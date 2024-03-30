use std::{fs::File, os::unix::net::SocketAddr, path::Path, sync::Arc, time::{self, Duration, SystemTime, UNIX_EPOCH}};

use axum::{body::{Body, HttpBody}};
use cookie::time::{format_description::well_known::Rfc3339, OffsetDateTime};
use diesel::IntoSql;
use serde::Serialize;
use serde_json::json;
use tower_http::trace::{OnRequest, OnResponse};
use tracing::{info, Subscriber};


pub struct FileLogger {
    path: String,
    pub file: Option<File>
}

impl FileLogger {
    pub fn new(path: String) -> Self {
        return Self { path, file: None }
    }

    pub fn open(&mut self) {
        let exists = Path::new(&self.path).exists();

        if (exists) {
            let access_fd = File::options()
            .append(true)
            .write(true)
            .truncate(false)
            .open(&self.path)
            .expect("Failed to open access log file");
            self.file = Some(access_fd);

        } else {
            let access_fd = File::create(&self.path)
            .expect("Failed to open access log file");
            self.file = Some(access_fd);
        };

    }
}


#[derive(Clone)]
pub struct OnResponseLogger {

}

trait Serializable {}

// Implement the Serializable trait for all types that also implement Serialize
impl<T: Serialize> Serializable for T {}


impl OnResponseLogger {
   pub fn new() -> Self {
       return Self {  }
   }
}

impl OnResponse<Body> for OnResponseLogger
{
   fn on_response(self, response: &axum::http::Response<Body>, latency: time::Duration, span: &tracing::Span) {
       let value = json!({"status": response.status().as_str(), "latency": latency.as_nanos()/1000000 });
       info!("{}", value)
   }
}


#[derive(Clone)]
pub struct OnRequestLogger {

}

impl OnRequestLogger {
   pub fn new() -> Self {
       return Self {  }
   }
}

impl<B> OnRequest<B> for OnRequestLogger {
   fn on_request(&mut self, request: &axum::http::Request<B>, span: &tracing::Span) {
    let method = request.method();
    let uri = request.uri();
    let headers = request.headers();
    let offset_dt: OffsetDateTime = SystemTime::now().into();
    let offset_dt = offset_dt.format(&Rfc3339).expect("Could not format datetime");
    let value = format!("{:?} - {} {} - {:?}",offset_dt , method, uri, headers);

    
    info!("{}", value)
   }
}

