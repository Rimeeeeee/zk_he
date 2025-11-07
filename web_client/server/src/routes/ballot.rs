use actix_web::{post, web, HttpResponse, Scope};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{db::Database, models::{Ballot, TokenRecord}};

#[post("/elections/{id}/ballots")]
async fn submit_ballot(
    db: web::Data<Database>,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let election_id = path.into_inner();
    let token = body["token"].as_str().unwrap_or("").to_string();
    let candidate_id = body["candidate_id"].as_u64().unwrap_or(0) as u32;
    let ciphertext = body["ciphertext"].as_str().unwrap_or("").to_string();

    // Token verification
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

    // Mark token used
    record.used = true;
    record.used_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    db.put(&token_key, &serde_json::to_vec(&record).unwrap());

    // Store ballot
    let ballot_id = Uuid::new_v4().to_string();
    let ballot = Ballot {
        ballot_id: ballot_id.clone(),
        election_id: election_id.clone(),
        candidate_id,
        ciphertext,
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        token_hash,
    };
    db.put(
        &format!("ballots:{}", ballot_id),
        &serde_json::to_vec(&ballot).unwrap(),
    );

    HttpResponse::Ok().json(json!({ "ballot_id": ballot_id }))
}

pub fn routes() -> Scope {
    web::scope("").service(submit_ballot)
}
