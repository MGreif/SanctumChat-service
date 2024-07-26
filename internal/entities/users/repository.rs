use diesel::{r2d2::{PooledConnection, ConnectionManager}, PgConnection, sql_types::Text};
use diesel::prelude::*;
use diesel::query_dsl::*;
use crate::{schema::users::dsl::*};


use crate::{helper::sql::Count, schema, models::UserDTO};

pub trait UserRepositoryInterface {
    fn check_if_user_already_exists(&mut self, usern: &String) -> Result<bool, String>;
    fn get_user_by_username(&mut self, usern: &String) -> Result<UserDTO, String>;
    fn save_user(&mut self, user: &UserDTO) -> Result<(), String>;
    fn get_user_by_username_and_password(&mut self, usern: &String, passw: &String) -> Result<UserDTO, String>;
}

pub struct UserRepository {
    pub pg_pool: PooledConnection<ConnectionManager<PgConnection>>
}

impl UserRepositoryInterface for UserRepository {

    fn check_if_user_already_exists(&mut self, usern: &String) -> Result<bool, String> {
        let count = diesel::sql_query("SELECT COUNT(*) FROM users WHERE username = $1")
            .bind::<Text, _>(usern)
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

    fn get_user_by_username(&mut self, usern: &String) -> Result<UserDTO, String> {
        let user: UserDTO = match users.select(users::all_columns()).filter(username.eq(usern)).first(&mut self.pg_pool) {
            Ok(user) => user,
            Err(_) => return Err(String::from("Could not get user"))
        };

        return Ok(user)
    }

    fn save_user(&mut self, user: &UserDTO) -> Result<(), String> {

        let values = vec![user];
        let result = diesel::insert_into(schema::users::table).values(values).execute(&mut self.pg_pool);


        match result {
            Err(err) => Err(format!("Could not save user {:?}", err)),
            Ok(_) => Ok(())
        }
    }

    fn get_user_by_username_and_password(&mut self, usern: &String, passw: &String) -> Result<UserDTO, String> {
        let user: Result<UserDTO, _> = schema::users::table.select(schema::users::all_columns)
        .filter(username
            .eq(usern)
            .and(password.eq(passw)))
        .first::<UserDTO>(&mut self.pg_pool);

        let user = match user {
            Err(err) => return Err(format!("Could not get user, {}", err)),
            Ok(result) => result
        };

        Ok(user)
    }

}