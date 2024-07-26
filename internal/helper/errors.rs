use core::fmt;
use std::num::NonZero;

use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use serde_json::json;
use tower_http::classify::{
    ClassifiedResponse, ClassifyResponse, MapFailureClass, ServerErrorsAsFailures,
    ServerErrorsFailureClass,
};

#[derive(Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct HTTPResponse<G: Serialize> {
    pub message: Option<String>,
    pub status: StatusCode,
    pub data: Option<G>,
}

impl<T: Serialize> IntoResponse for HTTPResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let message = self.message.unwrap_or_else(|| String::from(""));
        let body = axum::Json(json!({"message": message, "data": self.data}));
        if self.status >= StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(target: "error::server_error", message)
        } else if StatusCode::BAD_REQUEST <= self.status
            && self.status < StatusCode::INTERNAL_SERVER_ERROR
        {
            tracing::error!(target: "error::client_error", message)
        }
        (self.status, body).into_response()
    }
}

impl<G: Serialize> HTTPResponse<G> {
    pub fn new_internal_error(message: String) -> HTTPResponse<G> {
        return Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            data: None,
            message: Some(message),
        };
    }
}
