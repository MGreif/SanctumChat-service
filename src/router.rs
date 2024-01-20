use std::sync::Arc;

use axum::{Router, routing::{get, post, patch}, middleware};
use tower_http::cors::CorsLayer;

use crate::{handler::{message_handler, friend_handler, user_handler, version_handler, ws_handler}, middlewares, config::{AppState, ConfigManager}};

pub fn get_main_router(app_state: &Arc<AppState>, config: ConfigManager, cors: CorsLayer) -> Router {
    let main = Router::new()
        .route("/messages/read", patch(message_handler::set_messages_read))
        .route("/messages", get(message_handler::get_messages))
        .route("/friends/active", get(friend_handler::get_active_friends))
        .route("/friends", get(friend_handler::get_friends))
        .route("/friend-requests", get(friend_handler::get_friend_requests).post(friend_handler::create_friend_request))
        .route("/friend-requests/:uuid", patch(friend_handler::patch_friend_request))
        .route("/token", post( user_handler::token))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), middlewares::auth::bearer_token_validation))
        .route("/logout", post(user_handler::logout))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), middlewares::token::token_mw))
        .route("/users", post(user_handler::create_user))
        .route("/ws", get(ws_handler::ws_handler))
        .route("/login", post( user_handler::login))
        .route("/version", get(version_handler::version_handler))
        .route_layer(middleware::from_fn(middlewares::cookies::cookie_mw))
        .layer(cors)
        .with_state(app_state.clone())
        .with_state(config.clone());
    return main
}