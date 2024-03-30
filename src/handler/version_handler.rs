use std::{os::unix::net::SocketAddr, sync::Arc};
use axum::{extract::{ConnectInfo, State}, http::StatusCode, response::IntoResponse};
use tracing::info;

use crate::{helper::errors::HTTPResponse, config::AppState};

pub async fn version_handler(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    return HTTPResponse::<String> {
        data: Some(app_state.config.env.APP_VERSION.clone()),
        message: None,
        status: StatusCode::OK
    }
}