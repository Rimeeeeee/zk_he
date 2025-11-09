// use actix_web::{post, web, HttpResponse, Scope};
// use homomorphic::FheEncrypt;
// use serde_json::json;
// use sha2::{Digest, Sha256};
// use uuid::Uuid;
// use std::time::{SystemTime, UNIX_EPOCH};
// use std::fs;

// use tfhe::{ClientKey, FheUint8};

// use crate::{
//     db::Database,
//     models::{Ballot, TokenRecord, Election},
// };

// #[post("/elections/{id}/ballots")]
// async fn submit_ballot(
//     db: web::Data<Database>,
//     path: web::Path<String>,
//     body: web::Json<serde_json::Value>,
// ) -> HttpResponse {
//     let election_id = path.into_inner();

//     // ---------------------
//     // ✅ Fetch Election
//     // ---------------------
//     let election_key = format!("elections:{}", election_id);
//     let Some(bytes) = db.get(&election_key) else {
//         return HttpResponse::NotFound().json(json!({ "error": "Election not found" }));
//     };

//     let election: Election = serde_json::from_slice(&bytes).unwrap();

//     // ---------------------
//     // ✅ Candidate mapping
//     // ---------------------
//     let chosen_id = body["candidate_id"].as_u64().unwrap_or(0) as u32;

//     // ---------------------
//     // ✅ Load Client Key directly (no metadata needed)
//     // ---------------------
//     let client_path = format!("keys/{}_client.key", election_id);

//     let client_bytes = match fs::read(&client_path) {
//         Ok(b) => b,
//         Err(_) => {
//             return HttpResponse::InternalServerError()
//                 .json(json!({ "error": "Client key missing" }));
//         }
//     };

//     let client_key: ClientKey = match bincode::deserialize(&client_bytes) {
//         Ok(k) => k,
//         Err(_) => {
//             return HttpResponse::InternalServerError()
//                 .json(json!({ "error": "Failed to parse client key" }));
//         }
//     };

//     // ---------------------
//     // ✅ Create encrypted vote vector
//     // ---------------------
//     let mut encrypted_vec = Vec::new();

//     for c in &election.candidates {
//         let value: u8 = if c.id == chosen_id { 1 } else { 0 };
//         encrypted_vec.push(FheUint8::encrypt(value, &client_key));
//     }

//     // ---------------------
//     // ✅ Token verification
//     // ---------------------
//     let token = body["token"].as_str().unwrap_or("").to_string();

//     let mut hasher = Sha256::new();
//     hasher.update(token.as_bytes());
//     let token_hash = format!("{:x}", hasher.finalize());

//     let token_key = format!("tokens:{}", token_hash);

//     let Some(bytes) = db.get(&token_key) else {
//         return HttpResponse::Unauthorized().json(json!({ "error": "Invalid token" }));
//     };

//     let mut record: TokenRecord = serde_json::from_slice(&bytes).unwrap();

//     if record.used {
//         return HttpResponse::Forbidden().json(json!({ "error": "Token already used" }));
//     }

//     // ✅ Mark token used
//     record.used = true;
//     record.used_at = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
//     db.put(&token_key, &serde_json::to_vec(&record).unwrap());

//     // ---------------------
//     // ✅ Store the encrypted ballot
//     // ---------------------
//     let ballot_id = Uuid::new_v4().to_string();

//     let ballot = Ballot {
//         ballot_id: ballot_id.clone(),
//         election_id: election_id.clone(),
//         encrypted_vector: encrypted_vec,
//         timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
//         token_hash,
//     };

//     db.put(
//         &format!("ballots:{}", ballot_id),
//         &bincode::serialize(&ballot).unwrap(),
//     );

//     HttpResponse::Ok().json(json!({ "ballot_id": ballot_id }))
// }

// pub fn routes() -> Scope {
//     web::scope("").service(submit_ballot)
// }
