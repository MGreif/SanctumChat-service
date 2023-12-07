use std::sync::Arc;
use axum::{extract::{Json, Query, State}, response::IntoResponse, http::{HeaderMap, header::{SET_COOKIE, self}, StatusCode}, Extension};
use futures::lock::Mutex;
use tracing::info;
use crate::{schema::users::dsl::*, utils::jwt::Token, helper::session::SessionManager};
use serde_json::json;
use crate::{config::AppState, models::{UserDTO, self}, schema::{self, users::{self, all_columns}}, validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR, utils::jwt::{hash_string, token_into_typed}};
use diesel::prelude::*;
use diesel::sql_types::Text;
use crate::helper::errors::HTTPResponse;
use crate::helper::sql::Count;
use crate::utils::jwt::encrypt_user_token;
use openssl::rsa::Rsa;
use base64;
use base64::Engine;

#[derive(serde::Deserialize)]
pub struct UserCreateDTO {
    pub name: String,
    pub age: i32,
    pub username: String,
    password: String,
    pub public_key: String,
    pub generate_key: bool
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
    let headers = HeaderMap::new();

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&body.username) {
        Err(err) => {
            return (headers, HTTPResponse::<Vec<u8>> {
                message: Some(format!("Username validation failed: {}", err)),
                data: None,
                status: StatusCode::BAD_REQUEST
            })
        },
        Ok(_) => {}
    }
    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&body.name) {
        Err(err) => {
            return (headers, HTTPResponse::<Vec<u8>> {
                message: Some(format!("Password validation failed: {}", err)),
                data: None,
                status: StatusCode::BAD_REQUEST
            })
        },
        Ok(_) => {}
    }
    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&body.password) {
        Err(err) => {
            return (headers, HTTPResponse::<Vec<u8>> {
                message: Some(format!("Password validation failed: {}", err)),
                data: None,
                status: StatusCode::BAD_REQUEST
            })
        },
        Ok(_) => {}
    }

    let mut private_key: Option<Vec<u8>> = None;
    let mut pub_key: Vec<u8> =  body.public_key.as_bytes().to_vec();
    if body.generate_key == true {
        let rsa_key = Rsa::generate(2048).unwrap();
        let rsa_private_key = rsa_key.private_key_to_pem().unwrap();
        let rsa_public_key = rsa_key.public_key_to_pem().unwrap();
        private_key = Some(rsa_private_key);
        let output = base64::engine::general_purpose::STANDARD.encode(rsa_public_key.as_slice());
        pub_key = output.as_bytes().to_vec();
    };

    let mut new_user = models::UserDTO {
        username: body.username,
        age: body.age,
        name: body.name,
        password: body.password,
        public_key: pub_key
    };

    let mut query = diesel::sql_query("SELECT COUNT(*) FROM users WHERE username = $1").bind::<Text, _>(&new_user.username).load::<Count>(&mut db_conn).expect("Could not get user count");
    let count = match query.pop() {
        None => 0,
        Some(t) => t.count
    };

    match count {
        0 => {},
        _ => return (headers, HTTPResponse::<Vec<u8>> {
                    message: Some(format!("Username is already in use")),
                    data: None,
                    status: StatusCode::BAD_REQUEST
                })
        }


    let encrypted_password = hash_string(&new_user.password, state.config.env.HASHING_KEY.clone().as_bytes());
    new_user.password = encrypted_password;
    info!("{:?}", new_user);



    let values = vec![new_user];
    diesel::insert_into(schema::users::table).values(&values).execute(&mut db_conn).expect("Could not insert data");



    (headers, HTTPResponse::<Vec<u8>> {
        message: Some(format!("User created successfully")),
        data: private_key,
        status: StatusCode::CREATED
    })
}

#[derive(serde::Deserialize)]
pub struct LoginDTO {
    pub username: String,
    pub password: String
}

pub async fn logout<'a>(State(state): State<Arc<AppState>>, token: Extension<Token>) -> impl IntoResponse {

    let session_manager = match state.remove_from_p2p(&token.sub).await {
        Ok(sm) => sm,
        Err(err) => return HTTPResponse::<()> {
            message: Some(err),
            data: None,
            status: StatusCode::INTERNAL_SERVER_ERROR
        },
    };

    session_manager.lock().await.notify_offline().await;

    HTTPResponse::<()> {
        message: Some(String::from("Successfully logged out")),
        data: None,
        status: StatusCode::OK
    }
}

pub async fn login<'a>(State(state): State<Arc<AppState>>, Json(body): Json<LoginDTO>) -> impl IntoResponse {
    let LoginDTO { password: pw, username: username_id } = body;
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());


    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&username_id) {
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
    let user_result: Result<(String, String, i32, String, Vec<u8> ), _> = users
        .select(users::all_columns)
        .filter(username
            .eq(&username_id)
            .and(password.eq(hash_string(&pw, state.config.env.HASHING_KEY.clone().as_bytes()))))
        .first::<(String, String, i32, String, Vec<u8> )>(&mut pool);

    let user = match user_result {
        Err(_) => return (headers, axum::Json(json!({"message": "login failed, wrong username or password"}))),
        Ok(result_id) => UserDTO { username: result_id.0, name: result_id.1, age: result_id.2, password: result_id.3, public_key: result_id.4 }
    };

    let session_token = encrypt_user_token(user.clone(), state.config.env.HASHING_KEY.as_bytes());
    let token = token_into_typed(&session_token, state.config.env.HASHING_KEY.as_bytes()).expect("Could not parse token");

    let session_manager = SessionManager::new(user.clone(), token, state.clone());
    session_manager.notify_online().await;
    state.insert_into_p2p(session_manager).await;


    headers.insert(SET_COOKIE, format!("session={}; Max-Age=2592000; Path=/; SameSite=None", session_token).parse().unwrap());


    (headers, axum::Json(json!({"message": "login successful", "token": session_token})))
}

pub async fn token<'a>(State(app_state): State<Arc<AppState>>, Extension(token): Extension<Token>, headers: HeaderMap) -> (StatusCode, String) {
    let mut pool = app_state.db_pool.get().expect("Could not get db pool");
    let user: UserDTO = match users.select(all_columns).filter(username.eq(&token.sub)).first(&mut pool) {
        Ok(user) => user,
        Err(_) => return (StatusCode::FORBIDDEN, axum::Json::from(json!({"message": "Could not get user"})).to_string())
    };

    let session_manager = SessionManager::new(user.clone(), token, app_state.clone());
    app_state.insert_into_p2p(session_manager).await;
    

    info!("token 5");

    let token = headers.get("authorization").unwrap().to_owned();
    let token = token.to_str().unwrap();
    let token = token.replace("Bearer ", "");

    (StatusCode::OK, axum::Json::from(json!({"token": token})).to_string())
}