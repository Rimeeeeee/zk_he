use base64::{engine::general_purpose, Engine as _};
use bincode;
use homomorphic::{tfhe_uint::TfheU32, *};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// A serializable wrapper for both keys
#[derive(Serialize, Deserialize)]
struct RawKeys {
    client_key: Vec<u8>,
    server_key: Vec<u8>,
}

#[wasm_bindgen]
pub fn generate_keys_b64() -> Result<JsValue, JsValue> {
    let (client_key, server_key) = TfheU32::keygen().map_err(|e| e.to_string())?;

    // Serialize both keys
    let ck_bytes = bincode::serialize(&client_key).map_err(|e| e.to_string())?;
    let sk_bytes = bincode::serialize(&server_key).map_err(|e| e.to_string())?;

    // Wrap together
    let raw = RawKeys {
        client_key: ck_bytes,
        server_key: sk_bytes,
    };
    let raw_bytes = bincode::serialize(&raw).map_err(|e| e.to_string())?;
    let b64 = general_purpose::STANDARD.encode(raw_bytes);

    Ok(JsValue::from_str(&b64))
}

#[wasm_bindgen]
pub fn extract_server_key_b64(keys_b64: &str) -> Result<String, JsValue> {
    let bytes = general_purpose::STANDARD
        .decode(keys_b64)
        .map_err(|e| e.to_string())?;
    let raw: RawKeys = bincode::deserialize(&bytes).map_err(|e| e.to_string())?;
    Ok(general_purpose::STANDARD.encode(&raw.server_key))
}

#[wasm_bindgen]
pub fn encrypt_vote_u32_b64(keys_b64: &str, vote_value: u32) -> Result<String, JsValue> {
    // Decode combined keys
    let bytes = general_purpose::STANDARD
        .decode(keys_b64)
        .map_err(|e| e.to_string())?;
    let raw: RawKeys = bincode::deserialize(&bytes).map_err(|e| e.to_string())?;

    // Deserialize client key
    let client_key: ClientKey = bincode::deserialize(&raw.client_key).map_err(|e| e.to_string())?;

    // Encrypt using your abstraction
    let ct = TfheU32::encrypt(&client_key, &vote_value).map_err(|e| e.to_string())?;

    // Serialize ciphertext
    let ct_bytes = bincode::serialize(&ct).map_err(|e| e.to_string())?;
    let ct_b64 = general_purpose::STANDARD.encode(ct_bytes);

    Ok(ct_b64)
}
