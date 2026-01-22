

## Chain Synchronization

This document describes how nodes discover peers and synchronize the blockchain.

Synchronization is decentralized and automatic.

No central coordinator is required.

---

## General Behavior

Each node maintains an independent copy of the blockchain.

On startup, a node will:

1. Load its local chain state from disk
2. Accept incoming peer connections
3. Connect to known peers, if configured
4. Request blocks beyond its current state
5. Validate all received data
6. Extend its chain only with valid blocks

Nodes remain synchronized by continuously exchanging newly observed blocks.

---

## Initial Synchronization

When a node starts, it announces its current block height to connected peers.

Peers respond by transmitting blocks with a greater height.

Blocks are sent in ascending order and validated sequentially.

Only valid blocks are retained.

---

## Block Validation

Every received block is independently verified before acceptance.

Validation includes, but is not limited to:

* correct reference to the previous block hash
* valid proof-of-work
* a non-empty transaction set
* a valid difficulty value

Blocks that fail validation are rejected and not relayed.

---

## Chain Selection

When multiple valid chains are observed, the node selects the chain with the greatest accumulated proof-of-work.

Chains with less work, or containing invalid blocks, are ignored.

---

## Ongoing Synchronization

After initial synchronization:

* newly mined blocks are broadcast to peers
* received blocks are validated immediately
* valid blocks are relayed to other peers

Over time, nodes converge on a single shared chain.

Synchronization continues for the lifetime of the node.

---

## Peer Connections

Nodes communicate directly over TCP.

Default peer-to-peer port:

```
8333
```

The peer-to-peer protocol does not use HTTP.

---

## Multiple Nodes (Local Testing)

Multiple nodes may operate on the same machine using distinct ports.

Example:

```
Node A: 0.0.0.0:8333
Node B: 0.0.0.0:8334
Node C: 0.0.0.0:8335
```

Nodes may be connected manually or via predefined peer addresses.

---

## Persistence

Accepted blocks are written to persistent storage.

A node that restarts resumes synchronization from its last known state.

---

## Failure Handling

Temporary network failures do not affect consensus.

A node that falls behind will automatically resynchronize once connectivity is restored.

---

## Summary

Chain synchronization is decentralized, continuous, and self-healing.

Each node independently verifies all data it receives.

Consensus emerges from shared rules, not coordination.

