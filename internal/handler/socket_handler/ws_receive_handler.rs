use std::sync::Arc;

use crate::{
    appstate::AppState,
    handler::ws_handler::SocketMessage,
    helper::{jwt::Token, session::ISessionManager},
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

pub trait Receivable<S: ISessionManager, C: IConnectionManager> {
    async fn handle_receive(
        &self,
        app_state: Arc<AppState<S, C>>,
        token: Token,
    ) -> Result<(), SocketMessageError>;
}

pub async fn ws_receive_handler<S: ISessionManager, C: IConnectionManager>(
    message: SocketMessage,
    app_state: Arc<AppState<S, C>>,
    token: Token,
) -> Result<(), SocketMessageError> {
    match message {
        SocketMessage::SocketMessageDirect(m) => return m.handle_receive(app_state, token).await,
        _ => return Ok(()),
    };
}
