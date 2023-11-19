use std::{collections::HashMap, sync::Arc, ops::Index};

use futures::lock::Mutex;
use tracing::info;

use crate::{models::UserDTO, handler::ws_handler::{SocketMessage, SocketMessageStatusChange, EEvent}, config::AppState};
use tokio::sync::broadcast;
#[derive(Debug, Clone)]
pub struct SessionManager {
    pub active_friends: Arc<Mutex<Vec<String>>>,
    pub user_socket: broadcast::Sender<SocketMessage>,
    pub user: UserDTO,
    pub app_state: Arc<AppState>
}

impl<'a> SessionManager {
    pub fn new(user: UserDTO, app_state: Arc<AppState>) -> SessionManager {
        SessionManager { active_friends: Arc::new(Mutex::new(Vec::new())), user_socket: broadcast::channel(20).0, user, app_state }
    }

    pub async fn send_direct_message(&self, message: SocketMessage) {
        self.user_socket.send(message).expect("Could not send message");
    }
    pub async fn notify_online(&self) {
        for friend_id in self.active_friends.lock().await.iter() {
            info!("{friend_id}");
            let p2p = self.app_state.p2p_connections.lock().await;
            let friend_session_manager = p2p.get(friend_id).unwrap().lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::ONLINE, user_id: self.user.id.clone() })
            ).await;
        }
    }

    pub async fn notify_offline(&self, p2p_state: &futures::lock::MutexGuard<'_, HashMap<std::string::String, Arc<futures::lock::Mutex<SessionManager>>>>) {
        for friend_id in self.active_friends.lock().await.iter() {
            let friend_session_manager = p2p_state.get(friend_id).unwrap().lock().await;
            friend_session_manager.send_direct_message(
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange { status: EEvent::OFFLINE, user_id: self.user.id.clone() })
            ).await;
        }
    }

    pub async fn remove_friend(&self, friend_id: &str) {
        let mut friends = self.active_friends.lock().await;
        let index = friends.iter().position(|x| *x == friend_id).unwrap();
        friends.remove(index);
    }

    pub async fn add_friend(&self, friend_id: String) {
        let mut friends = self.active_friends.lock().await;
        friends.push(friend_id);
    }

    pub async fn add_self_to_friend(&self, friend_session_manager: SessionManager) {
        friend_session_manager.active_friends.lock().await.push(self.user.id.clone());
    }

    pub async fn get_friends(&self) -> futures::lock::MutexGuard<'_, Vec<std::string::String>> {
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

pub fn get_friends_in_p2p<'a>(p2p_state: &futures::lock::MutexGuard<'_, HashMap<std::string::String, Arc<futures::lock::Mutex<SessionManager>>>>) -> HashMap<String, Arc<Mutex<SessionManager>>> {
    let mut friends: HashMap<String, Arc<Mutex<SessionManager>>> = HashMap::new();
    
    // TODO let friends = get_friend_ids_from_db

    // TODO iterate through friends and push friends session manager into vec

    // TODO: Iterate through friends
    // Check if friend is in p2p_state
    // If yes, Append self as FriendSessionManager to friends 'friends'
    // If not, friend is offline and not connected
    // Currently not need because im getting friends directrly from p2p_state

    // Currently just iterating over entire p2p_state
    for (user_id, session_manager) in p2p_state.iter() {
        friends.insert(user_id.to_owned(),session_manager.to_owned());
    }

    friends

}


pub async fn update_user_friends<'a>(user: &UserDTO, app_state: Arc<AppState>) {
    let p2p_state = app_state.p2p_connections.lock().await;
    let friends_in_p2p_state = get_friends_in_p2p(&p2p_state); // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends

    for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
        if friend_id == &user.id {
            continue
        };

        // Insert self into friends of friends session manager
        let friend_session = friend_session_manager.lock().await;
        friend_session.active_friends.lock().await.push(user.id.clone());
    }
}

pub async fn prepare_user_session_manager<'a>(user: &UserDTO, app_state: Arc<AppState>) -> Arc<Mutex<SessionManager>> {
    // get friends from some source
    // Currently only getting other active users, because 'friends' is not implemented yet
    info!("prepare 0");
    let app_state = app_state.clone();
    let p2p_state = app_state.p2p_connections.lock().await;
    info!("prepare 0.5");
    let friends_in_p2p_state = get_friends_in_p2p(&p2p_state); // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends
    info!("prepare 1");
    drop(p2p_state);
    let self_session_manager = Arc::new(Mutex::new(SessionManager::new(user.clone(), app_state.clone())));
    info!("prepare 2");

    for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
        if friend_id == &user.id {
            continue
        };

        // Insert friends into self-session currently online friends
        let self_session_manager = self_session_manager.lock().await;
        let mut self_friends = self_session_manager.active_friends.lock().await;
        self_friends.push(friend_id.to_owned());

        drop(self_friends);
        drop(self_session_manager);
    }

    info!("prepare 3");


    let self_session_manager_locked = self_session_manager.lock().await;
    info!("{:?} {:?} {:?}", user, self_session_manager_locked, friends_in_p2p_state);
    info!("prepare 4");

    self_session_manager_locked.notify_online().await;
    info!("prepare 5");

    info!("{} has {:?} online friends", user.name, self_session_manager_locked.active_friends.lock().await.len());
    drop(self_session_manager_locked);
    self_session_manager.to_owned()

}
