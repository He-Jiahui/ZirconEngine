---
related_code:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/channel_layout.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/events.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/manager/acoustics.rs
  - zircon_runtime/src/core/framework/sound/manager/automation_timeline.rs
  - zircon_runtime/src/core/framework/sound/manager/backend.rs
  - zircon_runtime/src/core/framework/sound/manager/dynamic_events.rs
  - zircon_runtime/src/core/framework/sound/manager/mixer_graph.rs
  - zircon_runtime/src/core/framework/sound/manager/output_device.rs
  - zircon_runtime/src/core/framework/sound/manager/playback.rs
  - zircon_runtime/src/core/framework/sound/manager/render.rs
  - zircon_runtime/src/core/framework/sound/manager/runtime_settings.rs
  - zircon_runtime/src/core/framework/sound/manager/source.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/preset.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/framework/sound/tests.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/output_device.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/render.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait/source.rs
implementation_files:
  - zircon_runtime/src/core/framework/sound/mod.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_runtime/src/core/framework/sound/channel_layout.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/error.rs
  - zircon_runtime/src/core/framework/sound/events.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/ids.rs
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/manager/acoustics.rs
  - zircon_runtime/src/core/framework/sound/manager/automation_timeline.rs
  - zircon_runtime/src/core/framework/sound/manager/backend.rs
  - zircon_runtime/src/core/framework/sound/manager/dynamic_events.rs
  - zircon_runtime/src/core/framework/sound/manager/mixer_graph.rs
  - zircon_runtime/src/core/framework/sound/manager/output_device.rs
  - zircon_runtime/src/core/framework/sound/manager/playback.rs
  - zircon_runtime/src/core/framework/sound/manager/render.rs
  - zircon_runtime/src/core/framework/sound/manager/runtime_settings.rs
  - zircon_runtime/src/core/framework/sound/manager/source.rs
  - zircon_runtime/src/core/framework/sound/mix.rs
  - zircon_runtime/src/core/framework/sound/options.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/preset.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/framework/sound/tests.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
plan_sources:
  - .codex/plans/Sound 插件核心完善计划.md
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
tests:
  - zircon_runtime/src/core/framework/sound/tests.rs
  - default_sound_plugin_options_match_runtime_contract
  - default_stereo_mixer_graph_keeps_master_track_and_event_namespace
  - sound_channel_layouts_name_speaker_order_for_multichannel_formats
  - clip_source_defaults_to_master_track_and_neutral_spatial_contract
  - sound_scene_component_type_ids_are_plugin_prefixed
  - cargo test -p zircon_runtime --lib sound --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-framework-contract --message-format short --color never (pending while active Cargo lanes are busy)
  - 2026-06-04: rustfmt --edition 2021 over Sound manager capability trait files, Sound runtime manager_trait delegate files, Sound editor live-output controller, and manager handle structural test (passed)
  - 2026-06-04: git diff --check over touched Sound framework/runtime/editor docs and source files (passed)
doc_type: module-detail
---

# Sound Framework Contracts

## Purpose

`zircon_runtime::core::framework::sound` is the neutral audio contract layer shared by runtime plugins, editor tooling, scripting, and asset-facing systems. It owns stable DTOs, IDs, component type names, manager traits, error/status records, and option shapes. It does not own DSP algorithms, output-device threads, CPAL sessions, ray tracing, editor mixer panels, or asset decoding. Those concrete behaviors live in `zircon_plugins/sound/runtime` and `zircon_plugins/sound/editor`.

This keeps Sound aligned with the current engine architecture: `zircon_runtime::core::framework` defines contracts, `zircon_runtime::core::manager` exposes stable access handles, and the Sound plugin registers the real `SoundModule`, driver, and manager implementation.

## Related Files

The framework is folder-backed and `mod.rs` is only the public re-export surface.

- `ids.rs` defines stable handles for clips, tracks, effects, sources, listeners, volumes, parameters, automation bindings, timeline sequences, impulse responses, external sources, output devices, and playbacks.
- `channel_layout.rs` defines neutral speaker-order metadata for mono, stereo, 5.1, 7.1, and discrete multichannel formats so editor meters, output devices, render blocks, and export tooling do not have to infer layout from a bare channel count.
- `graph.rs`, `effects.rs`, `mix.rs`, `output.rs`, `status.rs`, and `preset.rs` describe the mixer graph, effect chains, rendered mix blocks, output-device contracts, backend status, and preset descriptors.
- `components.rs` defines the three scene-facing component contract IDs and descriptors: `sound.Component.AudioSource`, `sound.Component.AudioListener`, and `sound.Component.AudioVolume`.
- `automation.rs` and `events.rs` define timeline automation, target addressing, dynamic event catalogs, handler descriptors, queued invocations, delivery records, and execution reports.
- `acoustics.rs` defines HRTF and ray-traced impulse-response DTOs without depending on a concrete ray-query or geometry provider.
- `manager.rs` is the structural service surface. It exports the composed `SoundManager` trait plus capability traits under `manager/{backend,output_device,runtime_settings,playback,mixer_graph,source,automation_timeline,dynamic_events,acoustics,render}.rs`, so consumers can depend on a narrow output, mixer, source, automation, event, acoustics, or render capability without implementing the whole audio service when a narrower contract is enough.

The runtime plugin consumes these contracts in `zircon_plugins/sound/runtime/src/module.rs`, registers component descriptors in `zircon_plugins/sound/runtime/src/components.rs`, and implements the trait through its `service_types` subtree.

## Behavior Model

The default sound contract is a stereo software-mixer-oriented graph: `SoundMixerGraph::default_stereo(48_000)` creates a graph with one fixed `Master` track, two channels, a `stereo` channel layout, an empty source set, no automation bindings, and an empty `sound.dynamic_events` event catalog at version `1`. `SoundTrackId::master()` is the stable master route used by playback defaults and clip-backed source descriptors.

Channel layout is first-class contract data. `SoundChannelLayout` names the semantic speaker order for mono, stereo, surround 5.1, surround 7.1, or unknown discrete channel counts. `SoundPluginOptions`, `SoundMixerGraph`, `SoundMixBlock`, `SoundOutputDeviceDescriptor`, `SoundBackendCapability`, and `SoundBackendStatus` all carry layout metadata alongside `channel_count`; concrete runtimes must keep those values aligned when normalizing options, configuring output devices, and rendering blocks.

Scene integration uses three component IDs:

- `sound.Component.AudioSource` carries clip, external, synth, or silence input plus output routing, sends, gain, playback state, spatial blend, attenuation, doppler, occlusion, convolution send, and parameter bindings.
- `sound.Component.AudioListener` carries listener pose, active flag, HRTF profile, doppler tracking, ear offsets, and mixer target.
- `sound.Component.AudioVolume` carries region shape, priority, gain/filter/reverb/convolution influence, and crossfade distance.

Effects are data-only descriptors. The framework names the supported families: gain, filter, reverb, convolution reverb, compressor with optional sidechain, wave shaper, flanger, phaser, chorus, delay, pan/stereo width, and limiter. Validation, state allocation, delay-line history, filter coefficient calculation, and actual sample processing stay in the plugin runtime.

The manager surface is intentionally capability-composed because sound is an engine service, not only a clip player. `SoundManager` remains the stable whole-service trait resolved through `zircon_runtime::core::manager`, but it is now built from narrower traits for backend status, output-device control, runtime settings, clip playback, mixer graph mutation, source lifecycle, automation/timeline sequence advancement, dynamic events, acoustics, and direct mix rendering. Editor or tooling code can accept only the capability subset it needs, while the runtime plugin still implements the full service.

## Reference Alignment

The contract split follows local reference-engine evidence:

- Godot separates audio buses, effects, listeners, streams, and stream players (`dev/godot/doc/classes/AudioBusLayout.xml`, `AudioEffect*.xml`, `AudioListener3D.xml`, and `AudioStreamPlayer3D.xml`). Zircon translates that into mixer tracks, effect descriptors, listener descriptors, and scene audio source components.
- Unreal separates scene audio components, audio volumes, SoundCue-style graph assets, and large mixer/submix tooling (`dev/UnrealEngine/.../UAudioComponent`, `AAudioVolume`, and `SoundCue` documentation/source families). Zircon keeps similar concepts but routes access through `SoundManager` instead of introducing non-network `server` naming.
- Fyrox provides the closest Rust-native precedent for sound buses, context, listener, HRTF, DSP filters, reverb, and renderer separation (`dev/Fyrox/fyrox-sound/src/{bus,context,listener,source,dsp,effects,renderer}`). Zircon mirrors that separation with neutral contracts in framework and concrete audio engine code in the plugin runtime.
- Bevy's `bevy_audio` package and examples are useful for ECS-friendly clip playback and simple spatial audio, but Zircon deliberately goes deeper to support mixer graphs, DSP, HRTF, dynamic events, and timeline automation.

## Control Flow

`zircon_plugins/sound/runtime` registers `SoundModule` with an immediate `SoundDriver`, a lazy `DefaultSoundManager`, and a public `SoundManager` handle. The concrete manager owns plugin runtime state, implements each framework capability trait through focused delegate modules, and satisfies the composed whole-service trait. Editor, app, scripting, and future VM plugins should resolve the manager/handle and pass neutral DTOs; they should not share concrete runtime objects or call DSP internals.

Project or profile options flow through `SoundPluginOptions`. The defaults enable sound, use `software-mixer`, set `48_000 Hz`, `2` channels with the `stereo` layout, `256` frame blocks, `128` voices, `64` tracks, the default spatial scale of `1.0`, enabled convolution, enabled timeline integration, enabled dynamic events, and disabled ray-tracing quality. Concrete runtime config conversion and option/catalog parity live in the plugin runtime.

## Edge Cases

The framework accepts DTOs that still require runtime validation. Invalid graph references, cycles, illegal effect parameter ranges, unsupported output backends, channel layout/count mismatches, missing clips, missing HRTF profiles, and unavailable ray-query providers should return typed `SoundError` values from the manager implementation, not panic from the framework. Ray-traced convolution may be represented as waiting for a provider, static impulse response, or ray-traced cache state; the framework does not require a geometry provider to exist.

The default dynamic event catalog is intentionally empty. Sound-specific events and third-party handler descriptors are registered by the runtime plugin and optional feature plugins so the framework stays generic enough for future VM and native plugin event adapters.

## Test Coverage

Framework-level tests now lock the default option contract, the master mixer graph and dynamic-event namespace, channel layout names and speaker order, clip-backed source defaults, and sound component type prefixes. Cargo validation for these tests is pending while other active Cargo lanes are running; current low-interference validation should use rustfmt, conflict-marker scans, and `git diff --check` until the build machine is quiet enough for `cargo test -p zircon_runtime --lib sound`.
