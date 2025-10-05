use homomorphic::HomomorphicEncryption;
use symmetric::SymmetricCipher;

pub mod chacha;
pub use chacha::ChaChaTfheTranscipher;

/// Trait for homomorphic decryption (transciphering) of a symmetric ciphertext
/// into a homomorphic ciphertext.
///
/// # Overview
/// Transciphering is the process of converting data encrypted under a **standard symmetric cipher**
/// (like ChaCha20) into data encrypted under a **homomorphic encryption (HE) scheme**
/// without ever revealing the plaintext. This allows operations on encrypted data
/// that was originally encrypted symmetrically.
///
/// For example:
/// 1. A client encrypts sensitive data using a fast symmetric cipher (ChaCha20) for performance.
/// 2. The server wants to perform computations on this data using homomorphic encryption (TFHE),
///    which allows arbitrary operations on encrypted data.
/// 3. Using **transciphering**, the server can convert the symmetric ciphertext into
///    a homomorphic ciphertext securely, enabling computation without access to the plaintext.
///
/// # Type Parameters
/// - `HE`: The homomorphic encryption scheme used (e.g., `TfheU32`).
pub trait Transcipher<Sym, He>
where
    Sym: SymmetricCipher,
    He: HomomorphicEncryption,
{
    /// Encrypts plaintext with symmetric cipher, then re-encrypts
    /// the symmetric key homomorphically.
    fn transcipher_encrypt(
        sym_key: &Sym::Key,
        sym_nonce: &Sym::Nonce,
        he_sk: &He::SecretKey,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<He::Ciphertext>), TranscipherError>;

    /// Decrypts ciphertext homomorphically and then applies symmetric decrypt.
    fn transcipher_decrypt(
        sym_nonce: &Sym::Nonce,
        he_sk: &He::SecretKey,
        sym_key_ct: &Vec<He::Ciphertext>,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, TranscipherError>;
}

#[derive(Debug)]
pub enum TranscipherError {
    SymmetricError,
    HeError,
}
