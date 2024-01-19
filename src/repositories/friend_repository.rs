use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection};
use diesel::prelude::*;
use diesel::query_dsl::*;
use crate::models::UserDTOSanitized;


use crate::models::UserDTO;

pub trait FriendRepositoryInterface {
    fn get_friends(&mut self, username: &String) -> Result<Vec<UserDTOSanitized>, String>;
    fn get_friend(&mut self, username: &String, friend_name: &String) -> Result<Option<UserDTOSanitized>, String>;
}

pub struct FriendRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl FriendRepositoryInterface for FriendRepository {
    fn get_friends(&mut self, username: &String) -> Result<Vec<UserDTOSanitized>, String> {
        let query = diesel::sql_query("SELECT users.username as username, users.password, users.public_key FROM friends as f LEFT JOIN users ON f.befriended_user_id = users.username WHERE f.user_id = $1")
        .bind::<diesel::sql_types::Text, _>(username);

    let friends_from_db: Vec<UserDTO> = query.load(&mut self.pg_pool).expect("[get_friends] could not get friends");
    
    let mut friends_sanitized: Vec<UserDTOSanitized> = Vec::new();

    for friend in friends_from_db {
        match friend.sanitize_and_serialize() {
            Ok(friend) => friends_sanitized.push(friend),
            Err(_) => {}
        }
    }

    Ok(friends_sanitized)
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