use axum::{response::IntoResponse, http::{StatusCode, Response}, body::Body};
use serde::Serialize;
use serde_json::json;


#[derive(Clone, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String
}


#[derive(Clone, PartialEq, Debug)]
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



impl<G: Serialize> HTTPResponse<G> {
    pub fn new_internal_error(message: String) -> HTTPResponse<G> {
        return Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            data: None,
            message: Some(message)
        }
    }
}