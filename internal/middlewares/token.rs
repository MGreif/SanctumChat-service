use std::sync::Arc;

use axum::{http::{Request, StatusCode}, middleware::Next, response::Response, extract::State, body::Body};

use crate::{config::AppState, helper::{errors::HTTPResponse, jwt::token_into_typed}};

pub async fn token_mw(State(app_state): State<Arc<AppState>>, mut request: Request<Body>, next: Next) -> Result<Response, HTTPResponse<()>> {
    let headers = request.headers();
    let auth_token = match headers.get("authorization") {
        None => return Err(HTTPResponse { message: Some(String::from("Authentication token not provided")), data: None, status: StatusCode::UNAUTHORIZED,  } ),
        Some(token) => token.to_str().unwrap().replace("Bearer ", "")
    };

    let auth_token = match token_into_typed(&auth_token, app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(HTTPResponse { message: Some(String::from("Could not deserialize token")), data: None, status: StatusCode::UNPROCESSABLE_ENTITY } ),
        Ok(token) => token
    };

    request.extensions_mut().insert(auth_token);
    let response: Response = next.run(request).await;
    Ok(response)

}