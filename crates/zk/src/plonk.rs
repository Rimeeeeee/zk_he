use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};
use plonky2::plonk::proof::ProofWithPublicInputs;

const D: usize = 2;
type C = PoseidonGoldilocksConfig;
pub type F = <C as GenericConfig<D>>::F;

/// ==========================
/// Parameters
/// ==========================
pub struct Parameters {
    pub circuit_data: CircuitData<F, C, D>,
    pub vote_targets: Vec<Vec<Target>>,
}

/// ==========================
/// Statement
/// ==========================
pub struct Statement {
    pub totals: Vec<F>,
}

/// ==========================
/// Witness
/// ==========================
pub struct Witness {
    pub votes: Vec<Vec<F>>,
}

/// ==========================
/// Proof
/// ==========================
pub struct Proof {
    pub proof: ProofWithPublicInputs<F, C, D>,
}

/// ==========================
/// Setup
/// ==========================
pub fn setup(num_voters: usize, num_candidates: usize) -> Parameters {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<F, D>::new(config);

    let mut vote_targets = vec![];

    for _ in 0..num_voters {
        let mut row = vec![];

        for _ in 0..num_candidates {
            let v = builder.add_virtual_target();

            // Enforce binary constraint
            let one = builder.one();
            let v_minus_one = builder.sub(v, one);
            let prod = builder.mul(v, v_minus_one);
            builder.assert_zero(prod);

            row.push(v);
        }

        vote_targets.push(row);
    }

    // Sum constraints
    for j in 0..num_candidates {
        let mut sum = builder.zero();
        for i in 0..num_voters {
            sum = builder.add(sum, vote_targets[i][j]);
        }
        builder.register_public_input(sum);
    }

    let circuit_data = builder.build::<C>();

    Parameters {
        circuit_data,
        vote_targets,
    }
}

/// ==========================
/// Prove
/// ==========================
pub fn prove(params: &Parameters, witness: &Witness) -> Proof {
    let mut pw = PartialWitness::new();

    for i in 0..witness.votes.len() {
        for j in 0..witness.votes[i].len() {
            let _ = pw.set_target(params.vote_targets[i][j], witness.votes[i][j]);
        }
    }

    let proof = params.circuit_data.prove(pw).unwrap();

    Proof { proof }
}

/// ==========================
/// Verify
/// ==========================
pub fn verify(params: &Parameters, statement: &Statement, proof: &Proof) -> bool {
    if params.circuit_data.verify(proof.proof.clone()).is_err() {
        return false;
    }

    // Ensure public inputs match expected totals
    for (i, expected) in statement.totals.iter().enumerate() {
        if proof.proof.public_inputs[i] != *expected {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::field::types::Field;

    fn to_field_matrix(matrix: Vec<Vec<u64>>) -> Vec<Vec<F>> {
        matrix
            .into_iter()
            .map(|row| row.into_iter().map(F::from_canonical_u64).collect())
            .collect()
    }

    fn to_field_vec(vec: Vec<u64>) -> Vec<F> {
        vec.into_iter().map(F::from_canonical_u64).collect()
    }

    #[test]
    fn test_valid_voting_proof() {
        let num_voters = 3;
        let num_candidates = 2;

        let params = setup(num_voters, num_candidates);

        // Votes:
        // V1: [1,0]
        // V2: [0,1]
        // V3: [1,0]
        let votes = to_field_matrix(vec![vec![1, 0], vec![0, 1], vec![1, 0]]);

        // Totals: [2,1]
        let totals = to_field_vec(vec![2, 1]);

        let witness = Witness {
            votes: votes.clone(),
        };
        let statement = Statement {
            totals: totals.clone(),
        };

        let proof = prove(&params, &witness);

        let verified = verify(&params, &statement, &proof);
        assert!(verified, "Valid voting proof failed");
    }

    #[test]
    fn test_invalid_totals_should_fail() {
        let num_voters = 3;
        let num_candidates = 2;

        let params = setup(num_voters, num_candidates);

        let votes = to_field_matrix(vec![vec![1, 0], vec![0, 1], vec![1, 0]]);

        // WRONG totals
        let wrong_totals = to_field_vec(vec![3, 0]);

        let witness = Witness {
            votes: votes.clone(),
        };
        let statement = Statement {
            totals: wrong_totals,
        };

        let proof = prove(&params, &witness);

        let verified = verify(&params, &statement, &proof);
        assert!(!verified, "Invalid totals should not verify");
    }

    #[test]
    #[should_panic]
    fn test_non_binary_vote_should_fail_proving() {
        let num_voters = 2;
        let num_candidates = 2;

        let params = setup(num_voters, num_candidates);

        // Invalid vote value: 2
        let votes = to_field_matrix(vec![vec![2, 0], vec![0, 1]]);

        let witness = Witness { votes };

        // This should panic because binary constraint fails
        let _proof = prove(&params, &witness);
    }
}
