# Security Model

Bitcoin Revelation follows a local-verification security model.

Nodes do not trust peers or infrastructure.

---

## Wallet Security

- Recovery phrase shown once
- Phrase is never stored
- Wallet encrypted with AES-256-GCM
- PBKDF2 key derivation

Loss of the recovery phrase results in permanent loss of funds.

---

## Node Security

- All blocks and transactions verified locally
- Invalid data is rejected

---

## Network Security

- No trusted peers
- No checkpoints
- No central authority

---

## Responsibility

Users are responsible for:
- Backups
- Machine security
- Running trusted builds
