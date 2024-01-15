use std::sync::Arc;

use axum::Extension;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{response::IntoResponse, extract::State};
use crate::config::AppState;
use crate::domain::message_domain::MessageDomain;
use crate::helper::errors::HTTPResponse;
use crate::helper::jwt::Token;
use crate::helper::pagination::Pagination;
use crate::models::Message;
use crate::repositories::message_repository::MessageRepository;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GetMessageDTO {
    pub origin: String,
    pub size: Option<u8>,
    pub index: Option<u8>
}

pub async fn get_messages(State(app_state): State<Arc<AppState>>, Query(query): Query<GetMessageDTO>, token: Extension<Token>) -> impl IntoResponse {
    let repo = MessageRepository {
        pg_pool: app_state.db_pool.get().expect("[get_messages] Could not get db pool")
    };
    let mut domain = MessageDomain::new(repo);

    let pagination = Pagination::new(query.size, query.index);
    let messages = domain.get_messages(&token.sub, &query.origin, pagination);

    match messages {
        Ok(res) => HTTPResponse::<Vec<Message>> {
            data: Some(res),
            status: StatusCode::OK,
            message: None
        }.into_response(),
        Err(err) => err.into_response()
    }
}