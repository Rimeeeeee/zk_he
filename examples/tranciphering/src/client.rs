use bincode::serialize;
use rand::RngCore;
use std::io::{Read, Write};
use std::net::TcpStream;

use homomorphic::{HomomorphicEncryption, tfhe::TfheU32};
use symmetric::{SymmetricCipher, chacha::ChaCha20Cipher};
use tfhe::FheUint32;
use transciphering::{ChaChaTfheTranscipher, Transcipher, TranscipherError};

pub fn run() -> Result<(), TranscipherError> {
    let sym_key = ChaCha20Cipher::keygen();
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);

    let (client_key, _server_key) = TfheU32::keygen().map_err(|_| TranscipherError::HeError)?;

    let message = b"Hello server via transciphering!";

    let (ciphertext, key_cts): (Vec<u8>, Vec<FheUint32>) =
        ChaChaTfheTranscipher::transcipher_encrypt(&sym_key, &nonce, &client_key, message)?;

    let mut stream = TcpStream::connect("127.0.0.1:4000").unwrap();
    let data = serialize(&(ciphertext, key_cts)).unwrap();
    stream
        .write_all(&(data.len() as u64).to_le_bytes())
        .unwrap();
    stream.write_all(&data).unwrap();

    // Receive server response
    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf).unwrap();
    let data_size = u64::from_le_bytes(size_buf) as usize;

    let mut buf = vec![0u8; data_size];
    stream.read_exact(&mut buf).unwrap();

    let (ciphertext_resp, key_cts_resp): (Vec<u8>, Vec<FheUint32>) =
        bincode::deserialize(&buf).unwrap();

    let decrypted = ChaChaTfheTranscipher::transcipher_decrypt(
        &nonce,
        &client_key,
        &key_cts_resp,
        &ciphertext_resp,
    )?;

    println!(
        "Decrypted message from server: {}",
        String::from_utf8_lossy(&decrypted)
    );

    Ok(())
}
