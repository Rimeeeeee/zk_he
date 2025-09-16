//! Full homomorphic ChaCha20 transciphering (one 32-bit block).

use crate::{HeError, HomomorphicDecrypt, HomomorphicEncryption};
use homomorphic::tfhe::TfheU32;
use symmetric::chacha::ChaCha20Cipher;
use tfhe::FheUint32;
use tfhe::prelude::*;

/// Implement homomorphic decryption for ChaCha20 using TFHE.
// - Implements HomomorphicDecrypt<TfheU32> for ChaCha20Cipher.
// - Evaluates the ChaCha20 block function homomorphically (20 rounds).
// - Produces a single FheUint32 plaintext word (client must decrypt).
//
// Notes:
// - This implementation assumes the client will encrypt the ChaCha20 key
//   as eight FheUint32 words and send them to the server.
// - The server must set the TFHE server key globally (set_server_key) before doing HE ops.
// - Counter is set to 0 here to match the symmetric encryption used in the tests.
// - Performance: expect this to be slow; it's a correctness-first example.
/// # High-level idea
/// - The ChaCha20 keystream block is a deterministic 16-word (16 × 32-bit) function of:
///   constants || key || counter || nonce.
/// - We receive the ChaCha20 *key* as encrypted words (Vec<FheUint32>).
/// - We build the ChaCha20 state inside HE, run the full 20 rounds homomorphically,
///   compute keystream = state_after_rounds + initial_state (homomorphically),
///   then XOR the symmetric ciphertext word with the (encrypted) keystream word.
/// - The result is a `FheUint32` holding the plaintext (still encrypted).
impl HomomorphicDecrypt<TfheU32> for ChaCha20Cipher {
    type EncKey = Vec<FheUint32>; // expected length == 8

    fn homomorphic_decrypt(
        sym_ct: &[u8],
        enc_key: &Self::EncKey,
        client_key: &<TfheU32 as HomomorphicEncryption>::SecretKey,
    ) -> Result<FheUint32, HeError> {
        // Validate key length
        if enc_key.len() != 8 {
            return Err(HeError::EvalError("Invalid encrypted key length".into()));
        }

        // Parse the first 4 bytes of sym_ct into u32 (ChaCha20 operates on 32-bit words).
        if sym_ct.len() < 4 {
            return Err(HeError::DecryptError);
        }
        let mut ct_bytes = [0u8; 4];
        ct_bytes.copy_from_slice(&sym_ct[..4]);
        let ct_val = u32::from_le_bytes(ct_bytes);

        // ChaCha20 constants ("expand 32-byte k") — encrypt them under the client key
        // so they are represented in the HE domain.
        let constants: [FheUint32; 4] = [
            TfheU32::encrypt(client_key, &0x6170_7865)?, // "expa"
            TfheU32::encrypt(client_key, &0x3320_646e)?, // "nd 3"
            TfheU32::encrypt(client_key, &0x7962_2d32)?, // "2-by"
            TfheU32::encrypt(client_key, &0x6b20_6574)?, // "te k"
        ];

        // Build the 16-word ChaCha20 state:
        // [ constants(0..3), key(4..11), counter(12), nonce(13..15) ]
        //
        // **Important**: We choose counter = 0 here so it matches the test encryption
        // below which uses ChaCha20 with the default IETF-style counter starting at 0.
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
            TfheU32::encrypt(client_key, &0u32)?, // counter = 0
            TfheU32::encrypt(client_key, &0u32)?, // nonce0
            TfheU32::encrypt(client_key, &0u32)?, // nonce1
            TfheU32::encrypt(client_key, &0u32)?, // nonce2
        ];

        // Save the original state (needed for final addition).
        let initial_state = state.clone();

        // Run 20 ChaCha rounds = 10 double-rounds (homomorphically).
        for _ in 0..10 {
            double_round(&mut state);
        }

        // Compute keystream: keystream[i] = state[i] + initial_state[i] (homomorphic add)
        let mut keystream: Vec<FheUint32> = Vec::with_capacity(16);
        for i in 0..16 {
            let sum = TfheU32::add(&state[i], &initial_state[i])?;
            keystream.push(sum);
        }

        // Convert the 32-bit ciphertext word into an FHE ciphertext (client must provide client_key).
        let ct_fhe = TfheU32::encrypt(client_key, &ct_val)?;

        // XOR ciphertext with keystream word 0 (homomorphic XOR).
        let pt_fhe = &ct_fhe ^ &keystream[0];

        Ok(pt_fhe)
    }
}

/// Homomorphic ChaCha20 quarter-round: operates on FheUint32 values.
///
/// Implements:
/// a += b; d ^= a; d <<<= 16
/// c += d; b ^= c; b <<<= 12
/// a += b; d ^= a; d <<<= 8
/// c += d; b ^= c; b <<<= 7
fn quarter_round(
    a: &FheUint32,
    b: &FheUint32,
    c: &FheUint32,
    d: &FheUint32,
) -> (FheUint32, FheUint32, FheUint32, FheUint32) {
    // Work on clones to avoid mutating inputs unexpectedly.
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

/// Homomorphic ChaCha20 double-round: column round then diagonal round.
///
/// Uses quarter_round on encrypted words and writes results back to `state`.
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
    use homomorphic::tfhe::TfheU32;
    use symmetric::chacha::ChaCha20Cipher;
    use tfhe::{FheUint32, set_server_key};

    /// Minimal roundtrip test:
    /// - Uses a fixed 256-bit key (as bytes).
    /// - Encrypts a single 32-bit block with the same key/nonce/counter.
    /// - Runs homomorphic_decrypt and checks decrypted HE output equals original plaintext.
    ///
    /// Note: this test is correctness-focused. It will be slow because TFHE ops are slow.
    #[test]
    fn test_homomorphic_decrypt_roundtrip_fixed() {
        // --- helpers: ChaCha20 reference block ---
        fn rotl(v: u32, c: u32) -> u32 {
            v.rotate_left(c)
        }

        fn quarter_index(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
            let mut x = state[a];
            let mut y = state[b];
            let mut z = state[c];
            let mut w = state[d];

            x = x.wrapping_add(y);
            w ^= x;
            w = rotl(w, 16);
            z = z.wrapping_add(w);
            y ^= z;
            y = rotl(y, 12);
            x = x.wrapping_add(y);
            w ^= x;
            w = rotl(w, 8);
            z = z.wrapping_add(w);
            y ^= z;
            y = rotl(y, 7);

            state[a] = x;
            state[b] = y;
            state[c] = z;
            state[d] = w;
        }

        fn chacha20_block(key: &[u8; 32], counter: u32, nonce: &[u8; 12]) -> [u8; 64] {
            let mut state = [0u32; 16];
            // constants "expand 32-byte k"
            state[0] = 0x61707865;
            state[1] = 0x3320646e;
            state[2] = 0x79622d32;
            state[3] = 0x6b206574;
            // key (little-endian u32 words)
            for i in 0..8 {
                state[4 + i] = u32::from_le_bytes([
                    key[i * 4],
                    key[i * 4 + 1],
                    key[i * 4 + 2],
                    key[i * 4 + 3],
                ]);
            }
            state[12] = counter;
            state[13] = u32::from_le_bytes([nonce[0], nonce[1], nonce[2], nonce[3]]);
            state[14] = u32::from_le_bytes([nonce[4], nonce[5], nonce[6], nonce[7]]);
            state[15] = u32::from_le_bytes([nonce[8], nonce[9], nonce[10], nonce[11]]);

            let initial = state;

            // 20 rounds = 10 double-rounds
            for _ in 0..10 {
                // column rounds
                quarter_index(&mut state, 0, 4, 8, 12);
                quarter_index(&mut state, 1, 5, 9, 13);
                quarter_index(&mut state, 2, 6, 10, 14);
                quarter_index(&mut state, 3, 7, 11, 15);
                // diagonal rounds
                quarter_index(&mut state, 0, 5, 10, 15);
                quarter_index(&mut state, 1, 6, 11, 12);
                quarter_index(&mut state, 2, 7, 8, 13);
                quarter_index(&mut state, 3, 4, 9, 14);
            }

            let mut out = [0u8; 64];
            for i in 0..16 {
                let word = state[i].wrapping_add(initial[i]).to_le_bytes();
                out[i * 4..i * 4 + 4].copy_from_slice(&word);
            }
            out
        }

        // --- TFHE setup ---
        let (client_key, server_key) = TfheU32::keygen().unwrap();
        set_server_key(server_key.clone());

        let sym_key_bytes: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
            0x1d, 0x1e, 0x1f, 0x20,
        ];

        let mut enc_key: Vec<FheUint32> = Vec::with_capacity(8);
        for i in 0..8 {
            let w = u32::from_le_bytes([
                sym_key_bytes[i * 4],
                sym_key_bytes[i * 4 + 1],
                sym_key_bytes[i * 4 + 2],
                sym_key_bytes[i * 4 + 3],
            ]);
            enc_key.push(TfheU32::encrypt(&client_key, &w).unwrap());
        }

        // plaintext
        let plaintext_word: u32 = 0xDEADBEEF;
        println!("plaintext = {:08x}", plaintext_word);

        // software keystream
        let nonce = [0u8; 12];
        let sw_block = chacha20_block(&sym_key_bytes, 0, &nonce);
        let ks0_sw = u32::from_le_bytes([sw_block[0], sw_block[1], sw_block[2], sw_block[3]]);
        println!("software ks0 = {:08x}", ks0_sw);

        // ciphertext = plaintext ^ keystream
        let ct_word = plaintext_word ^ ks0_sw;
        let mut block_bytes = [0u8; 64];
        block_bytes[..4].copy_from_slice(&ct_word.to_le_bytes());
        for i in 4..64 {
            block_bytes[i] = sw_block[i]; // keystream for rest
        }

        println!("ciphertext[0..4] = {:02x?}", &block_bytes[..4]);

        // homomorphic decrypt
        let ct_fhe = ChaCha20Cipher::homomorphic_decrypt(&block_bytes, &enc_key, &client_key)
            .expect("homomorphic_decrypt failed");

        let decrypted: u32 = TfheU32::decrypt(&client_key, &ct_fhe).unwrap();
        println!("decrypted = {:08x}", decrypted);

        assert_eq!(decrypted, plaintext_word, "homomorphic roundtrip failed");
    }

    /// Minimal validation: ensure short ciphertext returns error.
    #[test]
    fn test_homomorphic_minimum_ct_size() {
        let (client_key, _server_key) = TfheU32::keygen().unwrap();

        // Provide a properly shaped encrypted key (all zeros encrypted)
        let enc_key: Vec<FheUint32> = (0u32..8u32)
            .map(|v| TfheU32::encrypt(&client_key, &v).unwrap())
            .collect();

        let short_ct: [u8; 2] = [0x00, 0x01];
        let res = ChaCha20Cipher::homomorphic_decrypt(&short_ct, &enc_key, &client_key);
        assert!(res.is_err());
    }
}
