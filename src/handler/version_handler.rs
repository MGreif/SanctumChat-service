use std::env;
use axum::{response::IntoResponse, http::StatusCode};

use crate::helper::errors::HTTPResponse;

pub async fn version_handler() -> impl IntoResponse {
    return HTTPResponse::<String> {
        data: Some(env::var("CARGO_PKG_VERSION").unwrap()),
        message: None,
        status: StatusCode::OK
    }
}