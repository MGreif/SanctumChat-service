use std::time::SystemTime;

use diesel::{Queryable, backend::Backend, deserialize::FromSql, pg::Pg};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable, diesel::Insertable, Clone)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserDTO {
    pub name: String,
    pub age: i32,
    pub id: Uuid,
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