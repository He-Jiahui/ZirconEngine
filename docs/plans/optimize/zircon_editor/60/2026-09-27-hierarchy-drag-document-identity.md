---
title: Editor60 Hierarchy Drag Document Identity
category: zircon_editor
date: 2026-09-27
implementation_status: implemented
validation_status: batched_validation_pending
performance_status: product_gate_pending
related_code:
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/identity.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/reference_drop_payload.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/inputs.rs
  - zircon_editor/src/ui/retained_host/app/pane_surface_actions/component_showcase/activation.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/60/2026-09-27-hierarchy-reparent-drag-threshold.md
tests:
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/hierarchy_identity.rs
---

# Editor60 hierarchy drag document identity

## Source finding

The Editor1021 threshold slice made Reparent require a real pointer Move, but its press still kept bare `NodeId` values and a `scene://node/{id}` payload. On Primary Up, `app/hierarchy_pointer/events/drag.rs` resolved the target from the current hierarchy rows and submitted those old IDs to the current editing World. `app/reference_drop_payload.rs` likewise consumed the old SceneInstance payload without a final source check. A scene reload can replace a World with the same row count and IDs while the pointer is held. Row-count layout sync cannot distinguish those Worlds, and Up or field drop can arrive before retained-host hierarchy refresh.

This is the local ED60-P0-01 G04/G05/G06 identity slice following [Editor1021](2026-09-27-hierarchy-reparent-drag-threshold.md). The source-only counterexample and regression design are recorded here; grouped managed execution is pending.

## Authority and implementation

`app/hierarchy_pointer/identity.rs` records a private press token: `WorldDomain`, the complete `GatewaySessionIdentity`, active `HistoryContextId`, `(DocumentId, activation_revision)` from the document lifecycle authority, and `callback_source_window`. The document lifecycle accessor returns those two scalars while holding its existing route and state locks; it does not clone the project path or scene URI. Capturing and comparing the token does not scan World nodes or alter the public drag payload.

`AuthoringWorld::replace` publishes a new gateway through `EditorRuntimeGatewayHandle::replace`; the handle increments and atomically publishes `gateway_generation` before the host's next refresh. Thus the current `world_gateway_identity(Edit)` differs immediately after same-document reload, even if node IDs and row count are identical. The Up handler checks the token before reading a target and again immediately before Reparent dispatch. The SceneInstance drop consumer checks it before taking the payload. Mismatch clears the gesture, node IDs, token, and scene payload. Hierarchy refresh retires stale presses before reflow, and the existing replacement report also clears an active press. Competing asset and inspector presses clear the same private state when replacing the scene payload. Unqualified SceneInstance payloads explicitly staged by existing callers retain the previous drop precedence.

The reference consumer reports a stale source separately from no active payload. For each Asset, Instance, and Object `FieldDropped` action, stale source rejection stops the showcase event before its static demonstration fallback can synthesize a different reference. A genuine no-payload action still uses that existing demonstration fallback.

The local Unreal Scene Outliner reference, `dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SOutlinerTreeView.cpp`, detects drag after press and validates a drop before committing. Zircon's generation, document, and window token uses Zircon's existing authorities; the Unreal reference is not claimed to implement this token.

## Tests and gates

The new retained-host child `app/tests/drag_sources/hierarchy_identity.rs` uses `ChildWindowHostHarness`, native Down/Move/Up, the real World gateway replacement and history engine, event journal, and SceneInstance reference field. It covers:

| Regression | Required observation |
| --- | --- |
| `same_shape_world_replacement_rejects_stale_native_up_before_and_after_reflow` | A new default World has the same row count and source ID, but its gateway identity changes; stale Up before or after reflow leaves parent map, history, and journal untouched. |
| `stale_scene_instance_payload_cannot_fall_through_any_reference_field_action` | A stale SceneInstance cannot change any Asset, Instance, or Object reference field through the static demo fallback before retained refresh. |
| `reference_fields_keep_their_demo_fallback_without_an_active_payload` | The three genuine no-payload actions still receive their existing static demonstration references. |
| `unchanged_world_preserves_native_reparent_and_undo` | The existing 4px activated gesture still reparents once and Undo restores its parent map. |
| `document_binding_switch_retires_native_gesture_even_when_node_ids_match` | Changed active document history context rejects old node IDs. |
| `callback_window_route_cannot_consume_another_windows_hierarchy_press` | The callback window qualifier rejects an Up routed through another window. |
| `competing_reference_press_clears_hierarchy_payload_and_its_identity` | A new Inspector source clears the old scene source and token. |

Existing Editor1021 native threshold, row-count reset, terminal-repeat, transaction, cycle, and Undo tests remain in place. Static checks and exact foreign-byte inverse evidence live under `.codex/state/session-coordinator/async-validation-batches/2026-09-27-astra-optimize-batch-r-editor60-hierarchy-drag-document-identity-*`. The new inputs explicitly succeed frozen Editor1021 source hashes `64f9806bf96408209799df19c7915283c5aeae734c8b327c6c2f19d0cdfb9c92` for `events/drag.rs` and `8b4dab32214f2cbb1741b3d5232428c06ce263b72e9327d1cf44ae5735a1586b` for `drag_sources/mod.rs`; the older R manifest is unchanged.

Managed Windows tests and native platform acceptance are pending. This slice closes the Edit gateway/document/window stale-consumption path at source level, but does not claim all ED60 G04–G06 cases: same-gateway Play World replacement before its invalidation pump and real cross-window native capture need separate product evidence. ED60 G02/G03 native capture, cancellation and platform drag metrics remain pending. G30 bounded large Reparent, G34 100K memory/first-paint/scroll/selection/filter/1% churn, G35 1M lazy/paged hierarchy, G36 multi-window isolation, and G38–G40 latency/native feedback/cross-engine benchmark remain pending. The parent plan sets no fixed millisecond pass threshold, so no measured performance pass is claimed.
