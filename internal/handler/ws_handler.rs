use std::sync::Arc;

use super::socket_handler::ws_handle_direct::SocketMessageDirect;
use crate::{
    config::AppState,
    handler::socket_handler::ws_receive_handler::{ws_receive_handler, SocketMessageError},
    helper::jwt::{token_into_typed, validate_user_token},
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{lock::Mutex, sink::SinkExt, stream::StreamExt};
use serde_json::{from_str, to_string};
use tracing::info;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler<'a>(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
}

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
#[serde(untagged)]

pub enum SocketMessage {
    SocketMessageDirect(SocketMessageDirect),
    SocketMessageNotification(SocketMessageNotification),
    SocketMessageStatusChange(SocketMessageStatusChange),
    SocketMessageOnlineUsers(SocketMessageOnlineUsers),
    SocketMessageFriendRequest(SocketMessageFriendRequest),
}

async fn handle_socket<'a>(stream: WebSocket, app_state: Arc<AppState>, query: WsQuery) {
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

    let app_state_orig = app_state.clone();
    let is_validated_result = validate_user_token(
        query.token.clone(),
        &app_state_orig.config.env.HASHING_KEY.as_bytes(),
    );
    match is_validated_result {
        Err(_) => {
            let message = SocketMessageError::new(String::from("You are not authenticated"));
            match sender
                .lock()
                .await
                .send(Message::Text(
                    to_string(&message).expect("Could not serialize message"),
                ))
                .await
            {
                Err(err) => info!("{}", err),
                Ok(_) => {}
            };
            return;
        }
        Ok(_) => {}
    }

    let token = token_into_typed(
        &query.token,
        app_state_orig.config.env.HASHING_KEY.as_bytes(),
    )
    .unwrap();
    let token2 = token.clone();

    let current_user_connections_connection = app_state_orig.current_user_connections.lock().await;
    let client_session = current_user_connections_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await;

    let mut client_session_receiver = client_session.user_socket.subscribe();
    drop(client_session);
    drop(current_user_connections_connection);

    // get online friends at client start/initialization
    let friends = app_state_orig.get_friends_in_current_user_connections(&token.sub).await;

    let mut online_friends: Vec<String> = vec![];

    for (friend_id, _) in friends {
        online_friends.push(friend_id.to_owned());
    }

    let mess =
        SocketMessage::SocketMessageOnlineUsers(SocketMessageOnlineUsers::new(online_friends));

    sender
        .lock()
        .await
        .send(Message::Text(to_string(&mess).unwrap()))
        .await
        .expect("Failed sending online_friends message");

    let sender_clone = sender.clone();

    // Handle whenever someone sends a message to the internally saved session_receiver user_socket
    let mut handle_client_session_receive_task = tokio::spawn(async move {
        while let Ok(msg) = client_session_receiver.recv().await {
            // If any websocket error, break loop.
            if sender
                .lock()
                .await
                .send(Message::Text(
                    to_string(&msg).unwrap_or_else(|err| err.to_string()),
                ))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    let app_state_clone = app_state.clone();
    let app_state_clone2 = app_state.clone();

    let token = token.clone();
    // Handle whenever the server receives a message from the client (browser)
    let mut handle_receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message: Result<SocketMessage, serde_json::Error> = from_str(&text);
            let message = match message {
                Ok(m) => m,
                Err(_) => {
                    let mut sender = sender_clone.lock().await;
                    let message =
                        SocketMessageError::new(format!("Could not deserialize {}", text));
                    sender
                        .send(Message::Text(to_string(&message).unwrap()))
                        .await
                        .unwrap();
                    continue;
                }
            };

            if let Err(err) =
                ws_receive_handler(message, app_state_clone.clone(), token.clone()).await
            {
                let mut sender_in_receiver = sender_clone.lock().await;
                sender_in_receiver
                    .send(Message::Text(serde_json::to_string(&err).unwrap()))
                    .await
                    .unwrap();
            }
        }
    });

    tokio::select! {
        _ = (&mut handle_receive_task) => {
            handle_client_session_receive_task.abort();
            match app_state_clone2.remove_from_current_user_connections(&token2.sub).await {
                Ok(_) => {},
                Err(err) => return info!("Error ocurred removing user from current_user_connections: {}; Maybe the user session expired or the user already logged out", err)
            };
        },
        _ = (&mut handle_client_session_receive_task) => {
            handle_receive_task.abort();
            match app_state_clone2.remove_from_current_user_connections(&token2.sub).await {
                Ok(_) => {},
                Err(err) => info!("Error ocurred removing user from current_user_connections: {}; Maybe the user session expired or the user already logged out", err)
            };
        },
    };
}
