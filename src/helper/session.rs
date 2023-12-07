use std::sync::Arc;
use futures::lock::Mutex;
use tracing::info;
use crate::{models::UserDTO, handler::ws_handler::{SocketMessage, SocketMessageStatusChange, EEvent}, config::AppState, utils::jwt::Token};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub user_socket: broadcast::Sender<SocketMessage>,
    pub user: UserDTO,
    pub token: Token,
    pub app_state: Arc<AppState>
}

impl<'a> SessionManager {
    pub fn new(user: UserDTO, token: Token, app_state: Arc<AppState>) -> SessionManager {
        SessionManager {
            user_socket: broadcast::channel(20).0,
            user,
            app_state,
            token
        }
    }

    pub async fn send_direct_message(&self, message: SocketMessage) {
        match self.user_socket.send(message)  {
            Err(err) => info!("Error, probably no listeners {}", err),
            Ok(_) => {}
        };
    }
    pub async fn notify_online(&self) {
        let friends_in_p2p = self.app_state.get_friends_in_p2p(&self.user.username).await;
        for (friend_id, friend_session_manager) in friends_in_p2p {
            info!("{friend_id}");
            let friend_session_manager = friend_session_manager.lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::ONLINE, user_id: self.user.username.clone() })
            ).await;
        }
    }

    pub async fn notify_offline(&self) {
        let friends_in_p2p = self.app_state.get_friends_in_p2p(&self.user.username).await;
        for (friend_id, friend_session_manager) in friends_in_p2p {
            info!("{friend_id}");
            let friend_session_manager = friend_session_manager.lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::OFFLINE, user_id: self.user.username.clone() })
            ).await;
        }
    }
}



