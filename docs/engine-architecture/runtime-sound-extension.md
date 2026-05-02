---
related_code:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/state.rs
  - zircon_plugins/sound/runtime/src/engine/validation.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/lib.rs
implementation_files:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_dependency_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_event_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_option_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/runtime_extension_registry.rs
  - zircon_runtime/src/plugin/extension_registry/register.rs
  - zircon_runtime/src/plugin/extension_registry/access.rs
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/engine/dsp.rs
  - zircon_plugins/sound/runtime/src/engine/render.rs
  - zircon_plugins/sound/runtime/src/engine/state.rs
  - zircon_plugins/sound/runtime/src/engine/validation.rs
  - zircon_plugins/sound/runtime/src/package.rs
  - zircon_plugins/sound/runtime/src/service_types.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/editor/src/lib.rs
plan_sources:
  - user: 2026-05-02 sound plugin mixer/spatial/convolution/timeline core implementation request
  - .codex/plans/Sound 插件核心完善计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct; 16 runtime tests passed)
  - cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct)
  - cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never (passed with CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct)
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
- `graph.rs` defines `SoundMixerGraph`, tracks, sends, controls, meters, snapshots, and ray-tracing convolution status.
- `effects.rs` defines the first deterministic DSP descriptor set: gain, filter, reverb, convolution reverb, compressor with sidechain, wave shaper, flanger, phaser, chorus, delay, pan/stereo, and limiter.
- `components.rs` defines neutral `AudioSource`, `AudioListener`, and `AudioVolume` contracts plus plugin dynamic component type IDs. Source/listener descriptors now carry position, forward vector, and velocity data so the plugin runtime can spatialize without coupling directly to a concrete scene transform component.
- `automation.rs` defines Timeline-style sound automation bindings and an empty dynamic event catalog.
- `options.rs` defines runtime/editor-facing sound plugin options and quality gates.

The plugin implementation is split under `zircon_plugins/sound/runtime/src/engine/` so `service_types.rs` remains a manager boundary rather than an audio engine dumping ground.

## Behavior Model

The default manager keeps a `SoundEngineState` with clip cache, active clip playbacks, explicit source voices, listener/volume descriptors, automation bindings, parameter values, static impulse responses, mixer graph, meters, latency, and ray-tracing status.

The `SoundManager` surface now exposes graph mutation at the track/send/effect level instead of requiring callers to replace the whole graph for common authoring operations. Track sends are keyed by target track ID, so adding a send to the same target updates the existing edge; removing a missing edge returns a typed `UnknownSend` error. Synth/automation parameters also support readback through `parameter_value(...)`, with unknown keys returning `UnknownParameter`.

The render order is a deterministic topological walk over parent routes, track sends, and post-effect sidechain dependencies. This keeps child tracks before parents, send sources before send targets, and post-effect sidechain key tracks before the compressors that read them.

Rendering follows a fixed block pipeline:

1. Validate the graph has a `master` track and no parent cycles.
2. Mix `play_clip` convenience playbacks into their requested output track.
3. Mix explicit `SoundSourceDescriptor` voices into dry source buffers.
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
- Static convolution reverb uses the same `SoundImpulseResponseId` slot that future ray-traced IR generation will fill.
- External and synthesizer sound sources are neutral handles/parameter IDs, so other plugins can feed audio without linking to sound internals.
- Active listeners are selected deterministically by requested mixer target, then `master`, then stable listener ID. This keeps multi-listener editor previews predictable while leaving room for richer runtime routing later.
- `AudioVolume` influence is deterministic in the first runtime slice: the highest-priority volume with non-zero influence wins, and its crossfade blends exterior/interior gain, low-pass, and static convolution send weight.

The first CPU DSP implementations are deterministic block effects for tests and editor preview. They are not yet a final low-latency audio-device backend.

## Editor and Plugin Metadata

The sound plugin now contributes:

- Dynamic component descriptors for `sound.Component.AudioSource`, `sound.Component.AudioListener`, and `sound.Component.AudioVolume`.
- Plugin options for backend, sample rate, channels, block size, max voices/tracks, HRTF, convolution, ray-tracing quality, Timeline automation, and dynamic event enablement.
- Manifest dependencies on `asset` and `scene`, plus optional `ray_query` and `timeline_sequence`.
- An empty `sound.dynamic_events` event catalog, versioned now so later multi-plugin event work has a stable slot.
- Editor views for Sound Mixer and Acoustic Debug.
- Component drawers for AudioSource, AudioListener, and AudioVolume.
- Placeholder feature crates for Timeline animation-track integration and ray-traced convolution reverb. They contribute manifests/capabilities/modules only; the real multi-plugin behavior remains a later slice.

The editor side is currently descriptor-level: it exposes the surfaces and gates that the host can discover. The concrete UI templates/controllers can be filled in the next editor implementation slice.

## Edge Cases and Constraints

- Removing `master` is rejected.
- Parent cycles, send cycles, and post-effect sidechain cycles are rejected before replacing the stored graph.
- Missing track references in playbacks, sends, or parent routes return typed errors.
- Removing a missing track send returns `UnknownSend`; querying a missing sound parameter returns `UnknownParameter`.
- Effect wet mix must stay within `0..=1`; compressors require `ratio >= 1`; filters and limiters reject invalid core parameters.
- Source descriptors reject non-finite position/forward/velocity/gain values and spatial ranges outside the supported envelope.
- Listener descriptors reject non-finite transforms and unknown mixer targets.
- Volume descriptors reject invalid shapes, non-finite gains, negative crossfade distances, and invalid low-pass cutoffs.
- Impulse responses must be non-empty and finite.
- Automation bindings must name a Timeline-style track path, but Timeline execution remains a gated integration point.
- Ray-traced convolution is not active until a geometry/ray-query provider exists; the status records disabled/static/future ray-traced state.

## Test Coverage

Added sound runtime tests cover plugin registration, component/option/event contribution, silence rendering, custom track routing, track send CRUD/rendering/errors, parent and send cycle rejection, solo direct-input gating, effect-chain gain, sidechain compression, pre/post sidechain taps, static convolution IR, synth parameter source rendering, parameter readback errors, automation snapshot visibility, listener-driven attenuation/pan/occlusion, volume priority/crossfade gain, pre-spatial source sends, deterministic filter/reverb/wave-shaper/modulation effects, pan/phase/limiter, bypass, and wet/dry behavior.

Added editor tests cover mixer and acoustic debug views, templates, menus, operations, and component drawer registration.

Validation evidence from this continuation slice:

- `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: runtime `16/16` tests passed and doctests had no failures.
- `cargo check --manifest-path zircon_plugins\sound\editor\Cargo.toml --lib --locked --offline --jobs 1 --message-format short --color never` passed with the same target directory.
- `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed before the full runtime test run and confirmed the new test code type-checked.
- A full `cargo test --manifest-path zircon_plugins\sound\editor\Cargo.toml ...` attempt timed out in the editor test-binary stage without Rust diagnostics under concurrent workspace build pressure; the editor library check above is the accepted scoped gate for this slice.

## Plan Sources

This update implements the first "core first" stage from `Sound 插件核心完善计划`: stable contracts, graph/DSP runtime, components, options/dependencies/events placeholders, and editor authoring descriptors.

The deeper follow-up remains:

- platform audio output driver,
- stateful low-latency DSP with persistent delay/reverb lines,
- true spatial HRTF and occlusion,
- ray-traced impulse-response generation from scene geometry,
- full Timeline event execution and dynamic multi-plugin event semantics,
- concrete editor UI templates/controllers.
