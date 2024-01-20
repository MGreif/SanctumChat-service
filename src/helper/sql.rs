use crate::models::{UserDTO, UserDTOSanitized};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

pub async fn get_friends_for_user_from_db(pool: &mut PooledConnection<ConnectionManager<PgConnection>>, client_uuid: &String) -> Vec<UserDTOSanitized> {
    let query = diesel::sql_query("SELECT users.username as username, users.password, users.public_key FROM friends as f LEFT JOIN users ON f.befriended_user_id = users.username WHERE f.user_id = $1")
        .bind::<diesel::sql_types::Text, _>(client_uuid);

    let friends_from_db: Vec<UserDTO> = query.load(pool).expect("[get_friends] could not get friends");
    
    let mut friends_sanitized: Vec<UserDTOSanitized> = Vec::new();

    for friend in friends_from_db {
        match friend.sanitize_and_serialize() {
            Ok(friend) => friends_sanitized.push(friend),
            Err(_) => {}
        }
        
    }

    friends_sanitized
}


use diesel::sql_types::BigInt;

#[derive(QueryableByName)]
pub struct Count {
    #[sql_type = "BigInt"]
    pub count: i64,
}
