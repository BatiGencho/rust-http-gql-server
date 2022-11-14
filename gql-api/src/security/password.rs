use argon2::{self, Config};
use rand::Rng;

use crate::error::HashError;

pub fn hash_password(password: &[u8]) -> Result<String, HashError> {
    let salt = rand::thread_rng().gen::<[u8; 32]>();
    let config = Config::default();
    argon2::hash_encoded(password, &salt, &config).map_err(HashError::Encode)
}

pub fn verify_password(hash: &str, password: &[u8]) -> Result<bool, HashError> {
    argon2::verify_encoded(hash, password).map_err(HashError::Verify)
}
