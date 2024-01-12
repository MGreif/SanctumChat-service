use diesel::{r2d2::{PooledConnection, ConnectionManager, self}, PgConnection, sql_types::Text};
use diesel::prelude::*;

use crate::{helper::sql::Count, schema, models::UserDTO};
pub struct UserRepository {
    pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl UserRepository {
    pub fn new(pg_pool: PooledConnection<ConnectionManager<PgConnection>>) -> UserRepository {
        UserRepository { pg_pool }
    }

    pub fn check_if_user_already_exists(&mut self, username: &String) -> Result<bool, String> {
        let count = diesel::sql_query("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind::<Text, _>(username)
            .load::<Count>(&mut self.pg_pool);

        let count = match count {
            Ok(result) => result,
            Err(err) => return Err(format!("Could not get user count {}", err))
        };

        let count = match count.first() {
            None => return Ok(false),
            Some(count) => count
        };

        let user_exists = count.count > 0;
        return Ok(user_exists)

    }

    pub fn save_user(&mut self, user: &UserDTO) -> Result<(), String> {

        let values = vec![user];
        let result = diesel::insert_into(schema::users::table).values(values).execute(&mut self.pg_pool);


        match result {
            Err(err) => Err(format!("Could not save user {:?}", err)),
            Ok(_) => Ok(())
        }
    }

}