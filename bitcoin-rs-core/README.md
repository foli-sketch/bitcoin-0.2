# bitcoin-rs-core

A Bitcoin-inspired **full node core** implementation written **100% in Rust**.

This project explores a clean, modern, and auditable architecture for a Bitcoin-style blockchain node, with a strong emphasis on **consensus correctness**, **safety**, and **long-term maintainability**.

> ⚠️ **Status:** Early development / research phase  
> This software is **not production-ready** and must **not** be used with real funds.

---

## Goals

- Bitcoin-style **UTXO-based ledger**
- **Nakamoto Proof-of-Work** consensus
- Bitcoin-inspired fixed-supply monetary model
- Clear separation of consensus-critical code
- Rust-first design emphasizing safety and correctness
- Educational and experimental value

---

## Non-Goals

- ❌ Bitcoin mainnet compatibility (at this stage)
- ❌ Replacing Bitcoin Core
- ❌ High-throughput or smart-contract execution
- ❌ Rapid feature iteration at the expense of consensus safety

This project prioritizes **correctness over performance**.

---

## Architecture Overview

The repository is structured similarly to Bitcoin Core, but redesigned with Rust idioms and safety guarantees in mind:

bitcoin-rs-core/
├── chain/ # Chainstate, validation, reorg logic
├── consensus/ # Proof-of-Work, difficulty, consensus rules
├── mempool/ # Transaction pool and policy logic
├── p2p/ # Peer-to-peer networking
├── primitives/ # Blocks, transactions, headers
├── rpc/ # RPC interface (future)
├── storage/ # Persistent storage (blocks, UTXO)
├── utxo/ # UTXO set management
├── wallet/ # Wallet logic (optional, future)
├── src/
│ └── main.rs # Node entry point
├── Cargo.toml
└── Cargo.lock


Consensus-critical logic is **intentionally isolated** to minimize the risk of accidental or implicit rule changes.

---

## Building & Running

### Requirements

- Rust (stable)
- Cargo

### Build

```bash
cargo build
Run
cargo run
Design Philosophy
Consensus rules are sacred
Any change to consensus logic must be explicit, reviewed, and intentional.
Silent or accidental rule changes are treated as critical failures.

UTXO ownership over account balances
Ownership is defined by cryptographic proofs over unspent outputs, not mutable account balances.

Safety over performance
A slow, correct node is preferable to a fast, broken one.

Clarity over cleverness
Readable, auditable code is preferred over premature optimization or unnecessary abstraction.

License
MIT License.

Disclaimer
This project is for educational and research purposes only.

It is not affiliated with Bitcoin Core, the Bitcoin project, or any related organizations.
