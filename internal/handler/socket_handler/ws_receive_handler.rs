use std::sync::Arc;

use crate::{
    appstate::AppState,
    entities::friends::repository::IFriendRepository,
    handler::ws_handler::SocketMessage,
    helper::{
        jwt::Token,
        session::{ISession, ISessionManager},
    },
    persistence::connection_manager::IConnectionManager,
};

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageError {
    pub message: String,
    pub TYPE: String,
}

impl SocketMessageError {
    pub fn new(message: String) -> SocketMessageError {
        SocketMessageError {
            TYPE: String::from("SOCKET_MESSAGE_ERROR"),
            message,
        }
    }
}

pub trait Receivable<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>
{
    async fn handle_receive(
        &self,
        app_state: Arc<AppState<SM, S, C, F>>,
        token: Token,
    ) -> Result<(), SocketMessageError>;
}

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
