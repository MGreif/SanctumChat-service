use core::time;
use std::sync::Arc;
use axum::{http::{Method, StatusCode}, response::IntoResponse, BoxError};
use config::{EnvConfig, AppState};
use diesel::r2d2::{ConnectionManager, Pool};
use helper::errors::HTTPResponse;
use tower_http::cors::{CorsLayer, Any, AllowHeaders};
use std::net::SocketAddr;
use axum::Router;
use tracing;
mod models;
mod schema;
use diesel::prelude::*;
mod config;
mod handler;
mod validation;
mod repositories;
mod middlewares;
mod helper;
mod domain;
mod router;
mod models_test;
use router::get_main_router;

fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
    .allow_origin(Any)
    .allow_headers(AllowHeaders::any());

    let config = config::ConfigManager::new();
    
    let pool = get_connection_pool(config.env.clone());
    let app_state = Arc::new(AppState::new(pool, config.clone()));

    let app_state_clone = app_state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            app_state_clone.remove_expired_p2p_sessions().await;
        }
    });


    let main_router = get_main_router(&app_state, config, cors);
    let app = Router::new().nest("/api", main_router);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}

pub async fn error_handler(err: BoxError) -> impl IntoResponse {
    return HTTPResponse::<()> {
        data: None,
        message: Some(err.to_string()),
        status: StatusCode::INTERNAL_SERVER_ERROR
    }
 }