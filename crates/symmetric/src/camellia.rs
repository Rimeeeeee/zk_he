use camellia::Camellia256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};

use rand::RngCore;

use crate::{SymmetricCipher, SymmetricError};

/// Camellia-256 in CBC mode with PKCS7 padding
pub struct CamelliaCipher;

impl SymmetricCipher for CamelliaCipher {
    type Key = [u8; 32]; // 256-bit key
    type Nonce = [u8; 16]; // 128-bit IV

    fn keygen() -> Self::Key {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    fn encrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError> {
        let cipher = cbc::Encryptor::<Camellia256>::new_from_slices(key, nonce)
            .map_err(|_| SymmetricError::InvalidKeyIv)?;

        Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext))
    }

    fn decrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError> {
        let cipher = cbc::Decryptor::<Camellia256>::new_from_slices(key, nonce)
            .map_err(|_| SymmetricError::InvalidKeyIv)?;

        cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| SymmetricError::DecryptError(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SymmetricCipher, SymmetricError};

    #[test]
    fn test_camellia_encrypt_decrypt_ok() {
        let key = CamelliaCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"Hello Camellia CBC with PKCS7!";

        // Allocate buffer with padding space (Camellia block = 16 bytes)
        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len(), 0u8);

        // Encrypt
        let ciphertext = CamelliaCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Prepare mutable buffer for decryption
        let mut ct_buf = ciphertext.clone();

        // Decrypt
        let decrypted = CamelliaCipher::decrypt(&key, &iv, &mut ct_buf).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_camellia_decrypt_with_wrong_key() {
        let key = CamelliaCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"SecretCamelliaMsg";

        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len() + 16, 0u8);

        let ciphertext = CamelliaCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Wrong key
        let wrong_key = CamelliaCipher::keygen();
        let mut ct_buf = ciphertext.clone();
        let result = CamelliaCipher::decrypt(&wrong_key, &iv, &mut ct_buf);

        assert!(matches!(result, Err(SymmetricError::DecryptError(_))));
    }

    #[test]
    fn test_camellia_decrypt_with_wrong_iv() {
        let key = CamelliaCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"AnotherCamelliaMsg";

        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len() + 16, 0u8);

        let ciphertext = CamelliaCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Wrong IV
        let mut bad_iv = [0u8; 16];
        bad_iv[0] = 42;

        let mut ct_buf = ciphertext.clone();
        let result = CamelliaCipher::decrypt(&key, &bad_iv, &mut ct_buf);
        assert_ne!(result.unwrap_or_default(), plaintext);
    }
}
