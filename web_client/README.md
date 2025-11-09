<img width="1874" height="959" alt="Screenshot 2025-10-24 194111" src="https://github.com/user-attachments/assets/fe92bce9-6c8b-43be-8b20-311a85515cc8" />
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
ls server
cargo run
```
On terminal 2:
``` bash
ls client
npm run dev
```
### Conclusion

This system demonstrates a fully encrypted, privacy-preserving voting pipeline using modern FHE technology.
It ensures:

🎯 Trustless computation
🛡 Voter anonymity
💡 Cryptographic integrity
📊 Secure end-to-end tallying
