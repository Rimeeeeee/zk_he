This project is a part of our college final year and may contain prospect of research paper if it happens:)

### Symmetric Benchmarks

| Cipher       | Data Size | Time per op                       | Outliers | Mild | Severe |
| ------------ | --------- | --------------------------------- | -------- | ---- | ------ |
| AES-256-CBC  | 1 KB      | [1.5498 µs, 1.5629 µs, 1.5769 µs] | 4%       | 2%   | 2%     |
| ChaCha20     | 1 KB      | [833.42 ns, 845.06 ns, 857.85 ns] | 10%      | 5%   | 5%     |
| Camellia-256 | 1 KB      | [16.215 µs, 16.417 µs, 16.636 µs] | 5%       | 3%   | 2%     |

---

### HE Benchmarks

| Operation   | Data Size | Time per op                       | Outliers | Mild | Severe |
| ----------- | --------- | --------------------------------- | -------- | ---- | ------ |
| Encrypt u32 | 4 bytes   | [2.3689 ms, 2.3702 ms, 2.3720 ms] | 11%      | 5%   | 6%     |
| Add         | 4 bytes   | [627.39 ms, 628.10 ms, 629.02 ms] | 5%       | 4%   | 1%     |
| Mul         | 4 bytes   | [8.4997 s, 8.5060 s, 8.5123 s]    | 2%       | 2%   | 0%     |
| Decrypt u32 | 4 bytes   | [14.709 µs, 14.731 µs, 14.752 µs] | 3%       | 1%   | 2%     |

### ZK Benchmarks

# ZK Voting Benchmark (10 Voters, 3 Candidates)

| System  | Operation | Lower Bound | Median    | Upper Bound |
| ------- | --------- | ----------- | --------- | ----------- |
| Groth16 | Setup     | 7.0629 ms   | 7.1022 ms | 7.1502 ms   |
| Groth16 | Prove     | 14.047 ms   | 14.113 ms | 14.187 ms   |
| Groth16 | Verify    | 3.6628 ms   | 3.6798 ms | 3.6966 ms   |
| Plonky2 | Setup     | 2.3948 ms   | 2.4304 ms | 2.4810 ms   |
| Plonky2 | Prove     | 5.4701 ms   | 5.7193 ms | 5.9786 ms   |
| Plonky2 | Verify    | 1.2570 ms   | 1.2574 ms | 1.2579 ms   |
