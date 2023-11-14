use hmac::{Hmac, Mac};
use jwt::{SignWithKey, VerifyWithKey};
use sha2::Sha256;
use std::collections::BTreeMap;

use crate::models::UserDTO;

pub fn encrypt_user_cookie(user: UserDTO, secret_key: &[u8]) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", user.id);
    let token_str = claims.sign_with_key(&key).unwrap();
    token_str
}

pub fn validate_user_cookie(token: String, secret_key: &[u8]) -> Result<bool, String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret_key).unwrap();
    let claims_wrapped: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);


    let claims = match claims_wrapped {
        Err(err) => return Err("Error validating user cookie".to_owned()),
        Ok(res) => res,
    };

    return Ok(true)

}

pub fn hash_string(string: &str, secret_key: &[u8]) -> String {
    let mut key = Hmac::<Sha256>::new_from_slice(secret_key).expect("HMAC can take key of any size");
    key.update(string.as_bytes());
    let result = key.finalize().into_bytes();
    hex::encode(result)
}