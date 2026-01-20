# Security Considerations

This document outlines known security properties and limitations.

The system assumes an adversarial environment.

---

## Trust Model

Nodes do not trust peers.

All received data is validated locally.

---

## Proof-of-Work Security

Security relies on the assumption that honest nodes control the majority of total computational power.

An attacker controlling sufficient power may attempt to create an alternative chain.

---

## Double Spending

Transactions become increasingly resistant to reversal as blocks are added after them.

Shallow confirmations may be reversed by chain reorganization.

---

## Network Attacks

The following attacks are possible or partially mitigated:

* **Eclipse attacks**
  A node may be isolated by malicious peers.

* **Spam blocks**
  Invalid blocks are rejected but may consume bandwidth.

* **Denial of service**
  Limited protections exist in the current version.

---

## Limitations

This implementation does not currently include:

* peer authentication
* encrypted connections
* rate limiting
* mempool fee prioritization

These may be addressed in later versions.

---

## Operational Guidance

Nodes are encouraged to:

* connect to multiple peers
* remain online continuously
* verify all received data

---

## Summary

Security emerges from decentralization and verification.

The system favors simplicity and transparency over complexity.
