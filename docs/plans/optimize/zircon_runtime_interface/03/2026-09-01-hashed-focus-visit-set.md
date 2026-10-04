record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/focus.rs
related_tests:
  - tools/tests/test_runtime_interface03_focus_visit_set_performance_contract.py
  - zircon_runtime_interface/src/ui/focus.rs::performance_tests::runtime_interface03_batch22_hashed_focus_visit_set_release_benchmark
---

# Hashed focus visit set

## Scope

Focus-chain traversal used a `BTreeSet` only to reject duplicate or cyclic node visits. Its sorted
iteration order was never observed; product ordering comes from tree preorder and explicit tab
index sorting.

Traversal now uses a `HashSet` preallocated to the tree node count, reducing membership and insert
cost from O(log n) to expected O(1). Candidate order, cycle rejection, missing-node handling, tab
ordering, and the prior partitioned finalization remain unchanged.

## Verification

- TDD RED: the focused contract found `BTreeSet` traversal and no hash-set benchmark.
- Focused ECS/focus static contracts after implementation: `7/7` passed, including the existing
  focus partition contracts.
- Batched static regression: `65/65` passed (`53` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Latest batched static regression after Batch23/24: `69/69` passed (`57 + 9 + 3`).
- Rust behavior coverage compares duplicate membership and lookup results across ordered and hash
  sets without observing either set's iteration order.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending for a later multi-task batch while the external `E:\Git\zr_vm`
dependency worktree prevents immutable validation preflight.

Ownership receipt: exact-path lease request `03632e4438cd41b2b723668ec2ae6e0e`; baseline
attribution request `5e7066bfaf4e404d81a0ffa056ad46a8` (`attributed`).

## Performance contract

The ignored release benchmark inserts 65,536 node IDs into ordered and preallocated hash sets over
21 alternating sample pairs. It requires the hash set to improve P95 by at least 30%. Terminal
nanosecond values must come from the managed Windows receipt before integration, push, or WeCom
reporting.
