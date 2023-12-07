use diesel::{r2d2::{self, Pool, ConnectionManager}, PgConnection};
use dotenv::dotenv;
use futures::lock::Mutex;
use tracing::info;
use std::{env, sync::Arc, collections::HashMap};
use tokio::sync::broadcast;

use crate::{helper::{session::{SessionManager}, sql::get_friends_for_user_from_db}, utils::jwt::check_token_expiration, handler::ws_handler::SocketMessageNotification};
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

    pub async  fn logout_user(&self, user_id: &String) -> Result<(),String> {
        let mut p2p = self.p2p_connections.lock().await;
        let (_, session_manager) = match p2p.remove_entry(user_id) {
            None => {
                return Err(String::from("user not p2p pool"))
            },
            Some(user) => user,
        };
        drop(p2p);
        session_manager.lock().await.notify_offline().await;
        // Remove user from logged in sessions
        Ok(())
    }

    pub async fn remove_expired_p2p_sessions(&self) {
        let mut p2p = self.p2p_connections.lock().await;
        let p2pclone = p2p.clone();
        let mut to_be_removed: Vec<&String> = Vec::new();
        for (user_id, sm) in p2pclone.iter() {
            let sm = sm.lock().await;
            let token_is_expired = match check_token_expiration(sm.token.clone()) {
                Err(_) => true,
                Ok(_) => false
            };

            if !token_is_expired {
                continue;
            }
            info!("Removed {} from p2p sessions due to session expiration", user_id);
            // Token is expired

            to_be_removed.push(user_id);
        }

        for user_id in to_be_removed {
            let (_, sm) = p2p.remove_entry(user_id).expect("Could not remove p2p entry");
            let sm = sm.lock().await;
            sm.notify_offline().await;
            sm.send_direct_message(crate::handler::ws_handler::SocketMessage::SocketMessageNotification(SocketMessageNotification {
                message: String::from("Your session expired"),
                title: String::from("Important"),
                status: String::from("error")
            })).await;
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
    pub HASHING_KEY: String
}


impl EnvConfig {
    pub fn new() -> EnvConfig {
        dotenv().ok();
        
        EnvConfig {
            DATABASE_URL: env::var("DATABASE_URL").expect("missing env DATABASE_URL"),
            HASHING_KEY: env::var("HASHING_KEY").expect("missing env HASHING_KEY")
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