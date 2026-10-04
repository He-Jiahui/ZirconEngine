---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/59/2026-09-27-viewport-pointer-button-ownership.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
implementation_files:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
tests:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_input/pointer_button_ownership_tests.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/pointer_bridge/button_ownership.rs
  - zircon_editor/src/ui/retained_host/app/tests/native_viewport_cancel.rs
---

# Editor59 viewport pointer button ownership completion list

| Plan slice | Implemented contract | Acceptance boundary | Status |
| --- | --- | --- | --- |
| ED59-G06 button ownership | Primary admission preserves an existing drag; Primary Up preserves Orbit/Pan. Shared Surface Cancel precedes controller cancellation for Escape and focus loss. Ten real controller, retained bridge and native regressions cover current/stale extract chords, outside motion, matching release, gizmo rollback and history. Runtime1009 owns the shared Surface button-aware capture dependency. | Scoped static checks and preservation evidence are recorded with owned preimages. Independent source review passed against the final v2 manifest. Grouped Windows Runtime/Editor compile/test validation remains pending. Capture generation and native multi-device portions of G06 remain open. | implemented_pending_validation |
| ED59-G31/G32/G33 and native product gates | No full render packet construction, mesh payload copy, motion coalescing or edge-order change was introduced to pointer dispatch. | Native product verification and 100k/1M selectable, 1 kHz, large-selection, 4/16 viewport allocation and p95/p99 measurements remain pending. This correctness repair does not certify their performance gates. | product_gate_pending |
