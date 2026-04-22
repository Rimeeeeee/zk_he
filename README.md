This project is a part of our college final year and may contain prospect of research paper if it happens:)

### Symmetric Benchmarks

| Cipher       | Data Size | Time per op                        | Outliers | Mild | Severe |
| ------------ | --------- | ---------------------------------- | -------- | ---- | ------ |
| AES-256-CBC  | 1 KB      | [1.5498 us, 1.5629 us, 1.5769 us]  | 4%       | 2%   | 2%     |
| ChaCha20     | 1 KB      | [833.42 ns, 845.06 ns, 857.85 ns]  | 10%      | 5%   | 5%     |
| Camellia-256 | 1 KB      | [16.215 us, 16.417 us, 16.636 us]  | 5%       | 3%   | 2%     |

---

### HE Benchmarks

| Operation   | Data Size | Time per op                        | Outliers | Mild | Severe |
| ----------- | --------- | ---------------------------------- | -------- | ---- | ------ |
| Encrypt u32 | 4 bytes   | [2.3689 ms, 2.3702 ms, 2.3720 ms]  | 11%      | 5%   | 6%     |
| Add         | 4 bytes   | [627.39 ms, 628.10 ms, 629.02 ms]  | 5%       | 4%   | 1%     |
| Mul         | 4 bytes   | [8.4997 s, 8.5060 s, 8.5123 s]     | 2%       | 2%   | 0%     |
| Decrypt u32 | 4 bytes   | [14.709 us, 14.731 us, 14.752 us]  | 3%       | 1%   | 2%     |

### Transciphering Benchmarks

| Operation | Data Size | Time per op                        |
| --------- | --------- | ---------------------------------- |
| Encrypt   | 32 B      | [16.746 ms, 16.986 ms, 17.329 ms]  |
| Decrypt   | 32 B      | [142.42 us, 146.84 us, 152.66 us]  |
| Roundtrip | 32 B      | [15.103 ms, 15.620 ms, 16.218 ms]  |
| Encrypt   | 1 KB      | [14.829 ms, 15.162 ms, 15.543 ms]  |
| Decrypt   | 1 KB      | [118.17 us, 125.88 us, 135.18 us]  |
| Roundtrip | 1 KB      | [13.949 ms, 14.384 ms, 14.815 ms]  |
| Encrypt   | 4 KB      | [13.500 ms, 13.855 ms, 14.169 ms]  |
| Decrypt   | 4 KB      | [123.65 us, 130.85 us, 142.46 us]  |
| Roundtrip | 4 KB      | [14.068 ms, 14.897 ms, 15.745 ms]  |

### ZK Transciphering Benchmarks

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

### ZK Benchmarks

| System  | Operation | Lower Bound | Median    | Upper Bound |
| ------- | --------- | ----------- | --------- | ----------- |
| Groth16 | Setup     | 7.0629 ms   | 7.1022 ms | 7.1502 ms   |
| Groth16 | Prove     | 14.047 ms   | 14.113 ms | 14.187 ms   |
| Groth16 | Verify    | 3.6628 ms   | 3.6798 ms | 3.6966 ms   |
| Plonky2 | Setup     | 2.3948 ms   | 2.4304 ms | 2.4810 ms   |
| Plonky2 | Prove     | 5.4701 ms   | 5.7193 ms | 5.9786 ms   |
| Plonky2 | Verify    | 1.2570 ms   | 1.2574 ms | 1.2579 ms   |
