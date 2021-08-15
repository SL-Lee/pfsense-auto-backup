use std::{env, fs};

use aes::Aes256;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use block_modes::{block_padding::Pkcs7, BlockMode, Cbc};
use rand::Rng;
use serde::{Deserialize, Serialize};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

#[derive(Deserialize, Serialize)]
pub struct EncryptionKeyMetadata {
    iv: [u8; 16],
    encrypted_key: Vec<u8>,
}

pub fn generate_encryption_key() -> (String, EncryptionKeyMetadata) {
    let iv = rand::thread_rng().gen::<[u8; 16]>();
    let kek = retrieve_key_encryption_key().unwrap();
    let dek = rand::thread_rng().gen::<[u8; 32]>();
    let cipher = Aes256Cbc::new_from_slices(&kek, &iv).unwrap();
    (
        dek.iter()
            .map(|byte| format!("{:0>2x}", byte))
            .collect::<String>(),
        EncryptionKeyMetadata {
            iv,
            encrypted_key: cipher.encrypt_vec(&dek),
        },
    )
}

pub fn retrieve_encryption_key(filename: &str) -> Result<String, String> {
    let encryption_key_metadata = match serde_json::from_str::<
        EncryptionKeyMetadata,
    >(
        match &fs::read_to_string(filename) {
            Ok(file_contents) => file_contents,
            Err(error) => return Err(error.to_string()),
        },
    ) {
        Ok(encryption_key_metadata) => encryption_key_metadata,
        Err(_) => {
            return Err("Metadata seems to be of invalid format.".to_string())
        }
    };
    let kek = retrieve_key_encryption_key().unwrap();
    let cipher =
        match Aes256Cbc::new_from_slices(&kek, &encryption_key_metadata.iv) {
            Ok(cipher) => cipher,
            Err(error) => return Err(error.to_string()),
        };
    match cipher.decrypt_vec(&encryption_key_metadata.encrypted_key) {
        Ok(encryption_key) => Ok(encryption_key
            .iter()
            .map(|byte| format!("{:0>2x}", byte))
            .collect::<String>()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn retrieve_key_encryption_key() -> Result<Vec<u8>, String> {
    let encryption_passphrase = env::var("ENCRYPTION_PASSPHRASE")
        .expect("'ENCRYPTION_PASSPHRASE' environment variable is not set.");
    let encryption_passphrase_bytes = encryption_passphrase.as_bytes();

    let kek_info = fs::read_to_string(".kek-info").unwrap();
    let parsed_hash = PasswordHash::new(&kek_info).expect(
        "The .kek-info file seems to be invalid. Try removing the file and \
        restarting the application to fix this.",
    );

    if Argon2::default()
        .verify_password(encryption_passphrase_bytes, &parsed_hash)
        .is_ok()
    {
        Ok(parsed_hash.hash.unwrap().as_bytes().to_vec())
    } else {
        Err("Unable to retrieve key encryption key.".to_string())
    }
}
