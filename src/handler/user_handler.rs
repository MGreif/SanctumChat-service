use std::sync::Arc;
use axum::{extract::{Json, Query}, response::IntoResponse};
use tracing::info;
use super::super::schema::users::dsl::*;
use serde_json::json;
use crate::{config::AppState, models::{UserDTO, self}, schema, validation::string_validate::DEFAULT_INPUT_FIELD_STRING_VALIDATOR};
use diesel::prelude::*;
use rand::{thread_rng, Rng, distributions::Alphanumeric};


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
    pub age: i32
}

#[derive(serde::Deserialize)]
pub struct GetUserQueryDTO {
    pub name: Option<String>
}

pub async fn get_users(state: axum::extract::Extension<Arc<AppState>>, Query(query_params): Query<GetUserQueryDTO>) -> String {

    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let mut query: _ = users.into_boxed();

    if let Some(query_name) = query_params.name {
        query = query.filter(name.like(format!("%{}%", query_name)));
    }

    let names: Vec<UserDTO> = query.select(schema::users::all_columns).load(&mut db_conn).expect("could not select users");
    format!("{}", serde_json::to_string(&names).unwrap())
}

pub async fn create_user(state: axum::extract::Extension<Arc<AppState>>, Json(body): Json<UserCreateDTO>) -> impl IntoResponse {
    let mut db_conn = state.db_pool.get().expect("could not get database pool");
    let new_user = models::UserDTO { 
        id: generate_random_string(10),
        age: body.age,
        name: body.name
    };

    match DEFAULT_INPUT_FIELD_STRING_VALIDATOR.validate(&new_user.name) {
        Err(err) => {
            info!("{} - Validation error: {}", "name", err);
            return axum::Json(json!({"message": err, "field": "name", }))
        },
        Ok(_) => {}
    }

    let values = vec![new_user];
    diesel::insert_into(schema::users::table).values(&values).execute(&mut db_conn).expect("Could not insert data");

    axum::Json(json!({"message": "User created successfully"}))
}