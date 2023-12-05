use std::sync::Arc;
use axum::{response::Response, middleware::Next, http::{Request, StatusCode, HeaderMap}, extract::{State}, Extension, body::Body};
use tracing::info;
use crate::{config::AppState, utils::jwt::validate_user_token};
use crate::middlewares::cookies::Cookies;

pub async fn auth<'a>( State(app_state): State<Arc<AppState>>, Extension(cookies): Extension<Cookies>, request: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    info!("{:?}", cookies);

    let session_cookie = match cookies.cookies.get("session") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(cookie) => cookie
    };

    match validate_user_token(session_cookie.to_owned(), app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {},
    };

    let response: Response = next.run(request).await;
    Ok(response)
}

pub async fn authBearer<'a>( State(app_state): State<Arc<AppState>>, headers: HeaderMap, request: Request<Body>, next: Next) -> Result<Response, StatusCode> {

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