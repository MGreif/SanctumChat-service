use diesel::{r2d2::{self, Pool, ConnectionManager}, PgConnection};
use dotenv::dotenv;
use std::{env, sync::Arc};

pub struct AppState {
    pub db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
}

impl AppState {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Arc<Self> {
        Arc::new(AppState { db_pool: pool })
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