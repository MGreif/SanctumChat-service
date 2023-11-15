use std::sync::Arc;

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::info;
use crate::{config::AppState, utils::jwt::{validate_user_cookie, token_into_typed}};

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(app_state): State<Arc<AppState>>, Query(query): Query<WsQuery>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app_state, query))
}

async fn handle_socket(stream: WebSocket, app_state: Arc<AppState>, query: WsQuery) {

    let (mut sender, mut receiver) = stream.split();
    info!("{}", query.token.clone());
    if !validate_user_cookie(query.token.clone(), app_state.config.env.HASHING_KEY.as_bytes()).unwrap() {
        sender.send(Message::Text(String::from("Not authorized"))).await.unwrap();
        return
    } 

    let token = token_into_typed(query.token.clone(), app_state.config.env.HASHING_KEY.as_bytes()).unwrap();

    let mut rx = app_state.broadcast.subscribe();

    sender.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");
    let msg = format!("{} joined.", token.sub);
    tracing::debug!("{msg}");
    let _ = app_state.broadcast.send(msg);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            // In any websocket error, break loop.
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = app_state.broadcast.clone();

    // Spawn a task that takes messages from the websocket, prepends the user
    // name, and sends them to all broadcast subscribers.
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            // Add username before message.
            let _ = tx.send(format!("{}: {text}", token.sub));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}