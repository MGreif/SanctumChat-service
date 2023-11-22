use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use diesel::sql_types::Uuid;

use crate::models::{Friend, UserDTO, FriendRequest, UserAliasDTO};
use crate::schema::users;
use crate::{config::AppState, utils::jwt::Token, schema::friends};
use crate::schema::friends::dsl::*;
use diesel::prelude::*;



pub async fn get_friends(State(app_state): State<Arc<AppState>>, token: Token) -> impl IntoResponse {

    let (u1, u2) = diesel::alias!(crate::schema::users as u1, crate::schema::users as u2);


    let mut pool = app_state.db_pool.get().expect("[get_friends] Could not get connection pool");
    let issuer: UserDTO = users::table.filter(users::id.eq(token.sub)).get_result(&mut pool).unwrap();
    let friend_relations: Vec<(Friend, Option<UserDTO>)> = Friend::belonging_to(&issuer).left_join(users::table).load(&mut pool).expect("Could not get friends");

    return "noice"
}