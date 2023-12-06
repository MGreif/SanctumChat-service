use std::{sync::Arc, time::SystemTime};

use axum::{
    extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State, Query}, response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt, lock::Mutex};
use serde_json::{from_str, to_string};
use tracing::info;
use uuid::Uuid;
use crate::{config::AppState, utils::jwt::{validate_user_token, token_into_typed}, models};


use crate::schema::messages::dsl::messages;
use diesel::prelude::*;

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
    pub message: String,
    pub message_self_encrypted: String
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct SocketMessageNotification {
    pub message: String,
    pub title: String,
    pub status: String
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
pub struct SocketMessageFriendRequest {
    pub sender_username: String,
    pub friend_request_id: Uuid
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

    let p2p_connection = app_state_orig.p2p_connections.lock().await;
    let client_session = p2p_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await;


    let mut client_rx = client_session.user_socket.subscribe();



    // get online friends at client start/initialization
    let friends = client_session.active_friends.lock().await;

    let mut online_friends: Vec<String> = vec![];

    for friend_id in friends.iter() {
        online_friends.push(friend_id.to_owned());
    }

    drop(friends);
    drop(client_session);
    drop(p2p_connection);




    let mess = SocketMessage::SocketMessageOnlineUsers(SocketMessageOnlineUsers { online_users: online_friends });
    sender.lock().await.send(Message::Text(to_string(&mess).unwrap())).await.expect("Failed sending joining message");

    sender.lock().await.send(Message::Text(format!("You joined the channel"))).await.expect("Failed sending joining message");

    let msg = format!("{} joined.", token.sub);
    tracing::debug!("{msg}");
    let _ = app_state_orig.broadcast.send(msg);


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

    let token = token.clone();

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let message: SocketMessageDirect = from_str(&text).expect(&format!("Could not deserialize {}", text));
            if let None = message.recipient {
                continue;
            }

            // Get fresh connection to get latest state
            let client_session = app_state_clone.p2p_connections.lock().await.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await.clone();
            let recipient = message.recipient.unwrap();

            let message = SocketMessageDirect { sender: Some(token.sub.clone()), recipient: Some(recipient.clone()), message: message.message.clone(), message_self_encrypted: message.message_self_encrypted.clone() };
            let message_clone = message.clone();
            // Save message in db
            let message_db = models::Message {
                content: message_clone.message,
                content_self_encrypted: message_clone.message_self_encrypted,
                id: Uuid::new_v4(),
                recipient: recipient.clone(),
                sender: token.sub.clone(),
                sent_at: SystemTime::now()
            };


            let mut pool = app_state_clone.db_pool.get().expect("Could not get db connection to db to save sent message");
            diesel::insert_into(messages).values(&message_db).execute(&mut pool).expect(format!("Could not save message {:?}", &message_db).as_str());

            client_session.send_direct_message(SocketMessage::SocketMessageDirect(message.clone())).await;

            let p2p = app_state_clone.p2p_connections.lock().await.clone();
            let recipient_session_manager = p2p.get(&recipient).clone();
            match recipient_session_manager {
                None => {},
                Some(sm) => {
                    sm.lock().await.send_direct_message(SocketMessage::SocketMessageDirect(message.clone())).await
                    
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut receive_task) => {
            sender_receive_task.abort();
         //   let own_p2p = app_state_clone2.p2p_connections.lock().await;
         //   let own_p2p = own_p2p.get(&token2.sub.clone());
           // if let Some(sm) = own_p2p {
              //  let own_p2p = sm.lock().await;
              //  own_p2p.notify_offline().await;
           // };
        },
        _ = (&mut sender_receive_task) => {
            receive_task.abort();
           // let own_p2p = app_state_clone2.p2p_connections.lock().await;
           // let own_p2p = own_p2p.get(&token2.sub.clone());
            //if let Some(sm) = own_p2p {
              //  let own_p2p = sm.lock().await;
              //  own_p2p.notify_offline().await;
          //  };
        },
    };
}