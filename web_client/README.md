<img width="970" height="749" alt="Screenshot 2025-11-09 183536" src="https://github.com/user-attachments/assets/1c2bdf29-5203-4490-b105-7ce7beb453ba" />

## 🗳️ Private End-to-End Encrypted Voting System (FHE-Powered)

A secure, trustless voting platform built using **Fully Homomorphic Encryption (FHE)**.
Votes remain encrypted at all times, ensuring privacy even during tallying.

This system uses:

✅ *Rust (Actix-web backend)*
✅ *TFHE-rs for Homomorphic Encryption*
✅ *React + Vite frontend*
✅ *Local DB + filesystem for key & ballot storage*

### Core Idea

This platform enables users to:

* Register for a voting token

* Vote ONCE using that token

* Submit an encrypted vote vector

* Have the server tally results without decrypting individual votes

**No one — not even the server — can see which candidate a voter selected.**

### How to run:
On terminal 1:
``` bash
cd server
cargo +nightly run
```
On terminal 2:
``` bash
cd client
npm run dev
```
### Conclusion

This system demonstrates a fully encrypted, privacy-preserving voting pipeline using modern FHE technology.
It ensures:

🎯 Trustless computation
🛡 Voter anonymity
💡 Cryptographic integrity
📊 Secure end-to-end tallying
