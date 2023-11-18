use std::sync::Arc;

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt, lock::Mutex};
use tracing::info;
use crate::{config::AppState, utils::jwt::{validate_user_token, token_into_typed}};

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
    recipient: String //User ID
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>, Query(query): Query<WsQuery>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
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
    let recipient = query.recipient;

    //TODO get sender logged in tx from app state


    info!("amount of active p2p {}", app_state.p2p_connections.lock().await.len());
    info!("p2p {:?}", app_state.p2p_connections.lock().await);
    let recipient_channel =  match app_state.p2p_connections.lock().await.get(&recipient) {
        None => {
            sender.lock().await.send(Message::Text(String::from(format!("{} is currently offline", recipient)))).await.unwrap();
            return;
        },
        Some(channel) => channel.to_owned()
    };

    let sender_channel =  match app_state.p2p_connections.lock().await.get(&token.sub) {
        None => {
            sender.lock().await.send(Message::Text(String::from(format!("{} lol wtf sender channel not there", &token.sub)))).await.unwrap();
            return;
        },
        Some(channel) => channel.to_owned()
    };


    let mut broadcast_rx = app_state.broadcast.subscribe();

    sender.lock().await.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");
    let msg = format!("{} joined.", token.sub);
    tracing::debug!("{msg}");
    let _ = app_state.broadcast.send(msg);
    let sender2 = sender.clone();

    // broadcast receive task
    let mut broadcast_receive_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            // In any websocket error, break loop.
            if sender2.lock().await.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Broadcast sender
    let _tx = app_state.broadcast.clone();
    let recipient_tx = recipient_channel.clone();
    let mut sender_rx = sender_channel.subscribe();


    let sender3 = sender.clone();


    let mut sender_receive_task = tokio::spawn(async move {
        while let Ok(msg) = sender_rx.recv().await {
            // In any websocket error, break loop.
            if sender3.lock().await.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let sender = sender.clone();

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = recipient_tx.send(format!("{}: {}", token.sub, text));
            sender.lock().await.send(Message::Text(String::from(format!("you: {text}")))).await.unwrap();
        }
    });

    tokio::select! {
        _ = (&mut broadcast_receive_task) => {
            receive_task.abort();
            sender_receive_task.abort();

        },
        _ = (&mut receive_task) => {
            broadcast_receive_task.abort();
            sender_receive_task.abort();
        },
        _ = (&mut sender_receive_task) => {
            broadcast_receive_task.abort();
            receive_task.abort();
        },
    };
}