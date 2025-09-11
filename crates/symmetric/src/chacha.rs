use chacha20::cipher::{KeyIvInit, StreamCipher};
use rand::RngCore;

use crate::{SymmetricCipher, SymmetricError};

/// ChaCha20 symmetric stream cipher.
///
/// ChaCha20 is a modern, high-performance stream cipher designed by Daniel J. Bernstein
/// as a variant of the Salsa20 cipher. It operates on 256-bit keys and 96-bit nonces
/// (IETF variant), generating a pseudorandom keystream that is XORed with plaintext
/// to produce ciphertext. The same operation is used for both encryption and decryption,
/// making it a symmetric cipher.
///
/// # Features
///
/// - **Key size**: 256 bits (32 bytes)
/// - **Nonce size**: 96 bits (12 bytes)
/// - **Stream cipher**: encrypts data of arbitrary length by XORing with a keystream
/// - **Deterministic and symmetric**: the same key and nonce combination always
///   produces the same keystream; encryption and decryption are identical operations
///
/// # Security
///
/// ChaCha20 is designed to resist timing attacks and provide high diffusion and
/// security guarantees. Nonce reuse with the same key completely compromises
/// security, so each encryption must use a unique nonce. While ChaCha20 provides
/// confidentiality, it does **not** provide authentication; it should be combined
/// with a MAC (e.g., Poly1305) for authenticated encryption if integrity is required.
pub struct ChaCha20Cipher;

impl SymmetricCipher for ChaCha20Cipher {
    type Key = [u8; 32]; // 256-bit key
    type Nonce = [u8; 12]; // 96-bit nonce (IETF variant)

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
        let mut cipher = chacha20::ChaCha20::new(key.into(), nonce.into());
        let mut buf = plaintext.to_vec();
        cipher.apply_keystream(&mut buf);
        Ok(buf)
    }

    fn decrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError> {
        let mut cipher = chacha20::ChaCha20::new(key.into(), nonce.into());
        let mut buf = ciphertext.to_vec();
        cipher.apply_keystream(&mut buf); // ChaCha20 is symmetric
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = ChaCha20Cipher::keygen();
        let nonce: [u8; 12] = rand::random(); // random 12-byte nonce

        let mut plaintext = b"Hello, ChaCha20!".to_vec();
        let ciphertext = ChaCha20Cipher::encrypt(&key, &nonce, &mut plaintext).unwrap();

        let mut ciphertext_clone = ciphertext.clone();
        let decrypted = ChaCha20Cipher::decrypt(&key, &nonce, &mut ciphertext_clone).unwrap();

        assert_eq!(decrypted, b"Hello, ChaCha20!");
    }

    #[test]
    fn test_different_nonce_produces_different_ciphertext() {
        let key = ChaCha20Cipher::keygen();
        let nonce1: [u8; 12] = rand::random();
        let nonce2: [u8; 12] = rand::random();

        let mut plaintext = b"Test message".to_vec();
        let ct1 = ChaCha20Cipher::encrypt(&key, &nonce1, &mut plaintext.clone()).unwrap();
        let ct2 = ChaCha20Cipher::encrypt(&key, &nonce2, &mut plaintext).unwrap();

        assert_ne!(ct1, ct2);
    }

    #[test]
    fn test_same_key_nonce_same_ciphertext() {
        let key = ChaCha20Cipher::keygen();
        let nonce: [u8; 12] = rand::random();

        let mut plaintext1 = b"Repeat message".to_vec();
        let mut plaintext2 = b"Repeat message".to_vec();

        let ct1 = ChaCha20Cipher::encrypt(&key, &nonce, &mut plaintext1).unwrap();
        let ct2 = ChaCha20Cipher::encrypt(&key, &nonce, &mut plaintext2).unwrap();

        assert_eq!(ct1, ct2);
    }
}
