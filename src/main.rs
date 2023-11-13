use std::sync::Arc;
use axum::Extension;
use config::{EnvConfig, AppState};
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
use handler::user_handler;


fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

fn get_app_state(pool: Pool<ConnectionManager<PgConnection>>) -> Arc<AppState> {
    AppState::new(pool)
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");


    let config = config::ConfigManager::new();
    let pool = get_connection_pool(config.env);
    let app_state = get_app_state(pool);

    let app = Router::new()
        .route("/users", get(user_handler::get_users).post(user_handler::create_user))
        .layer(Extension(app_state));


    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
