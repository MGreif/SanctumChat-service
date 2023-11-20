use std::sync::Arc;

use axum::extract::Query;
use axum::http::StatusCode;
use axum::{response::IntoResponse, extract::State, http::HeaderMap};
use serde_json::json;
use uuid::Uuid;

use crate::schema::messages::{*};
use crate::{config::AppState, utils::jwt::token_into_typed, models::Message, schema::messages};
use crate::{schema::messages::dsl::*};
use diesel::{prelude::*, BoolExpressionMethods};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GetMessageDTO {
    pub origin: Uuid
}

pub async fn get_messages(State(app_state): State<Arc<AppState>>, headers: HeaderMap, Query(query): Query<GetMessageDTO>) -> impl IntoResponse {

    let auth_header = match headers.get("authorization") {
        None => return (StatusCode::UNAUTHORIZED, axum::Json(json!({"message": "No auth header provided"}))),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };
    let token = token_into_typed(auth_header, app_state.config.env.HASHING_KEY.as_bytes()).expect("Could not get token");
    let mut pool = app_state.db_pool.get().expect("[get_messages] Could not get db pool");

    let client_sent_or_received = sender.eq(token.sub).or(recipient.eq(token.sub));
    let recipient_sent_or_received = sender.eq(query.origin).or(recipient.eq(query.origin));
    let db_messages: Vec<Message> = messages.select(all_columns).filter(client_sent_or_received).filter(recipient_sent_or_received).load(&mut pool).expect("[get_messages] Could not get messages");
    return (StatusCode::OK, axum::Json(json!(db_messages)))
}