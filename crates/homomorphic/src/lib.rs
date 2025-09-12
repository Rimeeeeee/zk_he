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

/// Trait all HE schemes should implement
pub trait HomomorphicEncryption {
    type SecretKey;
    type PublicKey;
    type Ciphertext;
    type Plaintext;

    /// Generate a new keypair
    fn keygen() -> Result<(Self::SecretKey, Self::PublicKey), HeError>;

    /// Encrypt a plaintext
    fn encrypt(pk: &Self::PublicKey, pt: &Self::Plaintext) -> Result<Self::Ciphertext, HeError>;

    /// Decrypt a ciphertext
    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::Plaintext, HeError>;

    /// Homomorphic addition
    fn add(ct1: &Self::Ciphertext, ct2: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError>;

    /// Homomorphic multiplication
    fn mul(ct1: &Self::Ciphertext, ct2: &Self::Ciphertext) -> Result<Self::Ciphertext, HeError>;
}
