use crate::{repositories::user_repository::UserRepositoryInterface, models::UserDTO};

pub struct UserRepositoryMock {}

impl UserRepositoryInterface for UserRepositoryMock {
    fn check_if_user_already_exists(&mut self, usern: &String) -> Result<bool, String> {
        if usern == "exists" {
            return Ok(true)
        } else if usern == "error" {
            return Err(String::from("Failed lol"))
        }
        return Ok(false)
    }

    fn get_user_by_username_and_password(&mut self, usern: &String, passw: &String) -> Result<UserDTO, String> {
        return Ok(UserDTO { username: usern.to_owned(), password: passw.to_owned(), public_key: vec![] })
    }
    fn save_user(&mut self, _: &UserDTO) -> Result<(), String> {
        return Ok(())
    }

    fn get_user_by_username(&mut self, usern: &String) -> Result<UserDTO, String> {
        return Ok(UserDTO {
            username: usern.to_owned(),
            password: String::from("TestPassword"),
            public_key: vec![]
        })
    }
}



#[cfg(test)]
mod integration_tests {
    use std::time::Duration;

    use crate::{domain::{user_domain_test::UserRepositoryMock, user_domain::UserDomain}, models::UserDTO, helper::jwt::{hash_string, create_user_token, generate_token_expiration}};

    #[test]
    fn test_user_creation() {
        let repo = UserRepositoryMock {};
        let mut domain = UserDomain::new(repo);
        let user_input = UserDTO {
            username: String::from("Test1"),
            password: String::from("Password1"),
            public_key: vec![]
        };

        let hashing_key = String::from("abc");
        let hashing_key = hashing_key.as_bytes();

        let x = domain.create_user(&user_input, &hashing_key);
        let hashed_password = hash_string(&user_input.password, &hashing_key);

        let user_expect = UserDTO {
            password: hashed_password,
            public_key: vec![],
            username: String::from("Test1")
        };

        let user_output = match x {
            Ok(usr) => usr,
            Err(_) => return assert!(false)
        };

        assert_eq!(user_expect, user_output)
    }

    #[test]
    fn test_login_user_and_prepare_token() {
        let repo = UserRepositoryMock {};
        let mut domain = UserDomain::new(repo);

        let hashing_key = String::from("abc");
        let hashing_key = hashing_key.as_bytes();

        let username = String::from("TestUser");
        let password = String::from("TestPassword");
        let user_expect = UserDTO {
            password: password.clone(),
            username: username.clone(),
            public_key: vec![]
        };

        let (valid_for, _) = generate_token_expiration(Duration::new(15*60, 0));
        let (token_expect, _) = create_user_token(user_expect.clone(), hashing_key, valid_for);
        let result = domain.login_user_and_prepare_token(&username, &password, hashing_key);

        let (user_output, token, _) = match result {
            Ok(res) => res,
            Err(_) => return assert!(false)
        };

        assert_eq!(user_expect, user_output);
        assert_eq!(token_expect.sub, token.sub);
    }


    #[test]
    fn test_renew_token() {
        let repo = UserRepositoryMock {};
        let mut domain = UserDomain::new(repo);

        let hashing_key = String::from("abc");
        let hashing_key = hashing_key.as_bytes();

        let username = String::from("TestUser");
        let password = String::from("TestPassword");
        let user_expect = UserDTO {
            password: password.clone(),
            username: username.clone(),
            public_key: vec![]
        };

        let (valid_for, _) = generate_token_expiration(Duration::new(15*60, 0));

        let (token_expect, _) = create_user_token(user_expect.clone(), hashing_key, valid_for);
        let result = domain.renew_token(&username, hashing_key);


        let (user_output, token, _) = match result {
            Ok(res) => res,
            Err(_) => return assert!(false)
        };

        assert_eq!(user_expect, user_output);
        assert_eq!(token_expect.sub, token.sub);
    }
}