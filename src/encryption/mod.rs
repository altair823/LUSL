use chacha20poly1305::{
    aead::{rand_core::RngCore, stream, OsRng},
    KeyInit, XChaCha20Poly1305,
};

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

fn make_aead(key: &[u8]) -> XChaCha20Poly1305 {
    XChaCha20Poly1305::new_from_slice(&key).unwrap()
}

pub fn make_encryptor(key: &[u8], nonce: &[u8]) -> stream::EncryptorBE32<XChaCha20Poly1305> {
    let aead = make_aead(key);
    stream::EncryptorBE32::from_aead(aead, nonce.as_ref().into())
}

pub fn make_decryptor(key: &[u8], nonce: &[u8]) -> stream::DecryptorBE32<XChaCha20Poly1305> {
    let aead = make_aead(key);
    stream::DecryptorBE32::from_aead(aead, nonce.as_ref().into())
}
