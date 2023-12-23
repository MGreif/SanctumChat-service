use diesel::{r2d2::{self, Pool, ConnectionManager}, PgConnection};
use dotenv::dotenv;
use futures::lock::Mutex;
use tracing::info;
use std::{env, sync::Arc, collections::HashMap};
use tokio::sync::broadcast;
use crate::{helper::{session::SessionManager, sql::get_friends_for_user_from_db, jwt::check_token_expiration}, handler::ws_handler::SocketMessageNotification};

#[derive(Debug)]
pub struct AppState {
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    pub broadcast: broadcast::Sender<String>,
    // Hashmap of currently logged in users
    pub p2p_connections: Mutex<HashMap<String, Arc<Mutex<SessionManager>>>>,
    pub config: ConfigManager
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, config: ConfigManager) -> Self {
        let (tx, _rx) = broadcast::channel(100);
         AppState { db_pool: pool, broadcast: tx, config: config, p2p_connections: Mutex::new(HashMap::new()) }
    }

    pub async fn insert_into_p2p(&self, session_manager: SessionManager) {
        let mut p2p = self.p2p_connections.lock().await;
        let username = &session_manager.clone().user.username;
        let session_manager = Arc::new(Mutex::new(session_manager));
        p2p.insert(username.to_owned(), session_manager);
    }

    pub async fn remove_from_p2p(&self, username: &String) -> Result<Arc<Mutex<SessionManager>>, String> {
        let mut p2p = self.p2p_connections.lock().await;
        let session_manager = match p2p.remove_entry(username) {
            None => {
                return Err(format!("user not p2p pool: {}", username))
            },
            Some(user) => user.1,
        };
        Ok(session_manager.clone())
    }

    pub async fn remove_expired_p2p_sessions(&self) {
        let p2p = self.p2p_connections.lock().await.clone();
        let p2p = p2p.iter()
        ;
        let mut to_be_removed: Vec<&String> = Vec::new();
        for (user_id, sm) in p2p {
            let sm = sm.lock().await;
            let token_is_expired = match check_token_expiration(sm.token.clone()) {
                Err(_) => true,
                Ok(_) => false
            };

            if !token_is_expired {
                continue;
            }
            // Token is expired
            to_be_removed.push(user_id);
        }

        for user_id in to_be_removed {
            info!("{} removing", user_id);
            let session_manager = self.remove_from_p2p(&user_id).await.expect("Could not remove from p2p");
            info!("Removed {} from p2p sessions due to session expiration", user_id);
        
            let session_manager = session_manager.lock().await;
            info!("{} notifzing offline", user_id);
            session_manager.notify_offline().await;
        
            info!("{} sending expiration message", user_id);
            session_manager.send_direct_message(crate::handler::ws_handler::SocketMessage::SocketMessageNotification(SocketMessageNotification::new(String::from("error"), String::from("Important"), String::from("Your session expired")))).await;
        }
    }

    pub async fn get_friends_in_p2p<'a>(&self, client_uuid: &String) -> HashMap<String, Arc<Mutex<SessionManager>>> {
        let mut pool = self.db_pool.get().expect("[get_friends] Could not get connection pool");
        let friends_from_db = get_friends_for_user_from_db(& mut pool, client_uuid).await;
        let mut friends: HashMap<String, Arc<Mutex<SessionManager>>> = HashMap::new();
        info!("1");
        let p2p_connections = self.p2p_connections.lock().await;
        info!("2");

        // Currently just iterating over entire p2p_state
        for user in friends_from_db.iter() {
    
            // Get sessionmanager from p2p pool
            info!("3");
    
            let session_manager = match p2p_connections.get(&user.username) {
                Some(sm) => sm,
                None => continue
            };
            friends.insert(user.username.clone(),session_manager.to_owned());
        }
        info!("4");
    
        friends
    
    }
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct EnvConfig {
    pub DATABASE_URL: String,
    pub HASHING_KEY: String,
    pub APP_VERSION: String
}


impl EnvConfig {
    pub fn new() -> EnvConfig {
        dotenv().ok();
        EnvConfig {
            DATABASE_URL: env::var("DATABASE_URL").expect("missing env DATABASE_URL"),
            HASHING_KEY: env::var("HASHING_KEY").expect("missing env HASHING_KEY"),
            APP_VERSION: option_env!("CARGO_PKG_VERSION").unwrap().to_string()
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigManager {
    pub env: EnvConfig
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let env = EnvConfig::new();
        println!("{:?}", serde_json::to_string(&env));
        ConfigManager { env }
    }
}