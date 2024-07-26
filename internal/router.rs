use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, patch, post},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    config::{AppState, ConfigManager},
    entities::{friends, messages, users},
    handler::{version_handler, ws_handler},
    middlewares,
};

pub fn get_main_router(
    app_state: &Arc<AppState>,
    config: ConfigManager,
    cors: CorsLayer,
) -> Router {
    let main = Router::new()
        .route(
            "/messages/read",
            patch(messages::controller::set_messages_read),
        )
        .route("/messages", get(messages::controller::get_messages))
        .route(
            "/friends/active",
            get(friends::controller::get_active_friends),
        )
        .route("/friends", get(friends::controller::get_friends))
        .route(
            "/friend-requests",
            get(friends::controller::get_friend_requests)
                .post(friends::controller::create_friend_request),
        )
        .route(
            "/friend-requests/:uuid",
            patch(friends::controller::patch_friend_request),
        )
        .route("/token", post(users::controller::token))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::auth::bearer_token_validation,
        ))
        .route("/logout", post(users::controller::logout))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            middlewares::token::token_mw,
        ))
        .route("/users", post(users::controller::create_user))
        .route("/ws", get(ws_handler::ws_handler))
        .route("/login", post(users::controller::login))
        .route("/version", get(version_handler::version_handler))
        .route_layer(middleware::from_fn(middlewares::cookies::cookie_mw))
        .layer(cors)
        .with_state(app_state.clone())
        .with_state(config.clone());
    return main;
}
