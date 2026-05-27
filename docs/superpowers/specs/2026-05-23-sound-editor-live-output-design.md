# Sound Editor Live Output Design

## Summary

Add editor-side live output wiring for the sound plugin without adding sound-specific branches to `zircon_editor` core. The slice introduces a plugin-local controller and view model over the neutral `SoundManager` output-device APIs, gives the Sound Mixer toolbar stable output-picker control metadata, and keeps runtime-owned audio state authoritative.

## Scope

This slice covers:

- a `zircon_plugins/sound/editor` live-output model for picker rows, selected status, latency, diagnostics, and backend state,
- a controller that calls `SoundManager::available_output_devices()`, `output_device_status()`, `configure_output_device(...)`, `start_output_device()`, and `stop_output_device()`,
- serializable operation result DTOs for future editor operation handlers or remote tooling,
- focused unit tests using a fake `SoundManager`,
- stable mixer toolbar control/event metadata for refresh, configure, start, and stop actions,
- docs, plan, and session evidence.

This slice does not cover:

- adding a sound-specific operation handler branch to `zircon_editor`,
- mapping `sound.mixer_console` into a native `PanePayload` variant,
- dynamic event ABI work,
- ray/occlusion acoustics,
- production DSP changes,
- persisted OS-level audio device IDs.

## Architecture

`zircon_runtime::core::framework::sound` remains the neutral contract owner. The editor plugin consumes `Arc<dyn SoundManager>` and only sees neutral descriptors, status, and errors. The concrete sound runtime remains in `zircon_plugins/sound/runtime`.

The new editor-owned boundary lives under `zircon_plugins/sound/editor/src/live_output/`:

- `model.rs` defines `SoundEditorOutputDeviceRow`, `SoundEditorOutputStatusModel`, `SoundEditorOutputSnapshot`, `SoundEditorOutputAction`, and `SoundEditorOutputActionReport`.
- `controller.rs` defines `SoundEditorLiveOutputController`, which owns no runtime state and only forwards to `SoundManager`.
- `mod.rs` is structural wiring and public re-export.

The crate root re-exports the live-output types. It does not parse payloads or execute workflows. `authoring_bindings.rs` continues to describe operation metadata only.

## Behavior

The controller builds a snapshot in one call:

- reads available device rows,
- reads output status,
- reads backend status,
- marks the selected row by comparing descriptor ID and backend,
- copies latency and diagnostics into editor-facing status data,
- records recoverable read errors as diagnostics instead of panicking.

Actions are explicit:

- `Refresh` returns the current snapshot without mutation.
- `Configure(descriptor)` calls `configure_output_device(...)`, then returns a refreshed snapshot.
- `Start` calls `start_output_device()`, then returns a refreshed snapshot.
- `Stop` calls `stop_output_device()`, then returns a refreshed snapshot.

If the manager call fails, the action returns a report with `success = false`, a string error, and the best-effort snapshot available after the failure. This lets the future UI show backend-unavailable diagnostics and still display the deterministic `software-null` row.

## Reference Evidence

- Godot's audio server exposes output device list, current output device, setter, and latency as editor-consumable server data. This supports exposing device picker/status as a first-class editor model rather than hardcoding CPAL in UI.
- Bevy keeps OS audio output optional and treats no-device environments as recoverable. This supports returning best-effort snapshots on start/configure errors.
- Fyrox keeps sound engine/runtime behavior separate from editor tooling. This supports a plugin-local editor controller over a manager trait instead of `zircon_editor` depending on concrete sound runtime state.

## Testing

Focused sound editor tests cover:

- snapshot projection marks the configured output row as selected,
- latency, backend state, and diagnostics are visible in the editor model,
- configure/start/stop actions invoke the manager and refresh state,
- manager failures produce unsuccessful action reports with best-effort diagnostics,
- sound editor plugin registration still contributes mixer operations and component drawers.

## Validation

Milestone validation should run:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_editor
cargo test --manifest-path "zircon_plugins\sound\editor\Cargo.toml" live_output --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-editor-live-output" --message-format short --color never
git diff --check -- "zircon_plugins\sound\editor" "docs\zircon_plugins\sound\editor.md" "docs\engine-architecture\runtime-sound-extension.md" "docs\superpowers\specs\2026-05-23-sound-editor-live-output-design.md" "docs\superpowers\plans\2026-05-23-sound-editor-live-output.md" ".codex\sessions\20260523-0748-sound-sequential-milestones.md"
```

Full workspace validation is deferred unless this slice unexpectedly changes shared editor/runtime wiring.
