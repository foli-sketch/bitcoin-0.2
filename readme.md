
# Bitcoin v0.2 — Revelation Edition

Bitcoin v0.2 — Revelation Edition is an experimental peer-to-peer electronic cash system written in Rust.

It is a standalone proof-of-work network intended to demonstrate the essential mechanics of decentralized consensus, block validation, and chain synchronization without reliance on any central authority.

This software is intended for study, experimentation, and long-running private node operation.
It is deliberately minimal.

---

## Design Intent

The objective of this project is not feature completeness, but correctness of the base protocol.

The system is designed so that every node independently verifies all rules required for consensus. No trusted services, checkpoints, or privileged roles exist.

The system provides:

* independent block validation by every node
* proof-of-work–based chain selection
* automatic fork resolution
* deterministic reconstruction of state from genesis

Any node may leave or rejoin the network at any time.

---

## Network Model

Each node maintains a full copy of the blockchain and enforces all consensus rules locally.

A node may:

* validate blocks independently
* mine new blocks
* relay blocks to peers
* synchronize missing blocks automatically
* reorganize when a stronger chain is discovered

Temporary divergence between nodes is expected and resolves naturally through accumulated proof-of-work.

There is no global coordinator.

---

## Consensus Rules

Consensus follows a Bitcoin-style longest-chain-by-work rule:

* blocks must reference a known parent
* blocks must satisfy the current proof-of-work difficulty
* multiple competing chains are permitted
* the chain with the most accumulated work is selected
* nodes reorganize automatically when a stronger chain appears

Block height is derived from ancestry.
There is no external clock and no checkpoint authority.

---

## Revelation Block

The initial block (height 0) is referred to as the *Revelation Block*.

* it has no parent (`prev_hash = 0x00…00`)
* it contains a single deterministic transaction
* it establishes the initial state of the system

Functionally, it serves the role of a genesis block.
The name is used to distinguish intent, not behavior.

All nodes share the same Revelation Block.

---

## Monetary Policy

The monetary rules are fixed and enforced by consensus:

* block subsidy is height-based
* the halving schedule is deterministic
* total supply is capped at 21 million units
* coinbase outputs require maturity before spending

No node may create coins outside these rules.

---

## Protocol Update — v0.2.1 (Consensus v2)

This release introduces **Consensus v2**, a height-activated protocol hardening.

### Summary

* Coinbase maturity is now enforced explicitly at the consensus level.
* The rule is activated by block height and does not invalidate historical blocks.
* Older nodes can still fully synchronize the blockchain.
* Nodes must upgrade to continue mining after activation.

### Activation

* **Consensus v2 activation height:** `1000`

Blocks below this height remain valid under legacy rules.
Blocks at or above this height must obey the new consensus rule.

### Compatibility

* Historical chain data remains valid.
* Full synchronization remains possible for old and new nodes.
* Mining participation after activation requires upgrading to v0.2.1.

### Release Tag

```
v0.2.1-consensus-v2
```

---

## Features

* proof-of-work mining
* fork-capable consensus with reorganization
* Merkle-root-based block structure
* persistent blockchain and UTXO storage
* peer-to-peer networking over raw TCP
* automatic block synchronization
* optional HTTP interface for inspection

---

## HTTP Interface

The HTTP interface is provided for inspection and development only.

Available endpoints:

* `/blocks` — list of known blocks
* `/block/height/:height` — block lookup by height
* `/tx/:txid` — transaction lookup
* `/address/:hash` — address balance and UTXO count

HTML views are available at:

* `/blocks.html`
* `/block/:height`
* `/tx.html/:txid`
* `/address.html/:hash`

The HTTP interface does not participate in consensus and may change without notice.

---

## Data Storage

State is stored locally on disk:

```
data/
├── blocks.json
└── utxos.json
```

Nodes may be stopped and restarted without loss of state.

All state can be reconstructed deterministically from the blockchain.

---

## Local Search and Indexing

The node does not maintain transaction, address, or block-height indexes by default.

It stores only:

* the blockchain
* the current UTXO set

These are sufficient to:

* validate blocks
* enforce consensus
* mine
* synchronize

Search and indexing are policy layers, not consensus requirements.

---

## Running on Mobile (Android / Termux)

A full validating node can be run on a mobile device using Termux.

This is possible because the node:

* maintains no mandatory indexes
* stores only minimal state (blocks and UTXOs)
* reconstructs all state deterministically

There is no mobile mode.
A phone running this software is a full node.

---

## Requirements

* Android device (ARM64 recommended)
* Termux installed from F-Droid
* approximately 200 MB of free storage
* stable internet connection

Termux from Google Play is deprecated.
Use the F-Droid distribution.

---

## Installation (Termux)

Install Termux:

[https://f-droid.org/packages/com.termux/](https://f-droid.org/packages/com.termux/)

Update packages:

```sh
pkg update && pkg upgrade
```

Install dependencies:

```sh
pkg install git rust clang
```

Verify installation:

```sh
rustc --version
cargo --version
```

Clone the repository:

```sh
git clone https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2.git
cd bitcoin-0.2
```

Build:

```sh
cargo build
```

Run:

```sh
cargo run
```

On first execution, the node will:

* create the Revelation Block
* initialize local storage
* connect to peers
* synchronize the blockchain

Validation behavior is identical to desktop operation.

---

## Building and Running (Desktop)

Build:

```sh
cargo build
```

Run:

```sh
cargo run
```

On startup, a node will:

* load the local blockchain from disk
* create the Revelation Block if none exists
* listen for peer connections
* request missing blocks from peers
* begin mining once synchronization stabilizes

---

## Running Multiple Nodes (Private Network)

Node A:

```sh
cargo run
```

Node B (separate directory and port):

```sh
cargo run
```

Each node must use a unique data directory and P2P port.

Nodes converge automatically on the strongest chain.

---

## Limitations

This software is experimental.

It intentionally omits:

* mandatory indexes
* advanced mempool policies
* network encryption
* denial-of-service protections
* fast synchronization or snapshots
* production-grade peer discovery

These may be added without modifying consensus rules.

---

## Release Status

This release freezes the consensus rules.

* independent network
* fixed monetary policy
* stable proof-of-work and fork-selection rules

Tag:

```
v0.2.1-consensus-v2
```

---

## Purpose

This project demonstrates that a peer-to-peer proof-of-work system can be implemented clearly, compactly, and without trusted parties.

If the software continues to run unchanged, the rules were sufficient.

---

## License

Open source.
Free to use, modify, and redistribute.

