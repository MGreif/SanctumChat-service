use std::sync::Arc;

use crate::{
    appstate::{AppState, IAppState},
    entities::friends::repository::IFriendRepository,
    helper::{
        jwt::{token_into_typed, validate_user_token},
        session::{ISession, ISessionManager},
    },
    interfaces::websockets::{
        socket_messages::{SocketMessage, SocketMessageError, SocketMessageOnlineUsers},
        ws_receive_handler::ws_receive_handler,
    },
    persistence::connection_manager::IConnectionManager,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::{from_str, to_string};
use tokio::sync::Mutex;
use tracing::info;

#[derive(serde::Deserialize)]
pub struct WsQuery {
    token: String,
}

pub async fn ws_handler<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    ws: WebSocketUpgrade,
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    Query(query): Query<WsQuery>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, app_state.to_owned(), query))
}

async fn handle_socket<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    stream: WebSocket,
    app_state: Arc<AppState<SM, S, C, F>>,
    query: WsQuery,
) {
    let (sender, mut receiver) = stream.split();
    let sender = Arc::new(Mutex::new(sender));

    let app_state_orig = app_state.clone();
    let is_validated_result = validate_user_token(
        query.token.clone(),
        &app_state_orig.get_config().env.HASHING_KEY.as_bytes(),
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
        app_state_orig.get_config().env.HASHING_KEY.as_bytes(),
    )
    .unwrap();
    let token2 = token.clone();

    let current_user_connections_connection = app_state_orig
        .get_session_manager()
        .get_current_user_connections()
        .lock()
        .await;
    let client_session = current_user_connections_connection.get(&token.sub).expect("Error getting client session. This should not appear because a session in create on login/token validations").lock().await;

    let mut client_session_receiver = client_session.get_user_socket().subscribe();
    drop(client_session);
    drop(current_user_connections_connection);

    // get online friends at client start/initialization
    let friends = app_state_orig
        .get_session_manager()
        .get_friends_in_current_user_connections(&token.sub)
        .await;

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
            match sender
                .lock()
                .await
                .send(Message::Text(
                    to_string(&msg).unwrap_or_else(|err| err.to_string()),
                ))
                .await
            {
                Err(err) => {
                    tracing::debug!(target: "application", "Websocket connection error: {}", &err);
                    break;
                },
                Ok(_) => {}
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
                let error_message = &err.message;
                tracing::error!(target: "websocket::handle_socket","{} - {}", &err.TYPE, &error_message);
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
            let session = match app_state_clone2.get_session_manager().remove_from_current_user_connections(&token2.sub).await {
                Ok(s) => s,
                Err(err) => return info!("Error ocurred removing user from current_user_connections: {}; Maybe the user session expired or the user already logged out", err)
            };
            tracing::debug!(target: "application", "Removed user due to session disconnect from browser");
            let session = session.lock().await;
            session.notify_offline(app_state_clone2.get_session_manager()).await;
        },
        _ = (&mut handle_client_session_receive_task) => {
            handle_receive_task.abort();
            let session = match app_state_clone2.get_session_manager().remove_from_current_user_connections(&token2.sub).await {
                Ok(s) => s,
                Err(err) => return info!("Error ocurred removing user from current_user_connections: {}; Maybe the user session expired or the user already logged out", err)
            };
            tracing::debug!(target: "application", "Removed user due to session disconnect from internal session state");
            let session = session.lock().await;
            session.notify_offline(app_state_clone2.get_session_manager()).await;

        },
    };
}
