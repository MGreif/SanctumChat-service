use std::str::FromStr;
use std::sync::Arc;

use crate::appstate::{AppState, IAppState};
use crate::entities::friends::repository::IFriendRepository;
use crate::helper::errors::HTTPResponse;
use crate::helper::jwt::Token;
use crate::helper::pagination::Pagination;
use crate::helper::session::{ISession, ISessionManager};
use crate::models::Message;
use crate::persistence::connection_manager::IConnectionManager;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use super::messages::MessageDomain;
use super::repository::MessageRepository;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct GetMessageDTO {
    pub origin: String,
    pub size: Option<u8>,
    pub index: Option<u8>,
}

pub async fn get_messages<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    Query(query): Query<GetMessageDTO>,
    token: Extension<Token>,
) -> impl IntoResponse {
    let repo = MessageRepository {
        pg_pool: app_state.get_db_pool(),
    };
    let mut domain = MessageDomain::new(repo);

    let pagination = Pagination::new(query.size, query.index);
    let messages = domain.get_messages(&token.sub, &query.origin, pagination);

    match messages {
        Ok(res) => HTTPResponse::<Vec<Message>> {
            data: Some(res),
            status: StatusCode::OK,
            message: None,
        }
        .into_response(),
        Err(err) => err.into_response(),
    }
}

#[derive(Deserialize, Serialize)]
pub struct SetMessageReadRequestQuery {
    pub ids: Vec<String>,
}

pub async fn set_messages_read<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
    Json(body): Json<SetMessageReadRequestQuery>,
) -> impl IntoResponse {
    let repo = MessageRepository {
        pg_pool: app_state.get_db_pool(),
    };
    let mut domain = MessageDomain::new(repo);

    let mut uuids: Vec<uuid::Uuid> = vec![];

    for string in body.ids {
        match uuid::Uuid::from_str(&string) {
            Err(_) => {
                return HTTPResponse::<()>::new_internal_error(format!(
                    "Could not parse {} as uuid",
                    &string
                ))
                .into_response()
            }
            Ok(res) => uuids.push(res.clone()),
        };
    }

    let result = domain.set_message_read(&uuids, &true, &token.sub);

    match result {
        Ok(_) => HTTPResponse::<()> {
            message: Some(String::from("Successfully edited messages")),
            data: None,
            status: StatusCode::OK,
        }
        .into_response(),
        Err(err) => HTTPResponse::<()>::new_internal_error(String::from(err)).into_response(),
    }
}
