use criterion::{Criterion, black_box, criterion_group, criterion_main};
use plonky2::field::types::Field;
use zk::groth;
use zk::plonk;

// ----------------------------
// Sample vote generator (u8)
// ----------------------------
fn sample_votes_u8(num_voters: usize, num_candidates: usize) -> (Vec<Vec<u8>>, Vec<u8>) {
    let mut votes = vec![vec![0u8; num_candidates]; num_voters];

    for i in 0..num_voters {
        votes[i][i % num_candidates] = 1;
    }

    let mut totals = vec![0u8; num_candidates];
    for row in &votes {
        for (j, v) in row.iter().enumerate() {
            totals[j] += v;
        }
    }

    (votes, totals)
}

// ----------------------------
// Groth16 Benchmarks
// ----------------------------
fn bench_groth(c: &mut Criterion) {
    let (votes, totals) = sample_votes_u8(10, 3);

    // Setup
    c.bench_function("groth_setup_10v_3c", |b| {
        b.iter(|| {
            black_box(groth::setup(&votes, &totals));
        })
    });

    let keys = groth::setup(&votes, &totals);

    // Prove
    c.bench_function("groth_prove_10v_3c", |b| {
        b.iter(|| {
            black_box(groth::prove(&keys, votes.clone(), totals.clone()));
        })
    });

    let proof = groth::prove(&keys, votes.clone(), totals.clone());

    // Verify
    c.bench_function("groth_verify_10v_3c", |b| {
        b.iter(|| {
            black_box(groth::verify(&keys, &proof, totals.clone()));
        })
    });
}

// ----------------------------
// Plonky2 Benchmarks
// ----------------------------
fn bench_plonk(c: &mut Criterion) {
    let num_voters = 10;
    let num_candidates = 3;

    let (votes_u8, totals_u8) = sample_votes_u8(num_voters, num_candidates);

    let params = plonk::setup(num_voters, num_candidates);

    // Convert to field format
    let votes_field: Vec<Vec<_>> = votes_u8
        .iter()
        .map(|row| {
            row.iter()
                .map(|&v| plonk::F::from_canonical_u64(v as u64))
                .collect()
        })
        .collect();

    let totals_field: Vec<_> = totals_u8
        .iter()
        .map(|&v| plonk::F::from_canonical_u64(v as u64))
        .collect();

    let witness = plonk::Witness {
        votes: votes_field.clone(),
    };

    let statement = plonk::Statement {
        totals: totals_field.clone(),
    };

    // Setup benchmark (already done once, but we measure it separately)
    c.bench_function("plonky_setup_10v_3c", |b| {
        b.iter(|| {
            black_box(plonk::setup(num_voters, num_candidates));
        })
    });

    // Prove
    c.bench_function("plonky_prove_10v_3c", |b| {
        b.iter(|| {
            black_box(plonk::prove(&params, &witness));
        })
    });

    let proof = plonk::prove(&params, &witness);

    // Verify
    c.bench_function("plonky_verify_10v_3c", |b| {
        b.iter(|| {
            black_box(plonk::verify(&params, &statement, &proof));
        })
    });
}

criterion_group!(benches, bench_groth, bench_plonk);
criterion_main!(benches);
