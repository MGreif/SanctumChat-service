use diesel::{r2d2::{self, Pool, ConnectionManager}, PgConnection};
use dotenv::dotenv;
use std::{env, sync::Arc, collections::HashMap};
use tokio::sync::{broadcast, Mutex};

pub struct WebSocketClient {
    pub id: String
}
pub struct AppState {
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
    pub broadcast: broadcast::Sender<String>

}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Arc<Self> {
        let (tx, _rx) = broadcast::channel(100);

        Arc::new(AppState { db_pool: pool, broadcast: tx })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EnvConfig {
    pub DATABASE_URL: String
}

impl EnvConfig {
    pub fn new() -> EnvConfig {
        dotenv().ok();
        
        EnvConfig { DATABASE_URL: env::var("DATABASE_URL").expect("could not establish connection") }
    }
}

#[derive(Debug)]
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