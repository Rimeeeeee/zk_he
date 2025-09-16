use homomorphic::{HeError, HomomorphicEncryption, tfhe::TfheU32};
use tfhe::FheUint32;

pub mod chacha;

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
pub trait HomomorphicDecrypt<HE: HomomorphicEncryption> {
    /// Type of the symmetric key, encrypted under HE.
    type EncKey;

    /// Homomorphically decrypt a symmetric ciphertext into a homomorphic ciphertext.
    ///
    /// # Parameters
    /// - `sym_ct`: The symmetric ciphertext (e.g., 32-bit block from ChaCha20).
    /// - `enc_key`: The symmetric key encrypted under HE (`FheUint32` array).
    /// - `client_key`: The secret key of the HE scheme, required to encrypt constants
    ///   and auxiliary values in the HE domain.
    ///
    /// # Returns
    /// A `FheUint32` representing the plaintext in encrypted form under HE.
    ///
    /// # Errors
    /// Returns `HeError` if:
    /// - The encrypted key length is invalid
    /// - The input ciphertext block is too short
    /// - Homomorphic operations fail
    fn homomorphic_decrypt(
        sym_ct: &[u8],
        enc_key: &Self::EncKey,
        client_key: &<TfheU32 as HomomorphicEncryption>::SecretKey,
    ) -> Result<FheUint32, HeError>;
}
