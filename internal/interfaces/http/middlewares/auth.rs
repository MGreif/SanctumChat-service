use crate::{
    appstate::{AppState, IAppState},
    helper::{jwt::validate_user_token, session::ISessionManager},
    persistence::connection_manager::IConnectionManager,
};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn bearer_token_validation<'a, S: ISessionManager, C: IConnectionManager>(
    State(app_state): State<Arc<AppState<S, C>>>,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = match headers.get("authorization") {
        None => return Err(StatusCode::UNAUTHORIZED),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };

    match validate_user_token(
        auth_header,
        app_state.get_config().env.HASHING_KEY.as_bytes(),
    ) {
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
        Ok(_) => {}
    };

    let response: Response = next.run(request).await;
    Ok(response)
}
