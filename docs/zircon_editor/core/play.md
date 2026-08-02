---
related_code:
  - zircon_editor/src/core/play/mod.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/mode.rs
  - zircon_editor/src/core/play/request.rs
  - zircon_editor/src/core/play/error.rs
  - zircon_editor/src/core/play/transition_report.rs
  - zircon_editor/src/core/play/backend
  - zircon_editor/src/core/play/process_backend
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/play/plugin_activation/contract.rs
  - zircon_editor/src/core/play/plugin_activation/native.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
tests:
  - zircon_editor/src/core/play/tests.rs
  - zircon_editor/src/tests/editor_event/runtime/stack_play.rs
  - tools/tests/test_editor04_play_session_controller_contract.py
  - tools/tests/test_editor04_process_play_backend_contract.py
doc_type: module-detail
---

# Editor Play Session Controller

## Purpose

`PlaySessionController` is the editor's single authority for `Edit`, `Building`, and `Playing`. UI chrome may display a derived session mode, but command enablement, build-to-play orchestration, and plugin bridge activation read the controller state directly.

The controller owns a typed `PlayBackend` lifecycle. `ProcessPlayBackend` now implements external runtime command construction, bounded stdout/stderr collection, poll/stop/reap, and versioned scene snapshot cleanup. It is intentionally not the startup default until Editor16 makes `runtime_preview` consume the joint Play arguments; the default `NoopPlayBackend` preserves the in-process editor gateway path without pretending to spawn a game.

The controller also owns the typed preference for the next run through `preferred_kind()` and `set_preferred_kind(PlayKind)`. The Workbench Run Mode menu changes that preference through `MenuAction::SelectPlayMode`; Enter Play reads it when constructing `PlayStartRequest`, so Play In Editor and Simulate no longer collapse to one hard-coded request. This preference is not a second lifecycle state: `mode()` remains the sole Edit/Building/Playing authority, and a selection made during a session only affects the next start.

## State transitions

- `request_play(immediate)` activates the plugin bridge, starts the selected backend, then moves `Edit -> Playing`.
- the next-run preference defaults to `PlayKind::Play`; selecting Simulate changes the kind carried by the next `PlayStartRequest` without entering play immediately.
- `request_play(after_build)` moves `Edit -> Building { play_after_build: true }` without activating plugins.
- `on_build_finished(true)` activates the bridge and moves `Building -> Playing`; failure moves to `Edit` without activation.
- a second play request while `Playing` returns `PlaySessionError::InvalidTransition`.
- `request_stop` is a no-op in `Edit`, cancels `Building`, and stops/reaps the backend before deactivating the bridge and moving `Playing -> Edit`.
- backend start failure rolls back plugin activation and remains in Edit. Backend stop or plugin deactivation failure keeps Playing so cleanup can be retried.
- `poll_backend` drains bounded output while running. Terminal exit performs snapshot cleanup, deactivates plugins, returns to Edit, and reports `PlayTransitionCause::Crashed { exit_code }`.

The dedicated transition mutex serializes lifecycle changes. State, activation, and backend owner locks are not held while calling external lifecycle code: each `Arc` is cloned first, then invoked under the transition gate.

## Process backend and snapshot ownership

`PlaySceneSource::from_world` captures the edit world through the current Plan11 `DynamicScene` writer. `PlaySnapshotStore` atomically publishes it below `.zircon/play/<instance>/play-scene.zrscene.json`; a `MaterializedPlayScene` owns that directory and removes it on stop, crash, spawn failure, or drop. Persisted scene paths are borrowed and never deleted.

`ProcessPlayBackend` constructs the jointly specified command:

`zircon_runtime --project <root> --runtime-session-profile runtime --play-scene <path> --play-report-pipe <name>`

stdout/stderr readers feed a bounded 1,024-line channel. Overflow is counted and surfaced as a diagnostic instead of allowing an unbounded editor-memory queue. Stop uses `kill` followed by `wait`; natural exits are reaped by poll. P1 process instances declare `backend_attachable = false`, so the host does not bind the editor's in-process gateway consumers to an external process.

## Plugin bridge activation

`PluginBridgeActivation` names the existing behavior truthfully: load project native plugins, take a `NativePluginLiveHost` activation snapshot, and restore it on exit. It does not claim to run a game, own a world, or present a viewport.

`NativePluginBridgeActivation` preserves the prior double-enter guard, tolerant empty deactivate, sorted diagnostics, and bridge diagnostics matrix. `NoopPluginBridgeActivation` is the default host fixture.

The retired `EditorRuntimePlayModeBackend`, `EditorPlayBridge`, and `bridge.rs` API do not have compatibility aliases.

## Host and command evaluation

The host owns one controller. Enter Play captures the current edit world into a typed scene source before lifecycle activation. Runtime event consumers are started only for an attachable backend and are still cleaned before stop; failed cleanup leaves both shell and controller in Playing for retry. The retained tick calls `poll_backend` through the existing runtime-consumer pump even when no consumer is attached, so a P1 terminal process can return shell/controller state to Edit.

`CommandEvalCtx.play_state` is projected from `PlaySessionController::mode()`, including `Building`. It no longer matches `EditorChromeSnapshot.session_mode`.

## Remaining work

- Editor16 runtime consumer for `--play-scene/--play-report-pipe`, followed by startup injection and real spawn/monitor/stop product evidence.
- P2 embedded runtime session, DTO world injection, viewport document, and zero-pollution evidence.
- play-domain attach/live sync, volatile history, edit policy, and explicit keep-simulation-changes command.
