use actix_web::{HttpResponse, Scope, get, post, web};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use homomorphic::{FheDecrypt, FheEncrypt, FheTrivialEncrypt};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use tfhe::{ConfigBuilder, ServerKey, generate_keys, set_server_key};
use uuid::Uuid;

use crate::{
    db::Database,
    models::{
        Ballot, Candidate, Election, ElectionKeys, ElectionResultRecord, EncryptedTallyEntry,
        ProofBundle, TallyEntry, TokenRecord,
    },
};
use std::path::Path;
use tfhe::{ClientKey, FheUint8};
use zk::groth;

// Ensure directory exists

// #[post("/admin/elections")]
// async fn create_election(
//     db: web::Data<Database>,
//     body: web::Json<serde_json::Value>,
// ) -> HttpResponse {
//     let id = Uuid::new_v4().to_string();
//     let now = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .unwrap()
//         .as_secs();

//     let name = body["name"].as_str().unwrap_or("Election").to_string();
//     let start_time = body["start_time"].as_u64().unwrap_or(now);
//     let end_time = body["end_time"].as_u64().unwrap_or(now + 3600);

//     let candidates: Vec<Candidate> =
//         serde_json::from_value(body["candidates"].clone()).unwrap_or_default();

//     let election = Election {
//         id: id.clone(),
//         name,
//         start_time,
//         end_time,
//         candidates,
//         closed: false,
//     };

//     let serialized = serde_json::to_vec(&election).unwrap();
//     db.put(&format!("elections:{}", id), &serialized);

//     HttpResponse::Ok().json(json!({ "election_id": id }))
// }
#[post("/admin/elections")]
async fn create_election(
    db: web::Data<Database>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    // --- Step 1: Generate election details ---
    let id = Uuid::new_v4().to_string();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let name = body["name"].as_str().unwrap_or("Election").to_string();
    let start_time = body["start_time"].as_u64().unwrap_or(now);
    let end_time = body["end_time"].as_u64().unwrap_or(now + 3600);

    let candidates: Vec<Candidate> =
        serde_json::from_value(body["candidates"].clone()).unwrap_or_default();

    let election = Election {
        id: id.clone(),
        name,
        start_time,
        end_time,
        candidates,
        closed: false,
    };

    // --- Step 2: Store election ---
    let serialized = serde_json::to_vec(&election).unwrap();
    db.put(&format!("elections:{}", id), &serialized);

    // --- Step 3: Generate FHE keys (directly inside this function) ---
    let key_path = format!("keys:{}", id);
    if db.exists(&key_path) {
        return HttpResponse::Conflict().json(json!({ "error": "Keys already exist" }));
    }

    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);
    let client_bytes = bincode::serialize(&client_key).unwrap();
    let server_bytes = bincode::serialize(&server_key).unwrap();
    let key_dir = Path::new("keys");
    if !key_dir.exists() {
        fs::create_dir_all(key_dir).expect("Failed to create keys directory");
    }
    let client_path = format!("keys/{}_client.key", id);
    let server_path = format!("keys/{}_server.key", id);

    fs::write(&client_path, &client_bytes).expect("Failed to write client key");
    fs::write(&server_path, &server_bytes).expect("Failed to write server key");

    let record = ElectionKeys {
        id: id.clone(),
        server: server_path.clone(),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    db.put(&key_path, &serde_json::to_vec(&record).unwrap());

    // --- Step 4: Return election id and keys together ---
    HttpResponse::Ok().json(json!({
        "election_id": id,
        "client_key_b64": STANDARD.encode(&client_bytes),
        "client_key": client_path,
        "server_key": server_path
    }))
}

#[post("/admin/elections/{id}/close")]
async fn close_election(db: web::Data<Database>, path: web::Path<String>) -> HttpResponse {
    let id = path.into_inner();
    let key = format!("elections:{}", id);

    if let Some(bytes) = db.get(&key) {
        let mut election: Election = serde_json::from_slice(&bytes).unwrap();
        election.closed = true;
        db.put(&key, &serde_json::to_vec(&election).unwrap());
        HttpResponse::Ok().json(json!({ "status": "closed" }))
    } else {
        HttpResponse::NotFound().json(json!({ "error": "Election not found" }))
    }
}

#[get("/elections")]
async fn list_elections(db: web::Data<Database>) -> HttpResponse {
    let mut elections = vec![];
    for (_key, value) in db.scan_prefix("elections:") {
        if let Ok(election) = serde_json::from_slice::<Election>(&value) {
            elections.push(election);
        }
    }
    HttpResponse::Ok().json(elections)
}

#[get("/elections/{id}")]
async fn get_election(db: web::Data<Database>, path: web::Path<String>) -> HttpResponse {
    let id = path.into_inner();
    let key = format!("elections:{}", id);

    if let Some(bytes) = db.get(&key) {
        let election: Election = serde_json::from_slice(&bytes).unwrap();
        HttpResponse::Ok().json(election)
    } else {
        HttpResponse::NotFound().json(json!({ "error": "Election not found" }))
    }
}

#[post("/elections/{id}/ballots")]
async fn submit_ballot(
    db: web::Data<Database>,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let election_id = path.into_inner();

    let election_key = format!("elections:{}", election_id);
    let Some(bytes) = db.get(&election_key) else {
        return HttpResponse::NotFound().json(json!({ "error": "Election not found" }));
    };
    let election: Election = serde_json::from_slice(&bytes).unwrap();

    let chosen_id = body["candidate_id"].as_u64().unwrap_or(0) as u32;
    println!("Vote for candidate ID: {}", chosen_id);

    let client_path = format!("keys/{}_client.key", election_id);
    let client_bytes = match fs::read(&client_path) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Client key missing" }));
        }
    };
    let client_key: ClientKey = bincode::deserialize(&client_bytes)
        .map_err(|_| HttpResponse::InternalServerError().json(json!({ "error": "Bad client key" })))
        .unwrap();

    let mut encrypted_vec = Vec::new();
    for c in &election.candidates {
        let bit: u8 = if c.id == chosen_id { 1 } else { 0 };
        encrypted_vec.push((c.id, FheUint8::encrypt(bit, &client_key)));
    }

    let token = body["token"].as_str().unwrap_or("").to_string();
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());
    let token_key = format!("tokens:{}", token_hash);

    let Some(bytes) = db.get(&token_key) else {
        return HttpResponse::Unauthorized().json(json!({ "error": "Invalid token" }));
    };
    let mut record: TokenRecord = serde_json::from_slice(&bytes).unwrap();

    if record.used {
        return HttpResponse::Forbidden().json(json!({ "error": "Token already used" }));
    }

    record.used = true;
    record.used_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );
    db.put(&token_key, &serde_json::to_vec(&record).unwrap());

    let ballot_id = Uuid::new_v4().to_string();
    let ballot = Ballot {
        ballot_id: ballot_id.clone(),
        election_id: election_id.clone(),
        encrypted_vector: encrypted_vec,
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        token_hash,
    };

    db.put(
        &format!("ballots:{}", ballot_id),
        &bincode::serialize(&ballot).unwrap(),
    );
    println!("Doneee");
    HttpResponse::Ok().json(json!({ "ballot_id": ballot_id }))
}

#[get("/elections/{id}/result")]
async fn calculate_winner(db: web::Data<Database>, path: web::Path<String>) -> HttpResponse {
    let election_id = path.into_inner();
    let election_key = format!("elections:{}", election_id);
    let result_key = format!("results:{}", election_id);

    let Some(election_bytes) = db.get(&election_key) else {
        return HttpResponse::NotFound().json(json!({ "error": "Election not found" }));
    };
    let election: Election = serde_json::from_slice(&election_bytes).unwrap();

    let mut ballots: Vec<(String, Vec<u8>, Ballot)> = db
        .scan_prefix("ballots:")
        .into_iter()
        .filter_map(|(key, value)| {
            let ballot = bincode::deserialize::<Ballot>(&value).ok()?;
            if ballot.election_id == election_id {
                Some((key, value, ballot))
            } else {
                None
            }
        })
        .collect();

    if ballots.is_empty() {
        return HttpResponse::Ok().json(json!({ "message": "No ballots found" }));
    }

    ballots.sort_by(|a, b| {
        a.2.timestamp
            .cmp(&b.2.timestamp)
            .then_with(|| a.2.ballot_id.cmp(&b.2.ballot_id))
    });

    let tally_hash = hash_ballots(&ballots);

    if let Some(cached_bytes) = db.get(&result_key) {
        if let Ok(cached) = serde_json::from_slice::<ElectionResultRecord>(&cached_bytes) {
            let cached_proof_hash = hash_proof_bundle(
                &cached.proof.proof_b64,
                &cached.proof.verification_key_b64,
                &cached.proof.public_totals,
            );
            if cached.tally_hash == tally_hash && cached.proof.proof_hash == cached_proof_hash {
                return HttpResponse::Ok().json(cached);
            }
        }
    }

    let server_path = format!("keys/{}_server.key", election_id);
    let client_path = format!("keys/{}_client.key", election_id);

    let server_bytes = match fs::read(&server_path) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Server key missing" }));
        }
    };
    let client_bytes = match fs::read(&client_path) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Client key missing" }));
        }
    };

    let server_key: ServerKey = bincode::deserialize(&server_bytes).unwrap();
    let client_key: ClientKey = bincode::deserialize(&client_bytes).unwrap();
    set_server_key(server_key);

    let num_candidates = election.candidates.len();
    let mut totals: Vec<FheUint8> = vec![FheUint8::encrypt_trivial(0u8); num_candidates];

    for (_, _, ballot) in &ballots {
        for (i, (_cid, vote_cipher)) in ballot.encrypted_vector.iter().enumerate() {
            totals[i] = &totals[i] + vote_cipher;
        }
    }

    let totals_plain = decrypt_totals(&totals, &election.candidates, &client_key);
    let vote_matrix = match decrypt_vote_matrix(&ballots, &election.candidates, &client_key) {
        Ok(matrix) => matrix,
        Err(message) => {
            return HttpResponse::BadRequest().json(json!({ "error": message }));
        }
    };
    let public_totals: Vec<u8> = totals_plain.iter().map(|entry| entry.votes).collect();
    let proof_bundle = build_proof_bundle(vote_matrix, public_totals.clone());
    let encrypted_totals = serialize_encrypted_totals(&totals, &election.candidates);

    let result = ElectionResultRecord {
        election_id: election_id.clone(),
        encrypted_totals,
        ballot_count: ballots.len(),
        tally_hash,
        generated_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        status: "Encrypted tally generated. Verify the proof and decrypt client-side.".to_string(),
        proof: proof_bundle,
    };

    db.put(&result_key, &serde_json::to_vec(&result).unwrap());

    HttpResponse::Ok().json(result)
}

fn hash_ballots(ballots: &[(String, Vec<u8>, Ballot)]) -> String {
    let mut hasher = Sha256::new();

    for (key, raw, ballot) in ballots {
        hasher.update(key.as_bytes());
        hasher.update(&ballot.timestamp.to_le_bytes());
        hasher.update(raw);
    }

    format!("{:x}", hasher.finalize())
}

fn decrypt_totals(
    totals: &[FheUint8],
    candidates: &[crate::models::Candidate],
    client_key: &ClientKey,
) -> Vec<TallyEntry> {
    let mut plain_totals = Vec::new();
    for (i, ct) in totals.iter().enumerate() {
        let count: u8 = ct.decrypt(client_key);
        plain_totals.push(TallyEntry {
            candidate_id: candidates[i].id,
            label: candidates[i].label.clone(),
            votes: count,
        });
    }

    plain_totals
}

fn decrypt_vote_matrix(
    ballots: &[(String, Vec<u8>, Ballot)],
    candidates: &[Candidate],
    client_key: &ClientKey,
) -> Result<Vec<Vec<u8>>, String> {
    ballots
        .iter()
        .map(|(_, _, ballot)| {
            let row: Vec<u8> = ballot
                .encrypted_vector
                .iter()
                .map(|(_, vote_cipher)| vote_cipher.decrypt(client_key))
                .collect();

            let valid_votes = row.iter().filter(|vote| **vote == 1).count();
            if row.len() != candidates.len() || valid_votes != 1 || row.iter().any(|vote| *vote > 1)
            {
                return Err("Invalid ballot encountered while building proof witness".to_string());
            }

            Ok(row)
        })
        .collect()
}

fn build_proof_bundle(vote_matrix: Vec<Vec<u8>>, public_totals: Vec<u8>) -> ProofBundle {
    let keys = groth::setup(&vote_matrix, &public_totals);
    let proof = groth::prove(&keys, vote_matrix, public_totals.clone());
    let verified = groth::verify(&keys, &proof, public_totals.clone());

    let proof_b64 = STANDARD.encode(&proof);
    let verification_key_b64 = STANDARD.encode(&keys.vk);
    let proof_hash = hash_proof_bundle(&proof_b64, &verification_key_b64, &public_totals);

    ProofBundle {
        system: "groth16".to_string(),
        proof_b64,
        verification_key_b64,
        public_totals,
        verified,
        proof_hash,
    }
}

fn hash_proof_bundle(proof_b64: &str, verification_key_b64: &str, public_totals: &[u8]) -> String {
    let payload = (proof_b64, verification_key_b64, public_totals);
    let encoded = serde_json::to_vec(&payload).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    format!("{:x}", hasher.finalize())
}

fn serialize_encrypted_totals(
    totals: &[FheUint8],
    candidates: &[Candidate],
) -> Vec<EncryptedTallyEntry> {
    totals
        .iter()
        .zip(candidates.iter())
        .map(|(ciphertext, candidate)| EncryptedTallyEntry {
            candidate_id: candidate.id,
            label: candidate.label.clone(),
            ciphertext_b64: STANDARD.encode(bincode::serialize(ciphertext).unwrap()),
        })
        .collect()
}

pub fn routes() -> Scope {
    web::scope("")
        .service(create_election)
        .service(close_election)
        .service(list_elections)
        .service(get_election)
        .service(submit_ballot)
        .service(calculate_winner)
}
