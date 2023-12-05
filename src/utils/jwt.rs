use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::{collections::BTreeMap, time::{SystemTime, UNIX_EPOCH, Duration}, ops::Add};
use serde::{Deserialize, Serialize};

use crate::models::UserDTO;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub sub: String,
    pub name: String,
    pub public_key: String,
    pub exp: Duration
}

pub fn get_time_since_epoch() -> Duration {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch
}

pub fn encrypt_user_token(user: UserDTO, secret_key: &[u8]) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let mut claims: BTreeMap<&str, String> = BTreeMap::new();
    let since_the_epoch = get_time_since_epoch();
    let jwt_valid_for = Duration::new(60 * 60, 0);

    let jwt_expires = since_the_epoch.add(jwt_valid_for);
    claims.insert("sub", user.username.to_string());
    claims.insert("name", user.name);
    claims.insert("exp", jwt_expires.as_secs_f32().to_string());
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

    let token = match token_into_typed(&token, secret_key) {
        Err(err) => return Err(err),
        Ok(t) => t
    };

    match check_token_expiration(token) {
        Err(err) => return Err(err),
        Ok(_) => {}
    }

    return Ok(true)
}

pub fn check_token_expiration(token: Token) -> Result<(), String> {
    let time_since_epoch = get_time_since_epoch();
    if time_since_epoch.gt(&token.exp) {
        return Err(String::from("Token is expired"));
    };
    return Ok(())
}

pub fn token_into_typed(token: &String, secret_key: &[u8]) -> Result<Token, String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let claims_wrapped: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);


    let claims = match claims_wrapped {
        Err(_) => return Err("Error validating user token".to_owned()),
        Ok(res) => res,
    };

    let uuid = claims.get("sub").unwrap();
    let exp = match claims.get("exp").unwrap().to_owned().parse::<u64>() {
        Err(err) => return Err(format!("Could not parse exp claim - {}", err)),
        Ok(res) => res
    };
    return Ok(Token {
        sub: uuid.to_owned(),
        name: claims.get("name").unwrap().to_owned(),
        public_key: claims.get("public_key").unwrap().to_owned(),
        exp: Duration::new(exp, 0)
    })
}

pub fn hash_string(string: &str, secret_key: &[u8]) -> String {
    let mut key = Hmac::<Sha256>::new_from_slice(secret_key).expect("HMAC can take key of any size");
    key.update(string.as_bytes());
    let result = key.finalize().into_bytes();
    hex::encode(result)
}