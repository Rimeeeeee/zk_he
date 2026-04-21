use crate::{db::Database, models::TokenRecord};
use actix_web::{HttpResponse, Scope, post, web};
use rand::{Rng, distributions::Alphanumeric};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[post("/token")]
async fn issue_token(db: web::Data<Database>) -> HttpResponse {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    let key = format!("tokens:{}", token_hash);
    if db.exists(&key) {
        return HttpResponse::Conflict().json(json!({"error": "Token already exists"}));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let record = TokenRecord {
        used: false,
        issued_at: now,
        used_at: None,
    };

    let serialized = serde_json::to_vec(&record).unwrap();
    db.put(&key, &serialized);

    HttpResponse::Ok().json(json!({ "token": token }))
}

pub fn routes() -> Scope {
    web::scope("/auth").service(issue_token)
}
