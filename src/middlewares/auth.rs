use std::{sync::Arc, collections::HashMap};
use axum::{response::Response, middleware::Next, http::{Request, StatusCode, HeaderValue, header::{SET_COOKIE, COOKIE}}, extract::{Query, State}, Extension};
use tracing::info;
use crate::{config::AppState, utils::jwt::validate_user_cookie};
use crate::middlewares::cookies::Cookies;

pub async fn auth<B>( State(app_state): State<Arc<AppState>>, Extension(cookies): Extension<Cookies>, request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    info!("{:?}", cookies);

    let session_cookie = match cookies.cookies.get("session") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(cookie) => cookie
    };

    match validate_user_cookie(session_cookie.to_owned(), app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {},
    };

    let response: Response = next.run(request).await;
    Ok(response)
}