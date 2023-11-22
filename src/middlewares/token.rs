use std::sync::Arc;

use axum::{http::{Request, StatusCode}, middleware::Next, response::Response, extract::State};

use crate::{utils::jwt::{validate_user_token, token_into_typed}, config::AppState, helper::errors::HTTPResponse};

pub async fn token_mw<B>(State(app_state): State<Arc<AppState>>, mut request: Request<B>, next: Next<B>) -> Result<Response, HTTPResponse> {
    let headers = request.headers();
    let auth_token = match headers.get("authorization").unwrap().to_str() {
        Err(_) => return Err(HTTPResponse { message: Some(String::from("Authentication token not provided")), data: (), status: StatusCode::UNAUTHORIZED,  } ),
        Ok(token) => token 
    };

    match validate_user_token(String::from(auth_token), app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(HTTPResponse { message: Some(String::from("Authentication token invalid")), data: (), status: StatusCode::FORBIDDEN } ),
        Ok(_) => {}
    }

    let auth_token = match token_into_typed(String::from(auth_token), app_state.config.env.HASHING_KEY.as_bytes()) {
        Err(_) => return Err(HTTPResponse { message: Some(String::from("Could not deserialize token")), data: (), status: StatusCode::UNPROCESSABLE_ENTITY } ),
        Ok(token) => token
    };

    request.extensions_mut().insert(auth_token);
    let response: Response = next.run(request).await;
    Ok(response)

}