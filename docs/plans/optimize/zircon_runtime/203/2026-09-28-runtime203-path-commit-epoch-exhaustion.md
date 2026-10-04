---
title: Runtime203 path commit epoch exhaustion fails closed
category: zircon_runtime
report_id: Runtime203-path-commit-epoch-exhaustion-2026-09-28
date: 2026-09-28
session_id: astra-optimize-20260926-batch-a
implementation_status: implemented_pending_validation
validation_status: static_checks_complete_managed_tests_pending
performance_status: not_applicable
related_code:
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence/registry.rs
  - zircon_runtime/src/foundation/runtime/config_manager/commit_fence/tests.rs
tests:
  - path_commit_epoch_exhaustion_never_reauthorizes_a_stale_fence
plan_sources:
  - docs/plans/optimize/zircon_runtime/203-runtime-preference-config-storage-authority-durability-migration-multiprocess-product-integration-current-working-tree-review.md
---

# Runtime203 path commit epoch exhaustion

## Change

`register_path_gate` previously advanced its per-path `u64` epoch with
`wrapping_add(1)`. A fence can remain live while that counter wraps; after
another registration reaches its old epoch, the old fence's equality check in
`ConfigCommitFence::commit` would authorize a superseded filesystem commit.

The registry now uses `checked_add(1)` while holding the shared path gate.
At `u64::MAX`, each later registration returns a path-bearing I/O error and
leaves the epoch unchanged. A live old fence cannot be reauthorized. Normal
last-fence reclamation still removes the path gate; a new gate may begin only
after no old fence can commit.

The regression sets a live gate to `u64::MAX - 1`, registers a latest fence at
`u64::MAX`, and checks two rejected registrations. It verifies that the old
fence's commit closure does not execute, the latest fence can still commit,
and the registry entry is reclaimed after the final fence drops. Under the
former wrapping increment, the first rejected-registration assertion fails.

## Evidence and limits

| Path | Preimage SHA-256 | Candidate SHA-256 |
|---|---|---|
| `zircon_runtime/src/foundation/runtime/config_manager/commit_fence/registry.rs` | `6129900cd0bd0a811e010cff35cb29bd6ca296242181f65804a7839c30a2f0bb` | `e7bddebd822c6a38a1cab4c67acb271e99d3887b39087978fd1c10d6e88a7e2a` |
| `zircon_runtime/src/foundation/runtime/config_manager/commit_fence/tests.rs` | `71e328b582ac1a491ed9b5be1ea2a10c6fa35398f1f6e59f2be1f16442d535a5` | `d73b0cb97b5de2c267fce54351e6ee9390c6abb4ee0dd0310c4906bc53753a09` |

These two clean preimages are outside the frozen v11 validation manifest. A
read-only coordinator ownership check found no live lease or active Session
with either exact path in its write scope, so this isolated maintenance slice
was edited without waiting for the congested coordinator. Grouped source
attribution completed under request `9006f4b3eef74e2a837772ecf0341d68`;
managed Cargo execution remains pending. The one grouped
successor batch will include the focused test and its Runtime package check.

Pinned Rustfmt 1.94.1 (edition 2021) passed for both Rust files. The scoped
diff and documentation structure checks are recorded in the separate static
receipt. No Cargo test, product startup, or performance profile has run for
this slice. This closes only the process-local epoch-wrap defect in
`CONFIG-P1-009`; the broader cross-process durability and conflict contract
remains open. No latency or allocation improvement is claimed.
