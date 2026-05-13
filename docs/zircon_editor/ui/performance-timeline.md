---
related_code:
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter.rs
  - zircon_editor/src/ui/host/editor_runtime_client.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
implementation_files:
  - zircon_runtime/src/core/diagnostics/profiling/tracy.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/app/profiling.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/retained_host/host_contract/presenter.rs
  - zircon_editor/src/ui/host/editor_runtime_client.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
plan_sources:
  - .codex/plans/Zircon 性能时间轴与 Tracy 集成设计.md
  - user: 2026-05-13 continue profiling timeline and Tracy integration milestone
tests:
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - target: cargo check -p zircon_editor --profile profiling --features profiling --locked
  - target: cargo check -p zircon_app --profile profiling --features "target-editor-host profiling profiling-tracy profiling-chrome" --locked
  - target: cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --profile profiling --features profiling --locked
  - target: cargo check -p zircon_runtime --profile profiling --features "profiling profiling-tracy" --locked
doc_type: module-detail
---

# Editor Performance Timeline

## Purpose

The editor side of the profiling milestone consumes the runtime diagnostics spine without becoming the recorder owner. The retained host adds editor-stream frame and span samples for authoring work, and it can merge a dynamic runtime session snapshot through `EditorRuntimeClient::profile_control` when the runtime cdylib supports the optional ABI hook.

This is the M1/M2 bridge toward a dedicated Performance Timeline panel. The current UI exposure is intentionally lightweight: the existing Runtime Diagnostics payload reports whether profiling is active and how many frames, spans, counters, and over-budget frames are visible.

## Editor Instrumentation

The retained host uses `zircon_runtime` profiling macros at stable CPU boundaries:

- `RetainedEditorHost::tick` records an `editor` frame called `retained_host_tick`.
- `recompute_if_dirty` records the shell presentation/layout recompute span.
- Viewport submission records `submit_viewport_extract` and `submit_extract_with_ui` spans.
- Viewport image polling records a span only when a new image is present.
- `SoftbufferHostPresenter::present` records presenter spans for planning/repaint, RGBA copy, and the softbuffer present call.

These spans stay in editor-owned modules and use the runtime profiling recorder as a shared diagnostic service. They do not move editor selection, workbench layout, native-window state, or viewport authoring state into `zircon_runtime`.

## Dynamic Runtime Merge

`EditorRuntimeClient` exposes a default `profile_control` method returning `Ok(None)`. `zircon_app::RuntimeSession` implements it by serializing `ProfileControlRequest`, calling the optional dynamic runtime `profile_control` function, decoding `ProfileControlResponse`, and freeing the returned ABI buffer.

When the `profiling` feature is enabled, `RetainedEditorHost::runtime_diagnostics_with_profile` starts from local editor diagnostics and asks the runtime client for a `Snapshot` response. If a runtime snapshot is returned, `app/profiling.rs` merges it into the editor snapshot:

- Runtime span ids and parent ids are offset when editor spans already exist.
- Frames, spans, and counters are appended so both `editor` and `runtime` streams are available to the diagnostics payload.
- Active and feature-enabled flags are OR-ed so either side can indicate live capture.
- Session ids are combined only when the editor and runtime report different ids.

The merge helper is feature-gated and lives in its own focused module to keep `host_lifecycle.rs` as lifecycle orchestration rather than a profiling transport implementation.

## Tracy Process Startup

When the editor or runtime preview is built with `profiling-tracy`, `zircon_app::EntryRunner` installs the runtime profiling Tracy sink before creating the editor host or runtime session. The dynamic runtime API entry installs the same sink for the cdylib image when the runtime library is loaded. This lets the editor host, app presenter, and dynamic runtime emit to Tracy through the same instrumentation macros while still treating Tracy as an external live viewer.

## UI Exposure

The Runtime Diagnostics pane now includes profiling detail rows when `ProfileSnapshot.feature_enabled` is true:

- `Profiling: active|inactive (<frames> frames, <spans> spans, <counters> counters)`.
- `Profiling over-budget frames: <count>`.

This gives M1/M2 validation evidence without adding the full M3 timeline panel yet. M3 can use the same merged `ProfileSnapshot` to render frame bars, span trees, top hotspots, capture controls, and export links.

## Test Coverage

`zircon_editor/src/tests/host/pane_presentation.rs` covers Runtime Diagnostics payload projection of profiling counts and over-budget frames. Existing fixture construction in `pane_presentation.rs` and `template_runtime/pane_payload_projection.rs` verifies explicit `RuntimeDiagnosticsSnapshot` initializers include the new `profile` field. The app editor-host profiling check verifies `zircon_app::RuntimeSession` still exposes the optional runtime snapshot hook only through the editor-host integration path.
