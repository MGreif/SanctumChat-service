use std::{
    env, io,
    time::{self, SystemTime},
};

use axum::body::Body;
use cookie::time::{format_description::well_known::Rfc3339, OffsetDateTime};
use serde::Serialize;
use serde_json::json;
use tower_http::trace::{OnRequest, OnResponse};
use tracing::{level_filters::LevelFilter, Level};
use tracing_appender::{non_blocking::WorkerGuard, rolling::Rotation};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Clone)]
pub struct OnResponseLogger {}

trait Serializable {}

// Implement the Serializable trait for all types that also implement Serialize
impl<T: Serialize> Serializable for T {}

impl OnResponseLogger {
    pub fn new() -> Self {
        return Self {};
    }
}

impl OnResponse<Body> for OnResponseLogger {
    fn on_response(
        self,
        response: &axum::http::Response<Body>,
        latency: time::Duration,
        span: &tracing::Span,
    ) {
        let value = json!({"status": response.status().as_str() });
        tracing::trace!(target:"http::response", "{}", value)
    }
}

#[derive(Clone)]
pub struct OnRequestLogger {}

impl OnRequestLogger {
    pub fn new() -> Self {
        return Self {};
    }
}

impl<B> OnRequest<B> for OnRequestLogger {
    fn on_request(&mut self, request: &axum::http::Request<B>, span: &tracing::Span) {
        let method = request.method();
        let uri = request.uri();
        let headers = request.headers();
        let offset_dt: OffsetDateTime = SystemTime::now().into();
        let offset_dt = offset_dt
            .format(&Rfc3339)
            .expect("Could not format datetime");
        let value = format!("{:?} - {} {} - {:?}", offset_dt, method, uri, headers);

        tracing::trace!(target:"http::request", "{}", value)
    }
}

pub fn initialize_logger() -> (WorkerGuard, WorkerGuard) {
    let stdout_log = tracing_subscriber::fmt::layer()
        .with_writer(io::stdout)
        .with_target(true)
        .with_line_number(false)
        .with_file(false)
        .log_internal_errors(true)
        .pretty();

    let access_log_writer = tracing_appender::rolling::Builder::new()
        .filename_prefix("access")
        .filename_suffix("log")
        .rotation(Rotation::DAILY)
        .build("./logs")
        .expect("Could not initiate access_log");

    let (access_log_handle, _access_guard) = tracing_appender::non_blocking(access_log_writer);

    let error_log_writer = tracing_appender::rolling::Builder::new()
        .filename_prefix("error")
        .filename_suffix("log")
        .rotation(Rotation::DAILY)
        .build("./logs")
        .expect("Could not initiate error_log");

    let (error_log_handle, _error_guard) = tracing_appender::non_blocking(error_log_writer);

    let access_log = tracing_subscriber::fmt::layer()
        .with_writer(access_log_handle)
        .with_line_number(false)
        .with_target(true)
        .json();

    let error_log = tracing_subscriber::fmt::layer()
        .with_writer(error_log_handle)
        .with_line_number(false)
        .with_target(true)
        .json();

    tracing_subscriber::registry()
        .with(stdout_log)
        .with(
            access_log.with_filter(
                filter::Targets::new()
                    .with_targets(vec![("http", LevelFilter::from_level(Level::TRACE))]),
            ),
        )
        .with(error_log.with_filter(filter::LevelFilter::from_level(Level::ERROR)))
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    return (_access_guard, _error_guard);
}
