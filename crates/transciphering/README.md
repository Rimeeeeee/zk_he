# Homomorphic ChaCha20 Transciphering

## Overview

This project demonstrates **transciphering**, the process of decrypting a ciphertext produced by a classical symmetric cipher (ChaCha20) **directly inside a homomorphic encryption (HE) domain**.  
The key idea is to combine **fast symmetric encryption** with **fully homomorphic operations**, allowing computations on encrypted data **without revealing the plaintext or the key**.

In this implementation:

- Symmetric encryption: **ChaCha20**
- Homomorphic encryption: **TFHE (FheUint32)**
- Transciphering: decrypting ChaCha20 inside HE to produce an **encrypted plaintext** block.

---

## Why ChaCha20?

ChaCha20 is chosen because:

1. **High Performance**: Optimized for software and fast on modern CPUs.
2. **Symmetric Stream Cipher**: Generates a keystream that can be XORed with plaintext, which maps nicely to homomorphic operations (addition/XOR).
3. **Security**: Strong against timing attacks and nonce reuse protects confidentiality.
4. **Deterministic & Predictable**: Same key + nonce combination produces the same keystream, which is important when moving operations into HE.

Using ChaCha20 in HE allows us to **demonstrate real symmetric cipher decryption without exposing the key**.

---

## Transciphering Process

### Step 1: Encrypt the Symmetric Key under HE

- The ChaCha20 256-bit key is split into **eight 32-bit words**.
- Each word is **encrypted using TFHE** to produce `FheUint32` values.
- This allows the key to be used in computations **while remaining encrypted**, keeping it confidential.

### Step 2: Convert Ciphertext Block to Integer

- ChaCha20 operates on **32-bit blocks**, but input is a byte array.
- Extract the first 4 bytes and convert into a `u32`.
- This block will later be XORed with the **homomorphic keystream**.

### Step 3: Build ChaCha20 State in HE

- The ChaCha20 state is a **16-word array**:

  | Index | Contents                            |
  | ----- | ----------------------------------- |
  | 0-3   | Constant words ("expand 32-byte k") |
  | 4-11  | Encrypted key words                 |
  | 12    | Counter (starts at 1)               |
  | 13-15 | Nonce (set to 0 in example)         |

- All values are either **TFHE-encrypted (`FheUint32`)** or known constants.
- This is necessary because ChaCha20 rounds operate on the **full state**.

### Step 4: Perform ChaCha20 Rounds Homomorphically

- ChaCha20 standard: **20 rounds (10 double-rounds)**.
- Operations performed in HE:
  - Homomorphic addition (`+`)
  - Homomorphic XOR (`^`)
  - Homomorphic rotation (`rotate_left`)
- This produces a **homomorphic keystream**, fully encrypted, without ever decrypting the key or state.

### Step 5: Add Original State to Transformed State

- ChaCha20 output is computed as `keystream = transformed_state + original_state`.
- Addition is done **homomorphically** using `FheUint32`.
- This ensures correctness while maintaining confidentiality.

### Step 6: XOR Ciphertext with Keystream

- Homomorphic XOR between the **ciphertext block** and the **first keystream word** produces the **plaintext in HE**.
- Resulting value: a `FheUint32` representing the plaintext **without exposing it** to the server or computation environment.

---

## Why This Approach Matters

1. **Confidentiality**: Both the ChaCha20 key and the decrypted plaintext remain encrypted.
2. **Bridging Symmetric and Homomorphic Encryption**: Transciphering enables applications that require **fast symmetric encryption for storage or communication**, while still performing **secure computations homomorphically**.
3. **Practical HE Applications**: Demonstrates a building block for systems like secure cloud computing, privacy-preserving analytics, or encrypted machine learning.

---

## Summary

- ChaCha20 encryption is performed as usual.
- The ChaCha20 key is **encrypted under HE**.
- A single 32-bit ciphertext block is **decrypted inside HE**, producing a homomorphic plaintext.
- All cryptographic operations (add, XOR, rotation) are performed **homomorphically**, preserving security.

This method provides a secure and practical foundation for **homomorphic processing of symmetric ciphertexts**, enabling advanced privacy-preserving computations.
