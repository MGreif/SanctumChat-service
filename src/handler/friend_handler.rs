use std::sync::Arc;
use axum::{Extension, extract::State, Json, response::IntoResponse};
use axum::extract::Path;
use axum::http::StatusCode;
use diesel::sql_types::{Bool, Text, Uuid, Nullable};

use crate::models::{UserDTO, FriendRequest};
use crate::schema::users;
use crate::{config::AppState, utils::jwt::Token};
use diesel::prelude::*;
use serde_json::json;
use tracing::info;
use crate::helper::errors::HTTPResponse;
use crate::helper::sql::get_friends_for_user_from_db;
use crate::schema::friend_requests::dsl::friend_requests;
use crate::validation::string_validate::UuidValidator;


pub struct FriendRequestGETDTO {
    pub id: Uuid,
    pub sender: Uuid,
    pub recipient: Uuid,
    pub accepted: Option<bool>
}



use diesel::sql_types::BigInt;

#[derive(QueryableByName)]
struct Count {
    #[sql_type = "BigInt"]
    count: i64,
}


#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct FriendRequestPOSTRequestDTO {
    recipient: String
}

#[derive(Debug, serde::Deserialize, serde::Serialize, QueryableByName)]
pub struct FriendRequestGETResponseDTO {
    #[diesel(sql_type = Uuid)]
    pub id: uuid::Uuid,
    #[diesel(sql_type = Uuid)]
    pub sender_id: uuid::Uuid,
    #[diesel(sql_type = Nullable<Text>)]
    pub sender_name: Option<String>,
    #[diesel(sql_type = Uuid)]
    pub recipient: uuid::Uuid,
    #[diesel(sql_type = Nullable<Bool>)]
    pub accepted: Option<bool>
}
pub async fn get_friend_requests(State(app_state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[get_friend_requests] Could not get connection pool");
    let issuer: UserDTO = users::table.filter(users::id.eq(token.sub)).get_result(&mut pool).unwrap();
    let query = diesel::sql_query("SELECT r.id as id, u.id as sender_id, u.name as sender_name, r.recipient as recipient, r.accepted as accepted FROM friend_requests as r INNER JOIN users as u ON u.id = r.sender WHERE r.recipient = $1 AND r.accepted IS NULL").bind::<diesel::sql_types::Uuid, _>(token.sub);
    println!("query: {}", diesel::debug_query::<diesel::pg::Pg, _>(&query).to_string());
    info!("sql querry {:?} {}", &query, token.sub.to_string());
    let friend_requests_results = query.load(&mut pool).expect("Could not get friend_requests");
    let friend_requests_results: Vec<FriendRequestGETResponseDTO> = friend_requests_results;
    return axum::Json(json!(friend_requests_results))
}

pub async fn create_friend_request(State(app_state): State<Arc<AppState>>, token: Extension<Token>, Json(body): Json<FriendRequestPOSTRequestDTO>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[create_friend_requests] Could not get connection pool");
    let recipient: uuid::Uuid = match uuid::Uuid::parse_str(body.recipient.as_str()) {
        Err(err) => return HTTPResponse::<FriendRequest> { status: StatusCode::BAD_REQUEST, message: Some(String::from("[create_friend_requests] Failed validating recipient uuid")), data: None },
        Ok(t) => t
    };

    let mut already_present = diesel::sql_query("SELECT COUNT(*) FROM friend_requests WHERE sender = $1 AND recipient = $2").bind::<diesel::sql_types::Uuid, _>(token.sub).bind::<Uuid, _>(recipient).load::<Count>(&mut pool).expect("Could not get friend requests");
    let already_present = match already_present.pop() {
        Some(i) => i.count,
        None => return     HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(format!("Could not get present friend-requests"))
        },
    };

    match already_present {
        0 => {},
        _ => return HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(format!("There is still a friend request present (Already created or pending or whatever. TODO: Change later)"))
        }
    };


    let new_request = FriendRequest {
        id: uuid::Uuid::new_v4(),
        accepted: None,
        recipient: recipient,
        sender: token.sub
    };
    let friend_request = diesel::insert_into(friend_requests).values(&new_request).execute(&mut pool).expect("[create_friend_requests] Could not insert friend request");
    HTTPResponse::<FriendRequest> {
        status: StatusCode::CREATED,
        data: Some(new_request),
        message: Some(format!("Inserted: {} rows", friend_request))
    }
}

#[derive(serde::Deserialize)]
pub struct FriendRequestPatchDTOBody {
    accepted: bool
}

pub async fn patch_friend_request(State(app_state): State<Arc<AppState>>, token: Extension<Token>,Path(uuid): Path<String>, Json(body): Json<FriendRequestPatchDTOBody>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[patch_friend_requests] Could not get connection pool");


    let validator = UuidValidator::new();

    if let Err(err) = validator.validate(uuid.as_str()) {
        return HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(String::from(err))
        }
    }

    let request_id: uuid::Uuid = match uuid::Uuid::parse_str(&uuid.as_str()) {
        Err(err) => return HTTPResponse::<FriendRequest> { status: StatusCode::BAD_REQUEST, message: Some(String::from("[patch_friend_requests] Failed validating id")), data: None },
        Ok(t) => t
    };

    match diesel::sql_query("SELECT COUNT(*) FROM friend_requests where id = $1 AND recipient = $2 AND accepted IS NULL")
        .bind::<diesel::sql_types::Uuid, _>(request_id)
        .bind::<diesel::sql_types::Uuid, _>(token.sub)
        .load::<Count>(&mut pool)
        .expect("Could not get count of friend requests").pop().expect("No rows").count {
        0 =>   return  HTTPResponse::<FriendRequest> {
            status: StatusCode::BAD_REQUEST,
            data: None,
            message: Some(format!("No Friendrequest available", ))
        },
        _ => {}
    }


    let mut query = diesel::sql_query("UPDATE friend_requests SET ").into_boxed();

    query = query.sql("accepted = $1 ").bind::<Bool, _>(body.accepted);

    let query = query.sql("WHERE id = $2").bind::<Uuid, _>(request_id);
    let patched = query.execute(&mut pool).expect("[patch_friend_requests] Could not patch friend request");
    HTTPResponse::<FriendRequest> {
        status: StatusCode::ACCEPTED,
        data: None,
        message: Some(format!("Inserted: {} rows", patched))
    }
}

pub async fn get_friends(State(app_state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {
    let mut pool = app_state.db_pool.get().expect("[get_friends] Could not get connection pool");
    let result = get_friends_for_user_from_db(& mut pool, token.sub).await;
    return HTTPResponse::<Vec<UserDTO>> {
        status: StatusCode::OK,
        data: Some(result),
        message: None
    }
}