use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection, deserialize::FromSqlRow, pg::Pg, row::Field};
use diesel::prelude::*;
use diesel::query_dsl::*;
use serde::Serialize;
use crate::models::UserDTOSanitized;
use diesel::sql_types::{Numeric};

use crate::models::UserDTO;

#[derive(Serialize, Debug, PartialEq, QueryableByName)]
pub struct FriendDTO {
    #[diesel(sql_type=diesel::sql_types::Text)]
    pub username: String,
    #[diesel(sql_type=diesel::sql_types::Text)]
    pub public_key: String,
    #[diesel(sql_type=diesel::sql_types::BigInt)]
    pub unread_message_count:i64,
}


pub trait FriendRepositoryInterface {
    fn get_friends(&mut self, username: &String) -> Result<Vec<FriendDTO>, String>;
    fn get_friend(&mut self, username: &String, friend_name: &String) -> Result<Option<UserDTOSanitized>, String>;
}

pub struct FriendRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl FriendRepositoryInterface for FriendRepository {
    fn get_friends(&mut self, username: &String) -> Result<Vec<FriendDTO>, String> {
        let query = diesel::sql_query(
            "SELECT
                users.username as username,
                users.public_key,
                COUNT(CASE WHEN messages.is_read = 'f' THEN 1 ELSE NULL END) AS unread_message_count
            FROM friends as f
            LEFT JOIN users
            ON f.befriended_user_id = users.username
            LEFT JOIN messages
            ON f.befriended_user_id = messages.sender AND messages.recipient = $1
            WHERE f.user_id = $1
            GROUP BY users.username"
        )
        .bind::<diesel::sql_types::Text, _>(username);

    let friends_from_db: Vec<FriendDTO> = query.load(&mut self.pg_pool).expect("[get_friends] could not get friends");
    

    Ok(friends_from_db)
    }

    fn get_friend(&mut self, username: &String, friend_name: &String) -> Result<Option<UserDTOSanitized>, String> {
        let query = diesel::sql_query("SELECT users.username as username, users.password, users.public_key FROM friends as f LEFT JOIN users ON f.befriended_user_id = users.username WHERE f.user_id = $1 AND f.befriended_user_id = $2")
        .bind::<diesel::sql_types::Text, _>(username)
        .bind::<diesel::sql_types::Text, _>(friend_name);

    let friends_from_db: Vec<UserDTO> = query.load(&mut self.pg_pool).expect("[get_friends] could not get friends");
    
    let mut friends_sanitized: Vec<UserDTOSanitized> = Vec::new();

    for friend in friends_from_db {
        match friend.sanitize_and_serialize() {
            Ok(friend) => friends_sanitized.push(friend),
            Err(_) => return Err(String::from("Could not sanitize user"))
        }
    }

    match friends_sanitized.pop() {
        Some(f) => Ok(Some(f)),
        None => Ok(None)
    }
    }
}