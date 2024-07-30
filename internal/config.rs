use crate::{
    handler::ws_handler::SocketMessageNotification,
    helper::{
        jwt::check_token_expiration, session::SessionManager, sql::get_friends_for_user_from_db,
    },
};
use diesel::{
    r2d2::{self, ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;
use futures::lock::Mutex;
use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::broadcast;
use tracing::info;

#[derive(Debug)]
pub struct AppState {
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    pub broadcast: broadcast::Sender<String>,
    // Hashmap of currently logged in users
    pub current_user_connections: Mutex<HashMap<String, Arc<Mutex<SessionManager>>>>,
    pub config: ConfigManager,
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, config: ConfigManager) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        AppState {
            db_pool: pool,
            broadcast: tx,
            config: config,
            current_user_connections: Mutex::new(HashMap::new()),
        }
    }

    pub async fn insert_into_current_user_connections(&self, session_manager: SessionManager) {
        let mut current_user_connections = self.current_user_connections.lock().await;
        let username = &session_manager.clone().user.username;
        let session_manager = Arc::new(Mutex::new(session_manager));
        current_user_connections.insert(username.to_owned(), session_manager);
    }

    pub async fn remove_from_current_user_connections(
        &self,
        username: &String,
    ) -> Result<Arc<Mutex<SessionManager>>, String> {
        let mut current_user_connections = self.current_user_connections.lock().await;
        let session_manager = match current_user_connections.remove_entry(username) {
            None => {
                return Err(format!(
                    "user not current_user_connections pool: {}",
                    username
                ))
            }
            Some(user) => user.1,
        };
        Ok(session_manager)
    }

    pub async fn remove_expired_current_user_connections_sessions(&self) {
        let current_user_connections = self.current_user_connections.lock().await.clone();
        let current_user_connections = current_user_connections.iter();
        let mut to_be_removed: Vec<&String> = Vec::new();
        for (user_id, sm) in current_user_connections {
            let sm = sm.lock().await;
            let token_is_expired = match check_token_expiration(sm.token.clone()) {
                Err(_) => true,
                Ok(_) => false,
            };

            if !token_is_expired {
                continue;
            }
            // Token is expired
            to_be_removed.push(user_id);
        }

        for user_id in to_be_removed {
            let session_manager = self
                .remove_from_current_user_connections(&user_id)
                .await
                .expect("Could not remove from current_user_connections");
            info!(
                "Removed {} from current_user_connections sessions due to session expiration",
                user_id
            );

            let session_manager = session_manager.lock().await;
            session_manager.notify_offline().await;
            session_manager
                .send_direct_message(
                    crate::handler::ws_handler::SocketMessage::SocketMessageNotification(
                        SocketMessageNotification::new(
                            String::from("error"),
                            String::from("Important"),
                            String::from("Your session expired"),
                        ),
                    ),
                )
                .await;
        }
    }

    pub async fn get_friends_in_current_user_connections<'a>(
        &self,
        client_uuid: &String,
    ) -> HashMap<String, Arc<Mutex<SessionManager>>> {
        let mut pool = self
            .db_pool
            .get()
            .expect("[get_friends] Could not get connection pool");
        let friends_from_db = get_friends_for_user_from_db(&mut pool, client_uuid).await;
        let mut friends: HashMap<String, Arc<Mutex<SessionManager>>> = HashMap::new();
        let current_user_connections = self.current_user_connections.lock().await;

        // Currently just iterating over entire current_user_connections_state
        for user in friends_from_db.iter() {
            // Get sessionmanager from current_user_connections pool
            let session_manager = match current_user_connections.get(&user.username) {
                Some(sm) => sm,
                None => continue,
            };
            friends.insert(user.username.clone(), session_manager.to_owned());
        }

        friends
    }
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct EnvConfig {
    pub DATABASE_URL: String,
    pub HASHING_KEY: String,
    pub APP_VERSION: String,
    pub CORS_ORIGIN: Option<String>,
}

impl EnvConfig {
    pub fn new() -> EnvConfig {
        dotenv().ok();
        EnvConfig {
            DATABASE_URL: env::var("DATABASE_URL").expect("missing env DATABASE_URL"),
            HASHING_KEY: env::var("HASHING_KEY").expect("missing env HASHING_KEY"),
            APP_VERSION: option_env!("CARGO_PKG_VERSION").unwrap().to_string(),
            CORS_ORIGIN: match env::var("CORS_ORIGIN") {
                Ok(r) => Some(r),
                Err(_) => None,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigManager {
    pub env: EnvConfig,
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let env = EnvConfig::new();
        println!("{:?}", serde_json::to_string(&env));
        ConfigManager { env }
    }
}
