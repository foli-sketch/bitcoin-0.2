# ⛓ Bitcoin v0.4.0 — Revelation Edition

A self-validating Proof-of-Work Layer-1 blockchain implementation  
**Consensus v4 — FINAL / FROZEN**

Repository:  
https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2

---

## Overview

Bitcoin Revelation is a minimal, deterministic implementation of a
Proof-of-Work peer-to-peer blockchain.

All Layer-1 consensus rules are explicitly defined, versioned, and enforced
locally by every node.

There are:
- No trusted authorities
- No checkpoints
- No governance keys
- No external dependencies

Each node independently verifies all blocks, transactions, and history.

---

## Consensus Status

- **Consensus Version:** v4
- **Specification Version:** v1.0
- **Status:** FINAL / FROZEN
- **Genesis Block:** Hard-coded and cryptographically pinned
- **Monetary Policy:** Fixed halving schedule
- **Fork Choice Rule:** Cumulative Proof-of-Work (not height)
- **Coinbase Maturity:** Enforced by consensus
- **Difficulty Adjustment:** Full 256-bit target arithmetic

⚠️ Any change to consensus rules requires an **explicit, version-gated hard fork**.

---

## Canonical Specification

The protocol specification is the **sole source of truth**.

```

src/spec.rs

```

Any modification to this file constitutes a **consensus fork**.

Developer documentation can be generated with:

```

cargo doc --no-deps

```

---

## What This Repository Contains

- Full validating node
- Proof-of-Work mining
- Deterministic UTXO ledger
- Encrypted HD wallet (BIP-39)
- Coinbase maturity enforcement
- Fee-based mempool
- Cumulative-work fork choice
- Transport-agnostic P2P networking
- Optional REST API (non-consensus)
- Persistent block and UTXO storage

---

## Network Model

- Permissionless
- Peer-to-peer
- No trusted bootstrap nodes
- No checkpoints
- Offline-capable (store-and-forward)

Connectivity affects **propagation speed**, not **validity**.

---

## Forking Philosophy

Forking is permitted by design.

Anyone may:
- Run a node
- Mine blocks
- Fork the code
- Propose or deploy alternative consensus rules

Consensus is **local, voluntary, and enforced by computation**.

---

## Security Model

- All data is locally verified
- Invalid blocks are rejected deterministically
- Invalid transactions never enter the ledger
- Networking does not override consensus

There is no mechanism to override validation.

---

## Disclaimer

This software is provided **as-is**.

There is:
- No warranty
- No authority
- No guarantee of value
- No recovery mechanism

Cryptographic systems are unforgiving.

---

⛓ *Time is the final judge.*
```

---
