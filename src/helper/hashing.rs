use pbkdf2::pbkdf2_hmac_array;
use sha2::Sha256;
use hex::encode;
pub fn hash_password(string: &str, secret_key: &[u8]) -> [u8; 20] {
    let rounds = 600_000;
    let hashed = pbkdf2_hmac_array::<Sha256, 20>(string.as_bytes(), secret_key, rounds);
    return hashed;
}

use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

pub fn hash_password_argon2(string: &str) -> Result<PasswordHash, String> {

    let password = string.as_bytes(); 

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    let salt: SaltString = SaltString::generate(&mut OsRng);

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = match argon2.hash_password(password, &salt) {
        Ok(hash) => hash,
        Err(err) => return Err(format!("Could not get password hash: {}", err))
    };



    return Ok(password_hash)
}

pub fn verify_password_argon2(password: String, password_hash: &String) -> Result<bool, String> {
    let password_hash = match PasswordHash::new(password_hash) {
        Ok(result) => result,
        Err(err) => return Err(format!("Could not create password hash from PHC string {}: {}", password_hash.clone(), err))
    };

    return Ok(Argon2::default().verify_password(password.as_bytes(), &password_hash).is_ok())
}