# Sound CPAL Adapter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Windows-first CPAL sound output adapter with a runtime-owned ring buffer while preserving deterministic `software-null` output for CI and editor preview.

**Architecture:** `zircon_runtime::core::framework::sound` remains neutral; no CPAL types cross the framework boundary. `zircon_plugins/sound/runtime` converts `src/output.rs` into a folder-backed output subsystem with a facade, software backend behavior, CPAL feature-gated device state, and a backend-neutral ring buffer. `DefaultSoundManager` continues to call the output facade and keeps concrete backend details out of `service_types.rs`.

**Tech Stack:** Rust 2021, optional `cpal` dependency, Cargo feature `cpal-backend`, existing `SoundManager` output APIs, current sound runtime tests and docs.

---

## Current Baseline

- The root workspace is dirty and shared with other active lanes. This plan intentionally stays in the existing `main` checkout and does not create worktrees or branches.
- `zircon_plugins/sound/runtime/src/output.rs` currently owns descriptor validation, backend catalog, unavailable-backend status, output lifecycle counters, and callback report accounting in one 260-line file. Adding CPAL there would mix software, OS-device, and buffer concerns, so this plan first splits it into `src/output/` modules.
- `zircon_plugins/sound/runtime/src/service_types.rs` is already a large manager implementation. It should keep using output facade methods; CPAL code must not be added there.
- `software-null` and `software-*` descriptors already work for deterministic tests. Existing tests in `zircon_plugins/sound/runtime/src/tests/output_device.rs` cover catalog, callback reports, unavailable backend status, stopped pulls, reconfigure recovery, and runtime format updates.
- `docs/superpowers/specs/2026-05-21-sound-cpal-adapter-design.md` is the approved design source for this plan.

## Source Map

- Modify `zircon_plugins/sound/runtime/Cargo.toml`: add `cpal-backend` feature and optional `cpal` dependency.
- Modify `zircon_plugins/Cargo.lock`: update only if Cargo needs to lock CPAL and transitive dependencies.
- Delete/replace `zircon_plugins/sound/runtime/src/output.rs` with `zircon_plugins/sound/runtime/src/output/mod.rs`.
- Create `zircon_plugins/sound/runtime/src/output/software.rs`: software backend IDs, catalog rows, backend support validation for `software-null` / `software-*`.
- Create `zircon_plugins/sound/runtime/src/output/ring_buffer.rs`: bounded interleaved `f32` FIFO with deterministic tests.
- Create `zircon_plugins/sound/runtime/src/output/cpal.rs`: feature-gated CPAL capability row, descriptor support, stream state, producer lifecycle, callback drain behavior, and feature-disabled fallback module.
- Modify `zircon_plugins/sound/runtime/src/engine/state.rs`: keep `SoundOutputDeviceRuntimeState::new(config)` construction valid after the output module split.
- Modify `zircon_plugins/sound/runtime/src/service_types.rs`: update imports and call facade methods for configure/start/stop/status/render/callback without CPAL-specific logic.
- Modify `zircon_plugins/sound/runtime/src/tests/output_device.rs`: add feature-gated and feature-disabled CPAL backend tests plus recovery assertions.
- Update `docs/engine-architecture/runtime-sound-extension.md`: document CPAL backend architecture, ring-buffer behavior, tests, and remaining backend gaps.
- Update `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md`: record active CPAL adapter milestone, touched modules, coordination warnings, and validation evidence.

## Milestone 1: Output Module Boundary And Software Baseline

Goal: Split the current output implementation into focused modules without changing behavior.

In-scope behaviors:

- Existing `software-null` and `software-*` backend descriptors keep working.
- Existing output lifecycle counters and callback reports keep their field names and semantics.
- `service_types.rs` keeps using a narrow output facade.

Dependencies:

- Current `SoundOutputDeviceDescriptor`, `SoundOutputDeviceStatus`, `SoundBackendCapability`, and `SoundBackendCallbackReport` DTOs remain unchanged.

Implementation slices:

- [x] Move the contents of `zircon_plugins/sound/runtime/src/output.rs` into `zircon_plugins/sound/runtime/src/output/mod.rs` as the facade.
- [x] Extract `SOFTWARE_NULL_BACKEND`, `available_output_backends()` software row construction, and `validate_backend_supported(...)` software matching into `zircon_plugins/sound/runtime/src/output/software.rs`.
- [x] Keep `pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability>` exported from `output/mod.rs`; it should combine software rows with CPAL rows in Milestone 3.
- [x] Keep `pub(crate) fn validate_output_device_descriptor(...)` in `output/mod.rs` unless it becomes purely software-specific; descriptor validation is backend-neutral.
- [x] Keep `SoundOutputDeviceRuntimeState` in `output/mod.rs` for this milestone. Do not create a broad trait hierarchy yet.
- [x] Update `zircon_plugins/sound/runtime/src/lib.rs` implicitly by preserving `mod output;`; no crate-root behavior should be added.
- [x] Update imports in `service_types.rs` only if module paths change. The manager should still import `super::output::available_output_backends` and call methods on `state.output_device`.
- [x] Add a short comment above `SoundOutputDeviceRuntimeState` explaining that it is the facade state that delegates backend-specific behavior instead of storing neutral DTOs only.

Testing stage:

- [x] Run formatting for the sound runtime package.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
```

- [x] Run the existing output-device tests to prove the split preserved behavior.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

- [x] If the test command fails before compiling the sound crate because unrelated dirty workspace code is broken, record the exact upstream diagnostic in the session note and do not edit unrelated lanes.

Exit evidence:

- Existing output-device tests pass or are blocked only by a recorded external compile failure.
- `git diff --check -- "zircon_plugins\sound\runtime\src\output" "zircon_plugins\sound\runtime\src\service_types.rs"` has no whitespace errors.

## Milestone 2: Ring Buffer Foundation

Goal: Add the backend-neutral bounded FIFO used by the CPAL callback and producer thread.

In-scope behaviors:

- Interleaved `f32` samples are written and read in FIFO order.
- Reads can request more samples than are available and report the shortage.
- Shortage fill is deterministic silence.
- Buffer capacity is fixed and derived by callers; overflow keeps the newest successfully written samples only if the API explicitly returns dropped count.

Dependencies:

- Output module split from Milestone 1.

Implementation slices:

- [x] Create `zircon_plugins/sound/runtime/src/output/ring_buffer.rs` with `SoundOutputRingBuffer`.
- [x] Use `VecDeque<f32>` internally for clarity; do not optimize to a lock-free structure in this milestone.
- [x] Provide `pub(crate) fn new(capacity_samples: usize) -> Self` that clamps capacity to at least one sample.
- [x] Provide `pub(crate) fn available_samples(&self) -> usize` and `pub(crate) fn capacity_samples(&self) -> usize` for status/debug assertions.
- [x] Provide `pub(crate) fn push_samples(&mut self, samples: &[f32]) -> usize` returning dropped sample count when input exceeds remaining capacity. The implementation should drop oldest samples before pushing new samples so the callback receives the most recent rendered audio after a producer overrun.
- [x] Provide `pub(crate) fn drain_into_with_silence(&mut self, output: &mut [f32]) -> usize` returning the underrun sample count and filling missing output samples with `0.0`.
- [x] Add private module tests inside `ring_buffer.rs`:

```rust
#[test]
fn ring_buffer_preserves_fifo_order_across_partial_reads() {
    let mut buffer = SoundOutputRingBuffer::new(8);
    assert_eq!(buffer.push_samples(&[0.1, 0.2, 0.3, 0.4]), 0);

    let mut first = [0.0; 2];
    assert_eq!(buffer.drain_into_with_silence(&mut first), 0);
    assert_eq!(first, [0.1, 0.2]);

    assert_eq!(buffer.push_samples(&[0.5, 0.6]), 0);
    let mut second = [0.0; 4];
    assert_eq!(buffer.drain_into_with_silence(&mut second), 0);
    assert_eq!(second, [0.3, 0.4, 0.5, 0.6]);
}

#[test]
fn ring_buffer_zero_fills_shortage_and_preserves_future_reads() {
    let mut buffer = SoundOutputRingBuffer::new(4);
    assert_eq!(buffer.push_samples(&[0.25]), 0);
    let mut first = [9.0; 3];
    assert_eq!(buffer.drain_into_with_silence(&mut first), 2);
    assert_eq!(first, [0.25, 0.0, 0.0]);

    assert_eq!(buffer.push_samples(&[0.5, 0.75]), 0);
    let mut second = [0.0; 2];
    assert_eq!(buffer.drain_into_with_silence(&mut second), 0);
    assert_eq!(second, [0.5, 0.75]);
}

#[test]
fn ring_buffer_drops_oldest_samples_on_overflow() {
    let mut buffer = SoundOutputRingBuffer::new(3);
    assert_eq!(buffer.push_samples(&[1.0, 2.0, 3.0]), 0);
    assert_eq!(buffer.push_samples(&[4.0, 5.0]), 2);
    let mut output = [0.0; 3];
    assert_eq!(buffer.drain_into_with_silence(&mut output), 0);
    assert_eq!(output, [3.0, 4.0, 5.0]);
}
```

- [x] Re-export `SoundOutputRingBuffer` within `output/mod.rs` only as `pub(crate)` for `cpal.rs`; do not expose it from crate root.

Testing stage:

- [x] Run formatting.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
```

- [x] Run output tests, which should include private ring-buffer tests because they are in the same crate test binary.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" ring_buffer --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

Exit evidence:

- Ring-buffer tests pass.
- Existing output-device tests still pass or remain blocked only by a recorded external compile failure.

## Milestone 3: CPAL Feature Gate And Backend Catalog

Goal: Add CPAL dependency wiring and deterministic diagnostics before starting a real stream.

In-scope behaviors:

- `cpal-backend` Cargo feature exists on `zircon_plugin_sound_runtime`.
- Backend catalog includes `cpal` only when the feature is compiled.
- Selecting `cpal` without the feature returns `BackendUnavailable` with a feature-disabled detail and preserves recovery to `software-null`.
- Existing `software-null` behavior is unchanged.

Dependencies:

- Output module facade from Milestone 1.
- Ring buffer foundation from Milestone 2.

Implementation slices:

- [x] Modify `zircon_plugins/sound/runtime/Cargo.toml`:

```toml
[features]
default = []
cpal-backend = ["dep:cpal"]

[dependencies]
zircon_runtime = { path = "../../../zircon_runtime", default-features = false }
cpal = { version = "0.15", optional = true }
```

- [x] Create `zircon_plugins/sound/runtime/src/output/cpal.rs` with these feature-gated constants and catalog functions:

```rust
pub(crate) const CPAL_BACKEND: &str = "cpal";

#[cfg(feature = "cpal-backend")]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: CPAL_BACKEND.to_string(),
        display_name: "CPAL Default Output".to_string(),
        realtime_capable: true,
        deterministic: false,
        min_sample_rate_hz: 8_000,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "uses the platform default output device through CPAL".to_string(),
            "availability depends on host audio devices and OS permissions".to_string(),
        ],
    }]
}

#[cfg(not(feature = "cpal-backend"))]
pub(crate) fn cpal_backend_capabilities() -> Vec<SoundBackendCapability> {
    Vec::new()
}
```

- [x] Add `pub(crate) fn cpal_backend_unavailable_detail() -> String` under `#[cfg(not(feature = "cpal-backend"))]` returning `"sound output backend `cpal` requires the `cpal-backend` feature".to_string()`.
- [x] Update `output/mod.rs::available_output_backends()` to append `cpal_backend_capabilities()` after software capabilities.
- [x] Update backend support validation so backend `cpal` is accepted only when `feature = "cpal-backend"`; without the feature it returns `SoundError::BackendUnavailable { detail: cpal_backend_unavailable_detail() }`.
- [x] In `zircon_plugins/sound/runtime/src/tests/output_device.rs`, update `output_backends_list_deterministic_null_backend` to keep asserting `software-null` behavior and add a new feature-disabled test:

```rust
#[cfg(not(feature = "cpal-backend"))]
#[test]
fn cpal_backend_reports_feature_disabled_when_not_compiled() {
    let sound = DefaultSoundManager::default();
    assert!(sound
        .available_output_backends()
        .unwrap()
        .iter()
        .all(|backend| backend.backend != "cpal"));

    let error = sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.disabled"),
            backend: "cpal".to_string(),
            display_name: "CPAL Disabled".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap_err();
    assert!(error.to_string().contains("cpal-backend"));
    assert_eq!(sound.backend_status().requested_backend, "cpal");
    assert_eq!(sound.backend_status().state, SoundBackendState::Unavailable);

    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.recovery"),
            backend: "software-null".to_string(),
            display_name: "Software Null Recovery".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();
    assert_eq!(sound.backend_status().state, SoundBackendState::Ready);
}
```

- [x] Add a feature-enabled catalog test:

```rust
#[cfg(feature = "cpal-backend")]
#[test]
fn cpal_backend_is_listed_when_feature_is_enabled() {
    let sound = DefaultSoundManager::default();
    let backend = sound
        .available_output_backends()
        .unwrap()
        .into_iter()
        .find(|backend| backend.backend == "cpal")
        .expect("cpal backend should be listed with cpal-backend feature");
    assert!(backend.realtime_capable);
    assert!(!backend.deterministic);
}
```

Testing stage:

- [x] Run non-CPAL formatting and tests.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

- [x] Run CPAL feature compile/test. This command may require network if CPAL is not already present in the local Cargo cache; if it fails only because offline mode cannot resolve CPAL, rerun without `--offline` after confirming `zircon_plugins/Cargo.lock` change is in scope.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" cpal_backend --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

Exit evidence:

- Feature-disabled CPAL diagnostics are covered without physical devices.
- Feature-enabled catalog test compiles and passes, or CPAL dependency acquisition is recorded as the only blocker.

## Milestone 4: CPAL Runtime State And Producer/Callback Lifecycle

Goal: Implement real Windows-first CPAL output startup, callback drain, producer rendering, stop, and recovery.

In-scope behaviors:

- CPAL stream state exists only under `output/cpal.rs` and only when the feature is enabled.
- CPAL callback drains `SoundOutputRingBuffer` and fills silence on underrun.
- Producer thread renders configured block-sized audio into the ring buffer.
- Stop/reconfigure drops stream state and joins the producer thread.
- Startup failure stores structured unavailable detail and remains recoverable.

Dependencies:

- CPAL feature gate/catalog from Milestone 3.
- Ring buffer foundation from Milestone 2.
- Existing `SoundEngineState::render_mix(&SoundConfig, frames)` path.

Implementation slices:

- [x] Add a backend session field to `SoundOutputDeviceRuntimeState` in `output/mod.rs`:

```rust
backend_session: SoundOutputBackendSession,
```

- [x] Define `enum SoundOutputBackendSession` in `output/mod.rs`:

```rust
enum SoundOutputBackendSession {
    None,
    #[cfg(feature = "cpal-backend")]
    Cpal(super::cpal::CpalOutputSession),
}
```

Use `mod cpal;` and `mod ring_buffer;` from `output/mod.rs`; adjust `super::cpal` to the correct module path after extraction.

- [x] Add `SoundOutputDeviceRuntimeState::clear_backend_session(&mut self)` that stops and drops any active backend session before descriptor replacement or stopped teardown.
- [x] Modify `configure(...)` to call `clear_backend_session()` before replacing descriptor.
- [x] Modify `stop(&mut self)` to call `clear_backend_session()` and then set state to `Stopped`.
- [x] Keep `record_rendered_block`, `record_callback_block`, and `record_callback_error` in the facade so software/manual output remains unchanged.
- [x] In `output/cpal.rs`, define `CpalOutputSession` with fields for stream, producer stop flag, producer thread handle, shared ring buffer, shared counters, and last error. Keep fields private.
- [x] In `output/cpal.rs`, define `CpalOutputSharedState` with `Mutex<SoundOutputRingBuffer>`, underrun count, callback count, last callback sequence, and last error guarded by `Mutex` or atomics where practical. Avoid exposing this type outside `output/cpal.rs`.
- [x] Add `pub(crate) fn start_cpal_session(...) -> Result<CpalOutputSession, SoundError>` under `#[cfg(feature = "cpal-backend")]`. Inputs should include descriptor, shared sound `Arc<Mutex<SoundEngineState>>`, shared config `Arc<Mutex<SoundConfig>>`, and enough facade callback/report helpers to record status. If direct helper borrowing is too complex, return backend-local counters and let `SoundOutputDeviceRuntimeState::status()` merge them.
- [x] Change `DefaultSoundManager::start_output_device` in `service_types.rs` so it delegates to a new facade method:

```rust
state.output_device.start_with_engine(self.state.clone(), self.config.clone())?;
```

`start_with_engine(...)` should handle `software-*` by setting `Started` immediately and `cpal` by calling `start_cpal_session(...)`.

- [x] In `start_cpal_session(...)`, use CPAL default host/default output device. On Windows this should use the default CPAL host; do not add WASAPI-specific APIs yet.
- [x] Choose a stream config from the device supported output configs by preferring descriptor sample rate/channel count and `f32` sample format. If the device only supports another sample format, return `BackendUnavailable` in this milestone rather than adding sample conversion.
- [x] Build an output stream whose data callback locks only the ring-buffer shared state, drains into the output slice, fills silence, and increments underrun counters. The callback must not lock `SoundEngineState`.
- [x] Spawn a producer thread before or immediately after stream play. The producer loops until stop flag is set, renders `descriptor.block_size_frames`, pushes samples into the ring buffer, and sleeps/yields when the buffer is near capacity. Keep the sleep deterministic and simple, based on block duration or a small `Duration::from_millis(1)` backoff.
- [x] On stream error callback, record error detail in shared backend state. Do not panic.
- [x] On producer render error, record error detail and push one silent block to avoid tight failure loops.
- [x] Implement `Drop` or explicit `stop(self)` for `CpalOutputSession` to signal the thread and join it. Avoid blocking indefinitely; if a join panic occurs, convert it into stored `last_error` if possible.
- [x] Make `SoundOutputDeviceRuntimeState::status()` merge CPAL backend-local callback/underrun/last-error counters with facade counters while a CPAL session is active.
- [x] Add a feature-gated smoke test in `output_device.rs`:

```rust
#[cfg(all(feature = "cpal-backend", target_os = "windows"))]
#[test]
fn cpal_backend_start_stop_is_structured_on_windows() {
    let sound = DefaultSoundManager::default();
    sound
        .configure_output_device(SoundOutputDeviceDescriptor {
            id: SoundOutputDeviceId::new("sound.output.cpal.windows"),
            backend: "cpal".to_string(),
            display_name: "CPAL Windows Default Output".to_string(),
            sample_rate_hz: 48_000,
            channel_count: 2,
            block_size_frames: 128,
            latency_blocks: 2,
        })
        .unwrap();

    match sound.start_output_device() {
        Ok(()) => {
            assert_eq!(sound.output_device_status().unwrap().state, SoundOutputDeviceState::Started);
            sound.stop_output_device().unwrap();
            assert_eq!(sound.output_device_status().unwrap().state, SoundOutputDeviceState::Stopped);
        }
        Err(error) => {
            assert!(error.to_string().contains("cpal") || error.to_string().contains("device"));
            assert_eq!(sound.output_device_status().unwrap().state, SoundOutputDeviceState::Stopped);
        }
    }
}
```

- [x] Add a private CPAL helper test, if possible without opening a device, that calls the callback-drain helper with an undersized ring buffer and asserts silence fill plus underrun counter increments.

Testing stage:

- [x] Run formatting.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
```

- [x] Run non-feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

- [x] Run CPAL feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

Exit evidence:

- CPAL feature build compiles.
- Feature-gated Windows smoke test either starts/stops or returns structured unavailable status.
- Callback-drain/ring-buffer behavior is covered without a device.

## Milestone 5: Documentation, Session, And Acceptance Validation

Goal: Record the CPAL backend behavior, validation evidence, and remaining backend gaps.

In-scope behaviors:

- Architecture docs mention the feature gate, ring-buffer model, and fallback/recovery behavior.
- Session note communicates active/complete status and coordination warnings.
- Plan checkboxes reflect implemented milestones.
- Validation commands are rerun after docs updates before claiming completion.

Dependencies:

- Milestones 1-4 implemented.

Implementation slices:

- [x] Update `docs/engine-architecture/runtime-sound-extension.md` header/frontmatter if present to include the new output module files, plan source, and tests. If the doc does not have YAML frontmatter, preserve its current style but add implementation files, plan sources, and validation evidence in the existing sections.
- [x] Document that `software-null` remains deterministic and `cpal` is feature-gated and host-device dependent.
- [x] Document that the CPAL callback drains a bounded ring buffer and fills underruns with silence instead of rendering under the callback lock.
- [x] Record remaining gaps: device enumeration, device picker/editor UX, per-host latency tuning, cross-platform manual validation, lock-free production scheduling, input/capture support.
- [x] Update `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md` with current CPAL adapter status, touched modules, validation commands/results, and warnings to avoid unrelated plugin manifest/render/editor lanes.
- [x] Update this plan's checkboxes as each milestone finishes.

Testing stage:

- [x] Check disk before Cargo validation. If the drive hosting `E:\cargo-targets\zircon-sound-cpal-adapter` has `<= 50 GB` free, run:

```powershell
cargo clean --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter"
```

- [x] Run final formatting.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
```

- [x] Run final non-feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

- [x] Run final CPAL feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never
```

- [x] Run final whitespace check.

```powershell
git diff --check -- "zircon_plugins\sound\runtime" "zircon_plugins\sound\runtime\Cargo.toml" "zircon_plugins\Cargo.lock" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-21-sound-cpal-adapter-design.md" "docs\superpowers\plans\2026-05-21-sound-cpal-adapter.md"
```

Exit evidence:

- Formatting passes.
- Non-feature output tests pass.
- CPAL-feature output tests pass or fail only with structured device-unavailable assertions accepted by the smoke test.
- Whitespace check has no errors. LF-to-CRLF warnings are acceptable if no whitespace errors appear.

Final evidence:

- `E:` free space was 42.80 GB, so `cargo clean --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter"` ran and removed 4214 files / 8.1 GiB before final Cargo validation.
- `cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime` passed.
- `cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never` passed: 7 passed, 0 failed, 81 filtered, with existing `zircon_runtime` warnings.
- `cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never` passed: 9 passed, 0 failed, 81 filtered, including the Windows CPAL structured start/stop smoke test, with existing `zircon_runtime` warnings.
- `cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" ring_buffer --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-adapter" --message-format short --color never` passed: 4 passed, 0 failed, 84 filtered, with existing `zircon_runtime` warnings.
- `git diff --check -- "zircon_plugins\sound\runtime" "zircon_plugins\sound\runtime\Cargo.toml" "zircon_plugins\Cargo.lock" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-21-sound-cpal-adapter-design.md" "docs\superpowers\plans\2026-05-21-sound-cpal-adapter.md"` reported no whitespace errors; only LF-to-CRLF warnings appeared.

## Acceptance Criteria

- `software-null` remains deterministic, always listed, and covered by existing output tests.
- `cpal-backend` feature exists and gates the optional CPAL dependency.
- Backend catalog lists `cpal` only when the feature is compiled.
- Selecting `cpal` without the feature gives a typed feature-disabled unavailable diagnostic and can recover by reconfiguring `software-null`.
- With `cpal-backend`, Windows default-output startup is attempted through CPAL and either starts/stops cleanly or returns structured unavailable detail.
- CPAL callback does not call `render_mix` or lock the sound engine state; it only drains ring-buffer samples and fills underruns with silence.
- Producer thread renders mixer blocks into the ring buffer and stops cleanly on output stop or reconfigure.
- Docs and session notes record implementation files, plan/spec sources, validation evidence, and remaining backend gaps.
