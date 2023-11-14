use std::sync::Arc;

use axum::{response::Response, middleware::Next, http::{Request, StatusCode, HeaderValue}, extract::{Query, State}};

use crate::config::AppState;

pub struct Q {
    pub name: String
}

pub async fn auth<B>( State(app_state): State<Arc<AppState>>, request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let headers = request.headers();
    if headers.get("auth").unwrap_or(&HeaderValue::from_static(&"")) == &"aaa" {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let response: Response = next.run(request).await;
    Ok(response)
}