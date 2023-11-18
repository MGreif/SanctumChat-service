use std::{sync::Arc, collections::HashMap};
use axum::{response::Response, middleware::Next, http::{Request, StatusCode, header::{SET_COOKIE, COOKIE}, HeaderMap}, extract::{Query, State}, Extension};
use tracing::info;
use crate::{config::AppState, utils::jwt::validate_user_token};
use crate::middlewares::cookies::Cookies;

pub async fn auth<B>( State(app_state): State<Arc<AppState>>, Extension(cookies): Extension<Cookies>, request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
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

pub async fn authBearer<B>( State(app_state): State<Arc<AppState>>, headers: HeaderMap, request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {

    let auth_header = match headers.get("authorization") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };

    match validate_user_token(auth_header, app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {},
    };

    let response: Response = next.run(request).await;
    Ok(response)
}