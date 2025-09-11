use aes::Aes256;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use rand::RngCore;

use crate::{SymmetricCipher, SymmetricError};

/// AES-256-CBC symmetric block cipher with PKCS7 padding.
///
/// AES (Advanced Encryption Standard) is a widely used symmetric block cipher
/// standardized by NIST. AES-256 uses a 256-bit key for encryption, providing
/// strong security against brute-force attacks. In CBC (Cipher Block Chaining) mode,
/// each plaintext block is XORed with the previous ciphertext block before being
/// encrypted, which ensures that identical plaintext blocks produce different ciphertexts.
///
/// Since AES operates on fixed-size 16-byte blocks, PKCS7 padding is used to
/// pad plaintexts that are not a multiple of the block size. This allows
/// arbitrary-length data to be securely encrypted.
///
/// # Features
///
/// - **Key size**: 256 bits (32 bytes)
/// - **Block size**: 128 bits (16 bytes)
/// - **Mode of operation**: CBC (Cipher Block Chaining)
/// - **Padding**: PKCS7, to handle plaintexts of arbitrary length
///
/// # Security
///
/// AES-256-CBC is considered secure when used with:
/// - A **random, unique IV** (initialization vector) for each encryption
/// - Secure key management (never reuse or expose the key)
///
/// CBC mode is vulnerable to certain padding oracle attacks if not implemented
/// carefully, so encryption/decryption routines must validate padding correctly.
pub struct AesCipher;

impl SymmetricCipher for AesCipher {
    type Key = [u8; 32]; // 256-bit key
    type Nonce = [u8; 16]; // 128-bit IV

    /// Generate a random 256-bit key
    fn keygen() -> Self::Key {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }

    /// Encrypts the plaintext in place and returns the ciphertext
    fn encrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError> {
        // Create an encryptor instance
        let cipher = cbc::Encryptor::<Aes256>::new(key.into(), nonce.into());

        // Encrypt with PKCS7 padding
        Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext))
    }

    /// Decrypts the ciphertext in place and returns the plaintext
    fn decrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError> {
        let cipher = cbc::Decryptor::<Aes256>::new(key.into(), nonce.into());

        // Decrypt and remove PKCS7 padding
        cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map(|ct| ct.to_vec())
            .map_err(|e| SymmetricError::DecryptError(format!("{:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SymmetricCipher, SymmetricError};

    #[test]
    fn test_aes_encrypt_decrypt_ok() {
        let key = AesCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"Hello AES CBC with PKCS7!";

        // Make a mutable buffer with extra space for padding
        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len(), 0u8); // allocate padding room

        // Encrypt
        let ciphertext = AesCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Prepare mutable buffer for decryption
        let mut ct_buf = ciphertext.clone();

        // Decrypt
        let decrypted = AesCipher::decrypt(&key, &iv, &mut ct_buf).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let key = AesCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"SecretMessage";

        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len(), 0u8);

        let ciphertext = AesCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Use wrong key for decryption
        let wrong_key = AesCipher::keygen();
        let mut ct_buf = ciphertext.clone();
        let result = AesCipher::decrypt(&wrong_key, &iv, &mut ct_buf);

        assert!(matches!(result, Err(SymmetricError::DecryptError(_))));
    }

    #[test]
    fn test_decrypt_with_wrong_iv() {
        let key = AesCipher::keygen();
        let iv = [0u8; 16];
        let plaintext = b"AnotherSecret";

        let mut buffer = plaintext.to_vec();
        buffer.resize(buffer.len(), 0u8);

        let ciphertext = AesCipher::encrypt(&key, &iv, &mut buffer.clone()).unwrap();

        // Wrong IV
        let mut bad_iv = [0u8; 16];
        bad_iv[0] = 42; // just flip something

        let mut ct_buf = ciphertext.clone();
        let result = AesCipher::decrypt(&key, &bad_iv, &mut ct_buf).unwrap();

        assert_ne!(buffer, result);
    }
}
