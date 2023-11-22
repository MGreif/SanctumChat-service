use axum::{response::IntoResponse, http::StatusCode};
use serde_json::json;

#[derive(Clone)]
pub struct HTTPResponse<G = ()> {
    pub message: Option<String>,
    pub status: StatusCode,
    pub data: G
}

impl IntoResponse for HTTPResponse {
    fn into_response(self) -> axum::response::Response {
        let body = axum::Json(json!({"message": self.message, "data": self.data}));
        (self.status, body).into_response()
    }
}