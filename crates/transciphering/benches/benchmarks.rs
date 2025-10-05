use criterion::{Criterion, criterion_group, criterion_main};
use rand::RngCore;
use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
use tfhe::{ConfigBuilder, generate_keys, set_server_key};
use transciphering::{Transcipher, chacha::ChaChaTfheTranscipher};

/// Benchmark full ChaCha20 → TFHE transciphering pipeline
fn bench_transciphering(c: &mut Criterion) {
    // --- TFHE KeyGen ---
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    // --- Symmetric key + nonce ---
    let sym_key = ChaCha20Cipher::keygen();
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    // --- Random plaintext block ---
    let plaintext: [u8; 32] = rand::random();

    // --- Encrypt via transciphering ---
    let (ciphertext, enc_key_cts) =
        ChaChaTfheTranscipher::transcipher_encrypt(&sym_key, &nonce, &client_key, &plaintext)
            .expect("transcipher encrypt");

    // --- Benchmark homomorphic decryption ---
    c.bench_function("ChaCha20 + TFHE transcipher decrypt", |b| {
        b.iter(|| {
            let _decrypted = ChaChaTfheTranscipher::transcipher_decrypt(
                &nonce,
                &client_key,
                &enc_key_cts,
                &ciphertext,
            )
            .unwrap();
        });
    });
}

criterion_group!(benches, bench_transciphering);
criterion_main!(benches);
