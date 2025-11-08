use crate::db::Database;
use actix_web::{HttpResponse, Scope, post, web};
use serde_json::json;

#[post("/elections/{id}/server-key")]
async fn store_server_key(
    db: web::Data<Database>,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let id = path.into_inner();
    let key_b64 = body["server_key"].as_str().unwrap_or("");
    if key_b64.is_empty() {
        return HttpResponse::BadRequest().json(json!({"error": "missing server_key"}));
    }

    db.put(&format!("server_keys:{}", id), key_b64.as_bytes());
    HttpResponse::Ok().json(json!({"status": "stored"}))
}

pub fn routes() -> Scope {
    web::scope("").service(store_server_key)
}
