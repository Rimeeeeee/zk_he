use criterion::{Criterion, criterion_group, criterion_main};
use homomorphic::{HomomorphicEncryption, tfhe::TfheU32};

fn bench_he(c: &mut Criterion) {
    let (pk, sk) = TfheU32::keygen().unwrap();
    let a: u32 = 42;
    let b: u32 = 7;

    c.bench_function("encrypt u32", |bch| {
        bch.iter(|| TfheU32::encrypt(&pk, &a).unwrap())
    });

    let ct_a = TfheU32::encrypt(&pk, &a).unwrap();
    let ct_b = TfheU32::encrypt(&pk, &b).unwrap();

    c.bench_function("add", |bch| {
        bch.iter(|| TfheU32::add(&ct_a, &ct_b).unwrap())
    });

    c.bench_function("mul", |bch| {
        bch.iter(|| TfheU32::mul(&ct_a, &ct_b).unwrap())
    });

    c.bench_function("decrypt", |bch| {
        bch.iter(|| TfheU32::decrypt(&pk, &ct_a).unwrap())
    });
}

criterion_group!(benches, bench_he);
criterion_main!(benches);
