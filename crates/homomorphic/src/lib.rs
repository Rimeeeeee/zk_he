//! Homomorphic Encryption Trait Definition

pub mod tfhe;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeError {
    #[error("Key generation failed")]
    KeyGenError,
    #[error("Encryption error")]
    EncryptError,
    #[error("Decryption error")]
    DecryptError,
    #[error("Evaluation error: {0}")]
    EvalError(String),
}

/// HomomorphicEncryption trait
pub trait HomomorphicEncryption {
    type SecretKey; // client/secret key (used for encrypt/decrypt)
    type PublicKey; // server/public key (used for evaluation)
    type Ciphertext;
    type Plaintext;

    /// Generate (secret_key, public/server_key)
    fn keygen() -> Result<(Self::SecretKey, Self::PublicKey), HeError>;

    /// ENCRYPT uses the SecretKey (client-side)
    fn encrypt(sk: &Self::SecretKey, pt: &Self::Plaintext) -> Result<Self::Ciphertext, HeError>;

    /// DECRYPT uses the SecretKey (client-side)
    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::Plaintext, HeError>;

    /// Homomorphic operations (server-side). They may require server key to be set globally
    fn add(ct1: &Self::Ciphertext, ct2: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError>;
    fn mul(ct1: &Self::Ciphertext, ct2: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError>;
}
