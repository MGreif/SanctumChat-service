use std::sync::Arc;
use futures::lock::Mutex;
use tracing::info;
use crate::{models::UserDTO, handler::ws_handler::{SocketMessage, SocketMessageStatusChange, EEvent}, config::AppState, utils::jwt::Token};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub active_friends: Arc<Mutex<Vec<String>>>,
    pub user_socket: broadcast::Sender<SocketMessage>,
    pub user: UserDTO,
    pub token: Token,
    pub app_state: Arc<AppState>
}

impl<'a> SessionManager {
    pub fn new(user: UserDTO, token: Token, app_state: Arc<AppState>) -> SessionManager {
        SessionManager {
            active_friends: Arc::new(Mutex::new(Vec::new())),
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
        for friend_id in self.active_friends.lock().await.iter() {
            info!("{friend_id}");
            let p2p = self.app_state.p2p_connections.lock().await;
            let friend_session_manager = p2p.get(friend_id).unwrap().lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::ONLINE, user_id: self.user.username.clone() })
            ).await;
        }
    }

    pub async fn notify_offline(&self, p2p: &futures::lock::MutexGuard<'_, std::collections::HashMap<String, Arc<Mutex<SessionManager>>>>) {
        for friend_id in self.active_friends.lock().await.iter() {
            let friend_session_manager = match p2p.get(friend_id) {
                Some(sm) => sm.lock().await,
                None => return
            };
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::OFFLINE, user_id: self.user.username.clone() })
            ).await;
        }
    }

    pub async fn update_active_friends_from_p2p_friends(&self) -> &SessionManager {
        info!("prepare 0.5");
        let friends_in_p2p_state = self.app_state.get_friends_in_p2p(&self.user.username).await;
        info!("prepare 2");
    
        let mut self_friends = self.active_friends.lock().await;
    
        for (friend_id, _) in friends_in_p2p_state.iter() {
            if friend_id == &self.user.username {
                continue
            };
    
            // Insert friends into self-session currently online friends
            if !self_friends.contains(friend_id) {
                self_friends.push(friend_id.to_owned())
            }
        }
    
        info!("prepare 3");
    
        drop(self_friends);
        self.notify_online().await;
        info!("prepare 5");
        self
    }
    

    pub async fn update_user_friends(&self) {
        let friends_in_p2p_state = self.app_state.get_friends_in_p2p(&self.user.username).await; // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends
    
        for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
            if friend_id == &self.user.username {
                continue
            };
    
            // Insert self into friends of friends session manager
            let friend_session = friend_session_manager.lock().await;
            friend_session.add_friend(self.user.username.clone()).await;
    
            let mut active_friends = friend_session.active_friends.lock().await;
            let index = active_friends.iter().position(|r| r == &self.user.username);
    
            match index {
                None => { active_friends.push(self.user.username.clone()) },
                Some(_) => {},
            };
        }
    }

    pub async fn remove_friend(&self, friend_id: &String) {
        let mut friends = self.active_friends.lock().await;
        let index = match friends.iter().position(|x| *x == friend_id.to_owned()) {
            None => return,
            Some(us) => us
        };
        friends.remove(index);
    }

    pub async fn add_friend(&self, friend_username: String) {
        let mut active_friends = self.active_friends.lock().await;
        let index = active_friends.iter().position(|r| r == &friend_username);

        match index {
            None => { active_friends.push(friend_username) },
            Some(_) => {},
        };
    }
}



