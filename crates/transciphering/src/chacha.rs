use crate::{HeError, HomomorphicDecrypt, HomomorphicEncryption};
use homomorphic::tfhe::TfheU32;
use symmetric::chacha::ChaCha20Cipher;
use tfhe::FheUint32;
use tfhe::prelude::*;

/// Implement homomorphic decryption for ChaCha20 using TFHE.
///
/// # Overview
/// This transciphering implementation allows decrypting a ChaCha20 ciphertext **inside the HE domain**.
/// - `sym_ct` is a 4-byte slice representing a 32-bit block of symmetric ciphertext.
/// - `enc_key` is the ChaCha20 key, encrypted under TFHE (8 × FheUint32 = 256-bit key).
/// - `client_key` is required to encrypt constants, counters, and nonces inside the HE domain.
///
/// The output is a `FheUint32` representing the plaintext block in encrypted form.
///
/// # Steps
/// 1. **Convert ciphertext bytes into a 32-bit integer**.
/// 2. **Encrypt ChaCha20 constants** ("expand 32-byte k") under the client key.
/// 3. **Build the ChaCha20 16-word state**: constants (0..3), key (4..11), counter (12), nonce (13..15).
/// 4. **Perform 20 ChaCha20 rounds** (10 double-rounds) using homomorphic addition, XOR, and rotations.
/// 5. **Add original state to transformed state** to generate keystream.
/// 6. **XOR the ciphertext block with the first keystream word** to produce the homomorphic plaintext.
impl HomomorphicDecrypt<TfheU32> for ChaCha20Cipher {
    type EncKey = Vec<FheUint32>; // 8 encrypted u32 words = 256-bit key

    fn homomorphic_decrypt(
        sym_ct: &[u8],
        enc_key: &Self::EncKey,
        client_key: &<TfheU32 as HomomorphicEncryption>::SecretKey,
    ) -> Result<FheUint32, HeError> {
        if enc_key.len() != 8 {
            return Err(HeError::EvalError("Invalid encrypted key length".into()));
        }

        // Step 1: Convert 4 bytes of sym_ct into a u32
        if sym_ct.len() < 4 {
            return Err(HeError::DecryptError);
        }
        let mut ct_bytes = [0u8; 4];
        ct_bytes.copy_from_slice(&sym_ct[..4]);
        let ct_val = u32::from_le_bytes(ct_bytes);

        // Step 2: Encrypt ChaCha20 constants under HE
        let constants: [FheUint32; 4] = [
            TfheU32::encrypt(client_key, &0x61707865)?, // "expa"
            TfheU32::encrypt(client_key, &0x3320646e)?, // "nd 3"
            TfheU32::encrypt(client_key, &0x79622d32)?, // "2-by"
            TfheU32::encrypt(client_key, &0x6b206574)?, // "te k"
        ];

        // Step 3: Build 16-word ChaCha20 state
        let mut state: [FheUint32; 16] = [
            constants[0].clone(),
            constants[1].clone(),
            constants[2].clone(),
            constants[3].clone(),
            enc_key[0].clone(),
            enc_key[1].clone(),
            enc_key[2].clone(),
            enc_key[3].clone(),
            enc_key[4].clone(),
            enc_key[5].clone(),
            enc_key[6].clone(),
            enc_key[7].clone(),
            TfheU32::encrypt(client_key, &1)?, // counter
            TfheU32::encrypt(client_key, &0)?, // nonce0
            TfheU32::encrypt(client_key, &0)?, // nonce1
            TfheU32::encrypt(client_key, &0)?, // nonce2
        ];

        // Step 4: Perform 20 ChaCha20 rounds (10 double-rounds)
        for _ in 0..10 {
            double_round(&mut state);
        }

        // Step 5: Add original state to transformed state to form keystream
        let mut keystream = Vec::with_capacity(16);
        for (i, s) in state.iter().enumerate() {
            let sum = TfheU32::add(s, &state[i])?;
            keystream.push(sum);
        }

        // Step 6: XOR first word of ciphertext with first word of keystream
        let ct_fhe = TfheU32::encrypt(client_key, &ct_val)?;
        let plaintext_fhe = ct_fhe ^ &keystream[0];

        Ok(plaintext_fhe)
    }
}

/// Homomorphic ChaCha20 quarter-round: operates entirely on FheUint32
///
/// Implements standard ChaCha20 quarter-round:
///
// — a += b; d ^= a; d <<< 16
// — c += d; b ^= c; b <<< 12
// — a += b; d ^= a; d <<< 8
// — c += d; b ^= c; b <<< 7
fn quarter_round(
    a: &FheUint32,
    b: &FheUint32,
    c: &FheUint32,
    d: &FheUint32,
) -> (FheUint32, FheUint32, FheUint32, FheUint32) {
    let mut a = a.clone();
    let mut b = b.clone();
    let mut c = c.clone();
    let mut d = d.clone();

    a = TfheU32::add(&a, &b).unwrap();
    d = &d ^ &a;
    d = d.rotate_left(16u32);

    c = TfheU32::add(&c, &d).unwrap();
    b = &b ^ &c;
    b = b.rotate_left(12u32);

    a = TfheU32::add(&a, &b).unwrap();
    d = &d ^ &a;
    d = d.rotate_left(8u32);

    c = TfheU32::add(&c, &d).unwrap();
    b = &b ^ &c;
    b = b.rotate_left(7u32);

    (a, b, c, d)
}

/// Homomorphic ChaCha20 double-round: column round + diagonal round
///
/// Each double-round performs 8 quarter-rounds, following the standard ChaCha20 design.
fn double_round(state: &mut [FheUint32; 16]) {
    // Column rounds
    let (a0, b0, c0, d0) = quarter_round(&state[0], &state[4], &state[8], &state[12]);
    state[0] = a0;
    state[4] = b0;
    state[8] = c0;
    state[12] = d0;

    let (a1, b1, c1, d1) = quarter_round(&state[1], &state[5], &state[9], &state[13]);
    state[1] = a1;
    state[5] = b1;
    state[9] = c1;
    state[13] = d1;

    let (a2, b2, c2, d2) = quarter_round(&state[2], &state[6], &state[10], &state[14]);
    state[2] = a2;
    state[6] = b2;
    state[10] = c2;
    state[14] = d2;

    let (a3, b3, c3, d3) = quarter_round(&state[3], &state[7], &state[11], &state[15]);
    state[3] = a3;
    state[7] = b3;
    state[11] = c3;
    state[15] = d3;

    // Diagonal rounds
    let (a0, b0, c0, d0) = quarter_round(&state[0], &state[5], &state[10], &state[15]);
    state[0] = a0;
    state[5] = b0;
    state[10] = c0;
    state[15] = d0;

    let (a1, b1, c1, d1) = quarter_round(&state[1], &state[6], &state[11], &state[12]);
    state[1] = a1;
    state[6] = b1;
    state[11] = c1;
    state[12] = d1;

    let (a2, b2, c2, d2) = quarter_round(&state[2], &state[7], &state[8], &state[13]);
    state[2] = a2;
    state[7] = b2;
    state[8] = c2;
    state[13] = d2;

    let (a3, b3, c3, d3) = quarter_round(&state[3], &state[4], &state[9], &state[14]);
    state[3] = a3;
    state[4] = b3;
    state[9] = c3;
    state[14] = d3;
}

#[cfg(test)]
mod tests {
    use super::*;
    use symmetric::SymmetricCipher;
    use tfhe::set_server_key;

    /// Test 1: Roundtrip correctness
    /// Encrypt a block with ChaCha20, then homomorphically decrypt and check it matches the original.
    #[test]
    fn test_homomorphic_decrypt_roundtrip() {
        let (client_key, server_key) = TfheU32::keygen().unwrap();
        set_server_key(server_key.clone());

        // Generate ChaCha20 key and encrypt under HE
        // Generate ChaCha20 key as 32 bytes
        let sym_key: [u8; 32] = rand::random();

        // Encrypt the key under HE (split into 8 u32 words)
        let mut enc_key = Vec::new();
        for chunk in sym_key.chunks(4) {
            let word = u32::from_le_bytes(chunk.try_into().unwrap());
            enc_key.push(TfheU32::encrypt(&client_key, &word).unwrap());
        }

        // Symmetric plaintext block
        let plaintext_block: u32 = 0xDEADBEEF;
        let mut sym_ct_bytes = plaintext_block.to_le_bytes();

        // Encrypt using ChaCha20
        let ciphertext = ChaCha20Cipher::encrypt(&sym_key, &[0u8; 12], &mut sym_ct_bytes).unwrap();

        // Homomorphic decryption
        let plaintext_fhe =
            ChaCha20Cipher::homomorphic_decrypt(&ciphertext, &enc_key, &client_key).unwrap();
        let decrypted = TfheU32::decrypt(&client_key, &plaintext_fhe).unwrap();

        assert_eq!(decrypted, plaintext_block);
    }

    /// Test 2: Minimum ciphertext length validation
    /// Ensures inputs shorter than 4 bytes produce an error.
    #[test]
    fn test_homomorphic_minimum_ct_size() {
        let (client_key, _server_key) = TfheU32::keygen().unwrap();

        // Proper encrypted key
        let sym_key: [u32; 8] = rand::random();
        let mut enc_key = Vec::new();
        for k in sym_key.iter() {
            enc_key.push(TfheU32::encrypt(&client_key, k).unwrap());
        }

        let short_ct: [u8; 2] = [0x12, 0x34]; // less than 4 bytes
        let result = ChaCha20Cipher::homomorphic_decrypt(&short_ct, &enc_key, &client_key);

        assert!(result.is_err());
    }
}
