use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::{os::unix::net::SocketAddr, sync::Arc};

use crate::{
    appstate::{AppState, IAppState},
    helper::{errors::HTTPResponse, session::ISessionManager},
    persistence::connection_manager::IConnectionManager,
};

pub async fn version_handler<S: ISessionManager, C: IConnectionManager>(
    State(app_state): State<Arc<AppState<S, C>>>,
) -> impl IntoResponse {
    return HTTPResponse::<String> {
        data: Some(app_state.get_config().env.APP_VERSION.clone()),
        message: None,
        status: StatusCode::OK,
    };
}
