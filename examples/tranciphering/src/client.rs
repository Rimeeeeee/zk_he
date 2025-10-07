use bincode::serialize;
use homomorphic::{HomomorphicEncryption, tfhe::TfheU32};
use std::io::{Read, Write};
use std::net::TcpStream;
use tfhe::FheUint32;
use transciphering::TranscipherError;

pub fn run() -> Result<(), TranscipherError> {
    let (client_key, server_key) = TfheU32::keygen().unwrap();
    println!("Client generated TFHE keys");

    let numbers = [3u32, 7, 5, 10];
    let claimed_sum = numbers.iter().sum::<u32>();
    println!("Numbers: {:?}", numbers);
    println!("Client claims sum = {}", claimed_sum);

    let numbers_ct: Vec<FheUint32> = numbers
        .iter()
        .map(|n| TfheU32::encrypt(&client_key, n).unwrap())
        .collect();
    println!("Client encrypted numbers");

    let mut stream = TcpStream::connect("127.0.0.1:4000").unwrap();
    println!("Connected to server");

    let data = serialize(&(client_key.clone(), server_key, numbers_ct, claimed_sum)).unwrap();
    stream
        .write_all(&(data.len() as u64).to_le_bytes())
        .unwrap();
    stream.write_all(&data).unwrap();
    println!("Sent keys, encrypted numbers, and claimed sum");

    let mut size_buf = [0u8; 8];
    stream.read_exact(&mut size_buf).unwrap();
    let data_size = u64::from_le_bytes(size_buf) as usize;

    let mut buf = vec![0u8; data_size];
    stream.read_exact(&mut buf).unwrap();
    let sum_ct: FheUint32 = bincode::deserialize(&buf).unwrap();
    println!("Received encrypted sum from server");

    let sum = TfheU32::decrypt(&client_key, &sum_ct).unwrap();
    println!("Decrypted sum on client: {}", sum);

    Ok(())
}
