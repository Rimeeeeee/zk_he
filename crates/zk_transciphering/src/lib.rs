use ark_bls12_381::{Bls12_381, Fr};
use ark_groth16::{
    Groth16, PreparedVerifyingKey, Proof, ProvingKey, VerifyingKey, prepare_verifying_key,
};
use ark_r1cs_std::{
    alloc::AllocVar,
    eq::EqGadget,
    fields::{FieldVar, fp::FpVar},
};
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};
use ark_snark::SNARK;
use ark_std::rand::rngs::OsRng;
use tfhe::FheUint32;
use transciphering::{Transcipher, TranscipherError, chacha::ChaChaTfheTranscipher};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DataCommitment {
    pub linear: u64,
    pub quadratic: u64,
}

impl DataCommitment {
    pub fn from_plaintext(plaintext: &[u8]) -> Self {
        let mut linear = 0u64;
        let mut quadratic = 0u64;

        for (index, byte) in plaintext.iter().enumerate() {
            let weight = (index as u64) + 1;
            let value = u64::from(*byte);
            linear += weight * value;
            quadratic += weight * weight * value;
        }

        Self { linear, quadratic }
    }
}

#[derive(Clone)]
struct DataCommitmentCircuit {
    plaintext: Vec<u8>,
    expected: DataCommitment,
}

impl ConstraintSynthesizer<Fr> for DataCommitmentCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let expected_linear = FpVar::new_input(cs.clone(), || Ok(Fr::from(self.expected.linear)))?;
        let expected_quadratic =
            FpVar::new_input(cs.clone(), || Ok(Fr::from(self.expected.quadratic)))?;

        let mut linear = FpVar::<Fr>::constant(Fr::from(0u64));
        let mut quadratic = FpVar::<Fr>::constant(Fr::from(0u64));

        for (index, byte) in self.plaintext.into_iter().enumerate() {
            let byte_var = FpVar::new_witness(cs.clone(), || Ok(Fr::from(byte as u64)))?;
            let weight = (index as u64) + 1;

            linear += byte_var.clone() * FpVar::constant(Fr::from(weight));
            quadratic += byte_var * FpVar::constant(Fr::from(weight * weight));
        }

        linear.enforce_equal(&expected_linear)?;
        quadratic.enforce_equal(&expected_quadratic)?;

        Ok(())
    }
}

pub struct ProofParameters {
    payload_len: usize,
    proving_key: ProvingKey<Bls12_381>,
    verifying_key: VerifyingKey<Bls12_381>,
    prepared_vk: PreparedVerifyingKey<Bls12_381>,
}

impl ProofParameters {
    pub fn setup(payload_len: usize) -> Result<Self, ZkTranscipherError> {
        let dummy_plaintext = vec![0u8; payload_len];
        let expected = DataCommitment::from_plaintext(&dummy_plaintext);
        let circuit = DataCommitmentCircuit {
            plaintext: dummy_plaintext,
            expected,
        };

        let mut rng = OsRng;
        let (proving_key, verifying_key) =
            Groth16::<Bls12_381>::circuit_specific_setup(circuit, &mut rng)
                .map_err(|err| ZkTranscipherError::ZkSetup(err.to_string()))?;
        let prepared_vk = prepare_verifying_key(&verifying_key);

        Ok(Self {
            payload_len,
            proving_key,
            verifying_key,
            prepared_vk,
        })
    }

    pub fn payload_len(&self) -> usize {
        self.payload_len
    }

    pub fn verify_key(&self) -> &VerifyingKey<Bls12_381> {
        &self.verifying_key
    }
}

pub struct DataProof {
    proof: Proof<Bls12_381>,
}

impl DataProof {
    pub fn prove(
        parameters: &ProofParameters,
        plaintext: &[u8],
    ) -> Result<(Self, DataCommitment), ZkTranscipherError> {
        if plaintext.len() != parameters.payload_len {
            return Err(ZkTranscipherError::PayloadLengthMismatch {
                expected: parameters.payload_len,
                actual: plaintext.len(),
            });
        }

        let commitment = DataCommitment::from_plaintext(plaintext);
        let circuit = DataCommitmentCircuit {
            plaintext: plaintext.to_vec(),
            expected: commitment.clone(),
        };
        let mut rng = OsRng;
        let proof = Groth16::<Bls12_381>::prove(&parameters.proving_key, circuit, &mut rng)
            .map_err(|err| ZkTranscipherError::ZkProve(err.to_string()))?;

        Ok((Self { proof }, commitment))
    }

    pub fn verify(&self, parameters: &ProofParameters, commitment: &DataCommitment) -> bool {
        let public_inputs = [Fr::from(commitment.linear), Fr::from(commitment.quadratic)];

        Groth16::<Bls12_381>::verify_with_processed_vk(
            &parameters.prepared_vk,
            &public_inputs,
            &self.proof,
        )
        .unwrap_or(false)
    }
}

pub struct ProvenTranscipherPackage {
    pub ciphertext: Vec<u8>,
    pub encrypted_key_words: Vec<FheUint32>,
    pub commitment: DataCommitment,
    pub proof: DataProof,
}

pub struct ProvenChaChaTfheTranscipher;

impl ProvenChaChaTfheTranscipher {
    pub fn transcipher_encrypt_and_prove(
        parameters: &ProofParameters,
        sym_key: &[u8; 32],
        nonce: &[u8; 12],
        client_key: &tfhe::ClientKey,
        plaintext: &[u8],
    ) -> Result<ProvenTranscipherPackage, ZkTranscipherError> {
        if plaintext.len() != parameters.payload_len {
            return Err(ZkTranscipherError::PayloadLengthMismatch {
                expected: parameters.payload_len,
                actual: plaintext.len(),
            });
        }

        let (ciphertext, encrypted_key_words) =
            ChaChaTfheTranscipher::transcipher_encrypt(sym_key, nonce, client_key, plaintext)
                .map_err(ZkTranscipherError::Transcipher)?;
        let (proof, commitment) = DataProof::prove(parameters, plaintext)?;

        Ok(ProvenTranscipherPackage {
            ciphertext,
            encrypted_key_words,
            commitment,
            proof,
        })
    }

    pub fn verify_package(
        parameters: &ProofParameters,
        package: &ProvenTranscipherPackage,
    ) -> bool {
        package.proof.verify(parameters, &package.commitment)
    }

    pub fn verify_and_decrypt(
        parameters: &ProofParameters,
        nonce: &[u8; 12],
        client_key: &tfhe::ClientKey,
        package: &ProvenTranscipherPackage,
    ) -> Result<Vec<u8>, ZkTranscipherError> {
        if !Self::verify_package(parameters, package) {
            return Err(ZkTranscipherError::InvalidProof);
        }

        ChaChaTfheTranscipher::transcipher_decrypt(
            nonce,
            client_key,
            &package.encrypted_key_words,
            &package.ciphertext,
        )
        .map_err(ZkTranscipherError::Transcipher)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ZkTranscipherError {
    #[error("payload length mismatch: expected {expected}, got {actual}")]
    PayloadLengthMismatch { expected: usize, actual: usize },

    #[error("transciphering failed: {0:?}")]
    Transcipher(TranscipherError),

    #[error("zk setup failed: {0}")]
    ZkSetup(String),

    #[error("zk proof generation failed: {0}")]
    ZkProve(String),

    #[error("zk proof did not verify")]
    InvalidProof,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;
    use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
    use tfhe::{ConfigBuilder, generate_keys, set_server_key};

    #[test]
    fn proof_verified_before_decrypt_roundtrip() {
        let payload = b"proof-gated transcipher result!!!";
        let parameters = ProofParameters::setup(payload.len()).expect("setup");

        let sym_key = ChaCha20Cipher::keygen();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);

        let package = ProvenChaChaTfheTranscipher::transcipher_encrypt_and_prove(
            &parameters,
            &sym_key,
            &nonce,
            &client_key,
            payload,
        )
        .expect("encrypt and prove");

        assert!(ProvenChaChaTfheTranscipher::verify_package(
            &parameters,
            &package
        ));

        let decrypted = ProvenChaChaTfheTranscipher::verify_and_decrypt(
            &parameters,
            &nonce,
            &client_key,
            &package,
        )
        .expect("verified decrypt");

        assert_eq!(decrypted, payload);
    }

    #[test]
    fn tampered_commitment_blocks_decrypt() {
        let payload = b"tamper detectable payload";
        let parameters = ProofParameters::setup(payload.len()).expect("setup");

        let sym_key = ChaCha20Cipher::keygen();
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);

        let config = ConfigBuilder::default().build();
        let (client_key, server_key) = generate_keys(config);
        set_server_key(server_key);

        let mut package = ProvenChaChaTfheTranscipher::transcipher_encrypt_and_prove(
            &parameters,
            &sym_key,
            &nonce,
            &client_key,
            payload,
        )
        .expect("encrypt and prove");

        package.commitment.linear += 1;

        let result = ProvenChaChaTfheTranscipher::verify_and_decrypt(
            &parameters,
            &nonce,
            &client_key,
            &package,
        );

        assert!(matches!(result, Err(ZkTranscipherError::InvalidProof)));
    }
}
