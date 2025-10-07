use crate::{HeError, HomomorphicEncryption};
use serde::{Deserialize, Serialize};
use tfhe::ClientKey;
use tfhe::prelude::*;
use tfhe::{ConfigBuilder, FheUint32, generate_keys};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TfheU32;

impl HomomorphicEncryption for TfheU32 {
    type SecretKey = tfhe::ClientKey;
    type PublicKey = tfhe::ServerKey;
    type Ciphertext = FheUint32;
    type Plaintext = u32;

    fn keygen() -> Result<(Self::SecretKey, Self::PublicKey), HeError> {
        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        Ok((client_key, server_key))
    }

    // encrypt now takes the ClientKey (SecretKey)
    fn encrypt(sk: &Self::SecretKey, pt: &Self::Plaintext) -> Result<Self::Ciphertext, HeError> {
        FheUint32::try_encrypt(*pt, sk).map_err(|_| HeError::EncryptError)
    }

    fn decrypt(sk: &Self::SecretKey, ct: &Self::Ciphertext) -> Result<Self::Plaintext, HeError> {
        Ok(ct.decrypt(sk))
    }

    fn add(ct1: &FheUint32, ct2: &FheUint32) -> Result<FheUint32, HeError> {
        Ok(ct1 + ct2)
    }

    fn mul(ct1: &FheUint32, ct2: &FheUint32) -> Result<FheUint32, HeError> {
        Ok(ct1 * ct2)
    }
}

impl TfheU32 {
    pub fn add(a: &FheUint32, b: &FheUint32) -> Result<FheUint32, HeError> {
        // Homomorphic addition (mod 2^32)
        Ok(a + b)
    }

    pub fn encrypt_public_with_client(ck: &ClientKey, value: u32) -> Result<FheUint32, HeError> {
        FheUint32::try_encrypt(value, ck).map_err(|_| HeError::EncryptError)
    }

    pub fn xor(a: &FheUint32, b: &FheUint32) -> Result<FheUint32, HeError> {
        // tfhe-rs implements bitwise operations natively:
        Ok(a ^ b)
    }

    pub fn rotl(a: &FheUint32, n: u32) -> Result<FheUint32, HeError> {
        // tfhe-rs supports rotate-left and rotate-right directly on FheUint32
        Ok(a.rotate_left(n))
    }

    pub fn rotr(a: &FheUint32, n: u32) -> Result<FheUint32, HeError> {
        Ok(a.rotate_right(n))
    }

    pub fn and(a: &FheUint32, b: &FheUint32) -> Result<FheUint32, HeError> {
        Ok(a & b)
    }

    pub fn or(a: &FheUint32, b: &FheUint32) -> Result<FheUint32, HeError> {
        Ok(a | b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tfhe::set_server_key;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let (client_key, server_key) = TfheU32::keygen().unwrap();

        // Server-side operations require the server key to be set globally:
        set_server_key(server_key.clone());

        let value: u32 = 12345;
        let ct = TfheU32::encrypt(&client_key, &value).unwrap();
        let dec = TfheU32::decrypt(&client_key, &ct).unwrap();

        assert_eq!(dec, value);
    }

    #[test]
    fn test_addition() {
        let (client_key, server_key) = TfheU32::keygen().unwrap();
        set_server_key(server_key.clone());

        let a: u32 = 5;
        let b: u32 = 7;

        let ct_a = TfheU32::encrypt(&client_key, &a).unwrap();
        let ct_b = TfheU32::encrypt(&client_key, &b).unwrap();
        let ct_sum = TfheU32::add(&ct_a, &ct_b).unwrap();
        let dec_sum = TfheU32::decrypt(&client_key, &ct_sum).unwrap();

        assert_eq!(dec_sum, a + b);
    }

    #[test]
    fn test_multiplication() {
        let (client_key, server_key) = TfheU32::keygen().unwrap();
        set_server_key(server_key.clone());

        let a: u32 = 2;
        let b: u32 = 3;

        let ct_a = TfheU32::encrypt(&client_key, &a).unwrap();
        let ct_b = TfheU32::encrypt(&client_key, &b).unwrap();

        let ct_prod = TfheU32::mul(&ct_a, &ct_b).unwrap();
        let dec_prod = TfheU32::decrypt(&client_key, &ct_prod).unwrap();

        assert_eq!(dec_prod, a * b);
    }

    #[test]
    fn test_chain_operations() {
        let (client_key, server_key) = TfheU32::keygen().unwrap();
        set_server_key(server_key.clone());

        let x: u32 = 9;
        let y: u32 = 4;
        let z: u32 = 2;

        let ct_x = TfheU32::encrypt(&client_key, &x).unwrap();
        let ct_y = TfheU32::encrypt(&client_key, &y).unwrap();
        let ct_z = TfheU32::encrypt(&client_key, &z).unwrap();

        // (x + y) * z
        let ct_sum = TfheU32::add(&ct_x, &ct_y).unwrap();
        let ct_expr = TfheU32::mul(&ct_sum, &ct_z).unwrap();

        let dec = TfheU32::decrypt(&client_key, &ct_expr).unwrap();

        assert_eq!(dec, (x + y) * z);
    }
}
