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
