## Private-First P2P Architecture

This document specifies the **networking layer** for Bitcoin v0.2 — Revelation Edition.

Networking is **non-consensus**.
All consensus rules are enforced exclusively by local validation and Proof-of-Work.

This design intentionally starts **private-first** and can be opened to public P2P later without changing consensus behavior.

## 1. Design Principles

1. **Networking is transport only**

   * Peers may transmit data
   * Peers MUST NOT define truth

2. **Consensus is local**

   * All blocks and transactions are validated locally
   * Network input is always untrusted

3. **Private by default**

   * No automatic peer discovery
   * No public listening unless explicitly enabled

4. **Public later, unchanged**

   * Opening the network MUST NOT require consensus changes
   * The same validation logic applies to private and public modes

## 2. Trust Model

* All peers are **untrusted**
* No peer input is authoritative
* No identity, reputation, or voting system exists
* Invalid data is silently dropped

Peers MAY propose data.
Nodes MUST decide validity.

## 3. Network Modes

### 3.1 Private Mode (Default)

Private mode is intended for:

* early testing
* multi-node validation
* controlled deployments

Characteristics:

* Static peer list
* No discovery
* Optional listening socket
* Explicit configuration required

Example:

```text
Node A ↔ Node B ↔ Node C
```

No peer outside the configured list can connect.

### 3.2 Public Mode (Optional, Future)

Public mode enables:

* peer discovery
* inbound connections
* open gossip

Public mode MUST:

* use the same message formats
* use the same validation logic
* impose strict resource limits

Public mode introduces **more peers**, not **more authority**.

## 4. Peer Configuration

Peers are defined explicitly in private mode.

Example configuration:

```toml
[p2p]
mode = "private"
listen = false
peers = [
  "127.0.0.1:8333",
  "192.168.1.10:8333"
]
```

No DNS seeds or discovery mechanisms are used in private mode.

## 5. Message Model

All network messages are length-prefixed binary frames.

### 5.1 Message Types

```rust
enum Message {
    Ping,
    Pong,
    GetTip,
    Tip { height: u64, hash: [u8; 32] },
    GetBlock { hash: [u8; 32] },
    Block { block: Block },
    GetTx { txid: [u8; 32] },
    Tx { tx: Transaction },
}
```

Message handling MUST NOT alter consensus logic.

## 6. Data Flow

### 6.1 Receiving a Block

```
receive bytes
    ↓
deserialize message
    ↓
extract block
    ↓
submit to chain validation
    ↓
IF valid:
    store
    rebroadcast
ELSE:
    drop
```

No peer is notified of validation results.

### 6.2 Receiving a Transaction

Transactions MAY be accepted into a mempool or ignored entirely.

Transactions MUST be re-validated when included in a block.

## 7. Validation Boundary

The networking layer MUST interact with consensus only through explicit boundaries.

Recommended interface:

```rust
fn submit_block(block: Block) -> Result<(), ValidationError>;
fn submit_transaction(tx: Transaction) -> Result<(), ValidationError>;
```

Networking code MUST NOT:

* modify blocks
* reorder transactions
* adjust difficulty
* influence chain selection

## 8. Resource Limits

To prevent abuse, the following limits SHOULD be enforced:

* Maximum message size
* Maximum blocks per peer per time window
* Connection timeouts
* Read/write rate limits

Peers exceeding limits MAY be disconnected without explanation.

## 9. Persistence and State

* Networking state is ephemeral
* No network-learned data is persisted without validation
* Disk state remains authoritative

Restarting the node MUST NOT require network access.

## 10. Revelation Handling

* Revelation block is assumed by all nodes
* Revelation is not transmitted over the network
* Nodes with mismatched revelation hashes will fail to synchronize naturally

## 11. Security Considerations

This design explicitly accepts the following risks:

* Sybil attacks
* Eclipse attempts
* Bandwidth abuse

These risks do NOT affect consensus correctness.

Security improvements MUST NOT alter validation rules.


## 12. Future Extensions (Non-Consensus)

The following MAY be added without consensus impact:

* Peer discovery
* Encryption
* Authentication
* Compression
* Inventory announcements
* Header-first sync

All extensions MUST preserve the validation boundary.


## 13. Final Principle

Networking distributes data.
Proof-of-Work determines truth.

Private first.
Public later.
Consensus unchanged.
