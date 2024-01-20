use std::time::SystemTime;

use diesel::{alias, QueryableByName, sql_types::Bool};
use uuid::Uuid;
use crate::schema;

alias!(schema::users as users_alias: UserAliasDTO);

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone, QueryableByName, PartialEq)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDTO {
    pub username: String,
    pub password: String,
    pub public_key: Vec<u8>
}

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct UserDTOSanitized {
    pub username: String,
    pub public_key: String
}

impl UserDTO {
    pub fn sanitize_and_serialize(&self) -> Result<UserDTOSanitized,String> {
        let public_key_utf8 = match std::str::from_utf8(&self.public_key) {
            Ok(res) => res.to_string(),
            Err(_) => return Err(String::from("Failed transforming Vec u8 into utf8 string")) 
        };
        return Ok(UserDTOSanitized {
            username: self.username.clone(),
            public_key: public_key_utf8
        })
    } 
}


#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub sent_at: SystemTime,
    pub content: String,
    pub content_self_encrypted: String,
    pub content_signature: String,
    pub content_self_encrypted_signature: String,
    pub is_read: bool,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone)]
#[diesel(table_name = crate::schema::friends)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Friend {
    pub id: Uuid,
    pub user_id: String,
    pub befriended_user_id: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone, QueryableByName)]
#[diesel(table_name = crate::schema::friend_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FriendRequest {
    pub id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub accepted: Option<bool>
}


