use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::{os::unix::net::SocketAddr, sync::Arc};
use tracing::info;

use crate::{
    appstate::AppState,
    appstate::IAppState,
    helper::{errors::HTTPResponse, session::ISessionManager},
};

pub async fn version_handler<S: ISessionManager>(
    State(app_state): State<Arc<AppState<S>>>,
) -> impl IntoResponse {
    return HTTPResponse::<String> {
        data: Some(app_state.get_config().env.APP_VERSION.clone()),
        message: None,
        status: StatusCode::OK,
    };
}
