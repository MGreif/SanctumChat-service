use std::time::SystemTime;

use diesel::{Queryable, backend::Backend, deserialize::FromSql, pg::Pg, associations::{Identifiable, Associations}, alias, QueryableByName};
use uuid::Uuid;

use crate::schema::{friends, self};

alias!(schema::users as users_alias: UserAliasDTO);

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone, QueryableByName)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDTO {
    pub username: String,
    pub name: String,
    pub age: i32,
    pub password: String
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


