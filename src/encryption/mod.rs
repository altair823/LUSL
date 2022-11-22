use chacha20poly1305::aead::{OsRng, rand_core::RngCore};

pub fn make_nonce() -> [u8; 19] {
    let mut nonce = [0u8; 19];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn make_key_from_password(password: &str) -> (Vec<u8>, [u8; 32]) {
    let argon2_config = argon2::Config::default();
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    let key = argon2::hash_raw(password.as_bytes(), &salt, &argon2_config).unwrap();
    (key, salt)
}