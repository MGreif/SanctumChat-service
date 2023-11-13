use std::future::Future;
use std::net::{TcpListener, TcpStream, AddrParseError};
use std::io::{Read, Write};
use std::sync::Arc;
use axum::Extension;
use axum::middleware::{ AddExtension };
use config::{ConfigManager, EnvConfig, AppState};
use diesel::r2d2::{ConnectionManager, Pool};
use serde::Serialize;
use std::net::SocketAddr;
use axum::{
    routing::{get, post},
    extract::{ Path, Query },
    http::StatusCode,
    response::{IntoResponse, Html},
    Json, Router,
};
use tracing;
use diesel::{pg, Connection, r2d2 };
mod models;
mod schema;
use self::models::UserDTO;
use diesel::prelude::*;
mod config;
use config::*;
mod handler;
use handler::user_handler;

#[derive(Debug, serde::Deserialize)]
struct QueryDTO {
    age: String
}

fn generate_random_data(prefix: String) -> Vec<UserDTO> {
    let mut random_data: Vec<UserDTO> = vec![];
    random_data
}




fn get_connection_pool(envConfig: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(envConfig.DATABASE_URL);
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

    //let values: Vec<RootDTO> = vec![RootDTO { name: String::from("SomeName") }];
    //let results = diesel::insert_into(root).values(values).execute(&mut connection).expect("Could not insert");



    async fn get_random_data() -> String {
        let random_data = generate_random_data(String::from("Preee"));
        serde_json::to_string(&random_data).unwrap()
    }


    let app = Router::new()
        .route("/getRandomData", get(get_random_data))
        .route("/users", get(user_handler::get_users).post(user_handler::create_user))
        .layer(Extension(app_state));


    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
