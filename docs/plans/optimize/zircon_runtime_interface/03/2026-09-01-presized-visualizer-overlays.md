record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/visualizer.rs
  - zircon_runtime_interface/src/ui/surface/render/visualizer/overlay_performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_visualizer_overlay_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/visualizer/overlay_performance_tests.rs::runtime_interface03_batch24_visualizer_overlay_capacity_release_benchmark
---

# Presized visualizer overlays

## Scope

`visualizer_overlays` always emits one wireframe overlay per paint element before optional clip,
text, batch, resource, and overdraw overlays, but its result vector previously started empty. It now
reserves the guaranteed `elements.len()` lower bound. Overlay order, optional overlay admission, and
payload values are unchanged; the implementation does not reserve for optional worst-case output.

## Verification

- TDD RED: the focused contract found `Vec::new()` and no capacity benchmark.
- Focused new and existing visualizer static contracts after implementation: `4/4` passed.
- Batched static regression: `69/69` passed (`57` RuntimeInterface03 performance contracts,
  `9` input-routing receipt contracts, and `3` asset-palette performance contracts).
- Rust behavior coverage compares the reserved product path with the former unreserved path for
  plain wireframe input.
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

The ignored release benchmark builds 4,097 plain overlays over 11 alternating samples. It requires
the reserved product capacity to equal the guaranteed output count and to be at least 33% below the
former unreserved growth capacity. It logs both capacities and observational P95 nanoseconds;
terminal values must come from the managed Windows receipt before integration, push, or WeCom
reporting.
