record_kind: optimization_validation
status: implementation_complete_managed_validation_pending
created_at: 2026-09-01
owner_session: root-runtime-interface03-activate-link-failure-20260831
related_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
related_code:
  - zircon_runtime_interface/src/ui/surface/render/frame_extract.rs
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/paint_element_capacity_tests.rs
related_tests:
  - tools/tests/test_runtime_interface03_render_frame_paint_capacity_performance_contract.py
  - zircon_runtime_interface/src/ui/surface/render/frame_extract/paint_element_capacity_tests.rs::runtime_interface03_batch20_render_frame_capacity_release_benchmark
---

# Presized render-frame paint elements

## Scope

`UiRenderFrameList::to_paint_elements_with_metrics` previously started with an empty vector and
grew it while projecting commands. Every render command guarantees at least one paint element, so
the command count is a valid lower bound.

The projection now reserves that lower bound once and continues appending directly into the same
buffer. The preexisting unreserved path is retained only under tests as the behavior and capacity
oracle. Direct append, paint order, payloads, and UI12's native-density raster normalization remain
unchanged.

## Verification

- TDD RED: the focused contract found no command-count reservation, oracle, or benchmark.
- Focused Pipeline/render-frame capacity static contracts after implementation: `6/6` passed.
- Batched RuntimeInterface03, input-routing, and Editor palette static regression: `61/61`
  passed (`49 + 9 + 3`).
- Rust behavior coverage compares complete element vectors and checks the exact retained capacity
  for 4,097 one-element commands.
- Scoped Rust 1.94.1 formatting and diff checks: passed.
- Managed Windows Rust 1.94.1 compile, behavior, and release benchmark: pending; no terminal
  performance number is claimed yet.

Managed submission: pending for a later multi-task batch while the external `E:\Git\zr_vm`
dependency worktree prevents immutable validation preflight.

## Performance contract

The ignored release benchmark projects 4,097 one-element commands eight times over 11 alternating
samples. Acceptance requires reserved capacity to equal 4,097 and to be at least 33% below the
unreserved growth result; alternating P95 nanoseconds are also printed as observational evidence.
Terminal capacity and timing values must come from the managed Windows receipt before integration,
push, or WeCom reporting.
