# Repository Ruleset — Fix-Only Policy

This repository implements a **live Proof-of-Work blockchain protocol**.

The consensus rules, revelation block, and monetary schedule are **frozen**.

From this point forward, the repository operates under a **FIX-ONLY policy**.

## 1. What Is Allowed

The following changes are permitted:

* Bug fixes that **do not change consensus**
* Crash fixes
* Determinism fixes
* Performance improvements that preserve identical results
* Documentation updates
* Comment and formatting improvements
* Build, CI, or tooling fixes
* Refactoring that produces **byte-for-byte identical behavior**

All fixes must preserve:

* Revelation hash
* Block hashes
* Transaction validity
* UTXO behavior
* Proof-of-Work rules

## 2. What Is NOT Allowed

The following changes are **strictly forbidden**:

* Changes to consensus rules
* Changes to block structure
* Changes to hashing algorithms
* Changes to difficulty logic semantics
* Changes to reward schedule or supply
* Revelation modification
* Hidden flags, admin controls, or governance logic
* “Temporary” exceptions or experimental paths

If a change alters chain history or validity, it will be rejected.

## 3. Contribution Rules

* All changes must be submitted via **Pull Request**
* Direct pushes to protected branches are disabled
* Pull Requests must describe:

  * What is fixed
  * Why it is safe
  * Why it does NOT change consensus

Maintainers may request:

* Reproduction steps
* Determinism proof
* Comparison of before/after block hashes

## 4. Consensus Freeze Principle

Consensus is defined by **what existing nodes already accept**.

If a bug is discovered:

* The fix must move **forward**
* History is not rewritten
* The network is not reset

This repository does not follow majority opinion.
It follows **existing Proof-of-Work reality**.

## 5. Final Statement

This is not an application repository.
This is a **protocol repository**.

The goal is correctness, stability, and predictability, not speed of change.

**Fixes only.
Rules are law.**
