use super::jwt::Token;
use crate::{
    appstate::IAppState,
    handler::ws_handler::{EEvent, SocketMessage, SocketMessageStatusChange},
    models::UserDTO,
};
use axum::async_trait;
use std::fmt::Debug;
use tokio::sync::broadcast;
use tracing::error;

#[derive(Debug)]
pub struct SessionManager {
    pub user_socket: broadcast::Sender<SocketMessage>,
    pub user: UserDTO,
    pub token: Token,
}

unsafe impl Send for SessionManager {}
unsafe impl Sync for SessionManager {}

#[async_trait]
pub trait ISessionManager: Clone + Debug + Send + 'static {
    fn get_user_socket(&self) -> broadcast::Sender<SocketMessage>;
    fn get_user(&self) -> UserDTO;
    fn get_token(&self) -> Token;
    async fn send_direct_message(&self, message: SocketMessage);
    async fn notify_online(&self, app_state: &impl IAppState<Self>);
    async fn notify_offline(&self, app_state: &impl IAppState<Self>);
    fn new(user: UserDTO, token: Token) -> Self;
}

impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            user_socket: self.user_socket.clone(),
            user: self.user.clone(),
            token: self.token.clone(),
        }
    }
}

#[async_trait]
impl ISessionManager for SessionManager {
    fn new(user: UserDTO, token: Token) -> Self {
        SessionManager {
            user_socket: broadcast::channel(20).0,
            user,
            token,
        }
    }
    fn get_token(&self) -> Token {
        self.token.clone()
    }
    fn get_user(&self) -> UserDTO {
        self.user.clone()
    }
    fn get_user_socket(&self) -> broadcast::Sender<SocketMessage> {
        self.user_socket.clone()
    }
    async fn send_direct_message(&self, message: SocketMessage) {
        match self.user_socket.send(message) {
            Err(err) => error!("Error, probably no listeners; {}", err),
            Ok(_) => {}
        };
    }
    async fn notify_online(&self, app_state: &impl IAppState<Self>) {
        let friends_in_current_user_connections = app_state
            .get_friends_in_current_user_connections(&self.user.username)
            .await;
        for (_, friend_session_manager) in friends_in_current_user_connections {
            let friend_session_manager = friend_session_manager.lock().await;
            friend_session_manager
                .send_direct_message(SocketMessage::SocketMessageStatusChange(
                    SocketMessageStatusChange::new(EEvent::ONLINE, self.user.username.clone()),
                ))
                .await;
        }
    }

    async fn notify_offline(&self, app_state: &impl IAppState<Self>) {
        let friends_in_current_user_connections = app_state
            .get_friends_in_current_user_connections(&self.user.username)
            .await;
        for (_, friend_session_manager) in friends_in_current_user_connections {
            let friend_session_manager = friend_session_manager.lock().await;
            friend_session_manager
                .send_direct_message(SocketMessage::SocketMessageStatusChange(
                    SocketMessageStatusChange::new(EEvent::OFFLINE, self.user.username.clone()),
                ))
                .await;
        }
    }
}
