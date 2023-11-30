use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use uuid::{Uuid, uuid};
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

use crate::models::UserDTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub sub: String,
    pub name: String,
    pub public_key: String
}

pub fn encrypt_user_token(user: UserDTO, secret_key: &[u8]) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let mut claims: BTreeMap<&str, String> = BTreeMap::new();
    claims.insert("sub", user.username.to_string());
    claims.insert("name", user.name);
    let public_key_base64 = String::from_utf8(user.public_key).expect("Could not parse public_key"); // Converting to string
    claims.insert("public_key", public_key_base64);
    let token_str = claims.sign_with_key(&key).unwrap();
    token_str
}

pub fn validate_user_token(token: String, secret_key: &[u8]) -> Result<bool, String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let claims_wrapped: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);


    match claims_wrapped {
        Err(_) => return Err("Error validating user token".to_owned()),
        Ok(res) => res,
    };

    return Ok(true)
}

pub fn token_into_typed(token: String, secret_key: &[u8]) -> Result<Token, String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let claims_wrapped: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);


    let claims = match claims_wrapped {
        Err(err) => return Err("Error validating user token".to_owned()),
        Ok(res) => res,
    };

    let uuid = claims.get("sub").unwrap();
    return Ok(Token { sub: uuid.to_owned(), name: claims.get("name").unwrap().to_owned(), public_key: claims.get("public_key").unwrap().to_owned() })
}

pub fn hash_string(string: &str, secret_key: &[u8]) -> String {
    let mut key = Hmac::<Sha256>::new_from_slice(secret_key).expect("HMAC can take key of any size");
    key.update(string.as_bytes());
    let result = key.finalize().into_bytes();
    hex::encode(result)
}