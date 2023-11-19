use std::sync::Arc;

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt, lock::Mutex};
use serde_json::{from_str, to_string, json};
use tracing::info;
use crate::{config::AppState, utils::jwt::{validate_user_token, token_into_typed}};

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler<'a>(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>, Query(query): Query<WsQuery>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
}



#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageDirect {
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub message: String
}
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]

pub enum EEvent {
    ONLINE,
    OFFLINE,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageEvent {
    event: EEvent,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageOnlineUsers {
    pub online_users: Vec<String>
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
pub struct SocketMessageStatusChange {
    pub status: EEvent,
    pub user_id: String
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]

pub enum SocketMessage {
    SocketMessageEvent(SocketMessageEvent),
    SocketMessageDirect(SocketMessageDirect),
    SocketMessageStatusChange(SocketMessageStatusChange),
    SocketMessageOnlineUsers(SocketMessageOnlineUsers)
}


async fn handle_socket<'a>(stream: WebSocket, app_state: Arc<AppState>, query: WsQuery) {
    info!("socket 1");
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));
    info!("socket 2");

    let app_state_orig = app_state.clone();

    let is_validated_result = validate_user_token(query.token.clone(), &app_state_orig.config.env.HASHING_KEY.as_bytes());
    match is_validated_result {
        Err(_) => {
            sender.lock().await.send(Message::Text(String::from("Not authorized"))).await.unwrap();
            return

        },
        Ok(_) => {}
    }
    info!("socket 3");

    let token = token_into_typed(query.token.clone(), app_state_orig.config.env.HASHING_KEY.as_bytes().clone()).unwrap();

    let p2p_connection = app_state_orig.p2p_connections.lock().await;
    let client_session = p2p_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await;

    info!("socket 4");

    let mut client_rx = client_session.user_socket.subscribe();



    // get online friends at client start/initialization
    let friends = client_session.active_friends.lock().await;
    info!("socket 5");

    let mut online_friends: Vec<String> = vec![];

    for friend_id in friends.iter() {
        online_friends.push(friend_id.to_owned());
    }

    drop(friends);
    drop(client_session);
    drop(p2p_connection);


    info!("socket 6");


    let mess = SocketMessage::SocketMessageOnlineUsers(SocketMessageOnlineUsers { online_users: online_friends });
    sender.lock().await.send(Message::Text(to_string(&mess).unwrap())).await.expect("Failed sending joining message");

    sender.lock().await.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");

    info!("socket 7");


    let msg = format!("{} joined.", token.sub);
    tracing::debug!("{msg}");
    let _ = app_state_orig.broadcast.send(msg);


    info!("socket 8");

    let mut sender_receive_task = tokio::spawn(async move {
        while let Ok(msg) = client_rx.recv().await {
            // In any websocket error, break loop.
            if sender.lock().await.send(Message::Text(to_string(&msg).unwrap_or_else(|err| err.to_string()))).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.

    let app_state_clone = app_state.clone();
    info!("socket 9");


    let token = token.clone();

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message: SocketMessageDirect = from_str(&text).expect(&format!("Could not deserialize {}", text));
            if let None = message.recipient {
                continue;
            }

            // Get fresh connection to get latest state
            let client_session = app_state_clone.p2p_connections.lock().await.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();
            let friends = client_session.active_friends.lock().await;
            let recipient = message.recipient.unwrap();
            info!("[name: {}]{} - friends {:?} - {}", token.name, &recipient, friends, friends.len());

            let p2p = app_state_clone.p2p_connections.lock().await;
            let recipient_sessin_manager = p2p.get(&recipient).unwrap().lock().await;

            // Send to recipient broadcast
            recipient_sessin_manager.send_direct_message(SocketMessage::SocketMessageDirect(SocketMessageDirect { sender: Some(token.sub.clone()), recipient: Some(recipient.clone()), message: message.message.clone() })).await;
            
            // Send back to client broadcast to reflect for sender
            client_session.send_direct_message(SocketMessage::SocketMessageDirect(SocketMessageDirect { sender: Some(token.sub.clone()), recipient: Some(recipient), message: message.message.clone() })).await; 
            
        }
    });

    info!("socket 10");

    tokio::select! {
        _ = (&mut receive_task) => {
            sender_receive_task.abort();
        },
        _ = (&mut sender_receive_task) => {
            receive_task.abort();
        },
    };
    info!("socket 11");

}