use diesel::{r2d2::{self, Pool, ConnectionManager}, PgConnection};
use dotenv::dotenv;
use futures::lock::Mutex;
use std::{env, sync::Arc, collections::HashMap};
use tokio::sync::broadcast;

pub struct AppState {
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    pub broadcast: broadcast::Sender<String>,
    // Hashmap of currently logged in users
    pub p2p_connections: Mutex<HashMap<String, broadcast::Sender<String>>>,
    pub config: ConfigManager
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>, config: ConfigManager) -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(100);

        Arc::new( AppState { db_pool: pool, broadcast: tx, config: config, p2p_connections: Mutex::new(HashMap::new()) })
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