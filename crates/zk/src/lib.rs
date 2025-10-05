//! Lib of zero-knowledge proof systems.

///Plonk Module
pub mod plonk;

/// A trait for Zero-Knowledge Proof systems.
pub trait ZeroKnowledge {
    type Parameters;
    type Proof;
    type Statement;
    type Witness;

    /// Generate trusted setup parameters.
    fn setup() -> Result<Self::Parameters, ZkError>;

    /// Create a proof given parameters, statement and witness.
    fn prove(
        params: &Self::Parameters,
        statement: &Self::Statement,
        witness: &Self::Witness,
    ) -> Result<Self::Proof, ZkError>;

    /// Verify a proof against a statement.
    fn verify(
        params: &Self::Parameters,
        statement: &Self::Statement,
        proof: &Self::Proof,
    ) -> Result<bool, ZkError>;
}

/// Error type for ZK operations.
#[derive(Debug, thiserror::Error)]
pub enum ZkError {
    #[error("Invalid setup parameters")]
    InvalidSetup,

    #[error("Proof generation failed")]
    ProveError,

    #[error("Proof verification failed")]
    VerifyError,

    #[error("Internal error: {0}")]
    Internal(String),
}
