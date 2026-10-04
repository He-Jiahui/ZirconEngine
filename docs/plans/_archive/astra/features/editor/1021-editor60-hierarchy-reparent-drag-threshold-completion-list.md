---
doc_type: completion-list
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-reparent-drag-threshold.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
implementation_files:
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/gesture.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/hierarchy_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/sync.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/input_policy.rs
  - zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize/tab_drag/lifecycle/move_event/start.rs
tests:
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/hierarchy_gesture.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/gesture.rs
---

# Editor60 hierarchy Reparent drag threshold completion list

| Plan slice | Implemented contract | Acceptance boundary | Status |
| --- | --- | --- | --- |
| ED60-P0-01 / G01 | The hierarchy bridge records press origin, activates from actual Move displacement at the established 4px threshold, and admits Up Reparent only for an activated gesture. Seven real host/world/history tests cover no Move, jitter, signed/diagonal displacement, equality, valid drag and Undo. | Scoped static and exact-preservation evidence are recorded under the R artifact prefix in the optimize record. Independent source review is complete; grouped Windows validation remains pending. | implemented_pending_validation |
| ED60-G08 local terminal slice | Release/cancel consumes the gesture, duplicate Up cannot submit again, and new press, invalid/outside owner motion, geometry/row-count reset retire prior activation. Two lower tests plus real host boundary tests cover these contracts. | Complete native capture/outside delivery and all cancellation sources are not covered. | implemented_pending_validation |
| ED60-G02/G03/G04/G05/G06 | Existing press-time reference payload consumers retain their behavior. | Platform metrics; native capture; Escape/right-click/focus/capture/window-close routing; same-count scene replacement; world/document/window/generation-qualified sessions remain open. This slice constrains Reparent, not all drop consumers. | product_gate_pending |
| ED60-G30/G34/G35/G36/G38/G39/G40 | Active Move adds fixed-size gesture state/arithmetic without scene scans or row copies; the existing transaction chain is reused. | Bounded large Reparent, 100K and 1M hierarchy measurements, multi-window/document isolation, latency/metrics, native product feedback and reproducible cross-engine benchmarks remain pending. No measured performance pass or new timing budget is claimed. | product_gate_pending |
