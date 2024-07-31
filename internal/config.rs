use crate::{
    handler::ws_handler::SocketMessageNotification,
    helper::{
        jwt::check_token_expiration,
        session::{ISession, Session},
        sql::get_friends_for_user_from_db,
    },
};
use axum::async_trait;
use diesel::{
    r2d2::{self, ConnectionManager, Pool},
    PgConnection,
};
use dotenv::dotenv;
use std::{borrow::Borrow, collections::HashMap, env, fmt::Debug, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use tracing::info;

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
