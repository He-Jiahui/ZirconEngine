# Sound CPAL Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add sound output device enumeration, picker-ready descriptors, and latency/status diagnostics on top of the existing CPAL adapter.

**Architecture:** `zircon_runtime::core::framework::sound` owns neutral DTOs and the `SoundManager` method. `zircon_plugins/sound/runtime/src/output/` owns software and CPAL enumeration, selected-device resolution, CPAL session queue diagnostics, and status merging. `DefaultSoundManager` remains a forwarding boundary.

**Tech Stack:** Rust 2021, serde DTOs, existing sound runtime plugin, optional `cpal-backend` feature, current output-device tests.

---

## Source Map

- Modify `zircon_runtime/src/core/framework/sound/output.rs`: add `SoundOutputDeviceInfo` and `SoundOutputLatencyStatus`; extend `SoundOutputDeviceStatus`.
- Modify `zircon_runtime/src/core/framework/sound/manager.rs`: add `available_output_devices()` to `SoundManager`.
- Modify `zircon_runtime/src/core/framework/sound/mod.rs`: re-export new output DTOs.
- Modify `zircon_plugins/sound/runtime/src/output/software.rs`: add deterministic software picker row construction.
- Modify `zircon_plugins/sound/runtime/src/output/cpal.rs`: add feature-gated CPAL enumeration, picker IDs, selected-device lookup, and queue diagnostics.
- Modify `zircon_plugins/sound/runtime/src/output/mod.rs`: expose device enumeration, status latency diagnostics, and CPAL queue-status merging.
- Modify `zircon_plugins/sound/runtime/src/service_types.rs`: implement `available_output_devices()` by forwarding to the output facade with current config.
- Modify `zircon_plugins/sound/runtime/src/tests/output_device.rs`: add deterministic enumeration/status tests and feature-gated CPAL assertions.
- Update `docs/engine-architecture/runtime-sound-extension.md`: record the picker contract, latency diagnostics, and validation.
- Update `.codex/sessions/20260523-0748-sound-sequential-milestones.md`: record this active slice and evidence.

## Milestone 1: Neutral Picker And Status Contracts

Goal: Add neutral contracts without introducing CPAL types into shared framework code.

Implementation slices:

- [x] Add `SoundOutputDeviceInfo { descriptor, is_default, available, diagnostic }` in `zircon_runtime/src/core/framework/sound/output.rs`.
- [x] Add `SoundOutputLatencyStatus { requested_latency_blocks, estimated_latency_frames, estimated_latency_seconds, queued_samples, capacity_samples }` in the same file.
- [x] Add `latency: SoundOutputLatencyStatus` and `diagnostics: Vec<String>` to `SoundOutputDeviceStatus`.
- [x] Add `fn available_output_devices(&self) -> Result<Vec<SoundOutputDeviceInfo>, SoundError>;` to `SoundManager`.
- [x] Re-export `SoundOutputDeviceInfo` and `SoundOutputLatencyStatus` from `sound/mod.rs`.

Testing stage:

- [ ] Defer compile/test execution until Milestone 4 per repository milestone-first cadence.

Exit evidence:

- Shared sound DTOs remain serde-friendly and CPAL-free.
- Only `DefaultSoundManager` needs a trait implementation update.

## Milestone 2: Software And CPAL Enumeration

Goal: Return picker rows for deterministic software output and feature-gated CPAL devices.

Implementation slices:

- [x] Add software device enumeration returning a `software-null` row with current config sample rate, channels, block size, and latency blocks.
- [x] Add `output::available_output_devices(&SoundConfig)` to combine software and CPAL rows.
- [x] Implement `DefaultSoundManager::available_output_devices()` by forwarding to the output facade.
- [x] Under `cpal-backend`, enumerate CPAL default and indexed output devices with neutral IDs.
- [x] Keep CPAL enumeration best-effort: software rows are still returned when host device enumeration fails.
- [x] Without `cpal-backend`, return no CPAL device rows.

Testing stage:

- [ ] Defer compile/test execution until Milestone 4.

Exit evidence:

- Picker rows exist without requiring OS audio.
- CPAL feature-disabled behavior remains deterministic.

## Milestone 3: Selected Device And Latency Diagnostics

Goal: Make picker descriptors usable and expose queue/latency status.

Implementation slices:

- [x] Add CPAL selected-device resolution for `sound.output.cpal.default` and `sound.output.cpal.device.<index>`.
- [x] Preserve manual CPAL descriptor configuration by falling back to the platform default when the ID is not a picker ID.
- [x] Add latency-status calculation in the output facade.
- [x] Extend CPAL session status with ring-buffer `queued_samples` and `capacity_samples`.
- [x] Merge CPAL queue diagnostics into `SoundOutputDeviceStatus`.
- [x] Add unavailable/backend error details to status diagnostics.

Testing stage:

- [ ] Defer compile/test execution until Milestone 4.

Exit evidence:

- Status reports estimated latency for software and CPAL descriptors.
- CPAL callback and producer counters remain backend-local and neutralized before crossing the manager contract.

## Milestone 4: Tests, Docs, And Acceptance Validation

Goal: Prove the CPAL polish slice and record evidence.

Implementation slices:

- [x] Add output-device tests for software enumeration, configure-from-picker descriptor, status latency, and diagnostics.
- [x] Add feature-disabled CPAL row assertions.
- [x] Add feature-enabled CPAL enumeration/helper tests that do not require a physical device.
- [x] Update runtime sound architecture docs with implementation files, plan/spec sources, and validation evidence.
- [x] Update active session note and this plan's checkboxes/evidence.

Testing stage:

- [x] Run formatting for the sound runtime package.

```powershell
cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime
```

- [x] Run rustfmt for touched shared sound DTO files.

```powershell
rustfmt --check zircon_runtime\src\core\framework\sound\output.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs
```

- [x] Run non-feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never
```

- [x] Run CPAL-feature output tests.

```powershell
cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never
```

- [x] Run whitespace check.

```powershell
git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound\runtime" "docs\engine-architecture\runtime-sound-extension.md" "docs\superpowers\specs\2026-05-23-sound-cpal-polish-design.md" "docs\superpowers\plans\2026-05-23-sound-cpal-polish.md" ".codex\sessions\20260523-0748-sound-sequential-milestones.md"
```

Exit evidence:

- Formatting passes.
- Output-device tests pass with and without `cpal-backend`, or any physical-device absence is accepted only through structured CPAL unavailable assertions.
- Docs and active session note record implementation files, validation commands, and remaining gaps.

Final evidence:

- `cargo fmt --check --manifest-path "zircon_plugins\Cargo.toml" -p zircon_plugin_sound_runtime` passed after CPAL polish implementation.
- `rustfmt --check zircon_runtime\src\core\framework\sound\output.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs` passed after CPAL polish implementation.
- `cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --locked --offline --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never` passed on the final rerun: 8 passed, 0 failed, 81 filtered out, with existing `zircon_runtime` warnings.
- `cargo test --manifest-path "zircon_plugins\sound\runtime\Cargo.toml" output_device --features cpal-backend --locked --jobs 1 --target-dir "E:\cargo-targets\zircon-sound-cpal-polish" --message-format short --color never` passed on the final rerun: 12 passed, 0 failed, 81 filtered out, with existing `zircon_runtime` warnings.
- `git diff --check -- "zircon_runtime\src\core\framework\sound" "zircon_plugins\sound\runtime" "docs\engine-architecture\runtime-sound-extension.md" "docs\superpowers\specs\2026-05-23-sound-cpal-polish-design.md" "docs\superpowers\plans\2026-05-23-sound-cpal-polish.md" ".codex\sessions\20260523-0748-sound-sequential-milestones.md"` reported no whitespace errors; LF-to-CRLF warnings only.

## Acceptance Criteria

- `available_output_devices()` exists on `SoundManager` and returns neutral picker rows.
- `software-null` is always listed and configurable from its picker descriptor.
- CPAL device rows are available only with `cpal-backend` and never leak CPAL types.
- CPAL picker descriptors select default or enumerated output devices when possible.
- `SoundOutputDeviceStatus` reports latency estimates, queue depth when available, and diagnostics.
- Existing CPAL adapter behavior remains recoverable and deterministic in no-device or no-feature environments.
