use bincode::deserialize;
use homomorphic::{HomomorphicEncryption, tfhe::TfheU32};
use std::io::{Read, Write};
use std::net::TcpListener;
use tfhe::FheUint32;
use transciphering::TranscipherError;

pub fn run() -> Result<(), TranscipherError> {
    let listener = TcpListener::bind("127.0.0.1:4000").unwrap();
    println!("Server listening on 127.0.0.1:4000");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Accepted client connection");

                let mut size_buf = [0u8; 8];
                stream.read_exact(&mut size_buf).unwrap();
                let data_size = u64::from_le_bytes(size_buf) as usize;
                println!("Reading {} bytes from client", data_size);

                let mut buf = vec![0u8; data_size];
                stream.read_exact(&mut buf).unwrap();

                let (client_key, server_key, numbers_ct, claimed_sum): (
                    tfhe::ClientKey,
                    tfhe::ServerKey,
                    Vec<FheUint32>,
                    u32,
                ) = deserialize(&buf).unwrap();
                println!(
                    "Received client key, server key, {} encrypted numbers, claimed sum {}",
                    numbers_ct.len(),
                    claimed_sum
                );

                tfhe::set_server_key(server_key);
                println!("Server key set on server");

                let mut sum_ct = numbers_ct[0].clone();
                for ct in numbers_ct.iter().skip(1) {
                    sum_ct = TfheU32::add(&sum_ct, ct).unwrap();
                }

                let sum = TfheU32::decrypt(&client_key, &sum_ct).unwrap();
                println!("Decrypted sum on server: {}", sum);

                if sum == claimed_sum {
                    println!("Sum is correct");
                } else {
                    println!("Sum is incorrect");
                }

                let data = bincode::serialize(&sum_ct).unwrap();
                stream
                    .write_all(&(data.len() as u64).to_le_bytes())
                    .unwrap();
                stream.write_all(&data).unwrap();
                println!("Sent encrypted sum back to client");
            }
            Err(e) => println!("Failed connection: {:?}", e),
        }
    }
    Ok(())
}
