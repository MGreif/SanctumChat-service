use std::sync::Arc;

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt, lock::Mutex};
use serde_json::{from_str, to_string};
use tracing::info;
use uuid::Uuid;
use crate::{config::AppState, handler::socket_handler::ws_receive_handler::ws_receive_handler, helper::jwt::{token_into_typed, validate_user_token}};
use super::socket_handler::ws_handle_direct::SocketMessageDirect;


#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler<'a>(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>, Query(query): Query<WsQuery>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageNotification {
    pub message: String,
    pub title: String,
    pub status: String,
    pub TYPE: String
}

impl SocketMessageNotification {
    pub fn new(status: String, title: String, message: String) -> SocketMessageNotification {
        SocketMessageNotification { 
            message,
            status,
            title,
            TYPE: String::from("SOCKET_MESSAGE_NOTIFICATION")
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
    pub TYPE: String
}

impl SocketMessageEvent {
    pub fn new(event: EEvent) -> SocketMessageEvent {
        SocketMessageEvent { event: EEvent::ONLINE, TYPE: String::from("SOCKET_MESSAGE_EVENT") }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageOnlineUsers {
    pub online_users: Vec<String>,
    pub TYPE: String
}

impl SocketMessageOnlineUsers {
    pub fn new(online_users: Vec<String>) -> SocketMessageOnlineUsers {
        SocketMessageOnlineUsers { 
            online_users,
            TYPE: String::from("SOCKET_MESSAGE_ONLINE_USERS")
         }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageStatusChange {
    pub status: EEvent,
    pub user_id: String,
    pub TYPE: String
}

impl SocketMessageStatusChange {
    pub fn new(status: EEvent, user_id: String) -> SocketMessageStatusChange {
        SocketMessageStatusChange { 
            status,
            user_id,
            TYPE: String::from("SOCKET_MESSAGE_STATUS_CHANGE")
         }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageFriendRequest {
    pub sender_username: String,
    pub friend_request_id: Uuid,
    pub TYPE: String
}

impl SocketMessageFriendRequest {
    pub fn new(friend_request_id: Uuid, sender_username: String) -> SocketMessageFriendRequest {
        SocketMessageFriendRequest { 
            friend_request_id,
            sender_username,
            TYPE: String::from("SOCKET_MESSAGE_FRIEND_REQUEST")
         }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]

pub enum SocketMessage {
    SocketMessageDirect(SocketMessageDirect),
    SocketMessageNotification(SocketMessageNotification),
    SocketMessageStatusChange(SocketMessageStatusChange),
    SocketMessageOnlineUsers(SocketMessageOnlineUsers),
    SocketMessageFriendRequest(SocketMessageFriendRequest)
}


async fn handle_socket<'a>(stream: WebSocket, app_state: Arc<AppState>, query: WsQuery) {
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

    let app_state_orig = app_state.clone();
    let is_validated_result = validate_user_token(query.token.clone(), &app_state_orig.config.env.HASHING_KEY.as_bytes());
    match is_validated_result {
        Err(_) => {
            match sender.lock().await.send(Message::Text(String::from("Not authorized"))).await {
                Err(err) => info!("{}", err),
                Ok(_) => {}
            };
            return

        },
        Ok(_) => {}
    }

    let token = token_into_typed(&query.token, app_state_orig.config.env.HASHING_KEY.as_bytes()).unwrap();
    let token2 = token.clone();

    let p2p_connection = app_state_orig.p2p_connections.lock().await;
    let client_session = p2p_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await;


    let mut client_rx = client_session.user_socket.subscribe();
    drop(client_session);
    drop(p2p_connection);

    // get online friends at client start/initialization
    let friends = app_state_orig.get_friends_in_p2p(&token.sub).await;

    let mut online_friends: Vec<String> = vec![];

    for (friend_id, _) in friends {
        online_friends.push(friend_id.to_owned());
    }

    let mess = SocketMessage::SocketMessageOnlineUsers(SocketMessageOnlineUsers::new(online_friends));
    sender.lock().await.send(Message::Text(to_string(&mess).unwrap())).await.expect("Failed sending joining message");

    sender.lock().await.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");

    let msg = format!("{} joined.", token.sub);

    let _ = app_state_orig.broadcast.send(msg);

    let sender_in_receiver = sender.clone();

    let mut sender_receive_task = tokio::spawn(async move {
        while let Ok(msg) = client_rx.recv().await {
            // In any websocket error, break loop.
            if sender.lock().await.send(Message::Text(to_string(&msg).unwrap_or_else(|err| err.to_string()))).await.is_err() {
                break;
            }
        }
    });

    let app_state_clone = app_state.clone();
    let app_state_clone2 = app_state.clone();

    let token = token.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message: SocketMessage = from_str(&text).expect(&format!("Could not deserialize {}", text));

            if let Err(err) = ws_receive_handler(message, app_state_clone.clone(), token.clone()).await {
                let mut sender_in_receiver = sender_in_receiver.lock().await;
                sender_in_receiver.send(Message::Text(serde_json::to_string(&err).unwrap())).await.unwrap();
            }

            
        }
    });

    tokio::select! {
        _ = (&mut receive_task) => {
            sender_receive_task.abort();
            match app_state_clone2.remove_from_p2p(&token2.sub).await {
                Ok(_) => {},
                Err(err) => info!("Error ocurred removing user from p2p: {}; Maybe the user session expired", err)
            };
        },
        _ = (&mut sender_receive_task) => {
            receive_task.abort();
            match app_state_clone2.remove_from_p2p(&token2.sub).await {
                Ok(_) => {},
                Err(err) => info!("Error ocurred removing user from p2p: {}; Maybe the user session expired", err)
            };
        },
    };
}