---
related_code:
  - zircon_plugins/sound/editor/Cargo.toml
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/live_output/mod.rs
  - zircon_plugins/sound/editor/src/live_output/model.rs
  - zircon_plugins/sound/editor/src/live_output/controller.rs
  - zircon_plugins/sound/editor/mixer_console.v2.ui.toml
  - zircon_plugins/sound/editor/acoustic_debug.v2.ui.toml
  - zircon_plugins/sound/editor/audio_source.drawer.v2.ui.toml
  - zircon_plugins/sound/editor/audio_listener.drawer.v2.ui.toml
  - zircon_plugins/sound/editor/audio_volume.drawer.v2.ui.toml
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
implementation_files:
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/live_output/mod.rs
  - zircon_plugins/sound/editor/src/live_output/model.rs
  - zircon_plugins/sound/editor/src/live_output/controller.rs
  - zircon_plugins/sound/editor/mixer_console.v2.ui.toml
  - zircon_plugins/sound/editor/acoustic_debug.v2.ui.toml
  - zircon_plugins/sound/editor/audio_source.drawer.v2.ui.toml
  - zircon_plugins/sound/editor/audio_listener.drawer.v2.ui.toml
  - zircon_plugins/sound/editor/audio_volume.drawer.v2.ui.toml
plan_sources:
  - .codex/plans/Sound 插件核心完善计划.md
  - docs/superpowers/specs/2026-05-23-sound-cpal-polish-design.md
  - docs/superpowers/plans/2026-05-23-sound-cpal-polish.md
  - docs/superpowers/specs/2026-05-23-sound-editor-live-output-design.md
  - docs/superpowers/plans/2026-05-23-sound-editor-live-output.md
tests:
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/live_output/controller.rs
  - cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_editor
  - cargo test --manifest-path "zircon_plugins\sound\editor\Cargo.toml" live_output --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-editor-live-output" --message-format short --color never
  - cargo test --manifest-path "zircon_plugins\sound\editor\Cargo.toml" sound_editor --locked --offline --jobs 1 --target-dir "D:\cargo-targets\zircon-sound-editor-template-routes" --message-format short --color never
  - cargo check --manifest-path "zircon_plugins\sound\editor\Cargo.toml" --tests --locked --offline --jobs 1 --target-dir "D:\cargo-targets\zircon-sound-editor-template-routes" --message-format short --color never
  - cargo metadata --manifest-path "zircon_plugins\sound\editor\Cargo.toml" --locked --offline --no-deps --format-version 1
doc_type: module-detail
---

# Sound Editor Plugin

## Purpose

`zircon_plugins/sound/editor` owns the authoring side of the sound plugin. It contributes Sound Mixer and Acoustic Debug views, component drawers for `AudioSource`, `AudioListener`, and `AudioVolume`, operation descriptors for mixer/source/listener/volume workflows, and the editor-facing live output model for output-device picker/status controls.

The editor plugin does not own audio runtime state. It consumes `zircon_runtime::core::framework::sound::SoundManager` through neutral DTOs, while concrete mixing, DSP, device startup, and CPAL behavior remain in `zircon_plugins/sound/runtime`.

## Live Output Boundary

The live output path is isolated under `src/live_output/`:

- `model.rs` defines serializable editor DTOs: device picker rows, status projection, output actions, snapshots, and action reports.
- `controller.rs` defines `SoundEditorLiveOutputController`, a thin controller over `Arc<dyn SoundManager>`.
- `mod.rs` re-exports the public boundary and contains no behavior.

The controller uses only neutral manager methods: `available_output_devices`, `output_device_status`, `backend_status`, `configure_output_device`, `start_output_device`, and `stop_output_device`. It marks a picker row as selected when the row descriptor matches the current status descriptor by `(id, backend)`. It copies latency, callback counters, last callback sequence, backend state, backend detail, and diagnostics into editor-facing snapshots.

Action reports are best-effort. If configure/start/stop fails, the report stores `success = false`, records the error string, and attaches the freshest snapshot that can still be read. This keeps no-device or CPAL-unavailable environments visible in the UI instead of turning live output into a panic path.

## Mixer Toolbar Metadata

`mixer_console.v2.ui.toml` now exposes stable live output controls:

- `SoundOutputDevicePicker`
- `SoundOutputRefreshButton`
- `SoundOutputStartButton`
- `SoundOutputStopButton`
- `SoundOutputStatusPanel`

The buttons route to sound output operation paths. `Sound.Output.Device.Refresh` is a non-mutating operation descriptor for snapshot refresh, while `Sound.Output.Device.Configure`, `Sound.Output.Device.Start`, and `Sound.Output.Device.Stop` are the existing output lifecycle paths. The picker remains a control slot; this slice intentionally does not add a native sound pane payload or a sound-specific branch in `zircon_editor` operation dispatch.

## Authoring Template Contract

The checked-in Sound editor UI templates are treated as static plugin assets, not editor-core code. `mixer_console.v2.ui.toml`, `acoustic_debug.v2.ui.toml`, and the three component drawer templates now have registration coverage in `src/lib.rs`: each template asset id must match the registered Sound surface or drawer asset, and every `route = "..."` event in the templates must target an operation descriptor contributed by `sound_editor_operation_descriptors()`.

This keeps the mixer toolbar buttons and future drawer/acoustic debug controls from drifting away from the operation table while preserving the current boundary: the editor host sees generic operation paths and template documents, and Sound-specific behavior remains in the Sound plugin.

## Edge Cases

- Device enumeration failure yields an empty device list plus a diagnostic when status is still readable.
- Backend unavailable errors from start/configure are returned in the action report and merged into snapshot diagnostics.
- CPAL picker IDs remain runtime-provided neutral descriptors. The editor model does not persist OS device IDs and does not import CPAL types.
- The deterministic `software-null` row remains the expected fallback for tests and editor preview.

## Test Coverage

The live-output unit tests use a fake `SoundManager` to cover selected-row projection, latency/status/backend-state projection, configure/start/stop action calls, and best-effort failure reports. Existing sound editor registration tests continue to cover mixer views, operation descriptors, payload schema IDs, menu items, and component drawer bindings. The authoring template contract tests read the static TOML templates with `include_str!`, verify template asset ids, and reject any routed UI event whose operation path is not registered by the Sound editor plugin.

Current validation evidence: `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor`, `cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1`, focused `cargo test --manifest-path zircon_plugins\sound\editor\Cargo.toml live_output --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-editor-live-output --message-format short --color never`, focused `cargo test --manifest-path zircon_plugins\sound\editor\Cargo.toml sound_editor --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-sound-editor-template-routes --message-format short --color never`, `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --tests --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-sound-editor-template-routes --message-format short --color never`, and scoped `git diff --check` passed. The earlier live-output focused test needed a warmed retry after a dependency-compilation timeout; the template-route focused test passed 3 registration/template tests, and the remaining output was limited to existing `zircon_runtime`, `zircon_editor`, and non-CPAL sound runtime warnings.
