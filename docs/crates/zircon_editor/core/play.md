---
related_code:
  - zircon_editor/src/core/play/mod.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/mode.rs
  - zircon_editor/src/core/play/request.rs
  - zircon_editor/src/core/play/error.rs
  - zircon_editor/src/core/play/transition_report.rs
  - zircon_editor/src/core/play/backend
  - zircon_editor/src/core/play/embedded_backend
  - zircon_editor/src/core/play/preview_frame.rs
  - zircon_editor/src/core/play/preview_input.rs
  - zircon_editor/src/core/play/simulate_camera.rs
  - zircon_editor/src/core/play/process_backend
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/play/plugin_activation/contract.rs
  - zircon_editor/src/core/play/plugin_activation/native.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/runtime_shutdown.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/play_preview_input.rs
  - zircon_editor/src/ui/host/editor_host_event_controller/simulate_camera.rs
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs
  - zircon_editor/src/ui/retained_host/app/simulate_camera_sync.rs
  - zircon_editor/src/ui/retained_host/app/native_keyboard_actions.rs
  - zircon_editor/src/ui/retained_host/app/viewport/game_input.rs
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime_interface/src/runtime_api/session/camera.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_app/src/entry/entry_runner/editor/play_session_factory.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
tests:
  - zircon_editor/src/core/play/tests.rs
  - zircon_editor/src/tests/editor_event/runtime/stack_play.rs
  - tools/tests/test_editor04_play_session_controller_contract.py
  - tools/tests/test_editor04_process_play_backend_contract.py
  - tools/tests/test_editor_embedded_play_session_contract.py
  - tools/tests/test_editor_pie_preview_frame_contract.py
  - tools/tests/test_editor_game_viewport_input_contract.py
doc_type: module-detail
---

# Editor Play Session Controller

## Purpose

`PlaySessionController` is the editor's single authority for `Edit`, `Building`, and `Playing`. UI chrome may display a derived session mode, but command enablement, build-to-play orchestration, and plugin bridge activation read the controller state directly.

The controller owns a typed `PlayBackend` lifecycle. Product GUI composition installs `EmbeddedPlayBackend`, including welcome-page startup before any project is open. `ProcessPlayBackend` remains an explicit external-process backend with bounded stdout/stderr collection, poll/stop/reap, and versioned scene snapshot cleanup; it is no longer the implicit product default. `NoopPlayBackend` is a non-attachable fixture/default for hosts that have not installed a product backend.

The controller also owns the typed preference for the next run through `preferred_kind()` and `set_preferred_kind(PlayKind)`. The Workbench Run Mode menu changes that preference through `MenuAction::SelectPlayMode`; Enter Play reads it when constructing `PlayStartRequest`, so Play In Editor and Simulate no longer collapse to one hard-coded request. This preference is not a second lifecycle state: `mode()` remains the sole Edit/Building/Playing authority, and a selection made during a session only affects the next start.

## State transitions

- `request_play(immediate)` rejects any stale attached play route, activates the plugin bridge, starts the selected backend, attaches the gateway carried by its start report, then moves `Edit -> Playing`.
- the next-run preference defaults to `PlayKind::Play`; selecting Simulate changes the kind carried by the next `PlayStartRequest` without entering play immediately.
- `request_play(after_build)` moves `Edit -> Building { play_after_build: true }` without activating plugins.
- `on_build_finished(true)` activates the bridge and moves `Building -> Playing`; failure moves to `Edit` without activation.
- a second play request while `Playing` returns `PlaySessionError::InvalidTransition`.
- `request_stop` is a no-op in `Edit`, cancels `Building`, and stops the backend before deactivating the bridge. An embedded backend then remains in a typed terminal-retirement state until consumers stop, the exact gateway identity detaches, and App destroys the lease.
- backend start failure rolls back plugin activation and remains in Edit. If a gateway attachment rollback cannot stop the already-created runtime, the controller remains `Playing` so stop can be retried. Backend retirement or plugin deactivation failure enters `CleanupFailed` with the remaining owner encoded explicitly.
- `poll_backend` drains bounded output while running. Terminal exit performs snapshot cleanup, deactivates plugins, returns to Edit, and reports `PlayTransitionCause::Crashed { exit_code }`.
- once the backend is terminal, normal stop, startup compensation, crash polling, project close, and host shutdown all detach the exact captured play gateway identity. The terminal check, identity capture, and detach share the controller transition gate; detach refuses active runtime modes even in release builds.

The dedicated transition mutex serializes lifecycle changes. State, activation, and backend owner locks are not held while calling external lifecycle code: each `Arc` is cloned first, then invoked under the transition gate.

## Embedded session ownership

`EmbeddedPlayBackend` materializes either the persisted scene or the current edit-world snapshot through `PlaySnapshotStore`, then asks an injected `PlaySessionFactory` for an opaque lease. Editor sees only the runtime gateway and `retire`; it does not import or copy ABI create/destroy function pointers.

`AppPlaySessionFactory` reloads the preflight-authenticated runtime artifact, rejects a changed BuildSet, and creates a separate `runtime` profile session with the project root and project-relative serialized scene. The authoring session remains the projectless `editor` profile. The lease retains the gateway and `Arc<RuntimeSession>` until terminal retirement.

Retirement is deliberately two-phase: runtime consumers stop, backend becomes terminal, `PlayDomainLink` detaches the captured gateway identity, then the App lease drops its gateway and requires `Arc::try_unwrap` before `RuntimeSession::try_destroy`. Outstanding gateway/output owners or destroy failure restore the stopped lease for retry. Snapshot cleanup runs only after successful session destruction. This follows the same isolation and inverse teardown principle as Unreal PIE without importing UObject world semantics.

## PIE preview frame ownership

The retained host keeps authoring Scene output, the SIE Scene override, and runtime Game output in separate slots. A Game pane never paints the authoring viewport. In Play, Scene continues to show authoring output and Game shows the runtime frame. In Simulate, the play-world frame temporarily overrides Scene while the authoring Scene slot keeps receiving updates, so clearing the override restores the latest authoring image immediately. The host captures `ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1` only when the destination pane is visible. Hidden panes skip the full-frame readback. `PlayPreviewFrame` copies validated RGBA into host-owned `Arc<[u8]>` and explicitly releases the provider allocation before the frame crosses into retained presentation state. Stop, crash, and other terminal transitions clear both runtime-owned presentation slots on the next tick.

This CPU capture path is the MVP fallback, not a claimed optimal endpoint. `play.preview.copy_bytes` exposes its per-frame readback volume, and `play.preview.hidden_capture_skipped_count` exposes avoided hidden-pane captures. Native-surface bind/present remains deferred until a profiling report identifies this copy as the bottleneck and the replacement preserves session, surface, and teardown ownership.

## PIE runtime input ownership

Native hit testing preserves `SceneViewport` and `GameViewport` as distinct targets and invokes distinct callbacks. Scene pointer events enter the authoring viewport dispatcher only. Game pointer move/button/wheel events are converted to runtime ABI events for `ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1`; keyboard press, repeat, and release events route there only while the Game view is focused.

`PlaySessionController::route_preview_input` is the single runtime input gate. It serializes input with lifecycle transitions, accepts only an attached `PlayKind::Play` session, and returns not-routed for Edit, Building, Simulate, and CleanupFailed. This keeps Simulate under editor input ownership and prevents a failed active Play dispatch from falling through into authoring shortcuts. Entering Play clears retained text focus once on the inactive-to-active edge. Workbench retains the Game descriptor and checks the focused instance under one session lock, so neither keyboard routing nor the active Play tick materializes the full view collection. A Game-to-other-view edge or native focus loss while Game is current sends the existing ABI background lifecycle event. Runtime background handling emits `InputEvent::FocusLost`, which clears active keys, pointer buttons, touches, and gamepad state instead of leaving stuck input after switching panes or Alt-Tab. Focus gain is not forwarded until the native callback carries source-window identity; otherwise a main or sibling window could incorrectly resume a detached Game window. Routed events increment `play.preview.input_routed_count`.

## Simulate view ownership

Simulate keeps the Scene viewport focused and leaves all mouse/keyboard input in the editor domain. Before the runtime tick, the retained host reads the authoring viewport's world-space camera snapshot and sends it only when changed through the versioned `VIEWPORT_CAMERA` event. The payload has a 4 KiB processing limit, and `PlaySessionController::route_simulate_camera` accepts only `PlayKind::Simulate`, making it mutually exclusive with the Game input route.

The runtime validates the transform and projection, then applies them to the cloned `RenderFrameExtract` selected camera. It does not write the duplicated gameplay world's camera entity. Runtime camera pipeline, HDR/exposure, MSAA, dynamic-resolution, and temporal settings remain authoritative; only transform, projection mode, FOV/ortho size, and clip planes are overridden for presentation. Unchanged camera sync and hidden Scene capture are measured by `play.simulate.camera_unchanged_skipped_count` and `play.simulate.hidden_capture_skipped_count`.

## Process backend and snapshot ownership

`PlaySceneSource::from_world` captures the edit world through the current Plan11 `DynamicScene` writer. `PlaySnapshotStore` atomically publishes it below `.zircon/play/<instance>/play-scene.zrscene.json`; a `MaterializedPlayScene` owns that directory and removes it on stop, crash, spawn failure, or drop. Persisted scene paths are borrowed and never deleted.

`ProcessPlayBackend` constructs the jointly specified command:

`zircon_runtime --project <root> --runtime-session-profile runtime --play-scene <path> --play-report-pipe <name>`

stdout/stderr readers feed a bounded 1,024-line channel. Overflow is counted and surfaced as a diagnostic instead of allowing an unbounded editor-memory queue. Stop uses `kill` followed by `wait`; natural exits are reaped by poll. P1 process instances carry no gateway, so `attachable()` is derived as false and the host does not bind in-process consumers to an external process. There is no separate attachability boolean that can drift from the actual transport.

## Plugin bridge activation

`PluginBridgeActivation` names the existing behavior truthfully: load project native plugins, take a `NativePluginLiveHost` activation snapshot, and restore it on exit. It does not claim to run a game, own a world, or present a viewport.

`NativePluginBridgeActivation` preserves the prior double-enter guard, tolerant empty deactivate, sorted diagnostics, and bridge diagnostics matrix. `NoopPluginBridgeActivation` is the default host fixture.

The retired `EditorRuntimePlayModeBackend`, `EditorPlayBridge`, and `bridge.rs` API do not have compatibility aliases.

## Host and command evaluation

The host owns one controller. Enter Play captures the current edit world into a typed scene source before lifecycle activation. Runtime event consumers are started only for an attachable backend and are still cleaned before stop; failed cleanup leaves both shell and controller in Playing for retry. The retained tick calls `poll_backend` through the existing runtime-consumer pump even when no consumer is attached, so a P1 terminal process can return shell/controller state to Edit.

The App-created projectless `editor` profile runtime gateway is not a play world and is never attached to `PlayDomainLink` during retained-host startup. Only the scene-loaded runtime-profile session produced by `AppPlaySessionFactory` may enter `WorldDomain::Play`. Project host paths reject a missing App-owned backend; the normal GUI path also installs it at welcome startup so a subsequently opened project has the same architecture.

`WorldDomain` and `PlayInstanceId` are also the single owner for scene selection identity. `SelectionModel` stores Play selections by instance, rejects the reserved zero identity on decode, and switches only after a real gateway attachment exists. Tick order is runtime lifecycle -> selection domain -> hierarchy -> Inspector, which covers immediate Play, after-build activation, crash, and terminal restoration without allowing equal Edit/Play entity numbers to share selection state.

`CommandEvalCtx.play_state` is projected from `PlaySessionController::mode()`, including `Building`. It no longer matches `EditorChromeSnapshot.session_mode`.

## Remaining work

- Editor16 runtime consumer for `--play-scene/--play-report-pipe`, followed by startup injection and real spawn/monitor/stop product evidence.
- relative mouse/cursor capture, IME, and gamepad product matrices; basic Game mouse/keyboard routing and Game document focus/restore are source-complete, while native-surface bind/present remains profiling-gated.
- query page/cursor/cancel, entity-exact invalidation, PIE viewport picking, volatile history, edit policy, and explicit keep-simulation-changes command. App-owned attach, live hierarchy, read-only Inspector, and instance-qualified selection are source-complete but still await managed/product validation.
- managed Cargo, real runtime session, repeated leak/zero-pollution, framebuffer, performance, and power evidence for the embedded product path.
