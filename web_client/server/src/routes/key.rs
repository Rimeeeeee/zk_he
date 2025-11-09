#[allow(deprecated)]
use crate::db::Database;
use actix_web::{HttpResponse, Scope, post, web};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use tfhe::{ConfigBuilder, generate_keys};

#[post("/{id}/keys")]
async fn generate_election_keys(db: web::Data<Database>, path: web::Path<String>) -> HttpResponse {
    let election_id = path.into_inner();

    let key_path = format!("keys:{}", election_id);
    if db.exists(&key_path) {
        return HttpResponse::Conflict().json(json!({ "error": "Keys already exist" }));
    }

    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);

    let client_bytes = bincode::serialize(&client_key).unwrap();
    let server_bytes = bincode::serialize(&server_key).unwrap();
    let client_b64 = base64::encode(client_bytes);
    let server_b64 = base64::encode(server_bytes);

    let record = json!({
        "client_key": client_b64,
        "server_key": server_b64,
        "created_at": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    db.put(&key_path, &serde_json::to_vec(&record).unwrap());

    HttpResponse::Ok().json(json!({
        "election_id": election_id,
        "server_key": server_b64
    }))
}

pub fn routes() -> Scope {
    // ✅ Important: prefix must match what frontend calls
    web::scope("/elections").service(generate_election_keys)
}
