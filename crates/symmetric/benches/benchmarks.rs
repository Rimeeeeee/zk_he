use criterion::{Criterion, criterion_group, criterion_main};
use symmetric::{
    SymmetricCipher, aes::AesCipher, camellia::CamelliaCipher, chacha::ChaCha20Cipher,
};

fn bench_symmetric(c: &mut Criterion) {
    let key_aes = AesCipher::keygen();
    let iv = [0u8; 16];
    let iv_cha = [0u8; 12];
    let mut plaintext = vec![42u8; 1024]; // 1KB

    c.bench_function("AES-256-CBC encrypt 1KB", |b| {
        b.iter(|| {
            let _ = AesCipher::encrypt(&key_aes, &iv, &mut plaintext).unwrap();
        })
    });

    let key_chacha = ChaCha20Cipher::keygen();
    c.bench_function("ChaCha20 encrypt 1KB", |b| {
        b.iter(|| {
            let _ = ChaCha20Cipher::encrypt(&key_chacha, &iv_cha, &mut plaintext).unwrap();
        })
    });

    let key_camellia = CamelliaCipher::keygen();
    c.bench_function("Camellia-256 encrypt 1KB", |b| {
        b.iter(|| {
            let _ = CamelliaCipher::encrypt(&key_camellia, &iv, &mut plaintext).unwrap();
        })
    });
}

criterion_group!(benches, bench_symmetric);
criterion_main!(benches);
