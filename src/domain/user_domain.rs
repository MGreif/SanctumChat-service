use axum::http::StatusCode;

use crate::{repositories::user_repository::UserRepository, models::UserDTO, helper::{errors::HTTPResponse, jwt::hash_string}};

pub struct UserDomain {
    user_repository: UserRepository
}

impl UserDomain {

    pub fn new(user_repository: UserRepository) -> Self {
        return UserDomain {
            user_repository
        }
    }

    pub fn create_user(&mut self, user: &UserDTO, hashing_key: &[u8]) -> Result<UserDTO, HTTPResponse<Vec<u8>>> {
        let mut user = user.clone();
        
        let user_exists = self.user_repository.check_if_user_already_exists(&user.username);
        let user_exists = match user_exists {
            Err(err) => return Err(HTTPResponse::new_internal_error(err)),
            Ok(exists) => exists
        };

        if !user_exists {
            return Err(HTTPResponse { message: Some(String::from("User with this username already exists")), status: StatusCode::BAD_REQUEST, data: None })
        }


        let encrypted_password = hash_string(&user.password, hashing_key);
        user.password = encrypted_password;


        

        match self.user_repository.save_user(&user) {
            Ok(_) => Ok(user),
            Err(err) => Err(HTTPResponse::new_internal_error(err))
        }
    }
}