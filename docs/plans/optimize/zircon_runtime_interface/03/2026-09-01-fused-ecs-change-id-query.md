record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/ecs.rs
  - zircon_runtime_interface/src/ui/ecs/query_performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_ecs_change_query_fusion_performance_contract.py
  - zircon_runtime_interface/src/ui/ecs/query_performance_tests.rs::runtime_interface03_batch21_fused_change_id_query_release_benchmark
---

# Fused ECS change-ID query

## Scope

`UiEcsProjectionDelta::node_ids_by_change_kind` previously called `changes_by_kind`, allocated an
intermediate vector of references, then allocated and populated the returned node-ID vector.

The query now filters the authoritative change slice and projects matching IDs in one iterator
pipeline. Output order and filtering semantics are unchanged. The former staged implementation is
retained under tests as the behavior and performance oracle.

## Verification

- TDD RED: the focused contract found the staged call and no fused benchmark.
- Focused ECS/focus static contracts after implementation: `7/7` passed.
- Batched static regression: `65/65` passed (`53` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Latest batched static regression after Batch23/24: `69/69` passed (`57 + 9 + 3`).
- Rust behavior coverage compares all Added, Removed, and Updated results with the staged oracle.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending for a later multi-task batch while the external `E:\Git\zr_vm`
dependency worktree prevents immutable validation preflight.

Ownership receipt: exact-path lease request `03632e4438cd41b2b723668ec2ae6e0e`; baseline
attribution request `5e7066bfaf4e404d81a0ffa056ad46a8` (`attributed`).

## Performance contract

The ignored release benchmark projects Updated IDs from 65,536 alternating changes 32 times over
11 alternating samples. It compares the two-allocation staged path with the fused path and requires
at least 20% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
