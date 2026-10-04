record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/hit.rs
related_tests:
  - zircon_runtime_interface/src/ui/surface/hit/route_validation_performance_tests.rs
  - tools/tests/test_runtime_interface03_hit_path_route_validation_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/hit/route_validation_performance_tests.rs::runtime_interface03_batch44_46_borrowed_hit_route_validation_release_benchmark
---

# Borrowed hit-route validation

## Scope

`UiHitPath::with_route` only used a temporary reversed `Vec<UiNodeId>` to validate a caller-supplied
bubble route, then discarded that allocation before retaining the authoritative root-to-leaf path.
The validation now compares the two borrowed iterators directly. Target consistency, route ordering,
panic text, retained ownership, and public signatures are unchanged.

## Verification

- TDD RED: the focused contract found a temporary `expected_bubble_route` collection.
- Focused Batch46 hit-route validation contract after implementation: `2/2` passed.
- Batched static regression after Batch44-46: `113/113` passed (`101` RuntimeInterface03
  performance contracts, `9` input-routing receipt contracts, and `3` asset-palette performance
  contracts).
- Rust behavior coverage compares borrowed validation with the former allocating oracle for matching,
  short, and content-mismatched routes.
- Existing input-response `#[should_panic]` contracts retain their expected diagnostic text.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Ownership receipt: exact-path lease request `46446f2cca454283bb3ff7b962cf69a7`; refreshed Batch46
  lease request `9ea5ada80b614585968dd61d34184218`; baseline attribution request
  `c8e8b7d33673418eb3cee024ded5831f` (`attributed`).

## Performance contract

The ignored release benchmark validates a 2,048-node route 131,072 times over 11 alternating samples.
It compares the former temporary reversed vector with direct iterator equality and requires at least
20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt before
integration, push, or WeCom reporting.

Batch44-46 was submitted in combined snapshot `2742` (request `714a35d557244e65b16f9f2055849148`);
managed validation request `9a89aca769ae4ed987c86b74aa6f817b` was rejected before queueing by the
external `E:\Git\zr_vm` dirty-worktree preflight. No validation ticket, Cargo run, or terminal
benchmark result exists for Batch46.
