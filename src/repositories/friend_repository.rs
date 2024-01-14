use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection, sql_types::Text};
use diesel::prelude::*;
use diesel::query_dsl::*;
use crate::{schema::users::dsl::*};


use crate::{helper::sql::Count, schema, models::UserDTO};

pub trait FriendRepositoryInterface {
    
}

pub struct FriendRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl FriendRepositoryInterface for FriendRepository {


}