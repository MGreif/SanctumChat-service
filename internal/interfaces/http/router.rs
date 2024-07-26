use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    middleware,
    routing::{get, patch, post},
    Router,
};
use tower_http::{
    cors::{AllowHeaders, AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

use crate::{
    config::{AppState, ConfigManager},
    entities::{friends, messages, users},
    handler::{version_handler, ws_handler},
    logging::{OnRequestLogger, OnResponseLogger},
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

pub fn initialize_http_server(app_state: &Arc<AppState>, config: ConfigManager) -> Router {
    let origin: AllowOrigin = match &config.env.CORS_ORIGIN {
        None => tower_http::cors::Any.into(),
        Some(r) => r.parse::<HeaderValue>().expect("Invalid cors url").into(),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PATCH])
        .allow_headers(AllowHeaders::any())
        .allow_origin(origin);

    let trace_layer: TraceLayer<
        tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
        DefaultMakeSpan,
        OnRequestLogger,
        OnResponseLogger,
    > = TraceLayer::new_for_http()
        .on_request(OnRequestLogger::new())
        .on_response(OnResponseLogger::new());

    let main_router = get_main_router(&app_state, config, cors);
    let app = Router::new().nest("/api", main_router).layer(trace_layer);
    app
}
