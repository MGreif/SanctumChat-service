use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    collections::BTreeMap,
    ops::Add,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::models::UserDTO;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token {
    pub sub: String,
    pub public_key: String,
    pub exp: Duration,
}

pub fn get_time_since_epoch() -> Duration {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch
}

pub fn generate_token_expiration(token_duration: Duration) -> (Duration, String) {
    let since_the_epoch = get_time_since_epoch();

    let jwt_expires = since_the_epoch.add(token_duration);

    (jwt_expires, jwt_expires.as_secs_f32().to_string())
}

pub fn create_user_token(user: UserDTO, secret_key: &[u8], expires: Duration) -> (Token, String) {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let mut claims: BTreeMap<&str, String> = BTreeMap::new();

    let public_key_base64 = String::from_utf8(user.public_key).expect("Could not parse public_key"); // Converting to string

    let token = Token {
        exp: expires,
        public_key: public_key_base64,
        sub: user.username.to_string(),
    };

    claims.insert("sub", token.sub.clone());
    claims.insert("exp", token.exp.as_nanos().to_string().clone());
    claims.insert("public_key", token.public_key.clone());

    let token_str = claims.clone().sign_with_key(&key).unwrap();

    (token, token_str)
}

pub fn validate_user_token(token: String, secret_key: &[u8]) -> Result<bool, String> {

    // the function token_into_typed validates the token
    let token = match token_into_typed(&token, secret_key) {
        Err(err) => return Err(err),
        Ok(t) => t,
    };

    match check_token_expiration(token) {
        Err(err) => return Err(err),
        Ok(_) => {}
    }
    return Ok(true);
}

pub fn check_token_expiration(token: Token) -> Result<(), String> {
    let time_since_epoch = get_time_since_epoch();
    if time_since_epoch.as_secs_f32().gt(&token.exp.as_secs_f32()) {
        return Err(String::from("Token is expired"));
    };
    return Ok(());
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
        Ok(res) => res,
    };

    return Ok(Token {
        sub: uuid.to_owned(),
        public_key: claims.get("public_key").unwrap().to_owned(),
        exp: Duration::from_nanos(exp),
    });
}

pub fn hash_string(string: &str, secret_key: &[u8]) -> String {
    let mut key =
        Hmac::<Sha256>::new_from_slice(secret_key).expect("HMAC can take key of any size");
    key.update(string.as_bytes());
    let result = key.finalize().into_bytes();
    hex::encode(result)
}
