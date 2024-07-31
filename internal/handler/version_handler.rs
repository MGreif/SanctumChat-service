use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    response::IntoResponse,
};
use std::{os::unix::net::SocketAddr, sync::Arc};

use crate::{
    appstate::{AppState, IAppState},
    entities::friends::repository::IFriendRepository,
    helper::{
        errors::HTTPResponse,
        session::{ISession, ISessionManager},
    },
    persistence::connection_manager::IConnectionManager,
};

pub async fn version_handler<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
) -> impl IntoResponse {
    return HTTPResponse::<String> {
        data: Some(app_state.get_config().env.APP_VERSION.clone()),
        message: None,
        status: StatusCode::OK,
    };
}
