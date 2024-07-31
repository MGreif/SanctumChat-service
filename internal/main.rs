use appstate::AppState;
use helper::session::SessionManager;
use interfaces::http::router::initialize_http_server;
use persistence::connection_manager::ConnectionManager;
use scheduler::session_cleanup::initialize_session_cleanup_schedule;
use std::net::SocketAddr;
use std::sync::Arc;
mod config;
mod entities;
mod handler;
mod helper;
mod interfaces;
mod logging;
mod models;
mod models_test;
mod scheduler;
mod schema;
mod validation;
use logging::initialize_logger;
mod appstate;
mod persistence;

#[tokio::main]
async fn main() {
    let config = config::ConfigManager::new();

    // This is needed. If the guards are _, the variables are deallocated and the logging does not work anymore
    let (_access_guard, _error_guard) = initialize_logger();

    let connection_manager = ConnectionManager::new(config.env.clone());
    let app_state = Arc::new(AppState::<SessionManager, ConnectionManager>::new(
        connection_manager,
        config.clone(),
    ));

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

mod appstate_test;
