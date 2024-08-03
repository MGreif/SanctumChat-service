use appstate::AppState;
use entities::friends::repository::FriendRepository;
use entities::friends::service::FriendDomain;
use helper::session::{Session, SessionManager};
use interfaces::http::router::initialize_http_server;
use persistence::connection_manager::{ConnectionManager, IConnectionManager};
use scheduler::session_cleanup::initialize_session_cleanup_schedule;
use std::net::SocketAddr;
use std::sync::Arc;
mod config;
mod entities;
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
mod tests;

#[tokio::main]
async fn main() {
    let config = config::ConfigManager::new();

    // This is needed. If the guards are _, the variables are deallocated and the logging does not work anymore
    let (_access_guard, _error_guard, _application_guard) = initialize_logger();
    let connection_manager = ConnectionManager::new(config.env.clone());

    let friend_domain = FriendDomain::new(FriendRepository {
        pg_pool: connection_manager.clone(),
    });
    let session_manager = SessionManager::new(friend_domain);
    let app_state = Arc::new(AppState::<
        SessionManager<Session, FriendRepository<ConnectionManager>>,
        Session,
        ConnectionManager,
        FriendRepository<ConnectionManager>,
    >::new(
        connection_manager, config.clone(), session_manager
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
