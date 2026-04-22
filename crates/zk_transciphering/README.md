# ZK-Gated Transciphering

This crate extends the `transciphering` flow with a zero-knowledge proof over a plaintext data commitment.

Flow:

1. The client performs ChaCha20 + TFHE transcipher encryption.
2. The client also creates a Groth16 proof for a commitment derived from the plaintext bytes.
3. The package carries ciphertext, encrypted key words, the public commitment, and the proof.
4. The client verifies the proof before allowing the transciphered result to be decrypted.

The proof in this crate binds the package to a plaintext commitment and enforces proof-first decryption. It does not yet prove the full ChaCha20 relation inside the circuit.

## Benchmarks

| Operation       | Data Size | Time per op                        |
| --------------- | --------- | ---------------------------------- |
| Prove           | 32 B      | [18.790 ms, 19.794 ms, 21.476 ms]  |
| Verify          | 32 B      | [2.0412 ms, 2.4491 ms, 2.7446 ms]  |
| Guarded Decrypt | 32 B      | [1.8230 ms, 2.0412 ms, 2.3333 ms]  |
| Proof Only      | 32 B      | [5.1370 ms, 6.8476 ms, 7.7740 ms]  |
| Prove           | 256 B     | [22.445 ms, 24.533 ms, 26.339 ms]  |
| Verify          | 256 B     | [2.0840 ms, 2.2807 ms, 2.4819 ms]  |
| Guarded Decrypt | 256 B     | [2.4478 ms, 2.6131 ms, 2.8225 ms]  |
| Proof Only      | 256 B     | [13.924 ms, 15.053 ms, 15.786 ms]  |
| Prove           | 1 KB      | [66.966 ms, 73.277 ms, 78.537 ms]  |
| Verify          | 1 KB      | [2.0005 ms, 2.2576 ms, 2.4663 ms]  |
| Guarded Decrypt | 1 KB      | [2.2946 ms, 2.5446 ms, 2.7014 ms]  |
| Proof Only      | 1 KB      | [36.368 ms, 42.097 ms, 46.188 ms]  |
