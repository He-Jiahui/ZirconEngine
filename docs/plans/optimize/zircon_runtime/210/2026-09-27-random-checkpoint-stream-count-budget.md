---
doc_type: optimization-record
status: source_candidate_pending_managed_validation
plan_source: docs/plans/optimize/zircon_runtime/210-runtime-random-authority-stream-checkpoint-replay-consumer-performance-current-working-tree-review.md
related_code:
  - zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
  - zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs
  - zircon_runtime/src/core/runtime/random/limits.rs
tests:
  - zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs
---

# Runtime210: random checkpoint stream count budget

Runtime210's `RT-RANDOM-P0-001` report predates the current working tree. The typed
`StreamAuthorityGenerationMismatch` error, v1 fail-closed test, and `zr_contracts`
workspace/runtime wiring already exist. Their exact contract source hashes match the
2026-09-18 Runtime08 focused closeout; this candidate does not claim or repeat that
repair.

## Bounded decode candidate

`RandomServiceCheckpoint` now caps its v2 stream sequence at 65,536 entries while
deserializing. The visitor grows only from admitted entries, ignores untrusted wire
size hints for allocation, and rejects the first excess element before decoding its
payload or pushing it into the vector. The public `try_new` constructor rejects an
already allocated oversized vector with typed `TooManyStreams { max, actual }`.
`RandomServiceLimits::MVP` reads the same contract cap. The serialized field layout
and bytes for valid checkpoints remain unchanged.

The focused tests cover an exact-cap round trip, the first excess wire element with
an intentionally malformed payload, and the constructor's typed count error. They
are source candidates pending grouped managed Cargo execution; static formatting
and diff checks alone do not establish Rust test results.

## Remaining gates

Runtime154-P1-005 and G09 remain **partial**. Generic serde deserialization still
does not enforce artifact bytes, depth, digest, deadline, or cancellation budgets;
the Runtime's custom smaller per-service limit is still checked at restore. The
complete replay artifact, product integration, Windows Release cost, allocation,
and compatibility gates remain open. No dynamic or performance pass is claimed.
