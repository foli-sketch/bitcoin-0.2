⛓ Bitcoin v0.3.3 — Revelation Edition  
Stable Node, Wallet, Mining & Public P2P Network

Consensus v3 — FINAL / FROZEN

Repository: 👉 https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2

---

## Overview

Bitcoin Revelation v0.3.3 is a stable release that finalizes the wallet, transaction, mempool, mining, API, and P2P networking layers on top of a **frozen Layer-1 Consensus v3**.

This release hard-codes the Genesis block and permanently freezes all consensus rules.  
No chain reset is required and all existing coins remain valid.

A live public seed node is available, allowing anyone to join and mine without manual peer coordination.

✅ No consensus rule changes  
✅ No chain reset required  
✅ Safe upgrade from v0.3.2  
✅ Public P2P network online  

---

## 🌍 Public Seed Node (Live)

```

bitcoin-0-2.fly.dev:8333

````

This node:
- Accepts inbound P2P connections
- Shares peer addresses (`getaddr`)
- Does not mine
- Does not alter consensus
- Exists purely for decentralized bootstrapping

Once connected, nodes discover peers automatically.

---

## Release Status

**v0.3.3 is a finalization release following v0.3.2.**

This release:
- Hard-codes the Genesis block
- Freezes Consensus v3 permanently
- Marks the protocol layer as final

Further consensus changes require a version-gated fork.

---

## What’s Included

- Deterministic HD wallets (BIP39)
- Encrypted wallet storage (AES-256-GCM + PBKDF2)
- ECDSA transaction signing & validation
- Coinbase maturity enforcement
- Mempool validation & transaction relay
- Miner transaction selection from mempool
- Full P2P block & transaction propagation
- Public seed node support
- REST API block explorer
- Command-line wallet interface
- Persistent blockchain & UTXO storage

---

## What’s NOT Changed

❌ No consensus rule changes  
❌ No reward schedule changes  
❌ No difficulty changes  
❌ No protocol fork  

➡ Consensus v3 remains frozen.

---

## Wallet System

The wallet operates above consensus and does not alter validation rules.

- Recovery phrase (BIP39) is shown **once** at wallet creation
- The phrase is **never stored**
- Wallet data is encrypted and stored locally in `data/wallet.dat`

If the recovery phrase is lost, funds cannot be recovered.

---

## Command-Line Wallet

```bash
cargo run wallet balance
cargo run wallet send <pubkey_hash_hex> <amount>
````

---

## REST API (Explorer)

Default endpoint:

```
http://127.0.0.1:8080
```

Endpoints:

* `/status`
* `/blocks`
* `/block/height/{n}`
* `/tx/{txid}`
* `/address/{pubkey_hash}`

---

## ⛏️ Mining & Running a Full Node

⚠️ Always download and build from **release tags**, not `main`.

👉 [https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2/tags](https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2/tags)
**Recommended:** `v0.3.3`

---

## Backward Compatibility

* Fully compatible with v0.3.0+ peers
* No fork, no replay risk
* Existing chains remain valid

---

## Release Identifier

* Tag: `v0.3.3`
* Client: Bitcoin Revelation v0.3.3
* Consensus: v3 (FINAL / FROZEN)

---

## Disclaimer

This software is provided as-is for research, experimentation, and independent node operation.

There is:

* No warranty
* No central authority
* No permission system

The rules are enforced by code, not humans.

⛓ Satoshi-Nakamoto-ITL — Bitcoin v0.3.3
