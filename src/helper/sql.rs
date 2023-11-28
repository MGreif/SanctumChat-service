use diesel::pg::Pg;
use crate::models::UserDTO;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use uuid::Uuid;

pub async fn get_friends_for_user_from_db(pool: &mut PooledConnection<ConnectionManager<PgConnection>>, client_uuid: String) -> Vec<UserDTO> {
    let query = diesel::sql_query("SELECT users.username as username, users.password, users.name, users.age FROM friends as f LEFT JOIN users ON f.befriended_user_id = users.username WHERE f.user_id = $1")
        .bind::<diesel::sql_types::Text, _>(client_uuid);
    println!("{}", diesel::debug_query::<Pg,_>(&query).to_string());

    let friends_from_db: Vec<UserDTO> = query.load(pool).expect("[get_friends] could not get friends");
    friends_from_db
}