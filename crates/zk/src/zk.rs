use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, prepare_verifying_key};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{FieldVar, fp::FpVar},
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_snark::SNARK;
use ark_std::rand::rngs::OsRng;

/// ==========================
/// Voting Circuit Definition
/// ==========================
#[derive(Clone)]
pub struct VotingCircuit {
    pub votes: Vec<Vec<u8>>, // private witness
    pub totals: Vec<u8>,     // public input
}

impl ConstraintSynthesizer<Fr> for VotingCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let num_voters = self.votes.len();
        let num_candidates = self.totals.len();

        // Public inputs
        let total_vars: Vec<FpVar<Fr>> = self
            .totals
            .into_iter()
            .map(|t| FpVar::new_input(cs.clone(), || Ok(Fr::from(t as u64))))
            .collect::<Result<_, _>>()?;

        // Private votes
        let vote_vars: Vec<Vec<FpVar<Fr>>> = self
            .votes
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|v| {
                        let var = FpVar::new_witness(cs.clone(), || Ok(Fr::from(v as u64)))?;

                        // Enforce binary: v*(v-1) == 0
                        let one = FpVar::constant(Fr::from(1u64));
                        let zero = FpVar::constant(Fr::from(0u64));
                        (&var * (&var - &one)).enforce_equal(&zero)?;

                        Ok(var)
                    })
                    .collect::<Result<Vec<_>, SynthesisError>>()
            })
            .collect::<Result<_, _>>()?;

        // Sum constraints
        for j in 0..num_candidates {
            let mut sum = FpVar::<Fr>::constant(Fr::from(0u64));

            for i in 0..num_voters {
                sum += &vote_vars[i][j];
            }

            sum.enforce_equal(&total_vars[j])?;
        }

        Ok(())
    }
}

/// ==========================
/// Serializable Key Container
/// ==========================
pub struct ZkKeys {
    pub pk: Vec<u8>,
    pub vk: Vec<u8>,
}

/// ==========================
/// Setup
/// ==========================
pub fn setup(votes: &[Vec<u8>], totals: &[u8]) -> ZkKeys {
    let circuit = VotingCircuit {
        votes: votes.to_vec(),
        totals: totals.to_vec(),
    };

    let mut rng = OsRng;

    let (pk, vk) =
        Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng).expect("setup failed");

    // Serialize using canonical serialization
    let mut pk_bytes = Vec::new();
    pk.serialize_compressed(&mut pk_bytes)
        .expect("pk serialization failed");

    let mut vk_bytes = Vec::new();
    vk.serialize_compressed(&mut vk_bytes)
        .expect("vk serialization failed");

    ZkKeys {
        pk: pk_bytes,
        vk: vk_bytes,
    }
}

/// ==========================
/// Prove
/// ==========================
pub fn prove(keys: &ZkKeys, votes: Vec<Vec<u8>>, totals: Vec<u8>) -> Vec<u8> {
    let pk = ProvingKey::<Bls12_381>::deserialize_compressed(&*keys.pk)
        .expect("pk deserialization failed");

    let circuit = VotingCircuit { votes, totals };

    let mut rng = OsRng;

    let proof =
        Groth16::<Bls12_381>::prove(&pk, circuit, &mut rng).expect("proof generation failed");

    let mut proof_bytes = Vec::new();
    proof
        .serialize_compressed(&mut proof_bytes)
        .expect("proof serialization failed");

    proof_bytes
}

/// ==========================
/// Verify
/// ==========================
pub fn verify(keys: &ZkKeys, proof_bytes: &[u8], totals: Vec<u8>) -> bool {
    let vk = VerifyingKey::<Bls12_381>::deserialize_compressed(&*keys.vk)
        .expect("vk deserialization failed");

    let proof = Proof::<Bls12_381>::deserialize_compressed(proof_bytes)
        .expect("proof deserialization failed");

    let pvk = prepare_verifying_key(&vk);

    let public_inputs: Vec<Fr> = totals.into_iter().map(|t| Fr::from(t as u64)).collect();

    Groth16::<Bls12_381>::verify_with_processed_vk(&pvk, &public_inputs, &proof).unwrap_or(false)
}
