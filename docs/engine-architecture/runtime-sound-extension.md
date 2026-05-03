---
related_code:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/events.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/preset.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/automation.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation.rs
  - zircon_plugins/sound/runtime/src/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration.rs
  - zircon_plugins/sound/runtime/src/output.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/state.rs
  - zircon_plugins/sound/runtime/src/engine/validation.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/presets.rs
  - zircon_plugins/sound/runtime/src/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/runtime/src/timeline.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/mixer_console.ui.toml
  - zircon_plugins/sound/editor/acoustic_debug.ui.toml
  - zircon_plugins/sound/editor/audio_source.drawer.ui.toml
  - zircon_plugins/sound/editor/audio_listener.drawer.ui.toml
  - zircon_plugins/sound/editor/audio_volume.drawer.ui.toml
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/presets.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
implementation_files:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/events.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/preset.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/automation.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation.rs
  - zircon_plugins/sound/runtime/src/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration.rs
  - zircon_plugins/sound/runtime/src/output.rs
  - zircon_plugins/sound/runtime/src/engine/dsp.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/state.rs
  - zircon_plugins/sound/runtime/src/engine/validation.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/presets.rs
  - zircon_plugins/sound/runtime/src/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/runtime/src/timeline.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/authoring_bindings.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_plugins/sound/editor/mixer_console.ui.toml
  - zircon_plugins/sound/editor/acoustic_debug.ui.toml
  - zircon_plugins/sound/editor/audio_source.drawer.ui.toml
  - zircon_plugins/sound/editor/audio_listener.drawer.ui.toml
  - zircon_plugins/sound/editor/audio_volume.drawer.ui.toml
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/presets.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
plan_sources:
  - user: 2026-05-02 sound plugin mixer/spatial/convolution/timeline core implementation request
  - docs/superpowers/specs/2026-05-04-sound-hrtf-profile-loading-design.md
  - docs/superpowers/plans/2026-05-04-sound-hrtf-profile-loading.md
  - docs/superpowers/specs/2026-05-03-sound-backend-seam-design.md
  - docs/superpowers/plans/2026-05-03-sound-backend-seam.md
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after Timeline sequence scheduling slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\ids.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after Timeline sequence scheduling slice)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml automation_curve --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-timeline-sequence --message-format short --color never (passed after Timeline sequence scheduling slice)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after dynamic event handler slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after dynamic event handler slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after dynamic event handler slice)
  - cargo check -p zircon_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-sound-framework --message-format short --color never (passed after dynamic event handler slice, existing warnings)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml dynamic_events --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events --message-format short --color never (passed after dynamic event handler slice)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events --message-format short --color never (passed after dynamic event handler slice)
  - cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-editor-lib --message-format short --color never (passed after dynamic event handler slice, existing warnings)
  - cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after linear resampling slice)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after linear resampling slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after linear resampling slice)
  - git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md (passed before this evidence text update)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linear-resample --message-format short --color never (passed after linear resampling slice)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml clip_and_external_inputs_resample_to_mixer_rate --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linear-resample --message-format short --color never (first attempt timed out after 304 seconds; retry was blocked before compile by shared zircon_plugins\Cargo.lock needing update after unrelated Cargo.toml/Cargo.lock changes)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after latest slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\components.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after latest slice)
  - cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --offline --no-deps --format-version 1 (passed)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state --message-format short --color never (latest retry timed out after 4 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device --message-format short --color never (timed out after 4 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-hrtf-preview --message-format short --color never (timed out after 4 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state --message-format short --color never (blocked before compile by zircon_plugins\Cargo.lock needing update)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never (timed out after 5 minutes under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never (timed out under concurrent Cargo load)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never (earlier blocked before sound tests by zircon_runtime graphics record_submission error E0599)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct; 16 runtime tests passed)
  - cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after mixer preset slice)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor (passed after mixer preset editor binding slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs zircon_runtime\src\core\framework\sound\preset.rs (passed after mixer preset slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after mixer preset slice)
  - cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after mixer preset editor binding slice)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never (passed after mixer preset slice)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never (timed out after 7 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-editor --message-format short --color never (timed out after 6 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after automation curve slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after automation curve slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after automation curve slice)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-curve --message-format short --color never (timed out after 7 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never (blocked before sound crate by missing zircon_runtime modules animation and physics in current worktree)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after automation-curve test-stability edit)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor (passed after dynamic-event editor placeholder edit)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after dynamic-event editor placeholder edit)
  - cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after dynamic-event editor placeholder edit)
  - git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md (passed after dynamic-event editor placeholder edit)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never (blocked before compile by shared zircon_plugins\Cargo.lock needing update)
  - cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-editor --message-format short --color never (blocked before compile by shared zircon_plugins\Cargo.lock needing update)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after dynamic event runtime queue and ray-traced IR submission slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\acoustics.rs zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\graph.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after dynamic event runtime queue and ray-traced IR submission slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after dynamic event runtime queue and ray-traced IR submission slice)
  - git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md (passed after dynamic event runtime queue and ray-traced IR submission slice)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-interface-gaps --message-format short --color never (timed out after 3 minutes without Rust diagnostics under concurrent Cargo load)
  - cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after graph configure import slice)
  - rustfmt zircon_runtime\src\core\framework\sound\acoustics.rs zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\graph.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after graph configure import slice)
  - cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime (passed after graph configure import slice)
  - rustfmt --check zircon_runtime\src\core\framework\sound\acoustics.rs zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\graph.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs (passed after graph configure import slice)
  - cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1 (passed after graph configure import slice)
  - git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md (passed after graph configure import slice before this evidence update)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-graph-config --message-format short --color never (passed after graph configure import slice)
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml graph_config --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-graph-config --message-format short --color never (first attempt timed out after 424 seconds; retry passed, 2 passed / 0 failed)
  - zircon_plugins/sound/runtime/src/tests.rs
  - zircon_plugins/sound/editor/src/lib.rs
doc_type: module-detail
---

# Runtime Sound Extension

## Purpose

`sound` is now documented as the first core slice of a plugin-owned audio engine, not only as the earlier WAV clip playback loop. The stable ownership remains the same:

- `zircon_runtime::core::framework::sound` owns neutral DTOs, IDs, errors, graph descriptions, component descriptors, automation contracts, plugin options, and the `SoundManager` trait.
- `zircon_plugins/sound/runtime` owns concrete state, clip/source playback, mixer graph validation, CPU DSP, meters, static impulse responses, and the default software render path.
- `zircon_plugins/sound/editor` owns editor contribution descriptors for mixer authoring, acoustic debug, and sound component drawers.

The design follows the repository boundary rule: heavy audio behavior stays in the independent sound plugin, while `zircon_runtime` exposes only neutral contracts and host-facing registry surfaces.

## Related Files

The sound contract is split by responsibility:

- `ids.rs` defines stable IDs for clips, playbacks, tracks, nodes, effects, sources, listeners, volumes, parameters, automation bindings, impulse responses, and external source handles.
- `graph.rs` defines `SoundMixerGraph`, tracks, sends, controls, meters, and snapshots.
- `acoustics.rs` defines ray-tracing convolution status, neutral HRTF profile descriptors, and the neutral ray-traced impulse-response submission descriptor used by optional geometry/ray-query providers.
- `effects.rs` defines the first deterministic DSP descriptor set: gain, filter, reverb, convolution reverb, compressor with sidechain, wave shaper, flanger, phaser, chorus, delay, pan/stereo, and limiter.
- `components.rs` defines neutral `AudioSource`, `AudioListener`, `AudioVolume`, and external source block contracts plus plugin dynamic component type IDs. Source/listener descriptors now carry position, forward vector, and velocity data so the plugin runtime can spatialize without coupling directly to a concrete scene transform component.
- Framework `automation.rs` defines Timeline-style sound automation bindings, serializable scalar automation curves/keyframes/interpolation modes, and neutral `SoundTimelineSequence` scheduling DTOs for binding curves to sound automation targets over time.
- Framework `events.rs` defines the versioned dynamic event catalog, descriptors, handler descriptors, opaque runtime invocation payloads, and deterministic delivery DTOs. Sound owns handler registration and queue fan-out mechanics while event meaning and side effects remain in the consuming plugins.
- `options.rs` defines runtime/editor-facing sound plugin options and quality gates.
- Framework `output.rs` defines the output-device and backend callback contract: descriptors, backend capabilities, start/stop state, callback reports, and render statistics. It is intentionally backend-neutral so the current software/null callback path and a later CPAL/platform driver can share the same manager surface.
- Framework `preset.rs` defines serializable mixer preset descriptors. The runtime owns the concrete built-in preset catalog in `zircon_plugins/sound/runtime/src/presets.rs`.

The plugin implementation is split under `zircon_plugins/sound/runtime/src/engine/` so `service_types.rs` remains a manager boundary rather than an audio engine dumping ground. Runtime automation value mapping lives in `zircon_plugins/sound/runtime/src/automation.rs`, sound-owned Timeline sequence scheduling lives in `zircon_plugins/sound/runtime/src/timeline.rs`, shared descriptor validation lives in `zircon_plugins/sound/runtime/src/descriptor_validation.rs`, full graph import lives in `zircon_plugins/sound/runtime/src/mixer_configuration.rs`, dynamic event registry/queue validation lives in `zircon_plugins/sound/runtime/src/dynamic_events.rs`, and ray-traced impulse-response provider submission lives in `zircon_plugins/sound/runtime/src/ray_tracing.rs`.

## Behavior Model

The default manager keeps a `SoundEngineState` with clip cache, external source blocks, active clip playbacks, explicit source voices, listener/volume descriptors, automation bindings, parameter values, static impulse responses, mixer graph, DSP runtime state, meters, latency, and ray-tracing status. Clip playbacks and explicit sources keep fractional source cursors so clip and external input blocks can be resampled deterministically into the mixer sample rate.

The `SoundManager` surface now exposes graph mutation at the track/send/effect level instead of requiring callers to replace the whole graph for common authoring operations. Track sends are keyed by target track ID, so adding a send to the same target updates the existing edge; removing a missing edge returns a typed `UnknownSend` error. Synth/automation parameters also support readback through `parameter_value(...)`, with unknown keys returning `UnknownParameter`.

`configure_mixer(...)` is now a full graph import path, not only a track-list replacement. It validates the incoming `SoundMixerGraph`, imports graph-owned `SoundSourceDescriptor` entries into the live source registry with fresh cursors, assigns IDs for authoring sources that omit one, imports graph automation bindings into the runtime binding table, synchronizes the dynamic event catalog, resets stale DSP/track state, and refreshes meters for the new track set. Duplicate configured source IDs or automation binding IDs return typed `InvalidParameter` errors before state is replaced.

Mixer preset application is now part of the same manager surface. `available_mixer_presets(...)` returns built-in serializable graphs for the default master-only graph, a Music/SFX/Ambience bus layout, and a Spatial Room layout with a room reverb return. `apply_mixer_preset(locator)` validates the selected graph, replaces the active mixer, clears DSP/track runtime state and meters that no longer match the graph, reroutes any active playbacks or sources pointing at removed tracks back to `master`, and removes stale source sends. Unknown preset locators return `InvalidLocator`.

External component and plugin audio is now represented by `ExternalAudioSourceHandle` plus `SoundExternalSourceBlock`. Other plugin systems can submit a finite interleaved audio block through `submit_external_source_block(...)`; any `AudioSource` using `SoundSourceInput::External(handle)` can render that block into its output track, with the source voice owning playback cursor, loop behavior, and sample-rate conversion into the current mixer rate. `clear_external_source(...)` removes the provider block and returns `UnknownExternalSource` for missing handles.

`AudioSource` parameter bindings are active at render time. Each `SoundSourceParameterBinding` maps a source parameter such as `gain`, `playing`, transform coordinates, or spatial controls to a stable synth parameter ID. The source descriptor stored in the graph is left unchanged; each block resolves a temporary descriptor from the current synth parameter table, so modulation can follow synthesizer output without forcing Timeline or synthesizer crates to own sound internals.

Timeline integration now has runtime value, curve, and sequence-entry points. `apply_automation_value(binding, value)` resolves a registered `SoundAutomationBinding`, checks finite input, and applies the scalar to the target track, effect, source, listener, volume, or synthesizer parameter. `apply_automation_curve_sample(binding, curve, time_seconds)` accepts a serializable `SoundAutomationCurve`, validates finite strictly-increasing keyframes, samples step/linear/smooth-step interpolation with endpoint clamping, and then applies the sampled scalar through the same typed binding path. `schedule_timeline_sequence(sequence)` registers or replaces a sound-owned sequence of automation tracks, `advance_timeline_sequences(delta_seconds)` advances all scheduled sequences, samples every bound curve, applies the values through the existing automation target path, returns per-binding sample reports, and removes completed non-looping sequences. Looping sequences wrap by duration. Timeline still owns editor sequence authoring and global playback orchestration, while sound owns deterministic sequence sampling, target resolution, validation, and local scheduling state.

Dynamic events are now an interface-level runtime queue with deterministic multi-plugin fan-out. `dynamic_event_catalog(...)`, `register_dynamic_event(...)`, and `unregister_dynamic_event(...)` manage the sound-owned versioned catalog; unregistering an event also removes its registered handlers and pending invocations. `register_dynamic_event_handler(...)` stores a plugin-owned handler for a known event, keyed by `(plugin_id, handler_id)` and ordered by `priority` for dispatch. `submit_dynamic_event(...)` validates event IDs, finite event time, and payload schema before appending an opaque invocation. `dispatch_dynamic_events(...)` drains pending invocations and returns one `SoundDynamicEventDelivery` per matching handler, sorted by higher priority first and then stable plugin/handler IDs; `drain_dynamic_events(...)` remains available as a raw queue escape hatch for hosts that want to perform their own dispatch.

Impulse-response cache lifecycle is explicit in the manager surface. `set_impulse_response(...)` installs or replaces a static IR, `remove_impulse_response(...)` invalidates it, and missing removals return `UnknownImpulseResponse`. The ray-traced convolution feature remains gated, but `set_ray_tracing_convolution_status(...)` lets the feature/capability layer publish `WaitingForGeometryProvider`, `StaticImpulseResponse`, or validated `RayTraced` status into mixer snapshots. The optional ray-query path can now call `submit_ray_traced_impulse_response(...)` with a `SoundRayTracedImpulseResponseDescriptor`; the runtime validates source/listener/volume references, format, rays, and finite IR samples, installs the IR into the same convolution cache used by static reverb, and updates `RayTraced { cached_cells, rays_per_update }`. `clear_ray_traced_impulse_response(...)` removes the provider-owned IR and returns the status to `WaitingForGeometryProvider` when the ray cache empties.

Output device lifecycle is now explicit in the manager surface. `available_output_backends(...)` reports the deterministic `software-null` backend capability used by tests and editor preview. `configure_output_device(...)` validates the backend ID, display name, sample rate, channel count, block size, and latency blocks, then updates the runtime config and mixer snapshot format. Reconfiguration stops the software output device and clears DSP runtime state because sample-rate/channel changes invalidate delay lines and effect history. `start_output_device(...)`, `stop_output_device(...)`, `output_device_status(...)`, `render_output_device_block(...)`, and `pull_output_backend_callback(...)` form the backend seam: a future platform callback can request exactly the configured block size while the sound runtime records callback sequence, rendered blocks, rendered frames, underrun count, and the last render error. This slice does not open an OS audio device; CPAL/platform adapters will bind to the callback contract later.

Spatial rendering now has deterministic HRTF profile loading plus the earlier preview fallback. `load_hrtf_profile(...)`, `remove_hrtf_profile(...)`, and `hrtf_profiles(...)` manage validated neutral profile descriptors with left/right FIR kernels. When an active `AudioListener` references a loaded `hrtf_profile`, the runtime applies the left/right kernels to stereo source audio before the usual source gain/pan step. If the listener references a missing profile, the runtime falls back to the deterministic ear-offset preview path, using listener ear offsets to estimate left/right ear distance, interaural gain, and capped interaural delay. Production SOFA/CIPIC database parsing, interpolation, and optimized binaural convolution remain future work.

DSP runtime state is keyed by `(SoundTrackId, SoundEffectId)` so effect tails belong to the track/effect instance rather than to the descriptor type. Delay, algorithmic reverb, convolution reverb, modulated delay history, modulation phase, phaser phase, and compressor release envelopes now keep deterministic state across `render_mix` calls. Track control delay has its own per-track state, and stale track/effect state is pruned when the mixer graph changes. Mixer snapshots also report a conservative latency frame count derived from enabled delay, reverb, convolution, flanger, chorus, and track delay controls.

The render order is a deterministic topological walk over parent routes, track sends, and post-effect sidechain dependencies. This keeps child tracks before parents, send sources before send targets, and post-effect sidechain key tracks before the compressors that read them.

Rendering follows a fixed block pipeline:

1. Validate the graph has a `master` track and no parent cycles.
2. Mix `play_clip` convenience playbacks into their requested output track, resampling clip frames into the configured mixer rate.
3. Mix explicit `SoundSourceDescriptor` voices into dry source buffers, including clip, external block, synth parameter, and silence inputs. Clip and external inputs use the same deterministic linear interpolation source cursor policy.
4. Apply the active listener policy, distance attenuation, stereo pan, source cone, deterministic occlusion fallback, Doppler preview gain, source convolution send, and strongest `AudioVolume` influence to each source buffer.
5. Route post-spatial source buffers into their output tracks, while `SoundSourceSend::pre_spatial` can tap the dry buffer for FMOD-style pre-spatial sends.
6. Process non-master tracks in topological order, preserving both pre-effect and post-effect sidechain tap buffers.
7. Run enabled effect chains, then track controls: gain, pan, L/R trims, delay, phase inversion, mute, solo direct-input gating, and bypass.
8. Route processed tracks into parent tracks and sends.
9. Process `master`, update meters, apply master gain, and clamp final samples.

`play_clip` is retained only as a convenience path into this graph. It is not a separate compatibility mixer.

## Design and Rationale

The contract deliberately models FMOD/Unreal/Godot/Fyrox-style audio structure:

- `master` is mandatory and cannot be removed.
- Custom tracks can route to `master` or another parent and can send into other tracks.
- Explicit send CRUD preserves the graph validator as the single source of truth for missing targets, self-sends, parent cycles, send cycles, and post-effect sidechain dependency cycles.
- Solo tracks mute non-solo direct playbacks/sources while still allowing solo-track audio to route through parent tracks to `master`.
- Effect descriptors are data-only so future native/VM plugin paths can serialize them.
- Sidechain compression references another track by ID instead of holding concrete Rust references. `pre_effects` reads the key track input buffer; post-effect taps require the key track to render earlier and read its processed control-applied buffer.
- Stateful DSP lives beside the mixer graph state, not in the neutral DTO layer. The neutral graph remains serializable while the plugin runtime owns delay lines, history buffers, modulation phase, and compressor envelopes that should not be serialized as authoring data.
- Automation writes are binding-driven instead of raw string writes into the graph. Track/effect/source/listener/volume targets expose a first scalar parameter set, unknown bindings return `UnknownAutomationBinding`, unsupported parameter names fail with `InvalidParameter` before mutating state, and curve samples reuse the same mutation path after validating curve data.
- Static convolution reverb and ray-traced provider submissions use the same `SoundImpulseResponseId` cache. Sound owns validation, cache insertion, status snapshots, and invalidation; scene geometry traversal and ray queries remain outside the core sound runtime and will publish through the new provider-facing descriptor.
- Output devices are modeled as backend-neutral DTOs instead of binding the sound plugin to a concrete OS audio library in this slice. The `software-null` backend is a deterministic adapter seam: it pulls from the same mixer path a real callback will use, records callback reports, and stays headless for CI.
- External and synthesizer sound sources are neutral handles/parameter IDs, so other plugins can feed audio without linking to sound internals. External sources deliberately submit block data through the manager contract; the sound runtime owns only resampling cursors and routing, not the upstream component. Source parameter bindings reuse the same stable parameter table so synth-driven modulation is local to sound but does not couple to a concrete synthesizer plugin.
- Active listeners are selected deterministically by requested mixer target, then `master`, then stable listener ID. This keeps multi-listener editor previews predictable while leaving room for richer runtime routing later.
- HRTF profile loading is intentionally neutral and deterministic: the runtime accepts explicit profile descriptors and FIR kernels rather than embedding a third-party database parser. It gives authored listener profile IDs real kernel-driven behavior now, while production profile database loading, interpolation, and optimized binaural convolution remain advanced backend concerns.
- `AudioVolume` influence is deterministic in the first runtime slice: the highest-priority volume with non-zero influence wins, and its crossfade blends exterior/interior gain, low-pass, and static convolution send weight.

The first CPU DSP implementations are deterministic block effects for tests and editor preview. Delay, algorithmic reverb, convolution, modulated delay, phaser LFO phase, compressor release, and track delay now preserve history across render calls, but the DSP path is still a first-stage software renderer rather than a final low-latency audio-device backend.

## Editor and Plugin Metadata

The sound plugin now contributes:

- Dynamic component descriptors for `sound.Component.AudioSource`, `sound.Component.AudioListener`, and `sound.Component.AudioVolume`.
- Plugin options for backend, sample rate, channels, block size, max voices/tracks, HRTF enablement/profile, convolution enablement/budget, ray-tracing quality, default mixer preset, Timeline automation, and dynamic event enablement.
- Manifest dependencies on `asset` and `scene`, plus optional `ray_query` and `timeline_sequence`.
- A versioned `sound.dynamic_events` event catalog plus manager-level event, handler, submit, raw-drain, and dispatch APIs. The runtime queue carries opaque payload bytes and schema IDs only; dispatch returns deterministic handler deliveries but does not execute plugin code.
- Editor views and UI template documents for Sound Mixer and Acoustic Debug. The mixer template exposes stable control slots for preset picker, toolbar controls, meters, send matrix, effect chain, sidechain picker, automation lane, and the empty dynamic event registry placeholder.
- Component drawer registrations and UI template documents for AudioSource, AudioListener, and AudioVolume. The drawer templates expose stable control slots for source input/output/sends/parameter bindings, listener HRTF/ear offsets, and volume gain/filter/reverb/crossfade authoring.
- Editor operation bindings for mixer track/send/effect/preset/sidechain/automation/dynamic-event/output/debug controls, plus AudioSource/AudioListener/AudioVolume component drawer operations. Each operation has a stable operation path and payload schema ID, and mutating operations are marked undoable for the future host-side handler layer.
- Placeholder feature crates for Timeline animation-track integration and ray-traced convolution reverb. They contribute manifests/capabilities/modules only; the real multi-plugin behavior remains a later slice.

The editor side now has template-level surfaces plus operation-level control contracts. Data-bound host handlers that execute those operations against a live `SoundManager` still belong to the next editor implementation slice.

## Edge Cases and Constraints

- Removing `master` is rejected.
- Parent cycles, send cycles, and post-effect sidechain cycles are rejected before replacing the stored graph.
- Missing track references in playbacks, sends, or parent routes return typed errors.
- Mixer preset application rejects unknown locators and cleans up live playbacks/source routes that point at tracks removed by the selected preset.
- Clip and external source resampling uses deterministic linear interpolation. Non-looping sources clamp interpolation at the final frame before reporting end-of-source, while looping sources interpolate across the wrap point.
- External source blocks reject empty provider handles, zero sample rate, zero channels, non-finite samples, and partial frames; clearing a missing external handle returns `UnknownExternalSource`.
- Source parameter bindings reject empty IDs and unsupported source parameter names before insertion. Synth parameter inputs reject empty parameter IDs and non-finite default values. Missing synth parameter values are treated as absent modulation for that block rather than as render failures.
- Output device descriptors reject empty IDs/backend/display names, zero sample rates, zero channel counts, zero block sizes, zero latency blocks, and unavailable backend IDs. Pulling output or backend callbacks before start returns `BackendUnavailable` with a stopped-device detail.
- Removing a missing impulse response returns `UnknownImpulseResponse`; ray-traced status rejects `RayTraced` snapshots with zero rays per update. Provider-submitted ray-traced IRs reject empty cell keys, zero sample rates/channels, zero ray counts, non-finite samples, and unknown source/listener/volume references.
- Dynamic event descriptors reject empty IDs, display names, or payload schemas; handler descriptors reject empty plugin IDs, handler IDs, event IDs, or display names and must reference a registered event. Submitted invocations reject unknown events, mismatched schemas, and non-finite event times. Unregistering an event also removes registered handlers and pending invocations for that event ID; removing a missing handler returns `UnknownDynamicEventHandler`.
- Removing a missing track send returns `UnknownSend`; querying a missing sound parameter returns `UnknownParameter`; applying or removing a missing automation binding returns `UnknownAutomationBinding`.
- Effect wet mix must stay within `0..=1`; compressors require `ratio >= 1`; filters and limiters reject invalid core parameters.
- Source descriptors reject non-finite position/forward/velocity/gain values and spatial ranges outside the supported envelope.
- Listener descriptors reject non-finite transforms and unknown mixer targets.
- HRTF profiles reject empty IDs/display names, zero sample rates, empty kernels, non-finite kernel samples, and all-zero kernels. Removing a missing profile returns `UnknownHrtfProfile`. Rendering with a missing listener profile falls back to preview instead of failing.
- Volume descriptors reject invalid shapes, non-finite gains, negative crossfade distances, and invalid low-pass cutoffs.
- Impulse responses must be non-empty and finite.
- Automation bindings must name a Timeline-style track path. Runtime value application, curve sampling, and sound-owned Timeline sequence scheduling are available for scalar targets. Timeline sequence descriptors reject empty IDs, non-positive or non-finite durations, empty track lists, duplicate bindings, missing automation bindings, invalid curves, and keyframes outside the sequence duration.
- Ray-traced convolution does not trace geometry itself until a geometry/ray-query provider exists; the new provider submission path accepts computed IRs and feeds the existing convolution cache.

## Test Coverage

Added sound runtime tests cover plugin registration, full component/option/event contribution, silence rendering, custom track routing, full graph configure import for sources and automation bindings, duplicate configured source/binding failures, built-in mixer preset discovery/application/reroute/error behavior, track send CRUD/rendering/errors, parent and send cycle rejection, solo direct-input gating, effect-chain gain, sidechain compression, pre/post sidechain taps, static convolution IR, IR invalidation errors, ray-tracing status publication/validation, ray-traced IR provider submission/cache status/convolution output/clear/invalid provider data, dynamic event descriptor registration/snapshot visibility/opaque invocation drain/invalid invocation failures, dynamic event multi-plugin handler registration/validation/deterministic dispatch/unregister cleanup, synth parameter source rendering, external source block rendering and lifecycle errors, output device configure/start/pull/status/reconfigure/errors, deterministic `software-null` backend capability listing/callback reports/stopped and unsupported backend failures, clip/external deterministic linear sample-rate resampling, source parameter binding to synth parameters, invalid source parameter binding errors, parameter readback errors, automation snapshot visibility, automation value application to synth/track/effect targets, automation curve sampling/application/endpoint clamping/invalid-curve failures, sound-owned Timeline sequence scheduling/non-looping completion/looping wrap/typed validation failures, clean automation binding/path/target failures, listener-driven attenuation/pan/occlusion, HRTF profile load/list/remove/validation/kernel rendering plus preview fallback behavior, volume priority/crossfade gain, pre-spatial source sends, deterministic filter/reverb/wave-shaper/modulation effects, stateful delay/reverb/convolution/modulated-delay tails across render blocks, phaser LFO phase continuity, compressor release envelopes, latency snapshot reporting, pan/phase/limiter, bypass, and wet/dry behavior.

Added editor tests cover mixer and acoustic debug views, templates, menus, operations, operation payload schema IDs, undoable mixer operations, preset and dynamic-event placeholder operation descriptors, and component drawer operation bindings.

Validation evidence from this continuation slice:

- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed after the Timeline sequence scheduling slice.
- `rustfmt --check zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\ids.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs` passed after the Timeline sequence scheduling slice.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml automation_curve --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-timeline-sequence --message-format short --color never` passed after the Timeline sequence scheduling slice: 5 passed, 47 filtered.
- A fresh retry of `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml automation_curve --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-timeline-sequence --message-format short --color never` after the doc/session update stopped before compiling the sound crate on an external dirty render-pipeline compile error: `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs:208:12` initializes `CompiledRenderPipeline` without the newly added dirty-field `pass_stages` from `zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs`. This sound slice did not edit that render pipeline lane.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed after formatting the backend-seam slice.
- `rustfmt --check zircon_runtime\src\core\framework\sound\output.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs` passed after the backend-seam slice.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml output_device --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-backend-seam --message-format short --color never` stopped before compiling the sound crate on an external dirty `zircon_runtime` graphics compile error: `zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs:139:26` mismatched mutability. This backend-seam slice did not edit that graphics lane.
- `git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md docs\superpowers\specs\2026-05-03-sound-backend-seam-design.md docs\superpowers\plans\2026-05-03-sound-backend-seam.md` passed after the backend-seam slice.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed after formatting the HRTF profile-loading slice.
- `rustfmt --check zircon_runtime\src\core\framework\sound\acoustics.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs` passed after formatting the HRTF profile-loading slice.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml hrtf --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-hrtf-profile --message-format short --color never` passed after the HRTF profile-loading slice: 4 passed, 54 filtered, with existing `zircon_runtime` warnings.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed after the dynamic-event handler slice.
- `rustfmt --check zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs` passed after the dynamic-event handler slice.
- `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1` passed after the dynamic-event handler slice.
- `cargo check -p zircon_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-sound-framework --message-format short --color never` passed after the dynamic-event handler slice with existing `zircon_runtime` warnings.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml dynamic_events --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events --message-format short --color never` passed after the dynamic-event handler slice: 4 passed, 46 filtered.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events --message-format short --color never` passed after the dynamic-event handler slice: 50 passed, 0 failed, doctests 0.
- `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-editor-lib --message-format short --color never` passed after the dynamic-event handler slice with existing `zircon_runtime`/`zircon_editor` warnings.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed for the sound runtime package after the resampling/source-binding/option additions.
- `rustfmt --check zircon_runtime\src\core\framework\sound\components.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\mod.rs` passed for the neutral sound files touched by this slice.
- `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, and `rustfmt zircon_runtime\src\core\framework\sound\ids.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs zircon_runtime\src\core\framework\sound\output.rs` passed after the output-device slice.
- `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` and `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime` passed after the HRTF preview slice.
- `cargo metadata --manifest-path zircon_plugins\Cargo.toml --locked --offline --no-deps --format-version 1` passed, and `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1` passed, confirming the plugin workspace and sound runtime manifest remain readable.
- `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor`, `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor`, and `cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1` passed after the editor operation-binding slice.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `rustfmt --check zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs zircon_runtime\src\core\framework\sound\preset.rs`, `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1`, and `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never` passed after the mixer preset slice.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor` and `cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1` passed after the editor mixer preset binding slice.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never` timed out after seven minutes without Rust diagnostics under concurrent Cargo load.
- `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-editor --message-format short --color never` timed out after six minutes without Rust diagnostics under concurrent Cargo load.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `rustfmt --check zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs`, and `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1` passed after the automation curve slice.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-curve --message-format short --color never` timed out after seven minutes without Rust diagnostics under concurrent Cargo load; the leftover sound Cargo processes were stopped afterward.
- Earlier `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never` stopped on an in-flight physics/animation cutover; that blocker was superseded when `zircon_runtime` removed the concrete `pub mod physics` / `pub mod animation` roots and moved their managers into `zircon_plugins/{physics,animation}/runtime`.
- `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor`, `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, and `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_editor` passed after the automation-curve test-stability and dynamic-event editor placeholder edits.
- `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1`, `cargo metadata --manifest-path zircon_plugins\sound\editor\Cargo.toml --locked --offline --no-deps --format-version 1`, and `git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md` passed after the latest edits.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets --message-format short --color never` and `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-editor --message-format short --color never` both stopped before compiling because `zircon_plugins/Cargo.lock` currently needs an update. The lockfile is shared dirty state outside this sound slice, so this continuation did not regenerate it.
- `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `rustfmt --check zircon_runtime\src\core\framework\sound\acoustics.rs zircon_runtime\src\core\framework\sound\automation.rs zircon_runtime\src\core\framework\sound\events.rs zircon_runtime\src\core\framework\sound\error.rs zircon_runtime\src\core\framework\sound\graph.rs zircon_runtime\src\core\framework\sound\manager.rs zircon_runtime\src\core\framework\sound\mod.rs`, `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1`, and `git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md` passed after the dynamic event runtime queue and ray-traced IR submission slice.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-interface-gaps --message-format short --color never` timed out after three minutes without Rust diagnostics under concurrent Cargo load.
- `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-editor-bindings --message-format short --color never` timed out after four minutes without Rust diagnostics under the current concurrent Cargo load.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device --message-format short --color never` timed out after four minutes without Rust diagnostics under the same concurrent Cargo load.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-hrtf-preview --message-format short --color never` timed out after four minutes without Rust diagnostics under the same concurrent Cargo load.
- Latest retry of `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state --message-format short --color never` timed out after four minutes without Rust diagnostics under the current concurrent Cargo load.
- `cargo check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state --message-format short --color never` stopped before compiling because Cargo reports `zircon_plugins\Cargo.lock` would need an update. The lockfile was already shared dirty state, so this sound slice did not regenerate it.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never` timed out after five minutes without Rust diagnostics while many unrelated Cargo/Rust jobs were compiling `zircon_runtime`.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never` and the same command with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct` also timed out under the same concurrent build pressure before emitting diagnostics.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation --message-format short --color never` compiled into `zircon_runtime` but stopped before sound tests on an external graphics error: `zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs:25:10` calls missing `with_evictable_page_ids` on `(HybridGiRuntimeFeedback, VirtualGeometryRuntimeFeedback)`.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: runtime `16/16` tests passed and doctests had no failures.
- `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --message-format short --color never` passed with the same target directory.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed before the full runtime test run and confirmed the new test code type-checked.
- A full `cargo test --manifest-path zircon_plugins\sound\editor\Cargo.toml ...` attempt timed out in the editor test-binary stage without Rust diagnostics under concurrent workspace build pressure; the editor library check above is the accepted scoped gate for this slice.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-graph-config --message-format short --color never` passed after the graph configure import and source-input validation slices, including the new graph-config tests at compile time. The focused `cargo test ... graph_config ...` execution initially timed out after 424 seconds without diagnostics, then passed on retry with 2 tests passed and 46 filtered.
- `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `cargo fmt --check --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime`, `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1`, and `git diff --check -- zircon_runtime\src\core\framework\sound zircon_plugins\sound docs\engine-architecture\runtime-sound-extension.md .codex\sessions\20260503-0228-sound-mixer-graph-continuation.md` passed after the deterministic linear resampling slice before this evidence text update.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linear-resample --message-format short --color never` passed after the deterministic linear resampling slice.
- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml clip_and_external_inputs_resample_to_mixer_rate --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-sound-linear-resample --message-format short --color never` first timed out after 304 seconds without Rust diagnostics. The immediate retry was blocked before compile because the shared `zircon_plugins\Cargo.lock` then needed an update after unrelated `zircon_plugins\Cargo.toml` / lockfile changes in the dirty workspace; this slice did not regenerate that shared lockfile.

## Plan Sources

This update implements the first "core first" stage from `Sound 插件核心完善计划`: stable contracts, graph/DSP runtime, components, options/dependencies, dynamic event registry/queue contracts, graph-owned source/automation import, ray-traced IR provider submission, and editor authoring descriptors.

The deeper follow-up remains:

- CPAL/platform OS audio-device adapter on top of the deterministic backend callback contract,
- production-grade low-latency DSP details such as higher-order filters/resamplers, denormal handling, SIMD/partition tuning, and audio-device scheduling,
- production HRTF database parsing/interpolation/optimized binaural convolution and geometry-backed occlusion,
- actual ray traversal/impulse-response generation from scene geometry,
- actual plugin-code execution for dynamic event handlers,
- host-side editor operation handlers that apply registered sound operations to a live `SoundManager`.
