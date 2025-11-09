use actix_web::{HttpResponse, Scope, get, post, web};
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
    models::{Ballot, Candidate, Election, ElectionKeys, TokenRecord},
};
use std::path::Path;
use tfhe::{ClientKey, FheUint8};

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

    // --- Load election ---
    let Some(election_bytes) = db.get(&election_key) else {
        return HttpResponse::NotFound().json(json!({ "error": "Election not found" }));
    };
    let election: Election = serde_json::from_slice(&election_bytes).unwrap();

    // --- Load server & client keys ---
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

    // --- Gather ballots ---
    let mut ballots: Vec<Ballot> = Vec::new();
    for (_k, v) in db.scan_prefix("ballots:") {
        if let Ok(ballot) = bincode::deserialize::<Ballot>(&v) {
            if ballot.election_id == election_id {
                ballots.push(ballot);
            }
        }
    }

    if ballots.is_empty() {
        return HttpResponse::Ok().json(json!({ "message": "No ballots found" }));
    }

    // --- Homomorphically add encrypted tallies ---
    let num_candidates = election.candidates.len();
    let mut totals: Vec<FheUint8> = vec![FheUint8::encrypt_trivial(0u8); num_candidates];

    for ballot in ballots {
        for (i, (_cid, vote_cipher)) in ballot.encrypted_vector.iter().enumerate() {
            totals[i] = &totals[i] + vote_cipher;
        }
    }

    // --- Call the separate decrypt + winner function ---
    let result = decrypt_and_find_winner(&totals, &election.candidates, &client_key);

    HttpResponse::Ok().json(json!({
        "election_id": election_id,
        "winner_label": result.0,
        "winner_id": result.1,
        "totals": result.2,
        "status": "Winner decrypted successfully"
    }))
}

/// Separate function that decrypts tallies and finds the winner
fn decrypt_and_find_winner(
    totals: &Vec<FheUint8>,
    candidates: &Vec<crate::models::Candidate>,
    client_key: &ClientKey,
) -> (String, u32, Vec<(String, u8)>) {
    let mut plain_totals = Vec::new();
    for (i, ct) in totals.iter().enumerate() {
        let count: u8 = ct.decrypt(client_key);
        plain_totals.push((candidates[i].label.clone(), count));
    }

    let (winner_label, _winner_votes) =
        plain_totals.iter().max_by_key(|(_, count)| *count).unwrap();

    let winner_id = candidates
        .iter()
        .find(|c| c.label == *winner_label)
        .map(|c| c.id)
        .unwrap_or(0);

    (winner_label.clone(), winner_id, plain_totals)
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
