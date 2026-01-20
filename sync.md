# Chain Synchronization

This document describes how nodes discover peers and synchronize the blockchain.

Synchronization is automatic and requires no central coordinator.

---

## General Behavior

Each node maintains its own copy of the blockchain.

When a node starts, it will:

1. Load its local blockchain from disk
2. Listen for incoming peer connections
3. Connect to known peers if configured
4. Request blocks it does not yet have
5. Validate received blocks
6. Extend its chain if the blocks are valid

Nodes remain synchronized by continuously exchanging new blocks.

---

## Initial Synchronization

On startup, a node sends a synchronization request to all connected peers.

The request includes the node’s current block height.

Peers respond by sending all blocks with a height greater than the requester’s height.

Blocks are sent sequentially in ascending order.

---

## Block Validation

Each received block is validated before being accepted.

Validation includes:

* correct reference to the previous block hash
* valid proof-of-work
* non-empty transaction list
* valid difficulty value

Invalid blocks are discarded and not relayed.

---

## Chain Selection

If multiple valid chains are observed, the node selects the chain with the most accumulated proof-of-work.

Shorter or invalid chains are ignored.

---

## Ongoing Synchronization

After initial synchronization:

* newly mined blocks are broadcast to peers
* received blocks are immediately validated and relayed
* all peers converge on the same chain over time

Synchronization continues for the lifetime of the node.

---

## Peer Connections

Nodes communicate directly using TCP.

Default peer-to-peer port:

```
8333
```

The peer-to-peer port does not use HTTP.

---

## Multiple Nodes (Local Testing)

Multiple nodes may be run on the same machine using different ports.

Example:

```
Node A: 0.0.0.0:8333
Node B: 0.0.0.0:8334
Node C: 0.0.0.0:8335
```

Nodes may be manually connected or connected using predefined seed addresses.

---

## Persistence

Synchronized blocks are written to disk.

A node that is shut down and restarted will resume synchronization from its last known height.

---

## Failure Handling

Temporary disconnections do not affect consensus.

A node that falls behind will automatically resynchronize when connectivity is restored.

---

## Summary

Synchronization is decentralized, continuous, and automatic.

No trusted servers or coordinators are required.

Each node independently verifies all data it receives.
