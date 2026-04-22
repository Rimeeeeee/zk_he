use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand::RngCore;
use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
use tfhe::{ConfigBuilder, generate_keys, set_server_key};
use transciphering::{Transcipher, chacha::ChaChaTfheTranscipher};

fn bench_transciphering(c: &mut Criterion) {
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    let sym_key = ChaCha20Cipher::keygen();
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    let mut group = c.benchmark_group("transciphering");

    for size in [32usize, 1024, 4096] {
        let plaintext = vec![0x42; size];
        let (ciphertext, enc_key_cts) =
            ChaChaTfheTranscipher::transcipher_encrypt(&sym_key, &nonce, &client_key, &plaintext)
                .expect("transcipher encrypt");

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("encrypt", size),
            &plaintext,
            |b, plaintext| {
                b.iter(|| {
                    let result = ChaChaTfheTranscipher::transcipher_encrypt(
                        &sym_key,
                        &nonce,
                        &client_key,
                        black_box(plaintext),
                    )
                    .expect("transcipher encrypt");
                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decrypt", size),
            &(ciphertext, enc_key_cts),
            |b, (ciphertext, enc_key_cts)| {
                b.iter(|| {
                    let result = ChaChaTfheTranscipher::transcipher_decrypt(
                        &nonce,
                        &client_key,
                        enc_key_cts,
                        black_box(ciphertext),
                    )
                    .expect("transcipher decrypt");
                    black_box(result);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("roundtrip", size),
            &plaintext,
            |b, plaintext| {
                b.iter(|| {
                    let (ciphertext, enc_key_cts) = ChaChaTfheTranscipher::transcipher_encrypt(
                        &sym_key,
                        &nonce,
                        &client_key,
                        black_box(plaintext),
                    )
                    .expect("transcipher encrypt");

                    let result = ChaChaTfheTranscipher::transcipher_decrypt(
                        &nonce,
                        &client_key,
                        &enc_key_cts,
                        black_box(&ciphertext),
                    )
                    .expect("transcipher decrypt");

                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_transciphering);
criterion_main!(benches);
