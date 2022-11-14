use bincode_aes::BincodeCryptor;

// Encryption Details:
/// key length (AES-256-CBC) KEY_LEN: usize = 32;
/// initialization vector length (AES-256-CBC) IV_LEN: usize = 16;
pub struct BincodeAesUtils {
    bc: BincodeCryptor,
}

impl BincodeAesUtils {
    pub fn new() -> Self {
        let key =
            bincode_aes::random_key().expect("Error creating a random key for bincode encryption");
        let bc = bincode_aes::with_key(key);
        Self { bc }
    }

    pub fn new_from_secret(secret: &str) -> Self {
        let key = bincode_aes::create_key(secret.as_bytes().to_vec())
            .expect("Error creating a key from secret");
        let bc = bincode_aes::with_key(key);
        Self { bc }
    }

    pub fn encrypt_data(&self, data: Option<String>) -> String {
        let encoded: Vec<u8> = self.bc.serialize(&data).unwrap();
        let encoded_hex = hex::encode(&encoded);
        encoded_hex
    }

    pub fn decrypt_data(&self, encrypted_data: &str) -> Option<String> {
        let mut decoded_hex = hex::decode(&encrypted_data).unwrap();
        let decoded_str: Option<String> = self.bc.deserialize(&mut decoded_hex).unwrap();
        decoded_str
    }
}
