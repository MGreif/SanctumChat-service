use config::{AppState, EnvConfig};
use diesel::r2d2::{ConnectionManager, Pool};
use interfaces::http::router::initialize_http_server;
use scheduler::session_cleanup::initialize_session_cleanup_schedule;
use std::net::SocketAddr;
use std::sync::Arc;
mod models;
mod schema;
use diesel::prelude::*;
mod config;
mod entities;
mod handler;
mod helper;
mod interfaces;
mod logging;
mod middlewares;
mod models_test;
mod scheduler;
mod validation;
use logging::initialize_logger;

fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

#[tokio::main]
async fn main() {
    let config = config::ConfigManager::new();

    // This is needed. If the guards are _, the variables are deallocated and the logging does not work anymore
    let (_access_guard, _error_guard) = initialize_logger();

    let pool = get_connection_pool(config.env.clone());
    let app_state = Arc::new(AppState::new(pool, config.clone()));

    initialize_session_cleanup_schedule(app_state.clone());

    let app = initialize_http_server(&app_state, config);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
