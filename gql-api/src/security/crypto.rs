use crate::error::Error;
use crate::error::UserError;
use ed25519_dalek::PublicKey;
use ed25519_dalek::Signature;
use ed25519_dalek::Verifier;
use ed25519_dalek::{ExpandedSecretKey, SecretKey};
use near_account_id::AccountId;
use rand::rngs::OsRng;

pub struct NearAccount {
    public_key: PublicKey,
    expanded_secret_key: ExpandedSecretKey,
    account_id: String,
}

impl NearAccount {
    pub fn new_implicit() -> Result<NearAccount, Error> {
        let mut csprng = OsRng {};

        // create secret key (64 bytes)
        let secret_key: SecretKey = SecretKey::generate(&mut csprng);
        let expanded_secret_key: ExpandedSecretKey = ExpandedSecretKey::from(&secret_key);

        // get pub key
        let pub_key: PublicKey = PublicKey::from(&expanded_secret_key);

        // get account id
        let account_id_hex = hex::encode(pub_key.to_bytes());

        // check account id is implicit
        if !check_implicit_account(&account_id_hex)? {
            return Err(Error::User(UserError::BadImplicitAccount));
        }

        Ok(Self {
            public_key: pub_key,
            expanded_secret_key: expanded_secret_key,
            account_id: account_id_hex,
        })
    }

    pub fn secret_key_b58_encoded(&self) -> String {
        let expanded_secret_key_bytes: [u8; 64] = self.expanded_secret_key.to_bytes();
        let b58_encoded_secret_key = bs58::encode(expanded_secret_key_bytes).into_string();
        b58_encoded_secret_key
    }

    pub fn pub_key_b58_encoded(&self) -> String {
        let pub_key_binary = self.public_key.to_bytes();
        let b58_encoded_pub_key = bs58::encode(pub_key_binary).into_string();
        b58_encoded_pub_key
    }

    pub fn account_id_hex_encoded(&self) -> &str {
        &self.account_id
    }

    pub fn sign_message(&self, message: &[u8]) -> ([u8; 64], String) {
        let signature: ed25519_dalek::Signature =
            self.expanded_secret_key.sign(message, &self.public_key);
        let sig_bytes = signature.to_bytes();
        let b58_encoded_signature = bs58::encode(sig_bytes).into_string();
        (sig_bytes, b58_encoded_signature)
    }

    pub fn verify_signature(
        &self,
        message: &[u8],
        signature_bytes: &[u8],
    ) -> Result<(), ed25519_dalek::ed25519::Error> {
        verify_signature_with_pub_key(&self.public_key, message, signature_bytes)
    }
}

pub fn verify_signature_with_pub_key(
    public_key: &PublicKey,
    message: &[u8],
    signature_bytes: &[u8],
) -> Result<(), ed25519_dalek::ed25519::Error> {
    let retrieved_signature = Signature::from_bytes(&signature_bytes)?;
    let is_ok = public_key.verify(message, &retrieved_signature);
    is_ok
}

pub fn check_implicit_account(account_id: &str) -> Result<bool, Error> {
    // check account id is implicit
    let near_account_id = account_id
        .parse::<AccountId>()
        .map_err(|e| Error::User(UserError::AccountParse(e)))?;

    Ok(near_account_id.is_implicit())
}

pub fn check_system_account(account_id: &str) -> Result<bool, Error> {
    // check account id is empty
    let near_account_id = account_id
        .parse::<AccountId>()
        .map_err(|e| Error::User(UserError::AccountParse(e)))?;

    Ok(near_account_id.is_empty())
}

pub fn check_normal_account(account_id: &str) -> Result<bool, Error> {
    // check account id is normal
    let near_account_id = account_id
        .parse::<AccountId>()
        .map_err(|e| Error::User(UserError::AccountParse(e)))?;

    if near_account_id.is_empty() || near_account_id.is_implicit() || near_account_id.is_system() {
        return Ok(false);
    }

    Ok(true)
}
