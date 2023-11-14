use std::sync::Arc;
use axum::{extract::{Json, Query, State}, response::{IntoResponse}, http::{HeaderMap, header::{SET_COOKIE, self}}};
use tracing::info;
use super::super::schema::users::dsl::*;
use serde_json::json;
use crate::{config::AppState, models::{UserDTO, self}, schema::{self, users}, validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR, utils::jwt::hash_string};
use diesel::prelude::*;
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use crate::utils::jwt::encrypt_user_cookie;

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
        Err(err) => return (headers, axum::Json(json!({"message": "login failed, wrong username or password"}))),
        Ok(result_id) => UserDTO {id: result_id.0, age: result_id.1, name: result_id.2, password: result_id.3 } 
    };

    let session_cookie = encrypt_user_cookie(user);
    headers.insert(SET_COOKIE, format!("session={}", session_cookie).parse().unwrap());


    (headers, axum::Json(json!({"message": "login successful"})))
}