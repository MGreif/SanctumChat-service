use std::time::Duration;

use axum::http::StatusCode;
use crate::{repositories::user_repository::UserRepositoryInterface, models::UserDTO, helper::{errors::HTTPResponse, jwt::{hash_string, Token, create_user_token, token_into_typed, generate_token_expiration}}};

pub struct UserDomain<I: UserRepositoryInterface> {
    user_repository: I
}

impl<I: UserRepositoryInterface> UserDomain<I> {

    pub fn new(user_repository: I) -> Self {
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

        if user_exists {
            return Err(HTTPResponse { message: Some(String::from("User with this username already exists")), status: StatusCode::BAD_REQUEST, data: None })
        }


        let encrypted_password = hash_string(&user.password, hashing_key);
        user.password = encrypted_password;


        

        match self.user_repository.save_user(&user) {
            Ok(_) => Ok(user),
            Err(err) => Err(HTTPResponse::new_internal_error(err))
        }
    }

    pub fn login_user_and_prepare_token(&mut self, usern: &String, passw: &String, hashing_key: &[u8]) -> Result<(UserDTO, Token, String), HTTPResponse<Token>> {
        
        let user = self.user_repository.get_user_by_username_and_password(usern, passw);
        let user = match user {
            Ok(user) => user,
            Err(_) => return Err(HTTPResponse {
                status: StatusCode::UNAUTHORIZED,
                data: None,
                message: Some(String::from("Username or password wrong"))
            })
        };

        let (valid_for, _) = generate_token_expiration(Duration::new(15*60, 0));
        
        let (token, token_str) = create_user_token(user.clone(), hashing_key, valid_for);

        Ok((user, token, token_str))
    }

    pub fn renew_token(&mut self, usern: &String, hashing_key: &[u8]) -> Result<(UserDTO, Token, String), String> {
        let user = self.user_repository.get_user_by_username(usern);
        let user = match user {
            Err(err) => return Err(format!("Could not find user {}: {}", usern, err)),
            Ok(user) => user
        };

        let (valid_for, _) = generate_token_expiration(Duration::new(15*60, 0));

        let (token, token_str) = create_user_token(user.clone(), hashing_key, valid_for);

        return Ok((user, token, token_str ))
    }
}