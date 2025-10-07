use criterion::{Criterion, criterion_group, criterion_main};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use zk::{
    ZeroKnowledge,
    plonk::{FibonacciTerm, Statement, Witness},
};

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

fn bench_fibonacci_proof(c: &mut Criterion) {
    let params = FibonacciTerm::setup().expect("setup failed");

    let result = fib100_as_field::<F>();
    let statement = Statement { result };
    let witness = Witness {};

    c.bench_function("FibonacciTerm proof generation", |b| {
        b.iter(|| {
            let _proof = FibonacciTerm::prove(&params, &statement, &witness).expect("prove failed");
        });
    });

    c.bench_function("FibonacciTerm proof verification", |b| {
        let proof = FibonacciTerm::prove(&params, &statement, &witness).expect("prove failed");

        b.iter(|| {
            let _verified =
                FibonacciTerm::verify(&params, &statement, &proof).expect("verify failed");
        });
    });
}

fn fib100_as_field<F: plonky2::field::types::Field>() -> F {
    let mut a = F::ZERO;
    let mut b = F::ONE;
    for _ in 0..99 {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

criterion_group!(benches, bench_fibonacci_proof);
criterion_main!(benches);
