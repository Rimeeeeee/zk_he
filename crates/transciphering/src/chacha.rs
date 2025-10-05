use crate::{HomomorphicEncryption, SymmetricCipher, Transcipher, TranscipherError};
use homomorphic::tfhe::TfheU32;
use symmetric::chacha::ChaCha20Cipher;

pub struct ChaChaTfheTranscipher;

impl Transcipher<ChaCha20Cipher, TfheU32> for ChaChaTfheTranscipher {
    fn transcipher_encrypt(
        sym_key: &<ChaCha20Cipher as SymmetricCipher>::Key,
        sym_nonce: &<ChaCha20Cipher as SymmetricCipher>::Nonce,
        he_sk: &<TfheU32 as HomomorphicEncryption>::SecretKey,
        plaintext: &[u8],
    ) -> Result<(Vec<u8>, Vec<<TfheU32 as HomomorphicEncryption>::Ciphertext>), TranscipherError>
    {
        // Encrypt plaintext using ChaCha20
        let mut pt = plaintext.to_vec();
        let ciphertext = ChaCha20Cipher::encrypt(sym_key, sym_nonce, &mut pt)
            .map_err(|_| TranscipherError::SymmetricError)?;

        //  Split 256-bit key (32 bytes) into 8 × 4-byte chunks
        let mut key_cts = Vec::with_capacity(8);
        for chunk in sym_key.chunks_exact(4) {
            let part = u32::from_le_bytes(chunk.try_into().unwrap());
            //  Encrypt each 32-bit chunk under TFHE
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
        // Decrypt each homomorphic key chunk
        let mut sym_key = [0u8; 32];
        for (i, ct) in sym_key_cts.iter().enumerate() {
            let part = TfheU32::decrypt(he_sk, ct).map_err(|_| TranscipherError::HeError)?;
            let bytes = part.to_le_bytes();
            sym_key[i * 4..(i + 1) * 4].copy_from_slice(&bytes);
        }

        //  Decrypt ChaCha20 ciphertext
        let mut ct = ciphertext.to_vec();
        let plaintext = ChaCha20Cipher::decrypt(&sym_key, sym_nonce, &mut ct)
            .map_err(|_| TranscipherError::SymmetricError)?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use tfhe::{ConfigBuilder, generate_keys};

    #[test]
    fn test_chacha_tfhe_transcipher_roundtrip() {
        // Generate symmetric key + nonce
        let sym_key = ChaCha20Cipher::keygen();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        // Generate homomorphic keys
        let config = ConfigBuilder::default().build();
        let (client_key, _server_key) = generate_keys(config);

        // Example plaintext
        let plaintext = b"transciphering test message";

        // Encrypt
        let (ciphertext, key_cts) =
            ChaChaTfheTranscipher::transcipher_encrypt(&sym_key, &nonce, &client_key, plaintext)
                .expect("encrypt");

        // Decrypt
        let decrypted =
            ChaChaTfheTranscipher::transcipher_decrypt(&nonce, &client_key, &key_cts, &ciphertext)
                .expect("decrypt");

        // Assert equality
        assert_eq!(plaintext.to_vec(), decrypted);
    }
}
