use crate::{HeError, HomomorphicEncryption};
use serde::{Deserialize, Serialize};
use tfhe::prelude::*;
use tfhe::{ClientKey, ConfigBuilder, FheBool, ServerKey, generate_keys};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfheBool;

impl HomomorphicEncryption for TfheBool {
    type SecretKey = ClientKey;
    type PublicKey = ServerKey;
    type Ciphertext = FheBool;
    type Plaintext = bool;

    /// Generates a new TFHE client + server keypair.
    fn keygen() -> Result<(Self::SecretKey, Self::PublicKey), HeError> {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        Ok((client_key, server_key))
    }

    /// Encrypts a boolean using the client (secret) key.
    fn encrypt(sk: &Self::SecretKey, pt: &Self::Plaintext) -> Result<Self::Ciphertext, HeError> {
        FheBool::try_encrypt(*pt, sk).map_err(|_| HeError::EncryptError)
    }

    /// Decrypts a ciphertext back into a boolean.
    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::Plaintext, HeError> {
        Ok(ct.decrypt(sk))
    }

    /// Homomorphic AND operation between two encrypted booleans.
    fn mul(ct1: &FheBool, ct2: &FheBool) -> Result<FheBool, HeError> {
        Ok(ct1 & ct2)
    }

    /// Homomorphic OR operation between two encrypted booleans.
    fn add(ct1: &FheBool, ct2: &FheBool) -> Result<FheBool, HeError> {
        Ok(ct1 | ct2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tfhe::set_server_key;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (client_key, server_key) = TfheBool::keygen().unwrap();
        set_server_key(server_key);

        let value = true;
        let ct = TfheBool::encrypt(&client_key, &value).unwrap();
        let dec = TfheBool::decrypt(&client_key, &ct).unwrap();
        assert_eq!(dec, value);

        let value = false;
        let ct = TfheBool::encrypt(&client_key, &value).unwrap();
        let dec = TfheBool::decrypt(&client_key, &ct).unwrap();
        assert_eq!(dec, value);
    }

    #[test]
    fn test_and_operation() {
        let (client_key, server_key) = TfheBool::keygen().unwrap();
        set_server_key(server_key);

        let a = TfheBool::encrypt(&client_key, &true).unwrap();
        let b = TfheBool::encrypt(&client_key, &false).unwrap();

        let ct_and = TfheBool::mul(&a, &b).unwrap();
        let dec = TfheBool::decrypt(&client_key, &ct_and).unwrap();
        assert_eq!(dec, false);
    }

    #[test]
    fn test_or_operation() {
        let (client_key, server_key) = TfheBool::keygen().unwrap();
        set_server_key(server_key);

        let a = TfheBool::encrypt(&client_key, &false).unwrap();
        let b = TfheBool::encrypt(&client_key, &true).unwrap();

        let ct_or = TfheBool::add(&a, &b).unwrap();
        let dec = TfheBool::decrypt(&client_key, &ct_or).unwrap();
        assert_eq!(dec, true);
    }

    #[test]
    fn test_chain_operations() {
        let (client_key, server_key) = TfheBool::keygen().unwrap();
        set_server_key(server_key);

        let a = TfheBool::encrypt(&client_key, &true).unwrap();
        let b = TfheBool::encrypt(&client_key, &false).unwrap();
        let c = TfheBool::encrypt(&client_key, &true).unwrap();

        // (a AND c) OR b
        let ct_and = TfheBool::mul(&a, &c).unwrap();
        let ct_expr = TfheBool::add(&ct_and, &b).unwrap();

        let dec = TfheBool::decrypt(&client_key, &ct_expr).unwrap();
        assert_eq!(dec, true);
    }
}
