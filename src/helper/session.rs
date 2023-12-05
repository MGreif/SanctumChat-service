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

    pub async fn notify_offline(&self) {
        for friend_id in self.active_friends.lock().await.iter() {
            let friend_session_manager = self.app_state.p2p_connections.lock().await;
            let friend_session_manager = friend_session_manager.get(friend_id).unwrap().lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::OFFLINE, user_id: self.user.username.clone() })
            ).await;
        }
    }

    pub async fn remove_friend(&self, friend_id: &String) {
        let mut friends = self.active_friends.lock().await;
        let index = friends.iter().position(|x| *x == friend_id.to_owned()).unwrap();
        friends.remove(index);
    }

    pub async fn add_friend(&self, friend_id: String) {
        let mut friends = self.active_friends.lock().await;
        friends.push(friend_id);
    }

    pub async fn add_self_to_friend(&self, friend_session_manager: SessionManager) {
        friend_session_manager.active_friends.lock().await.push(self.user.username.clone());
    }

    pub async fn get_friends(&self) -> futures::lock::MutexGuard<'_, Vec<String>> {
        self.active_friends.lock().await
    }

}

#[derive(Debug, Clone)]
pub struct FriendSessionManager {
    pub socket: broadcast::Sender<SocketMessage>
}

impl FriendSessionManager {
    pub async fn send_direct_message(&self, message: SocketMessage) {
        info!("{:?}", self.socket);
        self.socket.send(message).expect("Could not send message");
    }
}



pub async fn update_user_friends<'a>(user: &UserDTO, app_state: Arc<AppState>) {
    let friends_in_p2p_state = app_state.get_friends_in_p2p(&user.username).await; // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends

    for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
        if friend_id == &user.username {
            continue
        };

        // Insert self into friends of friends session manager
        let friend_session = friend_session_manager.lock().await;
        let mut active_friends = friend_session.active_friends.lock().await;
        let index = active_friends.iter().position(|r| r == &user.username);

        match index {
            None => { active_friends.push(user.username.clone()) },
            Some(_) => {},
        };
    }
}

pub async fn prepare_user_session_manager<'a>(user: &UserDTO, token: Token, app_state: Arc<AppState>) -> Arc<Mutex<SessionManager>> {
    info!("prepare 0.5");
    let friends_in_p2p_state = app_state.get_friends_in_p2p(&user.username).await;
    info!("prepare 1");
    let self_session_manager = Arc::new(Mutex::new(SessionManager::new(user.clone(), token, app_state.clone())));
    info!("prepare 2");

    for (friend_id, _) in friends_in_p2p_state.iter() {
        if friend_id == &user.username {
            continue
        };

        // Insert friends into self-session currently online friends
        let self_session_manager = self_session_manager.lock().await;
        let mut self_friends = self_session_manager.active_friends.lock().await;
        match self_friends.iter().position(|r| r == friend_id) {
            None => self_friends.push(friend_id.to_owned()),
            Some(_) => {}
        };


        drop(self_friends);
        drop(self_session_manager);
    }

    info!("prepare 3");


    let self_session_manager_locked = self_session_manager.lock().await;
    info!("prepare 4");

    self_session_manager_locked.notify_online().await;
    info!("prepare 5");

    drop(self_session_manager_locked);
    self_session_manager.to_owned()
}
