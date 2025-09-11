//! Lib of symmetric encryption algorithms.

/// AES Modules
pub mod aes;

/// A trait for symmetric encryption algorithms.
pub trait SymmetricCipher {
    type Key;
    type Nonce;

    fn keygen() -> Self::Key;
    fn encrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        plaintext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError>;
    fn decrypt(
        key: &Self::Key,
        nonce: &Self::Nonce,
        ciphertext: &mut [u8],
    ) -> Result<Vec<u8>, SymmetricError>;
}

/// Error type for symmetric encryption operations.
#[derive(Debug, thiserror::Error)]
pub enum SymmetricError {
    #[error("Invalid key or IV length")]
    InvalidKeyIv,

    #[error("Encryption failed")]
    EncryptError,

    #[error("Decryption failed: {0}")]
    DecryptError(String),
}
