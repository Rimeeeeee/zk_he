use criterion::{Criterion, criterion_group, criterion_main};
use homomorphic::{HomomorphicEncryption, tfhe::TfheU32};
use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
use transciphering::HomomorphicDecrypt;

/// Benchmark homomorphic decryption (transciphering) of a 32-bit ChaCha20 block
fn bench_tranciphering(c: &mut Criterion) {
    // --- Key generation ---
    let (client_key, _server_key) = TfheU32::keygen().unwrap();

    // --- Generate a random ChaCha20 key and encrypt under HE ---
    let sym_key: [u8; 32] = rand::random();
    let mut enc_key = Vec::new();
    for chunk in sym_key.chunks(4) {
        let word = u32::from_le_bytes(chunk.try_into().unwrap());
        enc_key.push(TfheU32::encrypt(&client_key, &word).unwrap());
    }

    // --- Symmetric plaintext block ---
    let plaintext_block: u32 = rand::random();
    let mut sym_ct_bytes = plaintext_block.to_le_bytes();

    // --- Symmetric encryption (ChaCha20) ---
    let _ciphertext = ChaCha20Cipher::encrypt(&sym_key, &[0u8; 12], &mut sym_ct_bytes).unwrap();

    // --- Benchmark homomorphic decryption ---
    c.bench_function("ChaCha20 → TFHE transciphering", |b| {
        b.iter(|| {
            let _pt_fhe =
                ChaCha20Cipher::homomorphic_decrypt(&sym_ct_bytes, &enc_key, &client_key).unwrap();
        });
    });
}

criterion_group!(benches, bench_tranciphering);
criterion_main!(benches);
