use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use rand::RngCore;
use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
use tfhe::{ConfigBuilder, generate_keys, set_server_key};
use zk_transciphering::{DataProof, ProofParameters, ProvenChaChaTfheTranscipher};

fn bench_zk_transciphering(c: &mut Criterion) {
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    set_server_key(server_key);

    let sym_key = ChaCha20Cipher::keygen();
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    let mut group = c.benchmark_group("zk_transciphering");

    for size in [32usize, 256, 1024] {
        let plaintext = vec![0x24; size];
        let parameters = ProofParameters::setup(size).expect("setup");
        let package = ProvenChaChaTfheTranscipher::transcipher_encrypt_and_prove(
            &parameters,
            &sym_key,
            &nonce,
            &client_key,
            &plaintext,
        )
        .expect("encrypt and prove");

        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("prove", size),
            &plaintext,
            |b, plaintext| {
                b.iter(|| {
                    let package = ProvenChaChaTfheTranscipher::transcipher_encrypt_and_prove(
                        &parameters,
                        &sym_key,
                        &nonce,
                        &client_key,
                        black_box(plaintext),
                    )
                    .expect("encrypt and prove");
                    black_box(package);
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("verify", size), &package, |b, package| {
            b.iter(|| {
                let verified =
                    ProvenChaChaTfheTranscipher::verify_package(&parameters, black_box(package));
                black_box(verified);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("guarded_decrypt", size),
            &package,
            |b, package| {
                b.iter(|| {
                    let plaintext = ProvenChaChaTfheTranscipher::verify_and_decrypt(
                        &parameters,
                        &nonce,
                        &client_key,
                        black_box(package),
                    )
                    .expect("verify and decrypt");
                    black_box(plaintext);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("proof_only", size),
            &plaintext,
            |b, plaintext| {
                b.iter(|| {
                    let proof = DataProof::prove(&parameters, black_box(plaintext)).expect("proof");
                    black_box(proof);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_zk_transciphering);
criterion_main!(benches);
