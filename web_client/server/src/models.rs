use serde::{Deserialize, Serialize};
use tfhe::FheUint8;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenRecord {
    pub used: bool,
    pub issued_at: u64,
    pub used_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candidate {
    pub id: u32,
    #[serde(rename = "name")]
    pub label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Election {
    pub id: String,
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub candidates: Vec<Candidate>,
    pub closed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElectionKeys {
    pub id: String,
    pub server: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Ballot {
    pub ballot_id: String,
    pub election_id: String,
    pub encrypted_vector: Vec<(u32, FheUint8)>,
    pub timestamp: u64,
    pub token_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TallyEntry {
    pub candidate_id: u32,
    pub label: String,
    pub votes: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProofBundle {
    pub system: String,
    pub proof_b64: String,
    pub verification_key_b64: String,
    pub public_totals: Vec<u8>,
    pub verified: bool,
    pub proof_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ElectionResultRecord {
    pub election_id: String,
    pub winner_label: String,
    pub winner_id: u32,
    pub totals: Vec<TallyEntry>,
    pub ballot_count: usize,
    pub tally_hash: String,
    pub generated_at: u64,
    pub status: String,
    pub proof: ProofBundle,
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct Tally {
//     pub candidate_id: u32,
//     pub encrypted_tally: String, // base64 of FheUint32 ciphertext
// }

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct EncryptedElectionTallies {
//     pub election_id: String,
//     pub tallies: HashMap<u32, Tally>,
// }
