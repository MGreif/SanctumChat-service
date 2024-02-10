use core::time;
use std::{fs::File, io::{self, stdout}, os::{self, unix::process}, process::exit, sync::Arc};
use axum::{http::{request, HeaderValue, Method, StatusCode}, response::IntoResponse, BoxError, extract::connect_info::MockConnectInfo};
use config::{EnvConfig, AppState};
use diesel::r2d2::{ConnectionManager, Pool};
use helper::errors::HTTPResponse;
use tower_http::{cors::{AllowHeaders, AllowOrigin, Any, CorsLayer}, trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, OnResponse, TraceLayer}};
use tracing_subscriber::layer::SubscriberExt;
use std::net::SocketAddr;
use axum::Router;
use tracing::{self, info, instrument::WithSubscriber, Level};
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
mod logging;
use logging::FileLogger;
use serde_json::json;
use router::get_main_router;
use tracing_subscriber::{filter, prelude::*};

fn get_connection_pool(env_config: EnvConfig) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(env_config.DATABASE_URL);
    let pool = Pool::new(manager).expect("Failed to create connection pool");
    pool
}

#[tokio::main]
async fn main() {

    let stdout_log = tracing_subscriber::fmt::layer()
        .with_writer(stdout)
        .pretty();

    let mut access_log_fd = FileLogger::new(String::from("access.log"));
    access_log_fd.open();
    let al_file = match access_log_fd.file {
        None => {exit(1)},
        Some(f) => f
    };


    let mut error_log_fd = FileLogger::new(String::from("error.log"));
    error_log_fd.open();
    let el_file = match error_log_fd.file {
        None => exit(1),
        Some(f) => f
    };

    let access_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(al_file))
        .with_target(false)
        .json();


    let error_log = tracing_subscriber::fmt::layer()
        .with_writer(Arc::new(el_file))
        .with_target(false)
        .json();


    tracing_subscriber::registry()
        .with(stdout_log)
        .with(access_log.with_filter(filter::LevelFilter::INFO))
        .with(error_log.with_filter(filter::LevelFilter::ERROR))
        .init();


    let config = config::ConfigManager::new();

    let origin: AllowOrigin = match &config.env.CORS_ORIGIN {
        None => Any.into(),
        Some(r) => r.parse::<HeaderValue>().expect("Invalid cors url").into()
    }; 

    let cors = CorsLayer::new()
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
    .allow_headers(AllowHeaders::any())
    .allow_origin(origin);

    
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

    let trace_layer = TraceLayer::new_for_http()
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(OnResponseLogger::new())
        .on_failure(DefaultOnFailure::new().level(Level::ERROR));

    let main_router = get_main_router(&app_state, config, cors);
    let app = Router::new().nest("/api", main_router)
        .layer(trace_layer);
    
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


 #[derive(Clone)]
 pub struct OnResponseLogger {

 }

impl OnResponseLogger {
    fn new() -> Self {
        return OnResponseLogger {  }
    }
}

 impl<B> OnResponse<B> for OnResponseLogger {
    fn on_response(self, response: &axum::http::Response<B>, latency: time::Duration, span: &tracing::Span) {
        let value = json!({"status": response.status().as_str(), "latency": latency });
        info!("{}", value)
    }
 }