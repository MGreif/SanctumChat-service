use std::sync::Arc;
use axum::{Extension, middleware};
use config::{EnvConfig, AppState, ConfigManager};
use diesel::r2d2::{ConnectionManager, Pool};
use std::net::SocketAddr;
use axum::{
    routing::get,
    Router,
};
use tracing;
mod models;
mod schema;
use diesel::prelude::*;
mod config;
mod handler;
mod validation;
use handler::user_handler;
mod middlewares;
mod utils;


fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

fn get_app_state(pool: Pool<ConnectionManager<PgConnection>>, config: ConfigManager) -> Arc<AppState> {
    AppState::new(pool, config)
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");


    let config = config::ConfigManager::new();
    let pool = get_connection_pool(config.env.clone());
    let app_state = get_app_state(pool, config.clone());

    let app = Router::new()
        .route("/login", get( user_handler::login))
        .route("/users", get(user_handler::get_users).post(user_handler::create_user))
        .route("/ws", get(handler::ws_handler::ws_handler))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), middlewares::auth::auth))
        .with_state(app_state)
        .with_state(config.clone());
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
