record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/query_performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_ecs_change_lookup_performance_contract.py
  - zircon_runtime_interface/src/ui/ecs/query_performance_tests.rs::runtime_interface03_batch23_binary_change_lookup_release_benchmark
---

# Binary ECS change lookup

## Scope

`UiEcsProjectionDelta::change` previously scanned every change. Canonical update-only deltas are
ordered by node ID, so the query now attempts `binary_search_by_key` and retains the prior linear
scan as a compatibility fallback for mixed add/remove, deserialized, or caller-mutated ordering.
The delta representation, output ordering, and missing-ID behavior are unchanged.

## Verification

- TDD RED: the focused contract found only the linear scan and no release benchmark.
- Focused static performance contract after implementation: `2/2` passed.
- Batched static regression: `69/69` passed (`57` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares binary/fallback and linear results for ordered, deliberately
  reordered, and missing node IDs.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending for the current multi-task batch while the external `E:\Git\zr_vm`
dependency worktree prevents immutable validation preflight.

Batch submission preflight on 2026-09-01 was rejected before queueing with
`validation_ticket_external_worktree_dirty` for external repo `E:\Git\zr_vm`; no ticket or
terminal benchmark result exists.

Ownership receipt: exact-path lease request `9ae8daf7bd9246d0afa90d12aeb9c672`; baseline
attribution request `a9f4bc0cb20c4475a9a16c160d53f24c` (`attributed`).

## Performance contract

The ignored release benchmark queries the last node in a 65,536-change ordered delta 32 times over
11 alternating samples. It compares the former linear scan with the binary fast path and requires
at least 80% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
