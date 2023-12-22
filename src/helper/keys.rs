use openssl::rsa::Rsa;

pub fn validate_private_key(private_key_string: String) -> Result<(), ()> {
    let private_key_regex = regex::Regex::new(r"^-----BEGIN PRIVATE KEY-----[A-z]+-----END PRIVATE KEY-----$").unwrap();

    match private_key_regex.captures(&private_key_string) {
        None => Err(()),
        Some(_) => Ok(()) 
    }
}

pub fn validate_public_key(private_key_string: String) -> Result<(), ()> {
    let private_key_regex = regex::Regex::new(r"^-----BEGIN PUBLIC KEY-----[A-z]+-----END PUBLIC KEY-----$").unwrap();

    match private_key_regex.captures(&private_key_string) {
        None => Err(()),
        Some(_) => Ok(()) 
    }
}

pub fn generate_rsa_key_pair() -> Result<(Vec<u8>, Vec<u8>), String> {
    let rsa_key = match Rsa::generate(2048) {
        Ok(key) => key,
        Err(err) => return Err(err.to_string())
    };
    let rsa_private_key = match rsa_key.private_key_to_pem() {
        Ok(key) => key,
        Err(err) => return Err(err.to_string())
    };
    let rsa_public_key = match rsa_key.public_key_to_pem() {
        Ok(key) => key,
        Err(err) => return Err(err.to_string())
    };

    return Ok((rsa_private_key, rsa_public_key))
}