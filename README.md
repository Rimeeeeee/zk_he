This project is a part of our college final year and may contain prospect of research paper if it happens:)

## For 1st review our checklist it:

- Add symmetric encryption decryption modules. As of now it is aes,camellia and chacha. -DONE
- Add homomorphic encryption and decryption modules tfhe -DONE
- Run Benchmark.Will be included here

# Symmetric Benchmarks

| Cipher       | Data Size | Time (µs) / ns                    | Outliers | Mild | Severe |
| ------------ | --------- | --------------------------------- | -------- | ---- | ------ |
| AES-256-CBC  | 1 KB      | [1.5498 µs, 1.5629 µs, 1.5769 µs] | 4%       | 2%   | 2%     |
| ChaCha20     | 1 KB      | [833.42 ns, 845.06 ns, 857.85 ns] | 10%      | 5%   | 5%     |
| Camellia-256 | 1 KB      | [16.215 µs, 16.417 µs, 16.636 µs] | 5%       | 3%   | 2%     |

# HE Benchmarks

### HE Benchmarks

| Operation    | Time per op                              | Outliers | Mild | Severe |
|--------------|------------------------------------------|----------|------|--------|
| Encrypt u32  | [2.3689 ms, 2.3702 ms, 2.3720 ms]        | 11%      | 5%   | 6%     |
| Add          | [627.39 ms, 628.10 ms, 629.02 ms]        | 5%       | 4%   | 1%     |
| Mul          | [8.4997 s, 8.5060 s, 8.5123 s]           | 2%       | 2%   | 0%     |
| Decrypt u32  | [14.709 µs, 14.731 µs, 14.752 µs]        | 3%       | 1%   | 2%     |
