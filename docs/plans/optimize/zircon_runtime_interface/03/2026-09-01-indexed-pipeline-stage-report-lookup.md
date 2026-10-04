record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/pipeline/stage.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report.rs
  - zircon_runtime_interface/src/ui/pipeline/frame_report/performance_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_pipeline_stage_report_lookup_performance_contract.py
  - zircon_runtime_interface/src/ui/pipeline/frame_report/performance_tests.rs::runtime_interface03_batch19_stage_report_index_release_benchmark
---

# Indexed pipeline stage-report lookup

## Scope

`UiPipelineFrameReport::stage_report` previously scanned the stage vector for every query even when
the report used the canonical complete runtime schedule order.

Runtime stages now probe their explicit canonical index first and validate the stored stage before
returning it. Sparse, reordered, archived, or malformed reports retain the original linear lookup.
The optimization does not change serialized order or require a new index field.

## Verification

- TDD RED: the focused contract found no indexed probe, linear oracle, or benchmark.
- Focused Pipeline/render-frame capacity static contracts after implementation: `6/6` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `61/61`
  passed (`49 + 9 + 3`).
- Rust behavior coverage compares indexed and linear lookup for canonical, sparse, reordered, and
  archived-stage reports.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending for a later multi-task batch while the external `E:\Git\zr_vm`
dependency worktree prevents immutable validation preflight.

## Performance contract

The ignored release benchmark resolves the last canonical runtime stage 100,000 times over 11
alternating samples. It compares the retained O(n) scan with the O(1) indexed probe and requires at
least 50% P95 improvement. Terminal nanosecond values must come from the managed Windows receipt
before integration, push, or WeCom reporting.
