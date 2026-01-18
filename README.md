# Bitcoin v0.2 — Revelation

Bitcoin v0.2 — Revelation is a **live Proof-of-Work blockchain network implemented in Rust**, following early Bitcoin design principles.

The network uses a deterministic genesis block, SHA-256 double-hash Proof-of-Work, and a UTXO-based ledger model.

There is **no central authority**, **no governance**, and **no upgrade mechanism**.

## Network Status

* Live private-origin mainnet
* Deterministic genesis (frozen)
* Proof-of-Work: double SHA-256
* UTXO-based ledger
* Fixed monetary schedule
* No governance
* No protocol upgrades

Nodes independently validate all rules from genesis.

## Requirements

* Rust (stable)
* Cargo

Install Rust: [https://rustup.rs](https://rustup.rs)

## Run a Node

Clone the repository:

```bash
git clone <REPO_URL>
cd bitcoin_v0_2_revelation
```

Run the node:

```bash
cargo run --release
```

On first run:

* The genesis block is created deterministically
* Blocks are mined using Proof-of-Work
* Chain state is stored in the `data/` directory

To preserve chain history, **do not delete** the `data/` directory.

## Consensus

Consensus rules are defined in `CONSENSUS.md`.

Nodes following the same rules and starting from genesis **must reach identical results**.

Consensus is enforced entirely by code and Proof-of-Work.

## Participation

* Anyone may run a node
* Anyone may mine
* No permission is required
* No registration is required

The network does not provide accounts, custodial services, or guarantees.

## Disclaimer

This project is software implementing a blockchain protocol.

It does not represent an investment offering, financial product, or promise of value.
