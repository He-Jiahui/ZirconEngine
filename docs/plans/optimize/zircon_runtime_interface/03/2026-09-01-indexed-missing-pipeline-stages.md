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
  - tools/tests/test_runtime_interface03_missing_pipeline_stage_performance_contract.py
  - zircon_runtime_interface/src/ui/pipeline/frame_report/performance_tests.rs::runtime_interface03_batch18_missing_stage_index_release_benchmark
---

# Indexed missing pipeline stages

## Scope

`UiPipelineFrameReport::missing_required_stages` previously called a linear `stage_report` scan for
each required runtime stage, repeating comparisons across the fixed schedule.

Each runtime stage now has an explicit internal index. One pass marks a fixed presence table and a
second pass emits missing stages in canonical order. Archived diagnostic stages map to `None` and
remain excluded exactly as before. The original nested lookup is retained as the behavior oracle.

## Verification

- TDD RED: the focused contract found no runtime index, presence table, oracle, or benchmark.
- Focused Clip/Pipeline static contracts after implementation: `4/4` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `57/57`
  passed (`45 + 9 + 3`).
- Rust behavior coverage compares indexed and nested projections while an archived diagnostic
  stage is present.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission:

- attribution request `69fe23f5649f453989cff0d1e0246421`;
- snapshot `2729`, create request `4e335ac3f8b646c8b69b8a009d5bb6e9`;
- validation idempotency request `6dff1b2c8ea54fa49db8ea0f41dd0e75` was rejected before
  queueing with `validation_ticket_external_worktree_dirty` because `E:\Git\zr_vm` is dirty;
- the unrelated external worktree is left unchanged; this batch will be resubmitted with later
  tasks after the managed preflight becomes available.

## Performance contract

The ignored release benchmark computes missing stages 100,000 times over 11 alternating samples
for a complete runtime report plus one archived stage. It compares the retained nested lookup with
the fixed-index projection and requires at least 50% P95 improvement. Terminal nanosecond values
must come from the managed Windows receipt before integration, push, or WeCom reporting.
