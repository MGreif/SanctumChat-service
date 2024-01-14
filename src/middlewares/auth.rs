use std::sync::Arc;
use axum::{response::Response, middleware::Next, http::{Request, StatusCode, HeaderMap}, extract::State, body::Body};
use tracing::info;
use crate::{config::AppState, helper::jwt::validate_user_token};

pub async fn bearer_token_validation<'a>( State(app_state): State<Arc<AppState>>, headers: HeaderMap, request: Request<Body>, next: Next) -> Result<Response, StatusCode> {

    let auth_header = match headers.get("authorization") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };

    info!("middleware");

    match validate_user_token(auth_header, app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {},
    };

    let response: Response = next.run(request).await;
    Ok(response)
}