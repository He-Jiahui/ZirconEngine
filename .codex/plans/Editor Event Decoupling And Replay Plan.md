# Editor Event Decoupling And Replay Plan

## Summary

- Add a new `editor_event` subsystem owned by `zircon_editor` as the single authority for editor UI event normalization, dispatch, execution, journaling, replay, and reflection refresh.
- Keep `zircon_editor_ui` and `zircon_ui` as protocol/reflection crates only. They keep typed bindings, codecs, descriptors, query/subscription shapes, and generic control surfaces, but they no longer own editor behavior execution.
- Refactor the current direct Slint callback path in [app.rs](E:/Git/ZirconEngine/zircon_editor/src/host/slint_host/app.rs) so Slint becomes only an adapter. The same dispatcher must also serve headless tests and future MCP control.
- Replace the current preview-style route translation in [reflection.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/reflection.rs) with real execution through the new dispatcher, so `InvokeBinding` / `CallAction` mutates real editor state and returns real event results.

## Public Interfaces

- Add `EditorEventId`, `EditorEventSequence`, `EditorEventSource`, `EditorEventEnvelope`, `EditorEvent`, `EditorEventResult`, `EditorEventEffect`, `EditorEventUndoPolicy`, `EditorEventRecord`, `EditorEventJournal`, `EditorEventReplay`, `EditorEventDispatcher`, `EditorEventRuntime`, and `EditorTransientUiState` under `zircon_editor::editor_event`.
- Define `EditorEvent` as the normalized journal format. It must cover the current mainline editor interactions: workbench menu actions, layout/docking actions, hierarchy selection, inspector batch apply, asset surface actions, viewport input, and host transient interaction state updates.
- Keep `EditorUiBinding`, `UiEventBinding`, `UiControlRequest`, and `EditorUiControlService` as the outer transport-neutral control API, but route execution must delegate into `EditorEventDispatcher` instead of directly decoding to preview JSON.
- Keep raw Slint callbacks and `nativeBinding` strings as adapter inputs only. They are not the canonical replay format and must not appear as the dispatcher’s core semantic model.

## Implementation Changes

- Create a focused `zircon_editor/src/editor_event/` module family for:
  - host adapters
  - semantic event normalization
  - dispatch/execution
  - transient UI state tracking
  - event journal + replay
  - reflection rebuild triggers
- Introduce an `EditorEventRuntime` that owns or coordinates `EditorState`, `EditorManager`, transient UI state, and the editor-facing control surface. All editor behavior must pass through this runtime.
- Refactor [app.rs](E:/Git/ZirconEngine/zircon_editor/src/host/slint_host/app.rs) so every `ui.on_*` callback only:
  - updates transient host state when needed
  - builds an `EditorEventEnvelope`
  - submits it to the dispatcher
  - consumes returned effects/results for presentation updates
- Remove direct `EditorState` / `EditorManager` mutation from Slint callbacks. Slint remains responsible for rendering and host event capture only.
- Refactor [reflection.rs](E:/Git/ZirconEngine/zircon_editor/src/workbench/reflection.rs) so workbench route registration binds to real dispatcher execution. Reflection actions must carry route metadata that resolves into `EditorEvent` execution, not into translation-only helper output.
- Rebuild reflection snapshots from three sources only:
  - stable editor state from `EditorState`
  - workbench/layout state from `EditorManager`
  - transient interaction state from `EditorTransientUiState`
- Record every dispatched event into a session-local journal with:
  - monotonic sequence id
  - source metadata (`slint`, `headless`, `mcp`, `replay`)
  - normalized event payload
  - before/after revision markers
  - emitted effects
  - undo policy
  - structured success/failure result
- Implement replay against normalized `EditorEvent` records. Replay must re-enter the same dispatcher path as live execution.
- Bridge undo in V1 instead of rebuilding it from scratch:
  - scene-editing events delegate to the existing `EditorHistory`
  - non-scene events are journaled and replayable now
  - inverse-event hooks and undo metadata are added now so layout/asset/editor-domain undo can be extended later without redesign
- Keep crate boundaries strict:
  - `zircon_editor` owns execution, journal, replay, and transient state
  - `zircon_editor_ui` owns editor-only binding and reflection schemas
  - `zircon_ui` owns generic binding/codec/reflection/control primitives only

## Test Plan

- Add unit tests proving one action executed from Slint adapter input, `EditorUiBinding`, and `CallAction` produces the same normalized `EditorEvent`, state mutation, and reflection diff.
- Add journal tests proving event records are serializable and preserve stable semantic payloads independent of Slint callback names.
- Add replay tests proving a recorded session reproduces the same editor snapshot, layout snapshot, and world snapshot.
- Add transient-state tests for hover, focus, pressed, drawer-resize, and drag bookkeeping being reflected without querying the live Slint tree.
- Update workbench reflection tests so `InvokeBinding` / `CallAction` executes real editor behavior instead of only returning translated preview JSON.
- Keep existing scene command undo/redo coverage passing and add tests that undoable scene events are correctly tagged and bridged through the event journal.

## Assumptions And Defaults

- V1 covers the current editor interaction mainline only: menu, docking/layout, hierarchy selection, inspector batch apply, asset surface operations, and viewport input.
- Event log/replay is session-local in this iteration. Records are serializable but are not automatically persisted to project files or global config yet.
- The canonical event log stores normalized `EditorEvent` records, not raw UI-library callbacks and not `nativeBinding` strings.
- Full undo parity for non-scene editor events is deferred, but V1 must include explicit undo policy/effect metadata so later undoable editor events can attach without another architectural split.
- This matches the reference-engine direction already present in the repo: central editor message/command ownership like Fyrox, and central selection/undo ownership like Godot, instead of letting widget callbacks own editor state.
