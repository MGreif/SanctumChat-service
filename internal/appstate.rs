use std::{collections::HashMap, fmt::Debug, sync::Arc};

use axum::async_trait;
use diesel::{
    r2d2::{self, ConnectionManager, Pool},
    PgConnection,
};
use futures::lock::Mutex;
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    config::ConfigManager,
    handler::ws_handler::SocketMessageNotification,
    helper::{
        jwt::check_token_expiration, session::ISessionManager, sql::get_friends_for_user_from_db,
    },
};

#[async_trait]
pub trait IAppState<T: ISessionManager>: Debug + Send + Sync {
    async fn insert_into_current_user_connections(&self, session_manager: T);
    async fn remove_from_current_user_connections(
        &self,
        username: &String,
    ) -> Result<Arc<Mutex<T>>, String>;

    async fn remove_expired_current_user_connections_sessions(&self);
    async fn get_friends_in_current_user_connections<'a>(
        &self,
        client_uuid: &String,
    ) -> HashMap<String, Arc<Mutex<T>>>;
    fn get_db_pool(&self) -> r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;
    fn get_current_user_connections(&self) -> &Mutex<HashMap<String, Arc<Mutex<T>>>>;
    fn get_config(&self) -> ConfigManager;
}

#[derive(Debug)]
pub struct AppState<T>
where
    T: ISessionManager,
{
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    pub broadcast: broadcast::Sender<String>,
    // Hashmap of currently logged in users
    pub current_user_connections: Arc<Mutex<HashMap<String, Arc<Mutex<T>>>>>,
    pub config: ConfigManager,
}

unsafe impl<T: ISessionManager> Send for AppState<T> {}
unsafe impl<T: ISessionManager> Sync for AppState<T> {}

impl<T: ISessionManager> AppState<T> {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, config: ConfigManager) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        AppState {
            db_pool: pool,
            broadcast: tx,
            config: config,
            current_user_connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl<T: ISessionManager> IAppState<T> for AppState<T> {
    fn get_config(&self) -> ConfigManager {
        self.config.clone()
    }
    fn get_current_user_connections(&self) -> &Mutex<HashMap<String, Arc<Mutex<T>>>> {
        &self.current_user_connections
    }
    fn get_db_pool(&self) -> r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>> {
        self.db_pool.get().expect("Could not get db pool")
    }
    async fn insert_into_current_user_connections(&self, session_manager: T) {
        let mut current_user_connections = self.current_user_connections.lock().await;
        let username = &session_manager.clone().get_user().username;
        let session_manager = Arc::new(Mutex::new(session_manager));
        current_user_connections.insert(username.to_owned(), session_manager);
    }

    async fn remove_from_current_user_connections(
        &self,
        username: &String,
    ) -> Result<Arc<Mutex<T>>, String> {
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

    async fn remove_expired_current_user_connections_sessions(&self) {
        let current_user_connections = self.current_user_connections.lock().await.clone();
        let current_user_connections = current_user_connections.iter();
        let mut to_be_removed: Vec<&String> = Vec::new();
        for (user_id, sm) in current_user_connections {
            let sm = sm.lock().await;
            let token_is_expired = match check_token_expiration(sm.get_token().clone()) {
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
                .clone()
                .remove_from_current_user_connections(&user_id)
                .await
                .expect("Could not remove from current_user_connections");
            info!(
                "Removed {} from current_user_connections sessions due to session expiration",
                user_id
            );

            let session_manager = session_manager.lock().await;
            session_manager.notify_offline(self).await;
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

    async fn get_friends_in_current_user_connections<'a>(
        &self,
        client_uuid: &String,
    ) -> HashMap<String, Arc<Mutex<T>>> {
        let mut pool = self
            .db_pool
            .get()
            .expect("[get_friends] Could not get connection pool");
        let friends_from_db = get_friends_for_user_from_db(&mut pool, client_uuid).await;
        let mut friends: HashMap<String, Arc<Mutex<T>>> = HashMap::new();
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
