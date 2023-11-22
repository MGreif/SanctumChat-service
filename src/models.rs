use std::time::SystemTime;

use diesel::{Queryable, backend::Backend, deserialize::FromSql, pg::Pg, associations::{Identifiable, Associations}, alias};
use uuid::Uuid;

use crate::schema::{friends, self};

alias!(schema::users as users_alias: UserAliasDTO);

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone, Identifiable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDTO {
    pub id: Uuid,
    pub name: String,
    pub age: i32,
    pub password: String
}

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: Uuid,
    pub sender: Uuid,
    pub recipient: Uuid,
    pub sent_at: SystemTime,
    pub content: String,
}

diesel::joinable!(crate::schema::friends -> crate::schema::users (id));
#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone, Identifiable, Associations)]
#[diesel(table_name = crate::schema::friends)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(UserDTO, foreign_key = user_id))]
#[diesel(belongs_to(UserAliasDTO, foreign_key = befriended_user_id))]
#[diesel(primary_key(user_id, befriended_user_id))]
pub struct Friend {
    pub id: Uuid,
    pub user_id: Uuid,
    pub befriended_user_id: Uuid,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone)]
#[diesel(table_name = crate::schema::friend_requests)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FriendRequest {
    pub id: Uuid,
    pub sender: Uuid,
    pub recipient: Uuid,
    pub accepted: Option<bool>
}
