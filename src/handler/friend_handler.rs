use std::sync::Arc;
use axum::{Extension, extract::State, Json, response::IntoResponse};
use axum::extract::Path;
use axum::http::StatusCode;
use diesel::sql_types::{Bool, Text, Uuid, Nullable};

use crate::domain::friend_request_domain::FriendRequestDomain;
use crate::helper::jwt::Token;
use crate::models::{FriendRequest, UserDTOSanitized};
use crate::config::AppState;
use crate::repositories::friend_request_repository::FriendRequestRepository;
use diesel::prelude::*;
use crate::handler::ws_handler::{SocketMessage, SocketMessageFriendRequest};
use crate::helper::errors::HTTPResponse;
use crate::helper::sql::get_friends_for_user_from_db;
use crate::validation::string_validate::UuidValidator;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct FriendRequestPOSTRequestDTO {
    recipient: String
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
    pub accepted: Option<bool>
}
pub async fn get_friend_requests(State(app_state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository { pg_pool: app_state.db_pool.get().expect("Could not get db_pool") };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);

    let friend_requests_result = match friend_request_domain.get_friend_requests_for_user(&token.sub) {
        Ok(res) => res,
        Err(err) => return err.into_response()
    };


    return HTTPResponse::<Vec<FriendRequestGETResponseDTO>> {
        data: Some(friend_requests_result),
        message: None,
        status: StatusCode::OK
    }.into_response()
}

pub async fn create_friend_request(State(app_state): State<Arc<AppState>>, token: Extension<Token>, Json(body): Json<FriendRequestPOSTRequestDTO>) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository { pg_pool: app_state.db_pool.get().expect("Could not get db_pool") };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);

    let recipient = body.recipient;

    let friend_request = match friend_request_domain.create_friend_request(&token.sub, &recipient) {
        Ok(res) => res,
        Err(err) => return err.into_response()
    };

    let receiver_session_manager = app_state.p2p_connections.lock().await;
    let receiver_session_manager = receiver_session_manager.get(&recipient.clone());
    if let Some(sm) = receiver_session_manager {
        let sm = sm.lock().await;
        let friend_request_message = SocketMessageFriendRequest::new(friend_request.id, token.sub.clone());
        sm.send_direct_message(SocketMessage::SocketMessageFriendRequest(friend_request_message)).await;
    };

    HTTPResponse::<FriendRequest> {
        status: StatusCode::CREATED,
        data: Some(friend_request),
        message: Some(format!("Successfully created friendrequest"))
    }.into_response()
}

#[derive(serde::Deserialize)]
pub struct FriendRequestPatchDTOBody {
    accepted: bool
}

pub async fn patch_friend_request(State(app_state): State<Arc<AppState>>, token: Extension<Token>,Path(uuid): Path<String>, Json(body): Json<FriendRequestPatchDTOBody>) -> impl IntoResponse {
    let friend_request_repository = FriendRequestRepository { pg_pool: app_state.db_pool.get().expect("Could not get db_pool") };
    let mut friend_request_domain = FriendRequestDomain::new(friend_request_repository);


    let validator = UuidValidator::new();

    if let Err(err) = validator.validate(uuid.as_str()) {
        return HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(String::from(err))
        }.into_response()
    }

    let request_id: uuid::Uuid = match uuid::Uuid::parse_str(&uuid.as_str()) {
        Err(_) => return HTTPResponse::<FriendRequest> { status: StatusCode::BAD_REQUEST, message: Some(String::from("[patch_friend_requests] Failed validating id")), data: None }.into_response(),
        Ok(t) => t
    };

    match friend_request_domain.accept_or_deny_friend_request(&request_id, &token.sub, body.accepted) {
        Ok(_) => HTTPResponse::<FriendRequest> {
            status: StatusCode::ACCEPTED,
            data: None,
            message: Some(String::from("Successfully updated friendrequest"))
        }.into_response(),
        Err(err) => err.into_response()
    }


    
}

pub async fn get_friends(State(app_state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[get_friends] Could not get connection pool");
    let result = get_friends_for_user_from_db(& mut pool, &token.sub).await;
    return HTTPResponse::<Vec<UserDTOSanitized>> {
        status: StatusCode::OK,
        data: Some(result),
        message: None
    }
}

pub async fn get_active_friends(State(app_state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {
    let result = app_state.get_friends_in_p2p(&token.sub).await;
    let result = result.iter().map(|u| u.0.to_owned()).collect::<Vec<String>>();
    return HTTPResponse::<Vec<String>> {
        status: StatusCode::OK,
        data: Some(result),
        message: None
    }
}