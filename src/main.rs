use axum::http::{HeaderValue, Method};
use axum::Router;
use config::{AppState, EnvConfig};
use core::time;
use diesel::r2d2::{ConnectionManager, Pool};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, Any, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};
mod models;
mod schema;
use diesel::prelude::*;
mod config;
mod entities;
mod handler;
mod helper;
mod logging;
mod middlewares;
mod models_test;
mod router;
mod validation;
use logging::{initialize_logger, OnRequestLogger, OnResponseLogger};
use router::get_main_router;

fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

#[tokio::main]
async fn main() {
    let config = config::ConfigManager::new();

    let origin: AllowOrigin = match &config.env.CORS_ORIGIN {
        None => Any.into(),
        Some(r) => r.parse::<HeaderValue>().expect("Invalid cors url").into(),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
        .allow_headers(AllowHeaders::any())
        .allow_origin(origin);

    // This is needed. If the guards are _, the variables are deallocated and the logging does not work anymore
    let (_access_guard, _error_guard) = initialize_logger();

    let pool = get_connection_pool(config.env.clone());
    let app_state = Arc::new(AppState::new(pool, config.clone()));

    let app_state_clone = app_state.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(time::Duration::from_secs(15));
        loop {
            interval.tick().await;
            app_state_clone
                .remove_expired_current_user_connections_sessions()
                .await;
        }
    });

    let trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
        DefaultMakeSpan,
        OnRequestLogger,
        OnResponseLogger,
    > = TraceLayer::new_for_http()
        .on_request(OnRequestLogger::new())
        .on_response(OnResponseLogger::new());

    let main_router = get_main_router(&app_state, config, cors);
    let app = Router::new().nest("/api", main_router).layer(trace_layer);

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
