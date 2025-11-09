use actix_web::{HttpResponse, Scope, get, post, web};
use homomorphic::FheEncrypt;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};
use tfhe::{ConfigBuilder, generate_keys};
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

    // ---------------------
    // ✅ Fetch Election
    // ---------------------
    let election_key = format!("elections:{}", election_id);
    let Some(bytes) = db.get(&election_key) else {
        return HttpResponse::NotFound().json(json!({ "error": "Election not found" }));
    };

    let election: Election = serde_json::from_slice(&bytes).unwrap();

    // ---------------------
    // ✅ Candidate mapping
    // ---------------------
    let chosen_id = body["candidate_id"].as_u64().unwrap_or(0) as u32;

    // ---------------------
    // ✅ Load Client Key directly (no metadata needed)
    // ---------------------
    let client_path = format!("keys/{}_client.key", election_id);

    let client_bytes = match fs::read(&client_path) {
        Ok(b) => b,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Client key missing" }));
        }
    };

    let client_key: ClientKey = match bincode::deserialize(&client_bytes) {
        Ok(k) => k,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": "Failed to parse client key" }));
        }
    };

    // ---------------------
    // ✅ Create encrypted vote vector
    // ---------------------
    let mut encrypted_vec = Vec::new();

    for c in &election.candidates {
        let value: u8 = if c.id == chosen_id { 1 } else { 0 };
        encrypted_vec.push(FheUint8::encrypt(value, &client_key));
    }

    // ---------------------
    // ✅ Token verification
    // ---------------------
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

    // ✅ Mark token used
    record.used = true;
    record.used_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    db.put(&token_key, &serde_json::to_vec(&record).unwrap());

    // ---------------------
    // ✅ Store the encrypted ballot
    // ---------------------
    let ballot_id = Uuid::new_v4().to_string();

    let ballot = Ballot {
        ballot_id: ballot_id.clone(),
        election_id: election_id.clone(),
        encrypted_vector: encrypted_vec,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        token_hash,
    };

    db.put(
        &format!("ballots:{}", ballot_id),
        &bincode::serialize(&ballot).unwrap(),
    );

    HttpResponse::Ok().json(json!({ "ballot_id": ballot_id }))
}


pub fn routes() -> Scope {
    web::scope("")
        .service(create_election)
        .service(close_election)
        .service(list_elections)
        .service(get_election)
        .service(submit_ballot)
}
