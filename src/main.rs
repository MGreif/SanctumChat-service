use std::future::Future;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::Arc;
use axum::Extension;
use diesel::r2d2::ConnectionManager;
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
mod person;
use diesel::{pg, Connection, r2d2 };
mod models;
mod schema;
use self::models::RootDTO;
use diesel::prelude::*;
use self::schema::root::dsl::*;
mod config;

#[derive(Debug, serde::Deserialize)]
struct QueryDTO {
    age: String
}

fn generate_random_data(prefix: String) -> Vec<RootDTO> {
    let mut random_data: Vec<RootDTO> = vec![];
    random_data
}




fn get_connection_pool() {
    let manager = ConnectionManager::<PgConnection>::new()
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    let config = config::ConfigManager::new();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    let database_uri = env::var("DATABASE_URL").expect("could not establish connectoin");
    let mut connection: PgConnection = pg::PgConnection::establish(&database_uri).unwrap();
    let values: Vec<RootDTO> = vec![RootDTO { name: String::from("SomeName") }];
    let results = diesel::insert_into(root).values(values).execute(&mut connection).expect("Could not insert");
    let mut p1 = person::Person::new("Carl", 20);
    async fn rootRoute(Extension(ext): Extension<String>) -> String {
        ext
    }

    async fn get_random_data() -> String {
        let random_data = generate_random_data(String::from("Preee"));
        serde_json::to_string(&random_data).unwrap()
    }


    let app = Router::new()
        .route("/getRandomData", get(get_random_data))
        .route("/change", get(rootRoute));


    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
