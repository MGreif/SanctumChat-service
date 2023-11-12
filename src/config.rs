use diesel::{r2d2, PgConnection};
use dotenv::dotenv;
use std::env;

pub struct AppState {
    db_pool: r2d2::Pool<r2d2::ConnectionManager<PgConnection>>,
}

#[derive(Debug, serde::Serialize)]
struct EnvConfig {
    pub DATABASE_URL: String
}

impl EnvConfig {
    pub fn new() -> EnvConfig {
        dotenv().ok();
        let mut envConfig: EnvConfig;
        envConfig.DATABASE_URL = env::var("DATABASE_URL").unwrap_or_default();
        envConfig
    }
}

#[derive(Debug)]
pub struct ConfigManager {
    env: EnvConfig
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let env = EnvConfig::new();
        println!("{:?}", serde_json::to_string(&env));
        ConfigManager { env }
    }
}