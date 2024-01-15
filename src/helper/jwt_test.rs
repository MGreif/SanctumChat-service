use std::time::Duration;

use tracing::info;

use crate::{models::UserDTO, helper::jwt::{Token, generate_token_expiration, token_into_typed}};

use super::jwt::create_user_token;

#[test]
pub fn test_create_user_token() {
    let user = UserDTO {
        password: String::from(""),
        public_key: vec![69, 69],
        username: String::from("User1")
    };
    let secret_key = String::from("abc");
    let secret_key = secret_key.as_bytes();

    let (valid_for, valid_for_claim) = generate_token_expiration(Duration::new(15*60, 0));
    let (token, token_str) = create_user_token(user, secret_key, valid_for);

    let token_expect = Token {
        exp: valid_for,
        public_key: String::from("EE"),
        sub: String::from("User1")
    };

    assert_eq!(token, token_expect);
}

#[test]
pub fn test_token_into_typed() {
    let secret_key = String::from("abc");
    let secret_key = secret_key.as_bytes();
    let user = UserDTO {
        password: String::from(""),
        public_key: vec![69, 69],
        username: String::from("User1")
    };
    let (valid_for, _) = generate_token_expiration(Duration::new(15*60, 0));

    let (token, token_str) = create_user_token(user, secret_key, valid_for);

    let token_decoded = token_into_typed(&token_str, secret_key).expect("Error");

    assert_eq!(token_decoded, token);
}