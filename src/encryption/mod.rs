use chacha20poly1305::aead::{OsRng, rand_core::RngCore};

pub const NONCE_LENGTH: usize = 19;
pub const SALT_LENGTH: usize = 32;

pub fn make_nonce() -> [u8; NONCE_LENGTH] {
    let mut nonce = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn make_new_key_from_password(password: &str) -> (Vec<u8>, [u8; SALT_LENGTH]) {
    let argon2_config = argon2::Config::default();
    let mut salt = [0u8; SALT_LENGTH];
    OsRng.fill_bytes(&mut salt);
    let key = argon2::hash_raw(password.as_bytes(), &salt, &argon2_config).unwrap();
    (key, salt)
}

pub fn make_key_from_password_and_salt(password: &str, salt: Vec<u8>) -> Vec<u8> {
    let argon2_config = argon2::Config::default();
    let key = argon2::hash_raw(password.as_bytes(), &salt, &argon2_config).unwrap();
    key
}