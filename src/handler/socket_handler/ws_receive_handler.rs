use std::sync::Arc;

use async_trait::async_trait;

use crate::{config::AppState, utils::jwt::Token, handler::ws_handler::SocketMessage};


#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageError {
    pub message: String,
    pub TYPE: String
}

impl SocketMessageError {
    pub fn new(message: String) -> SocketMessageError {
        SocketMessageError {
            TYPE: String::from("SOCKET_MESSAGE_ERROR"),
            message
        }
    }
}

#[async_trait]
pub trait Receivable {
    async fn handle_receive (&self, app_state: Arc<AppState>, token: Token) -> Result<(), SocketMessageError>;
}


pub async fn ws_receive_handler(message: SocketMessage, app_state: Arc<AppState>, token: Token) -> Result<(), SocketMessageError> {
    match message {
        SocketMessage::SocketMessageDirect(m) => return m.handle_receive(app_state, token).await,
        _ => return Ok(()),
    };
}