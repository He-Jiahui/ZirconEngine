# Sound CPAL Adapter Design

## Summary

Add a Windows-first CPAL output adapter for the sound runtime on top of the existing backend seam. The adapter should keep `software-null` as the deterministic headless backend, add `cpal` as a real device backend behind a `cpal-backend` feature, and use a runtime-owned ring buffer so CPAL's realtime callback does not lock and render the whole sound engine directly.

## Scope

This slice covers:

- a `cpal` backend id in the sound runtime backend catalog,
- optional `cpal` dependency wiring for `zircon_plugin_sound_runtime`,
- Windows-first default-output device opening through CPAL,
- sample-rate/channel/block-size negotiation through the existing `SoundOutputDeviceDescriptor`,
- a bounded runtime-local ring buffer between mixer rendering and the CPAL callback,
- producer-thread lifecycle management for CPAL output,
- typed unavailable diagnostics and recovery back to `software-null`,
- focused tests, docs, and session-note evidence.

This slice does not cover:

- input/capture devices,
- device picker UI or stable per-device IDs,
- WebAudio/mobile-specific adapters,
- lock-free or wait-free production-grade audio scheduling,
- exclusive-mode WASAPI, ASIO, JACK, or per-host advanced tuning,
- editor live operation routing beyond existing SoundManager output APIs.

## Architecture

`zircon_runtime::core::framework::sound` remains the neutral contract layer. It already exposes `SoundOutputDeviceDescriptor`, backend capability/status DTOs, and `SoundManager` output methods. No CPAL type, stream handle, callback closure, or OS device object may cross this boundary.

`zircon_plugins/sound/runtime` owns concrete output implementation. The current `output.rs` facade should become a folder-backed output subsystem before adding more backend behavior:

- `zircon_plugins/sound/runtime/src/output/mod.rs` stays the narrow facade used by `service_types.rs`.
- `zircon_plugins/sound/runtime/src/output/software.rs` owns deterministic `software-null` / `software-*` behavior.
- `zircon_plugins/sound/runtime/src/output/cpal.rs` owns CPAL feature-gated device opening, stream lifecycle, and producer/callback glue.
- `zircon_plugins/sound/runtime/src/output/ring_buffer.rs` owns the backend-neutral bounded interleaved `f32` buffer and tests.

`service_types.rs` should continue to configure, start, stop, and query the output facade. It should not contain CPAL-specific branches beyond forwarding errors from the output subsystem.

## Reference Evidence

- `dev/bevy/crates/bevy_audio/src/audio_output.rs` uses an internal `AudioOutput` resource with `Option<MixerDeviceSink>`. Missing output device logs a warning and systems skip playback rather than panicking.
- `dev/bevy/crates/bevy_audio/src/lib.rs` wires audio output as a plugin resource and gates playback systems with `audio_output_available`, which supports Zircon keeping CPAL backend availability as runtime status instead of requiring devices in CI.
- `dev/Fyrox/fyrox-sound/src/engine.rs` separates `SoundEngine::without_device` headless rendering from `initialize_audio_output_device()`, and output-device callbacks feed from engine render state. This supports keeping `software-null` and manual render pulls beside a real device path.
- `dev/godot/servers/audio/audio_driver_dummy.cpp` and `audio_driver_dummy.h` implement a dummy driver that repeatedly asks the shared audio server to mix into a buffer. This supports a deterministic non-device backend as a first-class driver rather than a testing hack.
- `dev/godot/drivers/wasapi/audio_driver_wasapi.cpp` and `audio_driver_wasapi.h` isolate platform device state, active flags, buffers, and restart details inside the driver. This supports keeping CPAL stream state inside the sound runtime output adapter rather than leaking it into neutral DTOs.

## Chosen Behavior

The backend catalog lists `software-null` unconditionally. It lists `cpal` when the `cpal-backend` feature is compiled. Without that feature, selecting backend `cpal` returns `SoundError::BackendUnavailable` with a feature-disabled detail. With the feature enabled, selecting `cpal` validates through CPAL device/config discovery during `start_output_device()`.

`configure_output_device()` stores the requested descriptor and updates mixer format exactly as the current output path does. For CPAL, descriptor validation still rejects empty IDs, empty backend/display name, zero sample rate, zero channels, zero block size, or zero latency blocks. It does not open the OS device until `start_output_device()`.

`start_output_device()` for CPAL opens the Windows default output device through CPAL, chooses an output stream config compatible with the descriptor where possible, creates a bounded ring buffer sized from `block_size_frames`, `channel_count`, and `latency_blocks`, starts a producer thread, starts the CPAL stream, and transitions status to `Started`. If device discovery, format negotiation, stream build, or stream play fails, the output state records `BackendUnavailable` detail and remains stopped.

The CPAL callback drains the ring buffer into the device-provided output slice and fills missing samples with silence. It must not call `SoundEngineState::render_mix` and must not lock the main sound engine mutex. The callback records underrun/error counters through thread-safe backend-local counters that are merged into `SoundOutputDeviceStatus`.

The producer thread repeatedly renders `SoundMixBlock`s by locking the sound engine state, calling the same mixer path used by `pull_output_backend_callback()`, and pushing interleaved samples into the ring buffer. If rendering fails, it stores the error, fills silence if necessary, and keeps the backend recoverable until stopped or reconfigured.

`stop_output_device()` for CPAL signals the producer, drops the CPAL stream, joins the producer thread, and leaves clips, sources, mixer graph, DSP state, and loaded profiles intact. Reconfiguring to `software-null` clears CPAL runtime state and restores deterministic manual pulls.

## Error Handling

Errors should remain structured through existing `SoundError` variants:

- feature-disabled CPAL: `BackendUnavailable` with a detail naming the missing `cpal-backend` feature,
- no default output device: `BackendUnavailable` with CPAL device detail,
- unsupported sample format/channel/rate: `BackendUnavailable` or `InvalidParameter` depending on whether the descriptor is invalid or the host cannot satisfy it,
- stream build/play failure: `BackendUnavailable` with CPAL error text,
- callback underrun: status counter increment plus silence fill, not a panic,
- producer render failure: status `last_error` and recoverable stopped/unavailable state when the failure is terminal,
- recovery: configuring `software-null` after a CPAL failure clears unavailable detail and returns `backend_status().state` to `Ready`.

## Testing

Focused runtime tests should cover deterministic behavior without requiring a physical audio device:

- backend catalog lists `software-null` always,
- backend catalog reports `cpal` only when `cpal-backend` is compiled,
- selecting `cpal` without `cpal-backend` returns a feature-disabled unavailable diagnostic,
- reconfiguring from failed/unavailable `cpal` back to `software-null` clears unavailable state,
- ring buffer preserves FIFO sample order across partial writes and partial reads,
- ring buffer returns zero-filled shortage counts without corrupting future reads,
- CPAL output state can be stopped after a failed or skipped startup without panics,
- CPAL status counters report callback underruns and producer errors through `SoundOutputDeviceStatus`.

Feature-gated smoke coverage may attempt to start the real CPAL default output on Windows. The test must accept either successful start/stop or a structured `BackendUnavailable` result, because CI and developer machines may not expose an output device.

## Validation

Milestone validation should run scoped sound runtime checks:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
git diff --check -- "zircon_plugins\sound\runtime" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-21-sound-cpal-adapter-design.md" "docs\superpowers\plans\2026-05-21-sound-cpal-adapter.md"
```

If adding CPAL requires updating `zircon_plugins/Cargo.lock`, that lockfile change is in scope for this milestone. Full plugin-workspace validation remains a later expansion gate if unrelated dirty workspace changes block it.

## Remaining Follow-Up

After this slice, remaining backend work includes device enumeration, device picker/editor UX, per-host latency tuning, exclusive-mode Windows support if needed, cross-platform manual validation on Linux/macOS, lock-free production audio scheduling, and input/capture support.
