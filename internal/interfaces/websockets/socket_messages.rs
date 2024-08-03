use std::sync::Arc;

use uuid::Uuid;

use crate::{
    appstate::AppState,
    entities::friends::repository::IFriendRepository,
    helper::{
        jwt::Token,
        session::{ISession, ISessionManager},
    },
    persistence::connection_manager::IConnectionManager,
};

use super::messages::SocketMessageDirect::SocketMessageDirect;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageNotification {
    pub message: String,
    pub title: String,
    pub status: String,
    pub TYPE: String,
}

impl SocketMessageNotification {
    pub fn new(status: String, title: String, message: String) -> SocketMessageNotification {
        SocketMessageNotification {
            message,
            status,
            title,
            TYPE: String::from("SOCKET_MESSAGE_NOTIFICATION"),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]

pub enum EEvent {
    ONLINE,
    OFFLINE,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageEvent {
    event: EEvent,
    pub TYPE: String,
}

impl SocketMessageEvent {
    pub fn new(event: EEvent) -> SocketMessageEvent {
        SocketMessageEvent {
            event: EEvent::ONLINE,
            TYPE: String::from("SOCKET_MESSAGE_EVENT"),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageOnlineUsers {
    pub online_users: Vec<String>,
    pub TYPE: String,
}

impl SocketMessageOnlineUsers {
    pub fn new(online_users: Vec<String>) -> SocketMessageOnlineUsers {
        SocketMessageOnlineUsers {
            online_users,
            TYPE: String::from("SOCKET_MESSAGE_ONLINE_USERS"),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageStatusChange {
    pub status: EEvent,
    pub user_id: String,
    pub TYPE: String,
}

impl SocketMessageStatusChange {
    pub fn new(status: EEvent, user_id: String) -> SocketMessageStatusChange {
        SocketMessageStatusChange {
            status,
            user_id,
            TYPE: String::from("SOCKET_MESSAGE_STATUS_CHANGE"),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageFriendRequest {
    pub sender_username: String,
    pub friend_request_id: Uuid,
    pub TYPE: String,
}

impl SocketMessageFriendRequest {
    pub fn new(friend_request_id: Uuid, sender_username: String) -> SocketMessageFriendRequest {
        SocketMessageFriendRequest {
            friend_request_id,
            sender_username,
            TYPE: String::from("SOCKET_MESSAGE_FRIEND_REQUEST"),
        }
    }
}

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

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]

pub enum SocketMessage {
    SocketMessageDirect(SocketMessageDirect),
    SocketMessageNotification(SocketMessageNotification),
    SocketMessageStatusChange(SocketMessageStatusChange),
    SocketMessageOnlineUsers(SocketMessageOnlineUsers),
    SocketMessageFriendRequest(SocketMessageFriendRequest),
}
