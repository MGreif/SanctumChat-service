use axum::{response::IntoResponse, http::StatusCode};
use axum::http::HeaderMap;
use serde::Serialize;
use serde_json::json;

#[derive(Clone)]
pub struct HTTPResponse<G: Serialize> {
    pub message: Option<String>,
    pub status: StatusCode,
    pub data: Option<G>,
}

impl<T: Serialize> IntoResponse for HTTPResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(json!({"message": self.message, "data": self.data}));
        (self.status, body).into_response()
    }
}