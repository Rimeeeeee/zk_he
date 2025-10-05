use bincode::deserialize;
use std::io::{Read, Write};
use std::net::TcpListener;
use tfhe::FheUint32;
use transciphering::TranscipherError;

pub fn run() -> Result<(), TranscipherError> {
    let listener = TcpListener::bind("127.0.0.1:4000").unwrap();
    println!("Server listening on 127.0.0.1:4000...");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        // Read message length
        let mut size_buf = [0u8; 8];
        stream.read_exact(&mut size_buf).unwrap();
        let data_size = u64::from_le_bytes(size_buf) as usize;

        // Read message
        let mut buf = vec![0u8; data_size];
        stream.read_exact(&mut buf).unwrap();

        let (ciphertext, key_cts): (Vec<u8>, Vec<FheUint32>) = deserialize(&buf).unwrap();

        // Optionally perform some operation on ciphertext
        // For now, just echo back
        let data = bincode::serialize(&(ciphertext, key_cts)).unwrap();
        stream
            .write_all(&(data.len() as u64).to_le_bytes())
            .unwrap();
        stream.write_all(&data).unwrap();
    }

    Ok(())
}
