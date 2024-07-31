use crate::appstate::{AppState, IAppState};
use crate::entities::friend_requests::friend_requests::FriendRequestDomain;
use crate::entities::friend_requests::repository::FriendRequestRepository;
use crate::entities::friends::repository::{FriendDTO, FriendRepository};
use crate::entities::friends::service::FriendDomain;
use crate::handler::ws_handler::{SocketMessage, SocketMessageFriendRequest};
use crate::helper::errors::HTTPResponse;
use crate::helper::jwt::Token;
use crate::helper::session::{ISession, ISessionManager};
use crate::models::FriendRequest;
use crate::persistence::connection_manager::IConnectionManager;
use crate::validation::string_validate::UuidValidator;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{extract::State, response::IntoResponse, Extension, Json};
use diesel::prelude::*;
use diesel::sql_types::{Bool, Nullable, Text, Uuid};
use std::borrow::BorrowMut;
use std::sync::Arc;

use super::repository::IFriendRepository;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct FriendRequestPOSTRequestDTO {
    recipient: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, QueryableByName)]
pub struct FriendRequestGETResponseDTO {
    #[diesel(sql_type = Uuid)]
    pub id: uuid::Uuid,
    #[diesel(sql_type = Text)]
    pub sender_id: String,
    #[diesel(sql_type = Text)]
    pub recipient: String,
    #[diesel(sql_type = Nullable<Bool>)]
    pub accepted: Option<bool>,
}
pub async fn get_friend_requests<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository {
        pg_pool: app_state.get_db_pool(),
    };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);

    let friend_requests_result =
        match friend_request_domain.get_friend_requests_for_user(&token.sub) {
            Ok(res) => res,
            Err(err) => return err.into_response(),
        };

    return HTTPResponse::<Vec<FriendRequestGETResponseDTO>> {
        data: Some(friend_requests_result),
        message: None,
        status: StatusCode::OK,
    }
    .into_response();
}

pub async fn create_friend_request<
    SM: ISessionManager<S, F>,
    S: ISession<F>,
    F: IFriendRepository,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
    Json(body): Json<FriendRequestPOSTRequestDTO>,
) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository {
        pg_pool: app_state.get_db_pool(),
    };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);

    let recipient = body.recipient;

    let friend_request = match friend_request_domain.create_friend_request(&token.sub, &recipient) {
        Ok(res) => res,
        Err(err) => return err.into_response(),
    };

    let receiver_session_manager = app_state
        .get_session_manager()
        .get_current_user_connections()
        .lock()
        .await;
    let receiver_session_manager = receiver_session_manager.get(&recipient.clone());
    if let Some(sm) = receiver_session_manager {
        let sm = sm.lock().await;
        let friend_request_message =
            SocketMessageFriendRequest::new(friend_request.id, token.sub.clone());
        sm.send_direct_message(SocketMessage::SocketMessageFriendRequest(
            friend_request_message,
        ))
        .await;
    };

    HTTPResponse::<FriendRequest> {
        status: StatusCode::CREATED,
        data: Some(friend_request),
        message: Some(format!("Successfully created friendrequest")),
    }
    .into_response()
}

#[derive(serde::Deserialize)]
pub struct FriendRequestPatchDTOBody {
    accepted: bool,
}

pub async fn patch_friend_request<
    SM: ISessionManager<S, F>,
    F: IFriendRepository,
    S: ISession<F>,
    T: IAppState<F, SM, S>,
>(
    State(app_state): State<Arc<T>>,
    token: Extension<Token>,
    Path(uuid): Path<String>,
    Json(body): Json<FriendRequestPatchDTOBody>,
) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository {
        pg_pool: app_state.get_db_pool(),
    };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);

    let validator = UuidValidator::new();

    if let Err(err) = validator.validate(uuid.as_str()) {
        return HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(String::from(err)),
        }
        .into_response();
    }

    let request_id: uuid::Uuid = match uuid::Uuid::parse_str(&uuid.as_str()) {
        Err(_) => {
            return HTTPResponse::<FriendRequest> {
                status: StatusCode::BAD_REQUEST,
                message: Some(String::from("[patch_friend_requests] Failed validating id")),
                data: None,
            }
            .into_response()
        }
        Ok(t) => t,
    };

    match friend_request_domain.accept_or_deny_friend_request(
        &request_id,
        &token.sub,
        body.accepted,
    ) {
        Ok(_) => HTTPResponse::<FriendRequest> {
            status: StatusCode::ACCEPTED,
            data: None,
            message: Some(String::from("Successfully updated friendrequest")),
        }
        .into_response(),
        Err(err) => err.into_response(),
    }
}

pub async fn get_friends<
    SM: ISessionManager<S, F>,
    F: IFriendRepository,
    S: ISession<F>,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
) -> impl IntoResponse {
    let friend_repository = FriendRepository {
        pg_pool: C::new(app_state.get_config().env),
    };
    let friend_domain = FriendDomain::new(friend_repository);
    let result = match friend_domain.get_friends(&token.sub) {
        Ok(res) => res,
        Err(err) => return HTTPResponse::<()>::new_internal_error(err).into_response(),
    };

    return HTTPResponse::<Vec<FriendDTO>> {
        status: StatusCode::OK,
        data: Some(result),
        message: None,
    }
    .into_response();
}

pub async fn get_active_friends<
    SM: ISessionManager<S, F>,
    F: IFriendRepository,
    S: ISession<F>,
    C: IConnectionManager,
>(
    State(app_state): State<Arc<AppState<SM, S, C, F>>>,
    token: Extension<Token>,
) -> impl IntoResponse {
    let app_state = app_state;
    let result = app_state
        .current_user_connections
        .get_friends_in_current_user_connections(&token.sub)
        .await;
    let result = result
        .iter()
        .map(|u| u.0.to_owned())
        .collect::<Vec<String>>();
    return HTTPResponse::<Vec<String>> {
        status: StatusCode::OK,
        data: Some(result),
        message: None,
    }
    .into_response();
}
