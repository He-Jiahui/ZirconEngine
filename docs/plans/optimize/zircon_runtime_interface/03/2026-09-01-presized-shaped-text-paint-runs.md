record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_text_paint_run_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/text_shape/performance_tests.rs::runtime_interface03_batch9_presized_shaped_text_runs_release_benchmark
---

# Presized shaped-text paint runs

## Scope

`text_paint_runs_from_shaped` previously appended every nonempty shaped cluster to a `Vec` that
started with zero capacity. The final upper bound is already present in the shaped line metadata,
so large or repeatedly projected text caused avoidable vector reallocations and element moves.

The projection now sums each line's cluster count and initializes the output vector with that
capacity. Empty clusters remain filtered, so sparse input may retain unused capacity but cannot
trigger growth; run order, fields, geometry, and serialized output are unchanged.

## Verification

- TDD RED: the focused contract found zero-capacity vector growth and no benchmark module.
- Focused text paint-run capacity static contracts after implementation: `3/3` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `47/47`
  passed (`26 + 9 + 12`).
- Rust behavior coverage compares the complete presized output with the prior unreserved path.
- Scoped Rust 1.94.1 formatting: passed.
- Managed Windows Rust 1.94.1 behavior and release benchmark: pending a multi-task asynchronous
  coordinator batch; no terminal performance number is claimed yet.

Managed submission:

- snapshot: `2709`;
- snapshot request: `a452d135acc047bd80a94be65d76034e`;
- attribution request: `65709559563c4d51a3197c9115933e94`;
- submit request: `7259a33853ac4ec09c4775f18564439c`;
- validation ticket: `24e43467b8ba4d9c98e3ff6e311b3743`;
- source manifest: `e42aba9a90335586891eb78f8ab382ac57653376d4569ea4dcd5e574053fb61f`;
- command: `cargo +1.94.1 test -p zircon_runtime_interface --locked --release
  runtime_interface03_batch9 -- --include-ignored --nocapture`;
- submitted state: `queued` (asynchronous; intentionally not polled).

## Performance contract

The ignored release benchmark repeatedly projects 128 shaped lines for 2,048 iterations over 11
alternating samples, comparing zero-capacity growth with upper-bound preallocation. The P95 gate
requires at least 5% improvement. Terminal nanosecond values must come from the managed Windows
receipt before integration, push, or WeCom reporting.
