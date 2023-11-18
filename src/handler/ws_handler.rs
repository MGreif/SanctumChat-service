use std::{sync::Arc, collections::HashMap};

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt, lock::Mutex};
use serde_json::from_str;
use tracing::info;
use tokio::sync::broadcast;
use crate::{config::AppState, utils::jwt::{validate_user_token, token_into_typed}, models::UserDTO};

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>, Query(query): Query<WsQuery>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
}

#[derive(Debug, Clone)]
pub struct SessionManager {
    pub friends: Arc<Mutex<HashMap<String, FriendSessionManager>>>,
    pub user_socket: broadcast::Sender<String>,
    pub user: UserDTO
}

impl SessionManager {
    pub fn new(user: UserDTO) -> SessionManager {
        
        SessionManager { friends: Arc::new(Mutex::new(HashMap::new())), user_socket: broadcast::channel(20).0, user }
    }
}

#[derive(Debug, Clone)]
pub struct FriendSessionManager {
    pub socket: broadcast::Sender<String>
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct SocketMessage {
    pub recipient: Option<String>,
    pub message: String
}

async fn handle_socket(stream: WebSocket, app_state: Arc<AppState>, query: WsQuery) {

    let (sender, mut receiver) = stream.split();

    let sender = Arc::new(Mutex::new(sender));




    let is_validated_result = validate_user_token(query.token.clone(), app_state.config.env.HASHING_KEY.as_bytes());
    match is_validated_result {
        Err(_) => {
            sender.lock().await.send(Message::Text(String::from("Not authorized"))).await.unwrap();
            return

        },
        Ok(_) => {}
    }

    let token = token_into_typed(query.token.clone(), app_state.config.env.HASHING_KEY.as_bytes()).unwrap();

    let app_state_current = app_state.clone();
    let p2p_connection = app_state_current.p2p_connections.lock().await;
    let client_session = p2p_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();
    // get friendSessionManager

    let mut client_rx = client_session.user_socket.subscribe();
    let client_tx = Arc::new(Mutex::new(client_session.user_socket.clone()));



    info!("amount of active p2p {}", p2p_connection.len());
    info!("p2p {:?}", p2p_connection);

    sender.lock().await.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");
    let msg = format!("{} joined.", token.sub);
    tracing::debug!("{msg}");
    let _ = app_state.broadcast.send(msg);



    let mut sender_receive_task = tokio::spawn(async move {
        while let Ok(msg) = client_rx.recv().await {
            // In any websocket error, break loop.
            if sender.lock().await.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message: SocketMessage = from_str(&text).expect(&format!("Could not deserialize {}", text));
            if let None = message.recipient {
                continue;
            }
            // Get fresh connection to get latest state
            let client_session = app_state.clone().p2p_connections.lock().await.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();
            let friends = client_session.friends.lock().await;

            info!("[name: {}]{} - friends {:?} - {}", token.name, message.recipient.clone().unwrap(), friends, friends.len());

            let recipient_session = friends.get(&message.recipient.unwrap());

            if let Some(session) = recipient_session {
                let recipient_tx = session.socket.clone();
        
                recipient_tx.send(format!("{}: {}", token.name, message.message)).expect("Could not send message");
                client_tx.lock().await.send(format!("you: {}", message.message)).unwrap();    
            } else if let None = recipient_session {
                client_tx.lock().await.send(format!("Recipient is offline")).unwrap();    
            }

        }
    });

    drop(p2p_connection);

    tokio::select! {
        _ = (&mut receive_task) => {
            sender_receive_task.abort();
        },
        _ = (&mut sender_receive_task) => {
            receive_task.abort();
        },
    };
}