use std::sync::Arc;

use axum::Extension;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{response::IntoResponse, extract::State};
use crate::helper::errors::HTTPResponse;
use crate::schema::messages::{*};
use crate::utils::jwt::Token;
use crate::{config::AppState, models::Message};
use crate::{schema::messages::dsl::*};
use diesel::{prelude::*, BoolExpressionMethods};

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GetMessageDTO {
    pub origin: String,
    pub skip: Option<i64>
}

pub async fn get_messages(State(app_state): State<Arc<AppState>>, Query(query): Query<GetMessageDTO>, token: Extension<Token>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[get_messages] Could not get db pool");

    let client_sent_or_received = sender.eq(token.sub.clone()).or(recipient.eq(token.sub.clone()));
    let recipient_sent_or_received = sender.eq(query.origin.clone()).or(recipient.eq(query.origin));
    let mut sql_query = messages.select(all_columns).order_by(sent_at.desc()).into_boxed();
    if let Some(skip) = query.skip {
        sql_query = sql_query.offset(skip);
    }
    let mut db_messages: Vec<Message> = sql_query.limit(15).filter(client_sent_or_received).filter(recipient_sent_or_received).load(&mut pool).expect("[get_messages] Could not get messages");
    db_messages.reverse();
    return HTTPResponse::<Vec<Message>> {
        data: Some(db_messages),
        status: StatusCode::OK,
        message: None
    }
}