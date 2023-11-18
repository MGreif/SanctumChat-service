use std::{sync::Arc, collections::HashMap};
use axum::{extract::{Json, Query, State, ws::Message}, response::IntoResponse, http::{HeaderMap, header::{SET_COOKIE, self}, StatusCode}};
use futures::lock::Mutex;
use tracing::info;
use super::{super::schema::users::dsl::*, ws_handler::FriendSessionManager};
use serde_json::json;
use crate::{config::AppState, models::{UserDTO, self}, schema::{self, users::{self, all_columns}}, validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR, utils::jwt::{hash_string, validate_user_token, token_into_typed}, handler::ws_handler::SessionManager};
use diesel::prelude::*;
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use crate::utils::jwt::encrypt_user_cookie;
use tokio::sync::broadcast;


fn generate_random_string(length: usize) -> String {
    let rng = thread_rng();

    let random_string: String = rng
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect();

    random_string
}

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

pub async fn get_users(State(state): State<Arc<AppState>>, Query(query_params): Query<GetUserQueryDTO>) -> String {

    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let mut query: _ = users.into_boxed();

    if let Some(query_name) = query_params.name {
        query = query.filter(name.like(format!("%{}%", query_name)));
    }

    let names: Vec<UserDTO> = query.select(schema::users::all_columns).load(&mut db_conn).expect("could not select users");
    format!("{}", serde_json::to_string(&names).unwrap())
}

pub async fn create_user(State(state): State<Arc<AppState>>, Json(body): Json<UserCreateDTO>) -> impl IntoResponse {
    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let mut new_user = models::UserDTO { 
        id: generate_random_string(10),
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
pub struct loginDTO {
    pub username: String,
    pub password: String
}

pub async fn logout(State(state): State<Arc<AppState>>, header: HeaderMap) -> impl IntoResponse {
    let token = header.get("authorization");
    let token = match token {
        None => {
            return axum::Json(json!({"message": "not logged in"}))
        },
        Some(token) => token_into_typed(token.to_str().unwrap().to_owned().replace("Bearer ", ""), state.config.env.HASHING_KEY.as_bytes()).unwrap()
    };

    let user_tx = state.p2p_connections.lock().await.remove_entry(&token.sub);

    info!("amount of active p2p {}", state.p2p_connections.lock().await.len());
    info!("p2p {:?}", state.p2p_connections.lock().await);
    let (user_id, user_tx) = match user_tx {
        None => {
            return axum::Json(json!({"message": "user not p2p pool"}))
        },
        Some(tx) => tx,
    };

    state.broadcast.send(format!("{} logged out", user_id)).unwrap(); // adjust this to send a json meta message that will be handled by the UI, which will then remove the 'online dot'

    axum::Json(json!({"message": "logged out"}))
}

pub async fn login(State(state): State<Arc<AppState>>, Json(body): Json<loginDTO>) -> impl IntoResponse {
    let loginDTO { password: pw, username } = body;
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
    let user_result: Result<(String, i32, String, String ), _> = users
        .select(users::all_columns)
        .filter(name
            .eq(&username)
            .and(password.eq(hash_string(&pw, state.config.env.HASHING_KEY.clone().as_bytes()))))
        .first::<(String, i32, String, String )>(&mut pool);

    let user = match user_result {
        Err(_) => return (headers, axum::Json(json!({"message": "login failed, wrong username or password"}))),
        Ok(result_id) => UserDTO { name: result_id.0, age: result_id.1, id: result_id.2, password: result_id.3 } 
    };



    let mut p2p_state = state.p2p_connections.lock().await;

    let session_manager = prepare_user_session_manager(&user, &mut p2p_state).await;

    p2p_state.insert(user.id.clone(), session_manager.to_owned());


    let session_cookie = encrypt_user_cookie(user, state.config.env.HASHING_KEY.as_bytes());
    headers.insert(SET_COOKIE, format!("session={}; Max-Age=2592000; Path=/; SameSite=None", session_cookie).parse().unwrap());


    (headers, axum::Json(json!({"message": "login successful", "token": session_cookie})))
}

#[derive(serde::Deserialize)]
pub struct TokenParams {
    token: String
}



pub async fn prepare_user_session_manager(user: &UserDTO, p2p_state: &mut futures::lock::MutexGuard<'_, HashMap<std::string::String, Arc<futures::lock::Mutex<SessionManager>>>>) -> Arc<Mutex<SessionManager>> {
    // get friends from some source
    // Currently only getting other active users, because 'friends' is not implemented yet
    let friends_in_p2p_state = p2p_state; // This has to be exchanged with an iteration and filtering only the p2p_connections that are the friends




    // TODO: Iterate through friends
    // Check if friend is in p2p_state
    // If yes, Append self as FriendSessionManager to friends 'friends'
    // If not, friend is offline and not connected
    // Currently not need because im getting friends directrly from p2p_state

    let self_session_manager = Arc::new(Mutex::new(SessionManager::new(user.clone())));


    for (friend_id, friend_session_manager) in friends_in_p2p_state.iter() {
        if friend_id == &user.id {
            info!("Found same id {} {}", friend_id, user.name);
            continue
        };

        let friend_session = friend_session_manager.lock().await;
        let friend_socket = friend_session.user_socket.clone();
        friend_session.friends.lock().await.insert(user.id.clone(), FriendSessionManager { socket: self_session_manager.lock().await.user_socket.clone() });
        let self_session_manager = self_session_manager.lock().await;
        let mut self_friends = self_session_manager.friends.lock().await;
        self_friends.insert(friend_id.clone(), FriendSessionManager { socket: friend_socket  });
        drop(self_friends);
        drop(self_session_manager);
    }





    let self_session_manager_locked = self_session_manager.lock().await;
    
    info!("{} has {:?} online friends", user.name, self_session_manager_locked.friends.lock().await.len());

    self_session_manager.to_owned()

}

pub async fn token(State(app_state): State<Arc<AppState>>, headers: HeaderMap) -> (StatusCode, String) {

    let auth_header = match headers.get("authorization") {
        None => return (StatusCode::UNAUTHORIZED, String::from("No auth header provided")),
        Some(header) => {
            let owned = header.to_owned();
            let bearer_string = owned.to_str().unwrap().to_owned();
            bearer_string.replace("Bearer ", "")
        }
    };

    let is_valid = validate_user_token(auth_header.clone(), app_state.config.env.HASHING_KEY.as_bytes());
    
    match is_valid {
        Err(err) => {return (StatusCode::INTERNAL_SERVER_ERROR, err)},
        Ok(_) => {}
    }

    let token = token_into_typed(auth_header.clone(), app_state.config.env.HASHING_KEY.as_bytes()).unwrap();

    let mut pool = app_state.db_pool.get().expect("Could not get db pool");
    let user: UserDTO = users.select(all_columns).first(&mut pool).expect("Could not get user");

    let mut p2p_state = app_state.p2p_connections.lock().await;

    let session_manager = prepare_user_session_manager(&user, &mut p2p_state).await;

    p2p_state.insert(token.sub, session_manager);

    info!("amount of active p2p {}", p2p_state.len());
    info!("p2p {:?}", p2p_state);


    (StatusCode::OK, axum::Json::from(json!({"token": auth_header})).to_string())
}