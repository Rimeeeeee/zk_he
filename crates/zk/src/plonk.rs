use plonky2::field::types::Field;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{GenericConfig, PoseidonGoldilocksConfig};

use crate::ZeroKnowledge;
use crate::ZkError;

/// A struct representing the Fibonacci term proving system.
pub struct FibonacciTerm {
    pub term: u32,
}

// Define parameters for this circuit
const D: usize = 2;
type C = PoseidonGoldilocksConfig;
type F = <C as GenericConfig<D>>::F;

/// Parameters, Statement, Witness, and Proof structures
pub struct Parameters {
    pub circuit_data: plonky2::plonk::circuit_data::CircuitData<F, C, D>,
}

pub struct Statement {
    pub result: F,
}

pub struct Witness {}

pub struct Proof {
    pub proof: plonky2::plonk::proof::ProofWithPublicInputs<F, C, D>,
}

impl ZeroKnowledge for FibonacciTerm {
    type Parameters = Parameters;
    type Statement = Statement;
    type Witness = Witness;
    type Proof = Proof;

    fn setup() -> Result<Self::Parameters, ZkError> {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, D>::new(config);

        // Build a circuit for the statement: "I know the 100th term of the Fibonacci sequence, starting from 0 and 1".
        let initial_a = builder.constant(F::ZERO);
        let initial_b = builder.constant(F::ONE);

        let mut prev_target = initial_a;
        let mut cur_target = initial_b;
        for _ in 0..99 {
            let temp = builder.add(prev_target, cur_target);
            prev_target = cur_target;
            cur_target = temp;
        }

        builder.register_public_input(cur_target);

        let circuit_data = builder.build::<C>();

        Ok(Parameters { circuit_data })
    }

    fn prove(
        params: &Self::Parameters,
        statement: &Self::Statement,
        _witness: &Self::Witness,
    ) -> Result<Self::Proof, ZkError> {
        let mut pw = PartialWitness::new();
        let _ = pw.set_target(
            params.circuit_data.prover_only.public_inputs[0],
            statement.result,
        );

        // Use the returned Result properly
        let proof_data = params
            .circuit_data
            .prove(pw)
            .map_err(|_| ZkError::ProveError)?;

        Ok(Proof { proof: proof_data })
    }

    fn verify(
        params: &Self::Parameters,
        statement: &Self::Statement,
        proof: &Self::Proof,
    ) -> Result<bool, ZkError> {
        let res = params.circuit_data.verify(proof.proof.clone());
        if res.is_err() {
            return Ok(false);
        }

        // Optional: check that the public input matches statement
        let public_input = proof.proof.public_inputs[0];
        Ok(public_input == statement.result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::field::types::Field;

    // Helper function to compute the 100th Fibonacci number as a field element
    fn fib100_as_field<F: Field>() -> F {
        let mut a = F::ZERO;
        let mut b = F::ONE;
        for _ in 0..99 {
            let c = a + b;
            a = b;
            b = c;
        }
        b
    }

    #[test]
    fn test_fibonacci_proof_and_verify() {
        // Setup the circuit
        let params = FibonacciTerm::setup().expect("setup failed");

        // Compute the correct result
        let result = fib100_as_field::<F>();

        let statement = Statement { result };
        let witness = Witness {};

        // Prove
        let proof = FibonacciTerm::prove(&params, &statement, &witness).expect("prove failed");

        // Verify
        let verified = FibonacciTerm::verify(&params, &statement, &proof).expect("verify failed");
        assert!(verified, "Proof did not verify");
    }
}
