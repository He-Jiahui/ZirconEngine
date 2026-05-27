---
related_code:
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/service_types/external_sources.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_validation.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/engine/math.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
implementation_files:
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/service_types/external_sources.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_validation.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/engine/math.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
plan_sources:
  - user: 2026-05-25 继续完善插件工作流以及sound插件作为样例完善
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/Sound 插件核心完善计划.md
tests:
  - zircon_plugins/sound/runtime/src/tests.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked -- --nocapture
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml common --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" first_party_sound_provider_preserves_manifest_maturity_and_capability_status -- --nocapture --test-threads=1
doc_type: module-detail
---

# Sound Runtime Plugin

## Purpose

`zircon_plugins/sound/runtime` is the concrete first-party provider for the neutral sound contracts in `zircon_runtime::core::framework::sound`. It owns the Sound module descriptor, runtime component descriptors, plugin options, dynamic-event catalog contribution, output-device integration, and the default software sound manager.

The crate is also the M4 sample for the Bevy-level plugin workflow. `zircon_runtime` describes the profile and availability contract without depending on this crate; `zircon_app` or an export host supplies `SoundRuntimePlugin::plugin_registration()` when the selected profile needs a linked first-party sound provider.

## Metadata Contract

Sound has three metadata sources that must stay aligned:

- `zircon_plugins/sound/plugin.toml` is the static package manifest consumed by Hub/export/catalog tooling.
- `runtime_plugin_descriptor()` is the provider-owned runtime descriptor used to build linked registration reports.
- `RuntimePluginDescriptor::builtin_catalog()` is the runtime-owned catalog entry used by profile availability.

The Sound provider reports `PluginMaturity::Beta` and `runtime.plugin.sound = CapabilityStatus::Partial`. That matches the static TOML and the built-in catalog. This matters because client/editor/dev profiles require Sound as a Bevy-default-style audio capability: a linked provider must not look lower maturity than the catalog entry that made the profile selectable.

## Runtime Contribution

`SoundRuntimePlugin` registers:

- `SoundModule`, `SoundDriver`, and `DefaultSoundManager`.
- `AudioSource`, `AudioListener`, and `AudioVolume` component descriptors.
- Sound plugin options such as backend, sample rate, block size, global volume, spatial scale, HRTF, convolution, ray tracing, timeline integration, and dynamic-event enablement.
- The empty versioned `sound.dynamic_events` event catalog used as a discoverable future integration point.

Runtime audio behavior remains in this crate. The runtime framework layer only owns DTOs, handles, and traits; it does not implement mixing, DSP, output callbacks, or Sound-specific editor behavior.

`src/engine/render.rs` remains the render orchestration surface for playbacks, source voices, track routing, sidechain taps, DSP application, meters, and final master gain. The module is being reduced from its original oversized shape by extracting pure support responsibilities first. `src/engine/math.rs` owns the internal 3D vector helpers used by spatial attenuation, HRTF preview, cone, pan, doppler, and volume calculations.

`src/engine/source_environment.rs` now owns source environment processing after dry source input generation: active listener selection, spatial attenuation, cone and doppler preview gain, ray-traced occlusion lookup, HRTF preview or loaded-profile application, `AudioVolume` influence/crossfade/filter/convolution send behavior, source convolution send, and final source pan. `render.rs` delegates this responsibility with cloned frame-state snapshots, keeping per-block orchestration separate from source-environment math and policy.

`src/service_types.rs` remains the public `DefaultSoundManager` service struct boundary and now only owns child-module wiring, `SoundDriver`, shared manager state fields, construction, and shared config snapshots. `src/service_types/manager_trait.rs` owns the `SoundManager` trait dispatch implementation and forwards each public contract method into the focused service modules. Playback clip asset-manager resolution, test clip injection, clip loading, and playback lifecycle controls now live in `src/service_types/playback.rs`; playback empty checks, playback status snapshots, and finished playback draining now live in `src/service_types/playback_status.rs`; playback settings validation, speed validation, and start/duration range calculation now live in `src/service_types/playback_validation.rs`. Source creation/update/removal and source playback controls now live in `src/service_types/sources.rs`; source empty checks, source status snapshots, range/cursor reporting, and finished source draining now live in `src/service_types/source_status.rs`. External audio source block submission and clearing now live in `src/service_types/external_sources.rs`. Output backend naming/status, output device configuration/start/stop/status/listing, software block rendering, backend capability listing, and backend callback pull behavior now live in `src/service_types/output_device.rs`. Mixer preset discovery/application, full mixer graph import, mixer snapshot, track CRUD, send CRUD, and effect CRUD now live in `src/service_types/mixer_graph.rs`. Dynamic event catalog registration, handler registration, pending-event queueing, dispatch fan-out, executor registration, and execution report assembly now live in `src/service_types/dynamic_events.rs`. Sound parameter storage, automation binding/application, automation curve sampling, and timeline sequence scheduling/advancement now live in `src/service_types/automation_timeline.rs`. Listener and `AudioVolume` registration, static impulse-response lifecycle, HRTF profile lifecycle, ray-tracing convolution status, and ray-traced impulse-response submission/listing/clearing now live in `src/service_types/acoustics.rs`. Global volume/default spatial-scale service configuration and direct software `render_mix` now live in `src/service_types/runtime_settings.rs`. This keeps the service root structural instead of owning asset loading, lifecycle, playback status reporting, playback validation, source status reporting, graph mutation, event-dispatch, timeline-control, acoustics-state, runtime-setting, external-source buffer, or trait-dispatch behavior directly.

## Test Coverage

`src/tests/manifest.rs` keeps static and generated metadata in sync. It checks option keys, runtime module contributions, dependency rows, event catalogs, component descriptors, and verifies that static TOML, runtime descriptor, generated package manifest, and built-in runtime catalog agree on maturity and capability status. The broader runtime test tree covers graph routing, DSP state, spatial/HRTF behavior, ray-traced impulse-response provider input, dynamic events, output-device behavior, presets, source lifecycle, automation, and manifest parity.

`src/tests/playback.rs` now owns the Bevy-inspired source and playback lifecycle regression cases that were previously embedded in the large runtime test aggregate: source speed/mute controls, sink-style source controls, start/duration ranges, cleanup intent, playback presets, invalid initial mix parameters, pause/resume/mute/speed status, seek/range handling, and finished playback reports. Keeping these tests folder-backed makes future playback work independent from mixer graph, DSP, spatial, and manifest coverage.

`src/tests/source_inputs.rs` now owns source-input regression coverage for external audio blocks, invalid or missing external handles, clip/external resampling to mixer rate, and synth-parameter source bindings. This keeps source ingestion coverage separate from playback lifecycle, mixer graph, DSP, and spatial tests.

`src/tests/automation_binding.rs` now owns synth-parameter visibility and automation binding coverage that used to live in the runtime test aggregate: snapshot visibility for bound synth parameters, automation value application to synth, track, and effect targets, and typed failures for invalid target paths or missing targets. `src/tests/automation_curve.rs` owns automation curve sampling, keyframe validation, one-shot timeline sequence advancement, and looping timeline behavior, so binding target resolution can evolve independently from curve sampling and timeline scheduling behavior.

`src/tests/runtime_core.rs` now owns runtime-plugin registration and default manager baseline coverage that used to live in the root test module: runtime module/component/option/event contribution, silent render format defaults, and final global-volume gain validation. `src/tests.rs` now remains a navigation and shared-fixture module instead of owning behavioral assertions.

`src/tests/convolution.rs` now owns static convolution and impulse-response lifecycle coverage that used to live in the root test module: master-track static IR processing, static IR cache invalidation when an impulse response is removed, and ray-tracing convolution status validation. Provider-fed ray-traced IR submission and occlusion cases remain in `ray_tracing.rs`.

`src/tests/common.rs` owns shared Sound test fixtures after the root test module cleanup: mono clip asset construction, default listener construction, effect descriptor construction, and near-equality sample assertions. `src/tests.rs` is now only the folder-backed test module index plus a re-export of those helpers for child modules.

`src/tests/spatial.rs` now owns HRTF profile behavior plus the remaining spatial source, spatial scale, AudioVolume, and pre-spatial source-send coverage that used to live in the runtime test aggregate. Keeping these tests together makes spatial attenuation, occlusion, HRTF, scale overrides, volume crossfade, and pre-spatial send behavior visible as one sound-domain test boundary.

`src/tests/mixer_graph.rs` now owns mixer graph and routing regression coverage that used to live in the runtime test aggregate: custom track routing through effect chains, track removal rerouting active playbacks, parent/send cycle rejection, send CRUD, solo routing, and sidechain pre/post-effect tap behavior. Keeping these tests together makes graph mutation and render routing failures visible without expanding the root test module.

`src/tests/dsp_state.rs` now owns both stateful DSP regression coverage and the deterministic single-block DSP effect checks that used to live in the runtime test aggregate: bypass/wet-dry behavior, delay, pan/phase, limiter, filter, reverb, waveshaper, flanger, phaser, chorus, state continuity, latency snapshots, parameter validation, and sidechain reference validation. Keeping these checks in one module makes DSP failures local to the effect/state boundary instead of mixing them with source, graph, spatial, or automation tests.

After this boundary extraction, focused validation on 2026-05-26 passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: 13 related tests passed, 0 failed, and 84 unrelated tests were filtered out. The run emitted only existing `zircon_runtime` warnings.

Focused validation after the source-input extraction passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` with the same target directory: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out. The run emitted only existing `zircon_runtime` warnings.

Focused validation after the spatial extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 13 spatial-related tests passed, 0 failed, and 84 unrelated tests were filtered out. One earlier cold-target attempt in the same target directory exited during dependency compilation at `unicode-bidi` with rustc exit code `1073807364` and no Sound diagnostics; the warmed retry completed and is the accepted evidence.

Focused validation after the mixer graph extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 8 mixer graph tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after the DSP extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 14 DSP tests passed, 0 failed, and 83 unrelated tests were filtered out.

Focused validation after the automation binding extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 4 automation-binding-related tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after the root runtime-core extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after the convolution extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 5 convolution-related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the common fixture extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml common --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 0 tests matched the `common` filter because it is helper-only, and the crate compiled successfully with 97 tests filtered out.

Focused validation after the render math extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 12 render-related tests passed, 0 failed, and 85 unrelated tests were filtered out.

Focused validation after the source environment extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 13 spatial-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the manager playback/source lifecycle extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source lifecycle related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same service lifecycle extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving external audio source block lifecycle into `src/service_types/external_sources.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving playback settings validation and start/duration range calculation into `src/service_types/playback_validation.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same playback validation extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving source status snapshots and finished-source draining into `src/service_types/source_status.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source lifecycle related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same source-status extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving playback status snapshots and finished-playback draining into `src/service_types/playback_status.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after moving the clip asset helper boundary into playback passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out. Two earlier attempts in the same target directory timed out while compiling dependencies under heavy concurrent workspace validation; the longer warmed retry completed and is the accepted evidence.

Focused validation after moving the `SoundManager` trait dispatch boundary into `src/service_types/manager_trait.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after the manager output-device extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after the manager mixer graph extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 mixer graph tests passed, 0 failed, and 89 unrelated tests were filtered out. Two earlier attempts in the same target directory timed out while compiling `zircon_runtime`; process inspection showed the Sound cargo/rustc jobs were still compiling rather than running a stuck Sound test binary, and the warmed retry completed successfully.

Focused validation after the manager dynamic-events extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 10 dynamic-event tests passed, 0 failed, and 87 unrelated tests were filtered out.

Focused validation after the manager automation/timeline extraction was attempted on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`. The command stopped while compiling `zircon_runtime` before Sound tests executed because active UI accessibility work still exposes `append_binding_report_diagnostic` as private while re-exporting/importing it across sibling action modules. The intended follow-up focused commands for this slice are the same `automation_binding` command and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` once that external compile blocker is cleared.

Focused validation after the manager acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 spatial/HRTF/volume related tests passed, 0 failed, and 84 unrelated tests were filtered out. The first cold attempt timed out during dependency compilation while other workspace validation jobs were active; the warmed retry completed successfully.

Focused validation after the same acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 convolution and impulse-response related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the same acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 ray-tracing impulse-response related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the manager runtime-settings extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Fresh validation on 2026-05-26 passed the full runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: 97 runtime tests passed, 0 failed, and doctests had no failures. The command was rerun after the playback and source-input test-boundary extractions with the same 97 passed / 0 failed result, rerun again on 2026-05-27 after the spatial extraction with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`, rerun after the mixer graph extraction, rerun after the DSP extraction, rerun after the automation binding extraction, rerun after the runtime-core/convolution root extraction, rerun after the common fixture extraction, rerun after the render math extraction, and rerun after the source environment extraction with the same target directory: 97 runtime tests passed, 0 failed, and doctests had no failures. The full runtime command was then rerun after the manager playback/source lifecycle extraction, manager output-device extraction, manager mixer graph extraction, manager acoustics extraction, manager runtime-settings extraction, manager playback asset-helper extraction, manager trait-dispatch extraction, manager external-source extraction, manager playback-validation extraction, manager source-status extraction, and manager playback-status extraction with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 97 runtime tests passed, 0 failed, and doctests had no failures. An intermediate full runtime rerun after the manager dynamic-events extraction had stopped in unrelated active UI accessibility work because `append_binding_report_diagnostic` was private to the `action::result` child module while being re-exported/imported across sibling action modules; the later full Sound runtime rerun now covers the Sound manager service-root split. The earlier app/provider command `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" first_party_sound_provider_preserves_manifest_maturity_and_capability_status -- --nocapture --test-threads=1` passed for the linked first-party provider path and proves Sound maturity, capability status, module, option, and dynamic-event catalog metadata stay preserved through app bootstrap.
