---
title: Editor60 Hierarchy Reparent Drag Threshold
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
---

# Editor60 hierarchy Reparent drag threshold

## Defect and scope

[ED60-P0-01 / G01](../60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md) requires a press followed by subthreshold motion and release over another row to remain a click. The prior [drag event owner](../../../../../zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs) staged node IDs and a reference payload on Down, then dispatched Reparent on Up without any displacement check. With the current 26px rows and 1px gaps, a press at row-local Y = 33.5 and release at Y = 35.5 crosses from the first row to the second with only 2px displacement; no Move is required to reach the faulty branch. This is a source-confirmed counterexample; managed dynamic reproduction remains pending.

The real call chain is [native hierarchy button dispatch](../../../../../zircon_editor/src/ui/retained_host/host_contract/native_pointer/button_dispatch/pane_callbacks/native_panes/hierarchy.rs) → pane callback → host drag event → [Reparent dispatch](../../../../../zircon_editor/src/ui/retained_host/callback_dispatch/hierarchy/edit.rs) → `EditorHierarchyEvent::ReparentNodes` → `EditorIntent::SetParents` → the existing transaction/parent validation. The lowest faulty support layer was the hierarchy gesture admission. Existing cycle checking, source-root normalization, atomic transaction and Undo behavior remain in the [editing owner](../../../../../zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs).

This R slice constrains **hierarchy Reparent submission**. Press-time `active_scene_drag_payload` remains available to existing reference-field drop consumers. This slice does not claim that every drag/drop consumer now uses a threshold.

## Implementation and reference evidence

The local Unreal primary reference separates press detection from drag activation: [SOutlinerTreeView.cpp](../../../../../dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SOutlinerTreeView.cpp) requests `DetectDrag` on left press (241–252), begins the operation only after drag detection (62–71), and validates the drop (98–105). [SlateApplication.cpp](../../../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp) detects a drag from Move (5784–5801) and compares squared displacement with squared trigger distance using `>=` (4107–4114).

- The [hierarchy gesture owner](../../../../../zircon_editor/src/ui/retained_host/hierarchy_pointer/gesture.rs) stores the press origin and whether motion reached the threshold. Actual Move displacement, including negative and diagonal deltas, controls activation. Up cannot activate an armed gesture by itself.
- The shared [retained-host input policy](../../../../../zircon_editor/src/ui/retained_host/input_policy.rs) uses the existing 4px [native tab-drag policy](../../../../../zircon_editor/src/ui/retained_host/host_contract/native_pointer/drag_resize/tab_drag/lifecycle/move_event/start.rs). Its tab caller retains the previous `hypot` comparison and behavior. This is not a new product performance threshold or a claim of platform-metric qualification.
- Up consumes the gesture once, retains normal hover/focus updates, and reaches Reparent only after activation. A new press replaces any old gesture. Explicit local cancellation, invalid/outside owner motion, pane geometry changes, row metrics changes and item-count changes retire it.
- Ordinary hover skips the new displacement work when no gesture exists. An active gesture adds constant-size state and arithmetic; it does not add scene scans, row copies or history work on Move. Actual cost remains unmeasured.

## Regression coverage

The tests-first snapshot preceded production edits. Seven tests in [hierarchy_gesture.rs](../../../../../zircon_editor/src/ui/retained_host/app/tests/drag_sources/hierarchy_gesture.rs) use the real retained host, world, transaction history and event journal. Press/Move/Up enter the native dispatch path. Boundary invalidation cases additionally call the real owner callback; they do not fabricate bridge/controller state. Scene-structure reset uses a real CreateNode intent and the normal hierarchy invalidation/reflow owner, and asserts the committed row count changed.

| Test | Observable contract |
| --- | --- |
| `hierarchy_native_press_release_without_move_preserves_world_and_history` | Adjacent-row release after only 2px displacement emits no Reparent and changes neither parents nor history. |
| `hierarchy_native_subthreshold_jitter_preserves_world_and_history` | 2px row-crossing, ±3.99px axial and ±(2,2) diagonal motion remain clicks; the diagonal cases detect an incorrect Manhattan-distance gate. |
| `hierarchy_native_threshold_drag_reparents_once_in_every_direction_and_undo_restores_world` | ±4px X/Y and ±(3,3) motion activate. One Reparent transaction changes the real parent, repeated Up creates no second event/history entry, and Undo restores the full parent map. |
| `hierarchy_native_new_press_resets_an_activated_gesture` | A fresh press cannot inherit an earlier threshold crossing. |
| `hierarchy_owner_outside_or_invalid_motion_retires_armed_and_dragging_gestures` | Outside/NaN points delivered to the owner cancel both phases; later in-pane movement and release cannot revive them. |
| `hierarchy_owner_geometry_reset_retires_armed_and_dragging_gestures` | A real callback size change cancels both phases, including after returning to the normal native geometry. |
| `hierarchy_owner_scene_structure_refresh_retires_an_activated_gesture` | A real node insertion/reflow cancels the prior gesture without adding a Reparent transaction. |

Two lower support tests in the gesture owner cover explicit cancellation/release idempotence and invalid press origins: `hierarchy_reparent_cancel_and_release_are_terminal` and `hierarchy_reparent_invalid_origin_never_arms_a_gesture`.

The existing [scene reference-payload tests](../../../../../zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs) remain unchanged. Existing [editing hierarchy tests](../../../../../zircon_editor/src/tests/editing/node_ops/hierarchy.rs) cover multi-node transaction atomicity, ancestor/descendant normalization, cycle rejection and Undo. No dynamic passing result is claimed for this batch.

## Evidence and acceptance

| Gate | Status |
| --- | --- |
| Ownership | Thirteen paths claimed by request `692ebfc984d941ae8399a2f3993f2fda`; they were checked outside sealed N/O/P/Q source inputs. Editor1021 was unoccupied. |
| Record authorization | Optimize maintenance request `b83381b241b84328bfdb6823bb9a3205`; Astra maintenance request `5925c21dfc984263b98ea09fb78f1872`. |
| Preservation | Exact preimages include both foreign `mod.rs` files. This slice adds only a declaration to each; inverse reconstruction must match their original bytes and saved foreign diffs. |
| Static evidence | Tests-first snapshot, scoped formatting, diff/structure checks, exact inverse and final source manifest use prefix `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-r-editor60-hierarchy-drag-threshold-*`. |
| Review / execution | Independent source review is complete; grouped Windows Editor validation remains pending. This implementation agent ran no Cargo, validation-state polling or submission. |
| ED60-G01 and local G08 slice | The threshold/release admission and repeat-terminal behavior are implemented; acceptance awaits managed execution. |
| ED60-G02/G03 | Native capture, cross-window/outside delivery, platform drag metrics, Escape, right-click, focus/capture loss and window-close cancellation remain open. Native routing does not always send outside motion to this owner; callback boundary tests do not close that gap. |
| ED60-G04/G05/G06 | Same-count scene replacement, world/document/window identity and generation-qualified session retirement remain open. Item-count reset is not a substitute for these contracts. |
| Product and performance | G30 bounded large Reparent; G34 100K memory/first-paint/scroll/selection/filter/1% churn; G35 1M lazy/paged hierarchy; G36 multi-window/document isolation; G38 latency/metrics; G39 native feedback matrix; G40 reproducible cross-engine benchmarks remain pending. The plan supplies no fixed millisecond pass threshold; none is invented or claimed. |
