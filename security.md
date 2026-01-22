

## Security Considerations

This document describes the security assumptions, properties, and known limitations of the system.

The system is designed to operate in a fully adversarial environment.

---

## Trust Model

Nodes do not trust peers by default.

All data received from the network is independently verified before acceptance.

Security is derived from verification, not trust.

---

## Proof-of-Work Security

Consensus security relies on the assumption that honest nodes control a majority of the total computational power.

An adversary with sufficient hashing power may attempt to construct an alternative chain.

The longest valid chain is considered authoritative.

---

## Double Spending

Transactions gain resistance to reversal as additional blocks are built on top of them.

Reorganizations are possible at low confirmation depths.

Finality increases probabilistically over time.

---

## Network Attacks

The system is exposed to known network-level attacks, including:

* **Eclipse attacks**
  A node may be isolated by a controlled set of peers.

* **Spam and invalid blocks**
  Invalid data is rejected, but may temporarily consume bandwidth and processing resources.

* **Denial-of-service attacks**
  Only limited mitigations are implemented in the current version.

---

## Limitations

This version intentionally omits several protections, including:

* peer authentication
* encrypted transport
* rate limiting and anti-spam mechanisms
* mempool fee prioritization

These features may be introduced in future revisions as the system evolves.

---

## Operational Guidance

For improved resilience, nodes are encouraged to:

* maintain connections to multiple independent peers
* remain online to observe chain activity
* verify all data locally and continuously

---

## Summary

Security emerges from decentralization, proof-of-work, and independent verification.

The system prioritizes simplicity, auditability, and correctness over complexity.

No trust is required beyond the rules themselves.

---

### Why this is better (briefly)

* Reads like a **whitepaper / protocol doc**, not a README
* Avoids marketing language
* States limitations clearly (very Satoshi-like)
* Uses **probabilistic finality language** correctly
* Emphasizes *rules over trust*

