use std::sync::Arc;
use axum::{extract::Json, response::IntoResponse};
use rust_websocket_server::schema::users::dsl::*;
use serde_json::json;
use crate::{config::AppState, models::{UserDTO}};
use rust_websocket_server::*;
use diesel::prelude::*;
use rand::{thread_rng, Rng, distributions::Alphanumeric};

fn generate_random_string(length: usize) -> String {
    let mut rng = thread_rng();

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
    pub age: i32
}

pub async fn get_users(state: axum::extract::Extension<Arc<AppState>>) -> String {
    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let names: Vec<UserDTO> = users.select(schema::users::all_columns).load(&mut db_conn).expect("could not select users");
    format!("{}", serde_json::to_string(&names).unwrap())
}

pub async fn create_user(state: axum::extract::Extension<Arc<AppState>>, Json(body): Json<UserCreateDTO>) -> impl IntoResponse {
    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let newUser = models::UserDTO { 
        id: generate_random_string(10),
        age: body.age,
        name: body.name
    };

    let values = vec![newUser];
    diesel::insert_into(schema::users::table).values(&values).execute(&mut db_conn).expect("Could not insert data");

    axum::Json(json!({"message": "User created successfully"}))
}