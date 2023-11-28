use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;
use tracing::info;
use uuid::Uuid;
use diesel::prelude::*;
use crate::{models::UserDTO, handler::ws_handler::{SocketMessage, SocketMessageStatusChange, EEvent}, config::AppState};
use tokio::sync::broadcast;
use crate::helper::sql::get_friends_for_user_from_db;

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

    pub async fn notify_offline(&self, p2p_state: &futures::lock::MutexGuard<'_, HashMap<String, Arc<futures::lock::Mutex<SessionManager>>>>) {
        for friend_id in self.active_friends.lock().await.iter() {
            let friend_session_manager = p2p_state.get(friend_id).unwrap().lock().await;
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

pub async fn get_friends_in_p2p<'a>(app_state: Arc<AppState>, client_uuid: String) -> HashMap<String, Arc<Mutex<SessionManager>>> {
    let mut pool = app_state.db_pool.get().expect("[get_friends] Could not get connection pool");
    let friends_from_db = get_friends_for_user_from_db(& mut pool, client_uuid).await;
    let mut friends: HashMap<String, Arc<Mutex<SessionManager>>> = HashMap::new();
    info!("FRIENDS {:?}", friends_from_db);
    let p2p_connections = app_state.p2p_connections.lock().await;
    info!("{:?}", p2p_connections);
    // Currently just iterating over entire p2p_state
    for (user) in friends_from_db.iter() {

        // Get sessionmanager from p2p pool

        let session_manager = match p2p_connections.get(&user.username) {
            Some(sm) => sm,
            None => continue
        };
        info!("FRIEND WITH SESSION MANAGER {}", user.name);
        friends.insert(user.username.clone(),session_manager.to_owned());
    }

    friends

}


pub async fn update_user_friends<'a>(user: &UserDTO, app_state: Arc<AppState>) {
    let friends_in_p2p_state = get_friends_in_p2p(app_state.clone(), user.username.clone()).await; // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends

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

pub async fn prepare_user_session_manager<'a>(user: &UserDTO, app_state: Arc<AppState>) -> Arc<Mutex<SessionManager>> {
    info!("prepare 0.5");
    let friends_in_p2p_state = get_friends_in_p2p(app_state.clone(), user.username.clone()).await;
    info!("prepare 1");
    let self_session_manager = Arc::new(Mutex::new(SessionManager::new(user.clone(), app_state.clone())));
    info!("prepare 2");

    for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
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

    info!("{} has {:?} online friends", user.name, self_session_manager_locked.active_friends.lock().await.len());
    drop(self_session_manager_locked);
    self_session_manager.to_owned()
}
