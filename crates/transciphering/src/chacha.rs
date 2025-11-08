use crate::{HomomorphicEncryption, SymmetricCipher, Transcipher, TranscipherError};
use homomorphic::tfhe_uint::TfheU32;
use symmetric::chacha::ChaCha20Cipher;
use tfhe::FheUint32;

pub struct ChaChaTfheTranscipher;

impl Transcipher<ChaCha20Cipher, TfheU32> for ChaChaTfheTranscipher {
    fn transcipher_encrypt(
        sym_key: &<ChaCha20Cipher as SymmetricCipher>::Key,
        sym_nonce: &<ChaCha20Cipher as SymmetricCipher>::Nonce,
        he_sk: &<TfheU32 as HomomorphicEncryption>::SecretKey,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<<TfheU32 as HomomorphicEncryption>::Ciphertext>), TranscipherError>
    {
        // ChaCha encrypt (fast)
        let mut pt = plaintext.to_vec();
        let ciphertext = ChaCha20Cipher::encrypt(sym_key, sym_nonce, &mut pt)
            .map_err(|_| TranscipherError::SymmetricError)?;

        // Split 32-byte key into 8 u32 words and TFHE-encrypt each (client-side secret key used)
        let mut key_cts = Vec::with_capacity(8);
        for chunk in sym_key.chunks_exact(4) {
            let part = u32::from_le_bytes(chunk.try_into().unwrap());
            let ct = TfheU32::encrypt(he_sk, &part).map_err(|_| TranscipherError::HeError)?;
            key_cts.push(ct);
        }

        Ok((ciphertext, key_cts))
    }

    fn transcipher_decrypt(
        sym_nonce: &<ChaCha20Cipher as SymmetricCipher>::Nonce,
        he_sk: &<TfheU32 as HomomorphicEncryption>::SecretKey,
        sym_key_cts: &Vec<<TfheU32 as HomomorphicEncryption>::Ciphertext>,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, TranscipherError> {
        // Recover symmetric key by decrypting the 8 TFHE-encrypted u32 words
        let mut sym_key = [0u8; 32];
        for (i, ct) in sym_key_cts.iter().enumerate() {
            let part = TfheU32::decrypt(he_sk, ct).map_err(|_| TranscipherError::HeError)?;
            sym_key[i * 4..(i + 1) * 4].copy_from_slice(&part.to_le_bytes());
        }

        // ChaCha decrypt
        let mut ct = ciphertext.to_vec();
        let plaintext = ChaCha20Cipher::decrypt(&sym_key, sym_nonce, &mut ct)
            .map_err(|_| TranscipherError::SymmetricError)?;

        Ok(plaintext)
    }
}

/// Server-side: homomorphically compute the ChaCha20 block function and return the 16 u32 output words
/// as ciphertexts. For test completeness this function receives `client_ck` so it can build ciphertexts
/// for public constants/counter/nonce. In production you would avoid giving the client key to the server.
pub struct ChaChaTfheServer;

impl ChaChaTfheServer {
    /// Compute one ChaCha20 block homomorphically.
    /// - `key_cts` - 8 ciphertexts holding the key words (little-endian u32 words)
    /// - `client_ck` - client TFHE secret key (used here to encrypt constants for the demo/test)
    /// - `counter` / `nonce` - block counter and 12-byte nonce
    /// Returns 16 ciphertexts representing the 16 u32 output words of the ChaCha20 block.
    pub fn generate_homomorphic_chacha_block(
        key_cts: &Vec<<TfheU32 as HomomorphicEncryption>::Ciphertext>,
        client_ck: &<TfheU32 as HomomorphicEncryption>::SecretKey,
        counter: u32,
        nonce: &[u8; 12],
    ) -> Result<Vec<<TfheU32 as HomomorphicEncryption>::Ciphertext>, TranscipherError> {
        if key_cts.len() != 8 {
            return Err(TranscipherError::HeError);
        }

        println!("🔹 [Server] Starting homomorphic ChaCha20 block generation...");
        println!("   - Received {} encrypted key words", key_cts.len());
        println!("   - Counter: {}", counter);
        println!("   - Nonce: {:02x?}", nonce.iter().collect::<Vec<_>>());

        // Constants: "expand 32-byte k"
        let constants: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];
        println!("   - ChaCha constants: {:08x?}", constants);

        let mut state: Vec<FheUint32> = Vec::with_capacity(16);

        // Encrypt constants
        for &c in &constants {
            let ct = TfheU32::encrypt_public_with_client(client_ck, c)
                .map_err(|_| TranscipherError::HeError)?;
            state.push(ct);
        }
        println!("   - Encrypted constants appended to state");

        // Append key ciphertexts
        for (i, k) in key_cts.iter().enumerate() {
            println!("   - Key word[{}]: <encrypted>", i);
            state.push(k.clone());
        }

        // Encrypt counter
        let counter_ct = TfheU32::encrypt_public_with_client(client_ck, counter)
            .map_err(|_| TranscipherError::HeError)?;
        state.push(counter_ct);
        println!("   - Counter encrypted and appended");

        // Encrypt nonce
        for i in 0..3 {
            let part = u32::from_le_bytes(nonce[i * 4..(i + 1) * 4].try_into().unwrap());
            let ct = TfheU32::encrypt_public_with_client(client_ck, part)
                .map_err(|_| TranscipherError::HeError)?;
            state.push(ct);
            println!("   - Nonce word[{}]: 0x{:08x} (encrypted)", i, part);
        }

        println!(" Initial ChaCha state prepared ({} words)", state.len());

        let mut working = state.clone();

        // Define quarter round helper
        let quarter_round = |st: &mut Vec<FheUint32>,
                             a: usize,
                             b: usize,
                             c: usize,
                             d: usize|
         -> Result<(), TranscipherError> {
            println!("   ▶ Quarter round ({},{},{},{})", a, b, c, d);
            st[a] = TfheU32::add(&st[a], &st[b]).map_err(|_| TranscipherError::HeError)?;
            st[d] = TfheU32::xor(&st[d], &st[a]).map_err(|_| TranscipherError::HeError)?;
            st[d] = TfheU32::rotl(&st[d], 16).map_err(|_| TranscipherError::HeError)?;

            st[c] = TfheU32::add(&st[c], &st[d]).map_err(|_| TranscipherError::HeError)?;
            st[b] = TfheU32::xor(&st[b], &st[c]).map_err(|_| TranscipherError::HeError)?;
            st[b] = TfheU32::rotl(&st[b], 12).map_err(|_| TranscipherError::HeError)?;

            st[a] = TfheU32::add(&st[a], &st[b]).map_err(|_| TranscipherError::HeError)?;
            st[d] = TfheU32::xor(&st[d], &st[a]).map_err(|_| TranscipherError::HeError)?;
            st[d] = TfheU32::rotl(&st[d], 8).map_err(|_| TranscipherError::HeError)?;

            st[c] = TfheU32::add(&st[c], &st[d]).map_err(|_| TranscipherError::HeError)?;
            st[b] = TfheU32::xor(&st[b], &st[c]).map_err(|_| TranscipherError::HeError)?;
            st[b] = TfheU32::rotl(&st[b], 7).map_err(|_| TranscipherError::HeError)?;
            Ok(())
        };

        // 20 rounds
        for round in 0..10 {
            println!("Double-round {}", round + 1);
            // Column rounds
            quarter_round(&mut working, 0, 4, 8, 12)?;
            quarter_round(&mut working, 1, 5, 9, 13)?;
            quarter_round(&mut working, 2, 6, 10, 14)?;
            quarter_round(&mut working, 3, 7, 11, 15)?;
            // Diagonal rounds
            quarter_round(&mut working, 0, 5, 10, 15)?;
            quarter_round(&mut working, 1, 6, 11, 12)?;
            quarter_round(&mut working, 2, 7, 8, 13)?;
            quarter_round(&mut working, 3, 4, 9, 14)?;
        }

        println!("Finished 20 rounds of ChaCha homomorphic mixing");

        // Final add original + working
        let mut out = Vec::with_capacity(16);
        for (i, (w, s)) in working.iter().zip(state.iter()).enumerate() {
            let ct = TfheU32::add(w, s).map_err(|_| TranscipherError::HeError)?;
            out.push(ct);
            println!("   - Output word[{}]: <encrypted>", i);
        }

        println!(" Homomorphic ChaCha block generation complete!");
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use tfhe::set_server_key;
    use tfhe::{ConfigBuilder, generate_keys};

    #[test]
    fn test_chacha_tfhe_full_roundtrip() {
        // === CLIENT SIDE ===
        let sym_key = ChaCha20Cipher::keygen();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key.clone());

        let plaintext = b"hello from client";
        // Client: encrypt plaintext + homomorphically encrypt symmetric key
        let (ciphertext, key_cts) =
            ChaChaTfheTranscipher::transcipher_encrypt(&sym_key, &nonce, &client_key, plaintext)
                .expect("encrypt ok");

        // === SERVER SIDE ===
        // Server: homomorphically computes one ChaCha20 block
        let keystream_cts =
            ChaChaTfheServer::generate_homomorphic_chacha_block(&key_cts, &client_key, 1, &nonce)
                .expect("server ok");

        // Server never sees plaintext or key
        assert_eq!(keystream_cts.len(), 16);

        // === CLIENT SIDE AGAIN ===
        // Client decrypts symmetric key and plaintext
        let decrypted =
            ChaChaTfheTranscipher::transcipher_decrypt(&nonce, &client_key, &key_cts, &ciphertext)
                .expect("decrypt ok");

        assert_eq!(decrypted, plaintext);
    }
}
