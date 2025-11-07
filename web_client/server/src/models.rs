use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenRecord {
    pub used: bool,
    pub issued_at: u64,
    pub used_at: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Candidate {
    pub id: u32,
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
pub struct Ballot {
    pub ballot_id: String,
    pub election_id: String,
    pub candidate_id: u32,
    pub ciphertext: String,
    pub timestamp: u64,
    pub token_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tally {
    pub candidate_id: u32,
    pub encrypted_tally: String, // base64 of FheUint32 ciphertext
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EncryptedElectionTallies {
    pub election_id: String,
    pub tallies: HashMap<u32, Tally>,
}
