<img width="609" height="677" alt="1000076284" src="https://github.com/user-attachments/assets/0fec946c-4bdc-4a6a-b955-dbc316293bb6" />
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
cargo run
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
