# Sound Backend Seam Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a deterministic backend-adapter seam for the sound runtime so future CPAL/OS backends can drive the mixer through a stable callback contract.

**Architecture:** `zircon_runtime::core::framework::sound` owns neutral backend capability, callback report, and status DTOs plus `SoundManager` trait methods. `zircon_plugins/sound/runtime` owns the concrete software/null adapter, output lifecycle state, callback accounting, and focused tests. Existing output-device APIs remain valid and delegate to the new seam instead of being replaced.

**Tech Stack:** Rust, Cargo, `zircon_runtime` neutral framework DTOs, `zircon_plugins/sound/runtime` software mixer, existing sound runtime tests and docs.

---

## Source Map

- Modify `zircon_runtime/src/core/framework/sound/output.rs`: add backend capability/range/callback DTOs and extend status fields only if required.
- Modify `zircon_runtime/src/core/framework/sound/manager.rs`: add manager methods for backend capability listing and backend callback pull.
- Modify `zircon_runtime/src/core/framework/sound/mod.rs`: re-export new output DTOs.
- Modify `zircon_plugins/sound/runtime/src/output.rs`: implement backend capability catalog, backend validation, callback report accounting, and the deterministic software/null adapter behavior.
- Modify `zircon_plugins/sound/runtime/src/service_types.rs`: wire `SoundManager` methods to the output adapter and existing `render_mix` path.
- Modify `zircon_plugins/sound/runtime/src/tests.rs`: import new output DTOs if needed.
- Modify `zircon_plugins/sound/runtime/src/tests/output_device.rs`: add backend seam tests.
- Modify `docs/engine-architecture/runtime-sound-extension.md`: document backend seam behavior, tests, and remaining backend gap.
- Modify `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md`: record implementation and validation evidence.

## Milestone 1: Neutral Backend Contract

Goal: Define the host-facing callback contract without adding platform dependencies.

- [ ] Add `SoundBackendCapability` to `zircon_runtime/src/core/framework/sound/output.rs`.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCapability {
    pub backend: String,
    pub display_name: String,
    pub realtime_capable: bool,
    pub deterministic: bool,
    pub min_sample_rate_hz: u32,
    pub max_sample_rate_hz: u32,
    pub min_channel_count: u16,
    pub max_channel_count: u16,
    pub min_block_size_frames: usize,
    pub max_block_size_frames: usize,
    pub notes: Vec<String>,
}
```

- [ ] Add `SoundBackendCallbackReport` to `zircon_runtime/src/core/framework/sound/output.rs`.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCallbackReport {
    pub device: SoundOutputDeviceId,
    pub backend: String,
    pub sequence_index: u64,
    pub requested_frames: usize,
    pub rendered_frames: usize,
    pub sample_count: usize,
    pub underrun: bool,
    pub error: Option<String>,
}
```

- [ ] Add `SoundBackendCallbackBlock` to `zircon_runtime/src/core/framework/sound/output.rs`.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundBackendCallbackBlock {
    pub report: SoundBackendCallbackReport,
    pub block: SoundMixBlock,
}
```

If `SoundMixBlock` is not currently imported in `output.rs`, add `use super::{SoundMixBlock, SoundOutputDeviceId};` and remove the old single-item import.

- [ ] Extend `SoundOutputDeviceStatus` with callback-specific counters.

```rust
pub callback_count: u64,
pub last_callback_sequence: Option<u64>,
```

Keep existing fields unchanged so old callers still have rendered block/frame and error data.

- [ ] Extend `SoundManager` in `zircon_runtime/src/core/framework/sound/manager.rs`.

```rust
fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError>;
fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError>;
```

Add the new DTOs to the `use super::{...}` list.

- [ ] Re-export the new DTOs in `zircon_runtime/src/core/framework/sound/mod.rs` from the `output` module export list.

## Milestone 2: Runtime Software/Null Adapter

Goal: Make the existing software output path behave like a backend adapter with validation and callback reports.

- [ ] In `zircon_plugins/sound/runtime/src/output.rs`, add constants for the deterministic backend ids.

```rust
pub(crate) const SOFTWARE_NULL_BACKEND: &str = "software-null";
pub(crate) const SOFTWARE_TEST_BACKEND: &str = "software-test";
```

Retain existing descriptors using other non-empty backend names where tests already rely on them, but make the capability catalog include `software-null` as the canonical backend seam.

- [ ] Add a backend capability catalog function.

```rust
pub(crate) fn available_output_backends() -> Vec<SoundBackendCapability> {
    vec![SoundBackendCapability {
        backend: SOFTWARE_NULL_BACKEND.to_string(),
        display_name: "Deterministic Software Null Output".to_string(),
        realtime_capable: false,
        deterministic: true,
        min_sample_rate_hz: 1,
        max_sample_rate_hz: 384_000,
        min_channel_count: 1,
        max_channel_count: 64,
        min_block_size_frames: 1,
        max_block_size_frames: 65_536,
        notes: vec![
            "headless backend for tests and editor preview".to_string(),
            "pulls blocks from the software mixer without opening an OS device".to_string(),
        ],
    }]
}
```

- [ ] Add backend-specific descriptor validation that accepts the canonical backend and preserves legacy software test descriptors.

```rust
fn validate_backend_supported(descriptor: &SoundOutputDeviceDescriptor) -> Result<(), SoundError> {
    if descriptor.backend == SOFTWARE_NULL_BACKEND || descriptor.backend.starts_with("software-") {
        return Ok(());
    }
    Err(SoundError::BackendUnavailable {
        detail: format!("sound output backend `{}` is not available", descriptor.backend),
    })
}
```

Call this from `SoundOutputDeviceRuntimeState::configure` after generic descriptor validation.

- [ ] Add callback sequence accounting fields to `SoundOutputDeviceRuntimeState`.

```rust
callback_count: u64,
last_callback_sequence: Option<u64>,
next_callback_sequence: u64,
```

Reset these fields in `new` and `configure`.

- [ ] Add a method that records successful callback reports.

```rust
pub(crate) fn record_callback_block(
    &mut self,
    requested_frames: usize,
    rendered_frames: usize,
    sample_count: usize,
) -> SoundBackendCallbackReport {
    let sequence_index = self.next_callback_sequence;
    self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
    self.callback_count = self.callback_count.saturating_add(1);
    self.last_callback_sequence = Some(sequence_index);
    self.record_rendered_block(rendered_frames, sample_count);
    let expected_samples = requested_frames.saturating_mul(self.descriptor.channel_count as usize);
    SoundBackendCallbackReport {
        device: self.descriptor.id.clone(),
        backend: self.descriptor.backend.clone(),
        sequence_index,
        requested_frames,
        rendered_frames,
        sample_count,
        underrun: rendered_frames != requested_frames || sample_count != expected_samples,
        error: None,
    }
}
```

- [ ] Add a method that records callback errors.

```rust
pub(crate) fn record_callback_error(
    &mut self,
    requested_frames: usize,
    error: &SoundError,
) -> SoundBackendCallbackReport {
    let sequence_index = self.next_callback_sequence;
    self.next_callback_sequence = self.next_callback_sequence.saturating_add(1);
    self.callback_count = self.callback_count.saturating_add(1);
    self.last_callback_sequence = Some(sequence_index);
    self.record_error(error);
    SoundBackendCallbackReport {
        device: self.descriptor.id.clone(),
        backend: self.descriptor.backend.clone(),
        sequence_index,
        requested_frames,
        rendered_frames: 0,
        sample_count: 0,
        underrun: true,
        error: Some(error.to_string()),
    }
}
```

- [ ] Update `SoundOutputDeviceRuntimeState::status` to fill the new callback status fields.

## Milestone 3: Manager Wiring And Tests

Goal: Expose the backend seam through `DefaultSoundManager` and cover lifecycle behavior.

- [ ] In `zircon_plugins/sound/runtime/src/service_types.rs`, import `available_output_backends` and implement `SoundManager::available_output_backends`.

```rust
fn available_output_backends(&self) -> Result<Vec<SoundBackendCapability>, SoundError> {
    Ok(available_output_backends())
}
```

- [ ] Implement `SoundManager::pull_output_backend_callback` in `DefaultSoundManager`.

Implementation shape:

```rust
fn pull_output_backend_callback(&self) -> Result<SoundBackendCallbackBlock, SoundError> {
    let frames = {
        let state = self.state.lock().expect("sound engine state poisoned");
        state.output_device.block_size_frames()?
    };

    match self.render_mix(frames) {
        Ok(block) => {
            let mut state = self.state.lock().expect("sound engine state poisoned");
            let report = state.output_device.record_callback_block(
                frames,
                block.frames,
                block.samples.len(),
            );
            Ok(SoundBackendCallbackBlock { report, block })
        }
        Err(error) => {
            let mut state = self.state.lock().expect("sound engine state poisoned");
            let report = state.output_device.record_callback_error(frames, &error);
            Err(SoundError::BackendUnavailable {
                detail: report.error.unwrap_or_else(|| error.to_string()),
            })
        }
    }
}
```

If `SoundMixBlock` uses a different frame field name, use the existing field from `zircon_runtime/src/core/framework/sound/mix.rs`.

- [ ] In `zircon_plugins/sound/runtime/src/tests/output_device.rs`, add `output_backends_list_deterministic_null_backend`.

Assertions:

```rust
let backends = sound.available_output_backends().unwrap();
let backend = backends
    .iter()
    .find(|backend| backend.backend == "software-null")
    .expect("software-null backend should be listed");
assert!(backend.deterministic);
assert!(!backend.realtime_capable);
assert!(backend.max_sample_rate_hz >= 48_000);
assert!(backend.max_channel_count >= 2);
```

- [ ] Add `software_null_backend_callback_reports_rendered_block`.

Use descriptor backend `software-null`, block size 2, stereo, insert the existing two-frame test clip, start device, call `pull_output_backend_callback`, and assert:

```rust
assert_eq!(callback.report.backend, "software-null");
assert_eq!(callback.report.sequence_index, 0);
assert_eq!(callback.report.requested_frames, 2);
assert_eq!(callback.report.rendered_frames, 2);
assert_eq!(callback.report.sample_count, 4);
assert!(!callback.report.underrun);
assert_eq!(callback.report.error, None);
assert_samples_near(&callback.block.samples, &[0.25, 0.25, 0.5, 0.5]);
```

Then query status and assert `callback_count == 1` and `last_callback_sequence == Some(0)`.

- [ ] Add `software_null_backend_rejects_stopped_callback_and_unsupported_backend`.

Assertions:

```rust
assert!(sound
    .pull_output_backend_callback()
    .unwrap_err()
    .to_string()
    .contains("stopped"));

let error = sound
    .configure_output_device(SoundOutputDeviceDescriptor {
        id: SoundOutputDeviceId::new("sound.output.unsupported"),
        backend: "cpal".to_string(),
        display_name: "Unsupported CPAL".to_string(),
        sample_rate_hz: 48_000,
        channel_count: 2,
        block_size_frames: 128,
        latency_blocks: 2,
    })
    .unwrap_err();
assert!(error.to_string().contains("not available"));
```

- [ ] Keep existing `render_output_device_block` tests passing by preserving current software backend behavior for `software-test` descriptors.

## Milestone 4: Documentation And Session Coordination

Goal: Keep docs synchronized with the new backend seam and record validation boundaries.

- [ ] Update `docs/engine-architecture/runtime-sound-extension.md` frontmatter `related_code`, `implementation_files`, and `tests` if new files or commands are added.
- [ ] Update the behavior model to describe backend capabilities, deterministic software/null callback pulls, and the fact that OS devices remain future work.
- [ ] Update edge cases to mention unsupported backend ids and stopped callback pulls.
- [ ] Update test coverage to mention backend capability listing and callback accounting.
- [ ] Update `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md` with the backend seam implementation and current validation evidence.

## Milestone 5: Testing Stage And Correction Loop

Goal: Validate the sound backend seam and document any external blockers honestly.

- [ ] Run formatting checks.

Commands:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
rustfmt --check "zircon_runtime\src\core\framework\sound\output.rs" "zircon_runtime\src\core\framework\sound\manager.rs" "zircon_runtime\src\core\framework\sound\mod.rs"
```

- [ ] Run focused output tests if the workspace compiles past unrelated dirty lanes.

Command:

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-backend-seam" --message-format short --color never
```

Expected sound result: all `output_device` tests pass.

- [ ] Run full sound runtime tests if focused tests pass and the external render-pipeline compile blocker is resolved.

Command:

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-backend-seam" --message-format short --color never
```

Expected sound result: all sound runtime unit tests pass.

- [ ] Run whitespace check over touched files.

Command:

```powershell
git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md"
```

- [ ] If Cargo fails before compiling sound on the known render-pipeline blocker, record the exact diagnostic in docs/session notes and do not edit render-pipeline files from this sound plan unless the user explicitly expands scope.

Known blocker signature:

```text
zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs:208:12
missing field `pass_stages` in initializer of `CompiledRenderPipeline`
```

## Acceptance Criteria

- `SoundManager` exposes backend capabilities and a backend callback pull API.
- The deterministic `software-null` backend is listed and can pull configured mixer blocks.
- Callback reports include backend id, device id, sequence, requested/rendered frames, sample count, underrun flag, and error text.
- Existing output-device configure/start/stop/render behavior still works for software descriptors.
- Docs and session notes state that the backend seam is done while true CPAL/OS backend remains open.
- Fresh validation evidence or exact external blockers are recorded.
