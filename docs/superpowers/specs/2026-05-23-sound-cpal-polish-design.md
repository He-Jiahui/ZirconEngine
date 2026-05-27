# Sound CPAL Polish Design

## Summary

Polish the CPAL output adapter by adding neutral device enumeration, a picker-ready output-device contract, and explicit latency/status diagnostics. The slice keeps `software-null` deterministic for tests and editor preview, keeps CPAL feature-gated, and does not move any CPAL types across `zircon_runtime::core::framework::sound`.

## Scope

This slice covers:

- a neutral `SoundOutputDeviceInfo` DTO for device picker rows,
- a `SoundManager::available_output_devices()` method,
- software output device enumeration that always works without OS audio,
- CPAL output device enumeration when `cpal-backend` is compiled,
- CPAL default-device and per-session enumerated-device IDs that can be fed back into `configure_output_device()`,
- latency diagnostics on `SoundOutputDeviceStatus`,
- deterministic tests for software enumeration and feature-gated CPAL contracts,
- docs/session/plan evidence.

This slice does not cover:

- editor live UI wiring,
- stable OS-level persisted device IDs,
- input/capture devices,
- exclusive-mode WASAPI, ASIO, JACK, or host-specific tuning,
- lock-free production scheduling,
- dynamic sound event ABI,
- ray/occlusion acoustics,
- production DSP changes.

## Reference Evidence

- `dev/godot/servers/audio/audio_server.h` exposes `get_output_device_list()`, `get_output_device()`, `set_output_device(...)`, and `get_latency()`. This supports making device enumeration, selected-device descriptor data, and latency a first-class neutral sound manager contract.
- `dev/bevy/crates/bevy_audio/src/audio_output.rs` keeps audio output availability optional and warns when no device exists. This supports keeping CPAL enumeration best-effort and preserving deterministic software output when host devices are absent.
- `dev/Fyrox/fyrox-sound/src/engine.rs` separates headless sound rendering from optional OS output-device initialization. This supports keeping `software-null` and CPAL as peers behind the runtime output facade.

## Chosen Behavior

`zircon_runtime::core::framework::sound` gains neutral DTOs only. `SoundOutputDeviceInfo` contains a ready-to-configure `SoundOutputDeviceDescriptor`, an `is_default` flag, an `available` flag, and an optional diagnostic. Picker UIs can display rows, filter unavailable rows, and pass the descriptor back into `configure_output_device()` without knowing CPAL internals.

`SoundManager::available_output_devices()` returns software rows unconditionally. With `cpal-backend`, it also returns CPAL rows discovered through the default CPAL host. Without the feature, CPAL contributes no device rows, matching the existing backend catalog behavior.

CPAL device IDs are neutral strings. `sound.output.cpal.default` means the current platform default output device. `sound.output.cpal.device.<index>` means the indexed output device from the current CPAL enumeration pass. The indexed IDs are picker-session IDs, not persisted OS identifiers. If callers configure CPAL with an arbitrary non-picker ID, CPAL falls back to the platform default output device to preserve descriptor-driven manual configuration.

`SoundOutputDeviceStatus` gains `latency` and `diagnostics`. Latency is calculated from the configured block size, latency blocks, and sample rate. Software output reports the configured estimate and no queue depth. CPAL status merges backend-local ring-buffer capacity and currently buffered sample count when a session is active. Diagnostics collect unavailable backend details, CPAL stream errors, and callback/producer errors without panicking.

## Error Handling

- Invalid descriptors still fail through `SoundError::InvalidParameter`.
- CPAL feature-disabled selection still reports `BackendUnavailable` naming `cpal-backend`.
- CPAL enumeration failures are represented as unavailable device rows or omitted CPAL rows; software rows remain available.
- CPAL selected-device lookup failure returns `BackendUnavailable` with an ID-specific detail.
- Callback underruns fill silence and increment counters; they do not fail the manager call.

## Testing

Focused tests cover:

- software device enumeration always returns a deterministic picker row,
- the software picker descriptor can be passed into `configure_output_device()`,
- status reports latency frames/seconds and diagnostics,
- CPAL feature-disabled builds expose no CPAL device rows,
- CPAL feature-enabled builds compile enumeration and expose structured rows when devices exist,
- CPAL ID parsing/selection helpers are deterministic without opening an OS device,
- existing output-device and ring-buffer tests continue passing.

## Validation

Milestone validation should run:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
rustfmt --check zircon_runtime\src\core\framework\sound\output.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never
git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound\runtime" "docs\engine-architecture\runtime-sound-extension.md" "docs\superpowers\specs\2026-05-23-sound-cpal-polish-design.md" "docs\superpowers\plans\2026-05-23-sound-cpal-polish.md" ".codex\sessions\20260523-0748-sound-sequential-milestones.md"
```

Full workspace validation is intentionally deferred unless this slice unexpectedly changes shared workspace wiring beyond sound framework DTOs and the sound runtime plugin.
