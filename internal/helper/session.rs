use super::jwt::{check_token_expiration, Token};
use crate::{
    entities::friends::{repository::IFriendRepository, service::FriendDomain},
    handler::ws_handler::{
        EEvent, SocketMessage, SocketMessageNotification, SocketMessageStatusChange,
    },
    models::UserDTO,
};
use axum::async_trait;
use futures::lock::Mutex;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::sync::broadcast;
use tracing::{error, field::debug, info};

#[async_trait]
pub trait ISessionManager<S: ISession<F>, F: IFriendRepository>:
    Debug + Send + Sync + 'static
{
    async fn insert_into_current_user_connections(&self, session_manager: S);
    async fn remove_from_current_user_connections(
        &self,
        username: &String,
    ) -> Result<Arc<Mutex<S>>, String>;

    async fn remove_expired_current_user_connections_sessions(&self);
    async fn get_friends_in_current_user_connections<'a>(
        &self,
        client_uuid: &String,
    ) -> HashMap<String, Arc<Mutex<S>>>;
    fn get_current_user_connections(&self) -> &Arc<Mutex<HashMap<String, Arc<Mutex<S>>>>>;
}
#[derive(Debug)]
pub struct SessionManager<S: ISession<F>, F: IFriendRepository> {
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<S>>>>>,
    friend_domain: FriendDomain<F>,
}

impl<S: ISession<F>, F: IFriendRepository> SessionManager<S, F> {
    pub fn new(friend_domain: FriendDomain<F>) -> Self {
        return Self {
            friend_domain: friend_domain,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        };
    }
}

#[async_trait]
impl<S: ISession<F>, F: IFriendRepository> ISessionManager<S, F> for SessionManager<S, F> {
    fn get_current_user_connections(&self) -> &Arc<Mutex<HashMap<String, Arc<Mutex<S>>>>> {
        return &self.sessions;
    }
    async fn insert_into_current_user_connections(&self, session: S) {
        let mut current_user_connections = self.sessions.lock().await;
        let username = &session.clone().get_user().username;
        let session_manager = Arc::new(Mutex::new(session));
        current_user_connections.insert(username.to_owned(), session_manager);
    }

    async fn remove_from_current_user_connections(
        &self,
        username: &String,
    ) -> Result<Arc<Mutex<S>>, String> {
        let mut current_user_connections = self.sessions.lock().await;
        let session_manager = match current_user_connections.remove_entry(username) {
            None => {
                return Err(format!(
                    "user not current_user_connections pool: {}",
                    username
                ))
            }
            Some(user) => user.1,
        };
        Ok(session_manager)
    }

    async fn remove_expired_current_user_connections_sessions(&self) {
        let current_user_connections = self.sessions.lock().await.clone();
        let current_user_connections = current_user_connections.iter();
        let mut to_be_removed: Vec<&String> = Vec::new();
        for (user_id, sm) in current_user_connections {
            let sm = sm.lock().await;
            let token_is_expired = match check_token_expiration(sm.get_token().clone()) {
                Err(_) => true,
                Ok(_) => false,
            };

            if !token_is_expired {
                continue;
            }
            // Token is expired
            tracing::debug!(target: "application", "[remove_expired_current_user_connections_sessions] User: {} token is expired, removing", &user_id);
            to_be_removed.push(user_id);
        }
        for user_id in to_be_removed {
            let session_manager = self
                .remove_from_current_user_connections(&user_id)
                .await
                .expect("Could not remove from current_user_connections");
            tracing::debug!(
                target: "application", "[remove_expired_current_user_connections_sessions] Removed {} from current_user_connections sessions due to session expiration",
                user_id
            );

            let session_manager = session_manager.lock().await;
            session_manager.notify_offline(self).await;
            session_manager
                .send_direct_message(
                    crate::handler::ws_handler::SocketMessage::SocketMessageNotification(
                        SocketMessageNotification::new(
                            String::from("error"),
                            String::from("Important"),
                            String::from("Your session expired"),
                        ),
                    ),
                )
                .await;
        }
    }

    async fn get_friends_in_current_user_connections<'a>(
        &self,
        username: &String,
    ) -> HashMap<String, Arc<Mutex<S>>> {
        let friends_from_db = self
            .friend_domain
            .get_friends(&username)
            .expect("Could not get friends");
        let mut friends: HashMap<String, Arc<Mutex<S>>> = HashMap::new();
        let current_user_connections = self.get_current_user_connections().lock().await;

        // Currently just iterating over entire current_user_connections_state
        for user in friends_from_db.iter() {
            // Get sessionmanager from current_user_connections pool
            let session_manager = match current_user_connections.get(&user.username) {
                Some(sm) => sm,
                None => continue,
            };
            friends.insert(user.username.clone(), session_manager.to_owned());
        }

        friends
    }
}

#[derive(Debug)]
pub struct Session {
    pub user_socket: broadcast::Sender<SocketMessage>,
    pub user: UserDTO,
    pub token: Token,
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}

#[async_trait]
pub trait ISession<F: IFriendRepository>: Clone + Debug + Send + 'static {
    fn get_user_socket(&self) -> broadcast::Sender<SocketMessage>;
    fn get_user(&self) -> UserDTO;
    fn get_token(&self) -> Token;
    async fn send_direct_message(&self, message: SocketMessage);
    async fn notify_online(&self, session_manager: &impl ISessionManager<Self, F>);
    async fn notify_offline(&self, session_manager: &impl ISessionManager<Self, F>);
    fn new(user: UserDTO, token: Token) -> Self;
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            user_socket: self.user_socket.clone(),
            user: self.user.clone(),
            token: self.token.clone(),
        }
    }
}

#[async_trait]
impl<F: IFriendRepository> ISession<F> for Session {
    fn new(user: UserDTO, token: Token) -> Self {
        Session {
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
    async fn notify_online(&self, session_manager: &impl ISessionManager<Self, F>) {
        let friends_in_current_user_connections = session_manager
            .get_friends_in_current_user_connections(&self.user.username)
            .await;
        for (_, friend_session) in friends_in_current_user_connections {
            let friend_session = friend_session.lock().await;
            // This is necessary to satisfy all trait bounds. Because in this context friend_session is a Session, but it could be anything implementing ISession
            <Self as ISession<F>>::send_direct_message(
                &friend_session,
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange::new(
                    EEvent::ONLINE,
                    self.user.username.clone(),
                )),
            )
            .await;
        }
    }

    async fn notify_offline(&self, session_manager: &impl ISessionManager<Self, F>) {
        let friends_in_current_user_connections = session_manager
            .get_friends_in_current_user_connections(&self.user.username)
            .await;
        for (_, friend_session) in friends_in_current_user_connections {
            let friend_session = friend_session.lock().await;
            <Self as ISession<F>>::send_direct_message(
                &friend_session,
                SocketMessage::SocketMessageStatusChange(SocketMessageStatusChange::new(
                    EEvent::OFFLINE,
                    self.user.username.clone(),
                )),
            )
            .await;
        }
    }
}
