---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/200/2026-09-26-product-input-to-frame-profile.md
  - docs/plans/optimize/zircon_runtime/200-runtime-ui-surface-input-focus-pointer-capture-ime-accessibility-frame-authority-current-working-tree-review.md
implementation_files:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render.rs
tests:
  - zircon_runtime/src/dynamic_api/session/tests/foundation_render/ui_input_frame_profile.rs
---

# Runtime200 product input to frame profile completion list

| Plan slice | Completed implementation | Acceptance boundary | Status |
| --- | --- | --- | --- |
| Dynamic UI input to WGPU frame evidence | Added an authored project UI behavior regression and an ignored 31-sample profile of touch dispatch, typed host output, runtime tick, WGPU capture, and total cycle for 1/16-click bursts. Each sample verifies rendered pixels, render passes, and UI commands. | Grouped managed Runtime behavior test and Windows WGPU marker `RUNTIME200_UI_INPUT_TO_CAPTURE_CYCLE_PROFILE_V1` must run and report p50/p95/p99. Visible action-response latency, allocation/RSS budgets, and the full `G-UI-25` product matrix remain open. | implemented_pending_validation |

The fixture does not alter production code or claim that the local valid-owner-route
helper ratio proves product performance.
