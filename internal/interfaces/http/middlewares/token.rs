use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    appstate::{AppState, IAppState},
    entities::friends::repository::IFriendRepository,
    helper::{
        errors::HTTPResponse,
        jwt::token_into_typed,
        session::{ISession, ISessionManager},
    },
    persistence::connection_manager::IConnectionManager,
};

pub async fn token_mw<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, HTTPResponse<()>> {
    let headers = request.headers();
    let auth_token = match headers.get("authorization") {
        None => {
            return Err(HTTPResponse {
                message: Some(String::from("Authentication token not provided")),
                data: None,
                status: StatusCode::UNAUTHORIZED,
            })
        }
        Some(token) => token.to_str().unwrap().replace("Bearer ", ""),
    };

    let auth_token = match token_into_typed(
        &auth_token,
        app_state.get_config().env.HASHING_KEY.as_bytes(),
    ) {
        Err(_) => {
            return Err(HTTPResponse {
                message: Some(String::from("Could not deserialize token")),
                data: None,
                status: StatusCode::UNPROCESSABLE_ENTITY,
            })
        }
        Ok(token) => token,
    };

    request.extensions_mut().insert(auth_token);
    let response: Response = next.run(request).await;
    Ok(response)
}
