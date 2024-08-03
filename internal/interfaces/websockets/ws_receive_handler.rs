use std::sync::Arc;

use crate::{
    appstate::AppState,
    entities::friends::repository::IFriendRepository,
    helper::{
        jwt::Token,
        session::{ISession, ISessionManager},
    },
    interfaces::websockets::socket_messages::{Receivable, SocketMessage, SocketMessageError},
    persistence::connection_manager::IConnectionManager,
};

pub async fn ws_receive_handler<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    message: SocketMessage,
    app_state: Arc<AppState<SM, S, C, F>>,
    token: Token,
) -> Result<(), SocketMessageError> {
    match message {
        SocketMessage::SocketMessageDirect(m) => return m.handle_receive(app_state, token).await,
        _ => return Ok(()),
    };
}
