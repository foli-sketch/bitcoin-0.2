# 🔗 Node Sync Guide (Public PoW Network)

This document explains **how to sync a full node** with the network **while other nodes are actively mining**.

Repository:
[https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2](https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2)

---

## 🧠 Core Principles

* The network **never stops mining**
* New nodes can **join and sync at any time**
* Consensus is decided by **cumulative Proof-of-Work**
* GitHub distributes code — **PoW decides truth**

---

## 0️⃣ Requirements

* Linux / macOS / WSL / Termux
* Rust (stable)
* Git
* Internet or private P2P connection

---

## 1️⃣ Clone the Public Code

```bash
git clone https://github.com/Satoshi-Nakamoto-ITL/bitcoin-0.2.git
cd bitcoin-0.2
```

⚠️ Cloning the repository does **not** grant trust.
All trust comes from **local validation + PoW**.

---

## 2️⃣ Build the Node

```bash
cargo build --release
```

Binary output:

```bash
./target/release/bitcoin
```

---

## 3️⃣ Start Node in Sync Mode

Sync mode disables mining while the node catches up.

```bash
./target/release/bitcoin \
  --node \
  --sync \
  --datadir ./data
```

The node will:

* Connect to peers
* Exchange chain metadata
* Compare cumulative difficulty
* Begin downloading blocks

Mining nodes continue mining during this process.

---

## 4️⃣ Peer Discovery

### Option A: Manual Peer Connections

```bash
./target/release/bitcoin \
  --node \
  --sync \
  --connect 192.168.1.10:8333 \
  --connect 203.xxx.xxx.xxx:8333
```

### Option B: Seed Nodes (Recommended)

Seed nodes help initial discovery but **do not control consensus**.

```rust
const SEED_NODES: &[&str] = &[
    "seed1.itlcoin.org:8333",
    "seed2.itlcoin.org:8333",
];
```

---

## 5️⃣ Headers-First Sync (Fast & Safe)

Sync process:

1. Download block headers
2. Validate Proof-of-Work per header
3. Select chain with highest cumulative difficulty
4. Download full blocks

```
HEADERS → VALIDATE → BEST CHAIN → BLOCKS
```

Forks are resolved automatically by cumulative PoW.

---

## 6️⃣ Full Block Validation

Every block is validated locally:

* Previous block hash
* Timestamp rules
* Proof-of-Work
* Transaction structure
* UTXO correctness

❌ No checkpoints
❌ No trusted nodes
❌ No GitHub authority

---

## 7️⃣ Receiving New Blocks During Sync

This is expected behavior.

* New blocks are queued
* Sync continues
* Latest blocks are applied
* Reorganization happens if required

No restart is needed.

---

## 8️⃣ Sync Completion

When synced:

```text
[SYNC COMPLETE]
Height: XXXXX
```

The node automatically switches to **normal mode**.

---

## 9️⃣ Enable Mining (Optional)

```bash
./target/release/bitcoin \
  --node \
  --mine \
  --threads 4
```

Mining never interferes with syncing or validation.

---

## 🔐 Security Rules (Mandatory)

* Trust cumulative Proof-of-Work only
* Validate everything locally
* Maintain multiple peers
* Support automatic chain reorganization
* No admin keys
* No forced updates

---

## 🔄 Updating the Software

```bash
git fetch
git pull
cargo build --release
```

⚠️ Consensus rule changes cause forks.
Users always choose which rules to run.

---

## 🧩 Mental Model

> GitHub distributes software
> Proof-of-Work decides consensus

This is a decentralized system by design.

---

End of document.
