# Sound HRTF Profile Loading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic runtime HRTF profile loading and kernel application for sound listeners.

**Architecture:** `zircon_runtime::core::framework::sound` defines neutral HRTF profile DTOs and `SoundManager` APIs. `zircon_plugins/sound/runtime` validates/stores profiles and applies loaded left/right FIR kernels during spatial source rendering, falling back to the existing preview when profiles are missing.

**Tech Stack:** Rust, Cargo, serde DTOs, existing sound runtime software renderer and spatial tests.

---

## Source Map

- Modify `zircon_runtime/src/core/framework/sound/acoustics.rs`: add `SoundHrtfProfileDescriptor`.
- Modify `zircon_runtime/src/core/framework/sound/error.rs`: add `UnknownHrtfProfile`.
- Modify `zircon_runtime/src/core/framework/sound/manager.rs`: add HRTF profile load/remove/list APIs.
- Modify `zircon_runtime/src/core/framework/sound/mod.rs`: export HRTF profile DTO.
- Modify `zircon_plugins/sound/runtime/src/engine/state.rs`: store loaded HRTF profiles.
- Modify `zircon_plugins/sound/runtime/src/descriptor_validation.rs`: validate HRTF profile descriptors.
- Modify `zircon_plugins/sound/runtime/src/service_types.rs`: implement manager APIs.
- Modify `zircon_plugins/sound/runtime/src/engine/render.rs`: apply loaded HRTF kernels during source environment rendering.
- Modify `zircon_plugins/sound/runtime/src/tests/spatial.rs`: add HRTF profile tests.
- Update `docs/engine-architecture/runtime-sound-extension.md` and `.codex/sessions/20260503-0228-sound-mixer-graph-continuation.md`.

## Milestone 1: Neutral Contract

- [ ] Add `SoundHrtfProfileDescriptor` in `acoustics.rs`.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundHrtfProfileDescriptor {
    pub profile_id: String,
    pub display_name: String,
    pub sample_rate_hz: u32,
    pub left_kernel: Vec<f32>,
    pub right_kernel: Vec<f32>,
    pub notes: Vec<String>,
}
```

- [ ] Add `UnknownHrtfProfile { profile_id: String }` to `SoundError`.
- [ ] Add manager APIs:

```rust
fn load_hrtf_profile(&self, profile: SoundHrtfProfileDescriptor) -> Result<(), SoundError>;
fn remove_hrtf_profile(&self, profile_id: &str) -> Result<(), SoundError>;
fn hrtf_profiles(&self) -> Result<Vec<SoundHrtfProfileDescriptor>, SoundError>;
```

- [ ] Re-export `SoundHrtfProfileDescriptor` from `mod.rs`.

## Milestone 2: Runtime Storage And Validation

- [ ] Add `hrtf_profiles: HashMap<String, SoundHrtfProfileDescriptor>` to `SoundEngineState` and initialize it in `new`.
- [ ] Add `validate_hrtf_profile_descriptor(profile: &SoundHrtfProfileDescriptor) -> Result<(), SoundError>`.

Validation rules:

- `profile_id` and `display_name` must be non-empty after trim.
- `sample_rate_hz` must be non-zero.
- `left_kernel` and `right_kernel` must be non-empty.
- all samples must be finite.
- at least one sample across both kernels must be non-zero.

- [ ] Implement manager methods in `service_types.rs`: validate and insert on load, remove or return `UnknownHrtfProfile`, list profiles sorted by `profile_id` for deterministic tests.

## Milestone 3: Kernel Application

- [ ] Pass `&self.hrtf_profiles` into `apply_source_environment` from `render.rs`.
- [ ] Update `apply_source_environment` signature to accept HRTF profiles.
- [ ] When the active listener has `hrtf_profile` and a matching loaded profile exists, apply deterministic left/right FIR kernels to stereo output.
- [ ] Keep existing ear-offset preview fallback when no loaded profile matches.
- [ ] Keep mono/multi-channel behavior safe: only process first two channels when `channels >= 2`; leave mono without HRTF kernel processing.

## Milestone 4: Tests And Docs

- [ ] Add spatial tests for profile load/list/remove validation and deterministic kernel rendering.
- [ ] Update docs and session note to describe HRTF profile loading, fallback behavior, validation, and remaining production HRTF database/interpolation gap.

## Milestone 5: Testing Stage

- [ ] Run formatting:

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
rustfmt --check "zircon_runtime\src\core\framework\sound\acoustics.rs" "zircon_runtime\src\core\framework\sound\error.rs" "zircon_runtime\src\core\framework\sound\manager.rs" "zircon_runtime\src\core\framework\sound\mod.rs"
```

- [ ] Run focused spatial/HRTF tests if workspace compiles past unrelated dirty lanes:

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" hrtf --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-hrtf-profile" --message-format short --color never
```

- [ ] Run whitespace check:

```powershell
git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound" "docs\engine-architecture\runtime-sound-extension.md" ".codex\sessions\20260503-0228-sound-mixer-graph-continuation.md" "docs\superpowers\specs\2026-05-04-sound-hrtf-profile-loading-design.md" "docs\superpowers\plans\2026-05-04-sound-hrtf-profile-loading.md"
```

- [ ] If Cargo fails before compiling sound on unrelated dirty graphics code, record the exact external diagnostic and do not edit graphics files from this sound slice.

## Acceptance Criteria

- HRTF profiles are neutral DTOs and managed through `SoundManager`.
- Runtime validates, stores, lists, and removes profiles deterministically.
- Loaded profiles affect stereo spatial rendering.
- Missing profiles still fall back to the existing preview path.
- Docs state that production HRTF databases/interpolation remain open.
