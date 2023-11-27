use std::sync::Arc;
use axum::{extract::{Json, Query, State}, response::IntoResponse, http::{HeaderMap, header::{SET_COOKIE, self}, StatusCode}};
use tracing::info;
use uuid::Uuid;
use crate::{schema::users::dsl::*, helper::session::get_friends_in_p2p};
use serde_json::json;
use crate::{config::AppState, models::{UserDTO, self}, schema::{self, users::{self, all_columns}}, validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR, utils::jwt::{hash_string, validate_user_token, token_into_typed}, helper::session::{prepare_user_session_manager, update_user_friends}};
use diesel::{prelude::*};
use crate::utils::jwt::encrypt_user_token;

#[derive(serde::Deserialize)]
pub struct UserCreateDTO {
    pub name: String,
    pub age: i32,
    password: String
}

#[derive(serde::Deserialize)]
pub struct GetUserQueryDTO {
    pub name: Option<String>
}

pub async fn get_users<'a>(State(state): State<Arc<AppState>>, Query(query_params): Query<GetUserQueryDTO>) -> String {

    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let mut query: _ = users.into_boxed();

    if let Some(query_name) = query_params.name {
        query = query.filter(name.like(format!("%{}%", query_name)));
    }

    let names: Vec<UserDTO> = query.select(schema::users::all_columns).load(&mut db_conn).expect("could not select users");
    format!("{}", serde_json::to_string(&names).unwrap())
}

pub async fn create_user<'a>(State(state): State<Arc<AppState>>, Json(body): Json<UserCreateDTO>) -> impl IntoResponse {
    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let mut new_user = models::UserDTO { 
        id: Uuid::new_v4(),
        age: body.age,
        name: body.name,
        password: body.password
    };

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&new_user.name) {
        Err(err) => {
            info!("{} - Validation error: {}", "name", err);
            return axum::Json(json!({"message": err, "field": "name", }))
        },
        Ok(_) => {}
    }
    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&new_user.password) {
        Err(err) => {
            info!("{} - Validation error: {}", "name", err);
            return axum::Json(json!({"message": err, "field": "password", }))
        },
        Ok(_) => {}
    }

    let encrypted_password = hash_string(&new_user.password, state.config.env.HASHING_KEY.clone().as_bytes());
    new_user.password = encrypted_password;
    info!("{:?}", new_user);

    let values = vec![new_user];
    diesel::insert_into(schema::users::table).values(&values).execute(&mut db_conn).expect("Could not insert data");

    axum::Json(json!({"message": "User created successfully"}))
}

#[derive(serde::Deserialize)]
pub struct LoginDTO {
    pub username: String,
    pub password: String
}

pub async fn logout<'a>(State(state): State<Arc<AppState>>, header: HeaderMap) -> impl IntoResponse {
    let token = header.get("authorization");
    info!("logout 1");
    let token = match token {
        None => {
            return axum::Json(json!({"message": "not logged in"}))
        },
        Some(token) => token_into_typed(token.to_str().unwrap().to_owned().replace("Bearer ", ""), state.config.env.HASHING_KEY.as_bytes()).unwrap()
    };
    info!("logout 2");

    let mut p2p = state.p2p_connections.lock().await;
    info!("logout 3");
    let (user_id, session_manager) = match p2p.remove_entry(&token.sub) {
        None => {
            return axum::Json(json!({"message": "user not p2p pool"}))
        },
        Some(user) => user,
    };

    info!("logout 4");

    session_manager.lock().await.notify_offline(&p2p).await;
    drop(p2p);
    // Remove user from logged in sessions

    info!("logout 5");

    let friends = get_friends_in_p2p(state.clone(), token.sub).await;
    // Remoe user from currently logged in friends 'active_friends'
    for (_, friend_user_session_manager) in friends {
        friend_user_session_manager.lock().await.remove_friend(&user_id).await;
    }
    info!("logout 6");



    axum::Json(json!({"message": "logged out"}))
}

pub async fn login<'a>(State(state): State<Arc<AppState>>, Json(body): Json<LoginDTO>) -> impl IntoResponse {
    let LoginDTO { password: pw, username } = body;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());


    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&username) {
        Err(err) => {
            info!("{} - Validation error: {}", "name", err);
            return (headers, axum::Json(json!({"message": err, "field": "username", })))
        },
        Ok(_) => {}
    }

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&pw) {
        Err(err) => {
            info!("{} - Validation error: {}", "name", err);
            return (headers, axum::Json(json!({"message": err, "field": "password", })))
        },
        Ok(_) => {}
    }


    let mut pool = state.db_pool.get().expect("Could not establish pool connection");
    let user_result: Result<(Uuid, String, i32, String ), _> = users
        .select(users::all_columns)
        .filter(name
            .eq(&username)
            .and(password.eq(hash_string(&pw, state.config.env.HASHING_KEY.clone().as_bytes()))))
        .first::<(Uuid, String, i32, String )>(&mut pool);

    let user = match user_result {
        Err(_) => return (headers, axum::Json(json!({"message": "login failed, wrong username or password"}))),
        Ok(result_id) => UserDTO { id: result_id.0, name: result_id.1, age: result_id.2, password: result_id.3 } 
    };


    let session_manager = prepare_user_session_manager(&user, state.clone()).await;
    update_user_friends(&user, state.clone()).await;

    let mut p2p_state = state.p2p_connections.lock().await;
    p2p_state.insert(user.id.clone(), session_manager.to_owned());


    let session_token = encrypt_user_token(user, state.config.env.HASHING_KEY.as_bytes());
    headers.insert(SET_COOKIE, format!("session={}; Max-Age=2592000; Path=/; SameSite=None", session_token).parse().unwrap());


    (headers, axum::Json(json!({"message": "login successful", "token": session_token})))
}

pub async fn token<'a>(State(app_state): State<Arc<AppState>>, headers: HeaderMap) -> (StatusCode, String) {
    info!("token 1");
    let auth_header = match headers.get("authorization") {
        None => return (StatusCode::UNAUTHORIZED, String::from("No auth header provided")),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };

    let is_valid = validate_user_token(auth_header.clone(), app_state.config.env.HASHING_KEY.as_bytes());
    info!("token 2");
    
    match is_valid {
        Err(err) => {return (StatusCode::INTERNAL_SERVER_ERROR, err)},
        Ok(_) => {}
    }

    let token = token_into_typed(auth_header.clone(), app_state.config.env.HASHING_KEY.as_bytes()).unwrap();
    info!("token 3");

    let mut pool = app_state.db_pool.get().expect("Could not get db pool");
    let user: UserDTO = users.select(all_columns).filter(id.eq(&token.sub)).first(&mut pool).expect("Could not get user");
    info!("token 3.5");

    info!("token 4");

    let session_manager = prepare_user_session_manager(&user, app_state.clone()).await;
    info!("token 4.5");
    update_user_friends(&user, app_state.clone()).await;
    let mut p2p_state = app_state.p2p_connections.lock().await;
    p2p_state.insert(token.sub, session_manager);
    info!("token 5");

    (StatusCode::OK, axum::Json::from(json!({"token": auth_header})).to_string())
}