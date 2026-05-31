---
related_code:
  - zircon_plugins/sound/plugin.toml
  - zircon_plugins/sound/runtime/Cargo.toml
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/mod.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/registration.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/Cargo.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/Cargo.toml
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/runtime/src/automation/mod.rs
  - zircon_plugins/sound/runtime/src/automation/binding.rs
  - zircon_plugins/sound/runtime/src/automation/curve.rs
  - zircon_plugins/sound/runtime/src/automation/values.rs
  - zircon_plugins/sound/runtime/src/automation/target/mod.rs
  - zircon_plugins/sound/runtime/src/automation/target/apply.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/mod.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/apply.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/common.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/delay.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/dynamics.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/filter.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/gain.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/modulation.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/reverb.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/shaper.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/stereo.rs
  - zircon_plugins/sound/runtime/src/automation/target/helpers.rs
  - zircon_plugins/sound/runtime/src/automation/target/listener.rs
  - zircon_plugins/sound/runtime/src/automation/target/source.rs
  - zircon_plugins/sound/runtime/src/automation/target/track.rs
  - zircon_plugins/sound/runtime/src/automation/target/volume.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/mod.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/common.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/external_source.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/hrtf.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/listener.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/mod.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/bindings.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/clip_range.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/input.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/spatial.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/tracks.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/values.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/volume.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/mod.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/callback.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/executor.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/request.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/slice.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/status.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/mod.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/automation.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/runtime_state.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/timeline.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/mod.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/provider.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/status.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/validation.rs
  - zircon_plugins/sound/runtime/src/package/mod.rs
  - zircon_plugins/sound/runtime/src/package/attach.rs
  - zircon_plugins/sound/runtime/src/package/dependencies.rs
  - zircon_plugins/sound/runtime/src/package/events.rs
  - zircon_plugins/sound/runtime/src/package/options.rs
  - zircon_plugins/sound/runtime/src/presets/mod.rs
  - zircon_plugins/sound/runtime/src/presets/catalog.rs
  - zircon_plugins/sound/runtime/src/presets/default.rs
  - zircon_plugins/sound/runtime/src/presets/locators.rs
  - zircon_plugins/sound/runtime/src/presets/music_sfx.rs
  - zircon_plugins/sound/runtime/src/presets/spatial_room.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/clip_assets.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/execution.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/registration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/unregistration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/service_types/external_sources.rs
  - zircon_plugins/sound/runtime/src/service_types/hrtf_profiles.rs
  - zircon_plugins/sound/runtime/src/service_types/impulse_responses.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/effects.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/snapshot.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/tracks.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_presets.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/status.rs
  - zircon_plugins/sound/runtime/src/service_types/output_render.rs
  - zircon_plugins/sound/runtime/src/service_types/parameters.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_validation.rs
  - zircon_plugins/sound/runtime/src/service_types/ray_tracing_convolution.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/source_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/source_seek.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/service_types/timeline_sequences.rs
  - zircon_plugins/sound/runtime/src/timeline/mod.rs
  - zircon_plugins/sound/runtime/src/timeline/advance.rs
  - zircon_plugins/sound/runtime/src/timeline/playback.rs
  - zircon_plugins/sound/runtime/src/timeline/schedule.rs
  - zircon_plugins/sound/runtime/src/timeline/validation.rs
  - zircon_plugins/sound/runtime/src/output/mod.rs
  - zircon_plugins/sound/runtime/src/output/catalog.rs
  - zircon_plugins/sound/runtime/src/output/descriptor_validation.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/mod.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/callback.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/config.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/session.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/start_stop.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/status.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/storage.rs
  - zircon_plugins/sound/runtime/src/output/status.rs
  - zircon_plugins/sound/runtime/src/output/software.rs
  - zircon_plugins/sound/runtime/src/output/ring_buffer.rs
  - zircon_plugins/sound/runtime/src/output/cpal/mod.rs
  - zircon_plugins/sound/runtime/src/output/cpal/capability.rs
  - zircon_plugins/sound/runtime/src/output/cpal/callback.rs
  - zircon_plugins/sound/runtime/src/output/cpal/device.rs
  - zircon_plugins/sound/runtime/src/output/cpal/device_thread.rs
  - zircon_plugins/sound/runtime/src/output/cpal/error.rs
  - zircon_plugins/sound/runtime/src/output/cpal/producer_thread.rs
  - zircon_plugins/sound/runtime/src/output/cpal/selection.rs
  - zircon_plugins/sound/runtime/src/output/cpal/session.rs
  - zircon_plugins/sound/runtime/src/output/cpal/shared_state.rs
  - zircon_plugins/sound/runtime/src/engine/math.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/controls.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/delay.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/dynamics.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/apply.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/chain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/sidechain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/gain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/meter.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/modulation.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/reverb.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/shaper.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/stereo.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/delay_line.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/effect_key.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/effect_runtime.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/history.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/track_runtime.rs
  - zircon_plugins/sound/runtime/src/engine/filter/mod.rs
  - zircon_plugins/sound/runtime/src/engine/filter/apply.rs
  - zircon_plugins/sound/runtime/src/engine/filter/coefficients.rs
  - zircon_plugins/sound/runtime/src/engine/filter/constants.rs
  - zircon_plugins/sound/runtime/src/engine/filter/shelf.rs
  - zircon_plugins/sound/runtime/src/engine/filter/state.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/mod.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/apply.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/key.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/prune.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/state.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/mod.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/constants.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/gain.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/query.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/ray_traced.rs
  - zircon_plugins/sound/runtime/src/engine/render/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/clip.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/finish.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mixing.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/pan.rs
  - zircon_plugins/sound/runtime/src/engine/render/routing.rs
  - zircon_plugins/sound/runtime/src/engine/render/runtime_state.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/frame.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/interpolation.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/position.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/step.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/external.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/input.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/parameters.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/range.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/apply.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/constants.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/convolution.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/loaded.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/preview.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/tail.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/listener.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/attenuation.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/cone.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/doppler.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/pan.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/profile.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/filter.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/influence.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/weight.rs
  - zircon_plugins/sound/runtime/src/engine/state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/state/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/engine/state/graph.rs
  - zircon_plugins/sound/runtime/src/engine/state/playback.rs
  - zircon_plugins/sound/runtime/src/engine/state/snapshot.rs
  - zircon_plugins/sound/runtime/src/engine/state/source.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/engine/validation/mod.rs
  - zircon_plugins/sound/runtime/src/engine/validation/effect.rs
  - zircon_plugins/sound/runtime/src/engine/validation/graph.rs
  - zircon_plugins/sound/runtime/src/engine/validation/ordering.rs
  - zircon_plugins/sound/runtime/src/engine/validation/references.rs
  - zircon_plugins/sound/runtime/src/engine/validation/track.rs
  - zircon_plugins/sound/runtime/src/engine/validation/values.rs
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
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/mod.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/descriptor.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/feature_manifest.rs
  - zircon_plugins/sound/runtime/src/runtime_plugin/registration.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/Cargo.toml
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/timeline_animation_track/editor/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/Cargo.toml
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/editor/src/lib.rs
  - zircon_plugins/sound/runtime/src/automation/mod.rs
  - zircon_plugins/sound/runtime/src/automation/binding.rs
  - zircon_plugins/sound/runtime/src/automation/curve.rs
  - zircon_plugins/sound/runtime/src/automation/values.rs
  - zircon_plugins/sound/runtime/src/automation/target/mod.rs
  - zircon_plugins/sound/runtime/src/automation/target/apply.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/mod.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/apply.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/common.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/delay.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/dynamics.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/filter.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/gain.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/modulation.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/reverb.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/shaper.rs
  - zircon_plugins/sound/runtime/src/automation/target/effect/stereo.rs
  - zircon_plugins/sound/runtime/src/automation/target/helpers.rs
  - zircon_plugins/sound/runtime/src/automation/target/listener.rs
  - zircon_plugins/sound/runtime/src/automation/target/source.rs
  - zircon_plugins/sound/runtime/src/automation/target/track.rs
  - zircon_plugins/sound/runtime/src/automation/target/volume.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/mod.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/common.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/external_source.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/hrtf.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/listener.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/mod.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/bindings.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/clip_range.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/input.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/spatial.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/tracks.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/source/values.rs
  - zircon_plugins/sound/runtime/src/descriptor_validation/volume.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/mod.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/callback.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/executor.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/request.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/slice.rs
  - zircon_plugins/sound/runtime/src/dynamic_event_abi/status.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/mod.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/automation.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/runtime_state.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/timeline.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/mod.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/provider.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/status.rs
  - zircon_plugins/sound/runtime/src/ray_tracing/validation.rs
  - zircon_plugins/sound/runtime/src/package/mod.rs
  - zircon_plugins/sound/runtime/src/package/attach.rs
  - zircon_plugins/sound/runtime/src/package/dependencies.rs
  - zircon_plugins/sound/runtime/src/package/events.rs
  - zircon_plugins/sound/runtime/src/package/options.rs
  - zircon_plugins/sound/runtime/src/presets/mod.rs
  - zircon_plugins/sound/runtime/src/presets/catalog.rs
  - zircon_plugins/sound/runtime/src/presets/default.rs
  - zircon_plugins/sound/runtime/src/presets/locators.rs
  - zircon_plugins/sound/runtime/src/presets/music_sfx.rs
  - zircon_plugins/sound/runtime/src/presets/spatial_room.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/sound/runtime/src/service_types/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/acoustics.rs
  - zircon_plugins/sound/runtime/src/service_types/automation_timeline.rs
  - zircon_plugins/sound/runtime/src/service_types/clip_assets.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/execution.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/registration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/unregistration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/service_types/external_sources.rs
  - zircon_plugins/sound/runtime/src/service_types/hrtf_profiles.rs
  - zircon_plugins/sound/runtime/src/service_types/impulse_responses.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/effects.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/snapshot.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/tracks.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_presets.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/status.rs
  - zircon_plugins/sound/runtime/src/service_types/output_render.rs
  - zircon_plugins/sound/runtime/src/service_types/parameters.rs
  - zircon_plugins/sound/runtime/src/service_types/playback.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_status.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_validation.rs
  - zircon_plugins/sound/runtime/src/service_types/ray_tracing_convolution.rs
  - zircon_plugins/sound/runtime/src/service_types/runtime_settings.rs
  - zircon_plugins/sound/runtime/src/service_types/source_controls.rs
  - zircon_plugins/sound/runtime/src/service_types/source_seek.rs
  - zircon_plugins/sound/runtime/src/service_types/source_status.rs
  - zircon_plugins/sound/runtime/src/service_types/sources.rs
  - zircon_plugins/sound/runtime/src/service_types/timeline_sequences.rs
  - zircon_plugins/sound/runtime/src/timeline/mod.rs
  - zircon_plugins/sound/runtime/src/timeline/advance.rs
  - zircon_plugins/sound/runtime/src/timeline/playback.rs
  - zircon_plugins/sound/runtime/src/timeline/schedule.rs
  - zircon_plugins/sound/runtime/src/timeline/validation.rs
  - zircon_plugins/sound/runtime/src/output/mod.rs
  - zircon_plugins/sound/runtime/src/output/catalog.rs
  - zircon_plugins/sound/runtime/src/output/descriptor_validation.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/mod.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/callback.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/config.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/session.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/start_stop.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/status.rs
  - zircon_plugins/sound/runtime/src/output/lifecycle/storage.rs
  - zircon_plugins/sound/runtime/src/output/status.rs
  - zircon_plugins/sound/runtime/src/output/software.rs
  - zircon_plugins/sound/runtime/src/output/ring_buffer.rs
  - zircon_plugins/sound/runtime/src/output/cpal/mod.rs
  - zircon_plugins/sound/runtime/src/output/cpal/capability.rs
  - zircon_plugins/sound/runtime/src/output/cpal/callback.rs
  - zircon_plugins/sound/runtime/src/output/cpal/device.rs
  - zircon_plugins/sound/runtime/src/output/cpal/device_thread.rs
  - zircon_plugins/sound/runtime/src/output/cpal/error.rs
  - zircon_plugins/sound/runtime/src/output/cpal/producer_thread.rs
  - zircon_plugins/sound/runtime/src/output/cpal/selection.rs
  - zircon_plugins/sound/runtime/src/output/cpal/session.rs
  - zircon_plugins/sound/runtime/src/output/cpal/shared_state.rs
  - zircon_plugins/sound/runtime/src/engine/math.rs
  - zircon_plugins/sound/runtime/src/engine/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/controls.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/delay.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/dynamics.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/apply.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/chain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/effects/sidechain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/gain.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/meter.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/modulation.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/reverb.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/shaper.rs
  - zircon_plugins/sound/runtime/src/engine/dsp/stereo.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/delay_line.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/effect_key.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/effect_runtime.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/history.rs
  - zircon_plugins/sound/runtime/src/engine/dsp_state/track_runtime.rs
  - zircon_plugins/sound/runtime/src/engine/filter/mod.rs
  - zircon_plugins/sound/runtime/src/engine/filter/apply.rs
  - zircon_plugins/sound/runtime/src/engine/filter/coefficients.rs
  - zircon_plugins/sound/runtime/src/engine/filter/constants.rs
  - zircon_plugins/sound/runtime/src/engine/filter/shelf.rs
  - zircon_plugins/sound/runtime/src/engine/filter/state.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/mod.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/apply.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/key.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/prune.rs
  - zircon_plugins/sound/runtime/src/engine/hrtf/state.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/mod.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/constants.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/gain.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/query.rs
  - zircon_plugins/sound/runtime/src/engine/occlusion/ray_traced.rs
  - zircon_plugins/sound/runtime/src/engine/render/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/clip.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/finish.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mixing.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/pan.rs
  - zircon_plugins/sound/runtime/src/engine/render/routing.rs
  - zircon_plugins/sound/runtime/src/engine/render/runtime_state.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/frame.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/interpolation.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/position.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/step.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/external.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/input.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/parameters.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/range.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/apply.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/constants.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/convolution.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/loaded.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/preview.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/tail.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/listener.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/attenuation.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/cone.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/doppler.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/pan.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/spatial/profile.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/mod.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/filter.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/influence.rs
  - zircon_plugins/sound/runtime/src/engine/source_environment/volume/weight.rs
  - zircon_plugins/sound/runtime/src/engine/state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/state/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/engine/state/graph.rs
  - zircon_plugins/sound/runtime/src/engine/state/playback.rs
  - zircon_plugins/sound/runtime/src/engine/state/snapshot.rs
  - zircon_plugins/sound/runtime/src/engine/state/source.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/engine/validation/mod.rs
  - zircon_plugins/sound/runtime/src/engine/validation/effect.rs
  - zircon_plugins/sound/runtime/src/engine/validation/graph.rs
  - zircon_plugins/sound/runtime/src/engine/validation/ordering.rs
  - zircon_plugins/sound/runtime/src/engine/validation/references.rs
  - zircon_plugins/sound/runtime/src/engine/validation/track.rs
  - zircon_plugins/sound/runtime/src/engine/validation/values.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
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
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs
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
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never
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
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-optional-feature-manifest cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-optional-feature-manifest cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-optional-feature-manifest cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --offline --jobs 1 --message-format short --color never
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

`src/lib.rs` remains the crate surface and now only declares internal modules plus the curated public Sound runtime API. Runtime-plugin wiring is folder-backed under `src/runtime_plugin/`: `descriptor.rs` builds the provider-owned runtime descriptor, `feature_manifest.rs` owns the optional timeline and ray-traced convolution feature bundle manifests, and `registration.rs` owns `SoundRuntimePlugin`, package-manifest materialization, project selection, linked registration reports, runtime capability rows, and runtime-extension registration. The linked optional feature provider crates under `zircon_plugins/sound/features/*/runtime` expose their own `feature_manifest()` and `plugin_feature_registration()` entry points, their `Cargo.toml` files keep `zircon_runtime` default features disabled because they only consume plugin/core metadata, and their local tests now pin those provider manifests to the owner/static Sound feature-bundle contract, including the editor module capability rows consumed by editor-host package projection. The sibling editor feature crates reuse the runtime provider `EDITOR_CAPABILITY` constants so descriptor capabilities and package manifests do not drift.

`src/config.rs` is the concrete provider configuration boundary. It derives its default from the neutral `SoundPluginOptions` DTO and exposes `SoundConfig::from_plugin_options(...)` so profile/export/editor option materialization can preserve backend, format, global volume, spatial scale, HRTF profile, convolution budget, ray-tracing quality, default mixer preset, Timeline integration, and dynamic-event gates before `DefaultSoundManager::with_config(...)` initializes runtime state.

Package manifest contributions are folder-backed under `src/package/`: `dependencies.rs` owns required and optional plugin dependencies, `options.rs` owns Sound option rows and capability gating, `events.rs` owns the versioned dynamic-event catalog namespace, and `attach.rs` owns the package-manifest merge order for dependency, option, event-catalog, and component contributions. Internal callers use these direct child modules while `src/lib.rs` keeps the curated public helper exports for external consumers.

Built-in mixer preset catalog construction is folder-backed under `src/presets/`: `locators.rs` owns stable preset locator constants, `catalog.rs` assembles the exposed preset descriptors, `default.rs` constructs the default master-only graph, `music_sfx.rs` constructs the Music/SFX/Ambience bus graph, and `spatial_room.rs` adds the Room Reverb return and sends. `src/service_types/mixer_presets.rs` imports the catalog directly and owns live state replacement/rerouting.

`SoundRuntimePlugin` registers:

- `SoundModule`, `SoundDriver`, and `DefaultSoundManager`.
- `AudioSource`, `AudioListener`, and `AudioVolume` component descriptors.
- Sound plugin options such as backend, sample rate, block size, global volume, spatial scale, HRTF, convolution, ray tracing, timeline integration, and dynamic-event enablement.
- The empty versioned `sound.dynamic_events` event catalog used as a discoverable future integration point.

Runtime audio behavior remains in this crate. The runtime framework layer only owns DTOs, handles, and traits; it does not implement mixing, DSP, output callbacks, or Sound-specific editor behavior.

Engine-owned live state is folder-backed under `src/engine/state/`: `storage.rs` owns `SoundEngineState` and its constructor, `dynamic_events.rs` owns executor callback wrappers and executor keys, `playback.rs` owns loaded clip and active playback runtime records, `source.rs` owns explicit source voice cursors and pending finish state, `graph.rs` owns track add/remove mutations that must revalidate graph shape and reroute active outputs, and `snapshot.rs` owns mixer snapshot projection. `mod.rs` only wires and re-exports these state concepts for the rest of the engine.

Render mixing is folder-backed under `src/engine/render/`: `mod.rs` is structural and only declares the render child modules; `orchestration.rs` owns the `SoundEngineState::render_mix` orchestration loop, graph validation, track buffer flow, sidechain taps, DSP application, meters, and final master gain; `playback/` owns active clip playback, with structural `mod.rs`, `mixing.rs` for active-playback routing and track-buffer dispatch, `clip.rs` for clip block sampling, `pan.rs` for per-channel pan/gain projection, and `finish.rs` for finished-playback event reporting; `source/` owns explicit `AudioSource` rendering, with `mod.rs` kept structural, `orchestration.rs` coordinating source buffers, environment delegation, sends, and finish reporting, `input.rs` mixing clip/external/synth source inputs, `external.rs` resampling provider blocks, `parameters.rs` resolving source parameter bindings, and `range.rs` calculating clip-backed source frame ranges; `sampling/` owns shared resampling and channel projection, with structural `mod.rs`, `step.rs` for resample step calculation, `position.rs` for source cursor/range advancement, `interpolation.rs` for frame interpolation, and `frame.rs` for source-frame sampling plus output-channel folding; `routing.rs` owns solo-track direct-input checks and buffer accumulation; and `runtime_state.rs` owns stale track/effect/HRTF state pruning plus latency-frame estimation. Loaded HRTF runtime state is folder-backed under `src/engine/hrtf/`: `mod.rs` exposes only loaded-profile application, render-state keys, render-state storage, and pruning; `key.rs` owns `(source, listener, profile)` identity, `state.rs` owns FIR tail history retention and tail checks, `apply.rs` owns loaded-profile convolution and history sampling, and `prune.rs` owns stale loaded-profile state retention. `src/engine/math.rs` owns the internal 3D vector helpers used by spatial attenuation, HRTF preview, cone, pan, doppler, and volume calculations.

DSP execution is folder-backed under `src/engine/dsp/`: `mod.rs` exposes only the render-facing entry points, `effects/` owns track effect-chain execution with structural `mod.rs`, `chain.rs` for enabled-effect iteration and wet/dry mixing, `apply.rs` for effect-kind dispatch into DSP families, and `sidechain.rs` for compressor sidechain tap lookup; `controls.rs` applies per-track delay/pan/gain controls, `meter.rs` calculates peaks and RMS values, and `delay.rs`, `reverb.rs`, `modulation.rs`, `dynamics.rs`, `stereo.rs`, `gain.rs`, and `shaper.rs` own the corresponding effect families. Stateful DSP data is folder-backed under `src/engine/dsp_state/`: `mod.rs` exposes the narrow runtime-state surface, `effect_key.rs` owns the track/effect lookup key, `effect_runtime.rs` owns per-effect delay/reverb/convolution/modulation/filter/compressor fields, `track_runtime.rs` owns per-track control-delay state, `delay_line.rs` owns circular delay-line samples and cursors, and `history.rs` owns cross-block history sampling/retention. Filter math is folder-backed under `src/engine/filter/`: `mod.rs` exposes only `apply_biquad_filter_block` and `SoundBiquadFilterState`, `state.rs` owns per-effect/per-channel direct-form history, `apply.rs` runs block filtering, `coefficients.rs` maps `SoundFilterEffect` modes to normalized low-pass/high-pass/band-pass/notch biquad coefficients with cutoff and Q clamps, `shelf.rs` owns low-shelf/high-shelf coefficient formulas, and `constants.rs` keeps cutoff, Q, shelf slope, and gain clamps private to the filter boundary.

Source environment processing is folder-backed under `src/engine/source_environment/` after dry source input generation. `mod.rs` is structural and exposes only the narrow `apply_source_environment`, active-listener, and HRTF-tail entry points; `apply.rs` coordinates gain, pan, volume, HRTF, and convolution effects; `listener.rs` owns active listener selection; `spatial/` owns source spatial profile composition, with `profile.rs` combining blend, occlusion, listener-right pan, and child gains, `attenuation.rs` owning attenuation curves, `cone.rs` owning source-cone gain, `doppler.rs` owning preview Doppler gain, and `pan.rs` owning stereo source pan application; `hrtf/` owns source-environment HRTF dispatch, with `mod.rs` exposing only the local HRTF entries, `loaded.rs` resolving active listener profiles into loaded-profile convolution state, `preview.rs` owning deterministic ear-distance gain and delay fallback, and `tail.rs` checking pending loaded-profile FIR tails; `volume/` owns `AudioVolume` policy, with `mod.rs` exposing only the local volume entries, `influence.rs` selecting the strongest priority/id influence and projecting gain, `weight.rs` calculating shape distance and crossfade weight, and `filter.rs` applying volume low-pass blocks; `convolution.rs` owns source and volume convolution sends; and `constants.rs` keeps the shared preview constants out of behavior files. `src/engine/occlusion/` owns the occlusion query DTO, deterministic fallback gain, provider-fed gain entry point, and ray-traced descriptor specificity matching, so spatial source processing can ask one narrow runtime boundary for occlusion attenuation without owning provider cache policy. `src/engine/render/source/orchestration.rs` delegates this responsibility with cloned frame-state snapshots, keeping per-block orchestration separate from the source render root and source-environment math/policy.

Engine validation is folder-backed under `src/engine/validation/`: `mod.rs` exposes the stable `validate_graph`, `validate_effect`, and `track_render_order` entry points; `graph.rs` owns whole-graph validation flow; `track.rs` owns track-control and send checks; `effect.rs` owns effect parameter constraints; `references.rs` owns sidechain track reference policy; `ordering.rs` owns deterministic render dependency sorting; and `values.rs` keeps shared finite/range guards private to validation.

Concrete output support is folder-backed under `src/output/`: `mod.rs` is a structural entry, `catalog.rs` composes backend capability rows and device picker rows, and `descriptor_validation.rs` owns backend-neutral descriptor validation plus backend availability checks. `lifecycle/` owns output-device runtime state and device transitions: `mod.rs` exposes only the lifecycle boundary, `storage.rs` stores `SoundOutputDeviceRuntimeState`, `config.rs` owns reconfiguration and backend-session clearing, `start_stop.rs` owns software/CPAL start and stop paths, `callback.rs` owns rendered-block counters, callback accounting, and unavailable-backend error recording, `status.rs` projects lifecycle state into `SoundOutputDeviceStatus`, and `session.rs` owns the backend-session enum. The sibling `output/status.rs` remains responsible for latency estimation and status diagnostic de-duplication, `software.rs` owns deterministic software output rows, `ring_buffer.rs` owns the bounded realtime FIFO, and `cpal/` owns the optional platform adapter behind `cpal-backend`. Inside `cpal/`, `mod.rs` is structural, `capability.rs` owns backend capability and feature availability rows, `device.rs` owns picker row enumeration, `selection.rs` owns CPAL device and stream-config selection, `session.rs` owns `CpalOutputSession`, `device_thread.rs` and `producer_thread.rs` own the two runtime threads, `callback.rs` owns realtime output draining, `shared_state.rs` owns queue/counter/error state, and `error.rs` maps backend-unavailable details.

`src/service_types/mod.rs` remains the public `DefaultSoundManager` service boundary and only owns child-module wiring plus the curated `DefaultSoundManager`/`SoundDriver` export. `src/service_types/manager_state.rs` owns `SoundDriver`, shared manager state fields, construction, and shared config snapshots. `src/service_types/manager_trait.rs` owns the `SoundManager` trait dispatch implementation and forwards each public contract method into the focused service modules. Clip asset-manager resolution, test clip injection, clip loading, and clip info snapshots now live in `src/service_types/clip_assets.rs`. Playback creation and stopped-playback completion events now live in `src/service_types/playback.rs`; playback pause/resume/toggle, gain/speed/seek, and mute controls now live in `src/service_types/playback_controls.rs`; playback empty checks, playback status snapshots, and finished playback draining now live in `src/service_types/playback_status.rs`; playback settings validation, speed validation, and start/duration range calculation now live in `src/service_types/playback_validation.rs`. Source creation/update/removal and stopped-source completion events now live in `src/service_types/sources.rs`; source pause/resume/toggle, gain/speed, and mute controls now live in `src/service_types/source_controls.rs`; source seek/cursor repositioning now lives in `src/service_types/source_seek.rs`; source empty checks, source status snapshots, range/cursor reporting, and finished source draining now live in `src/service_types/source_status.rs`. External audio source block submission and clearing now live in `src/service_types/external_sources.rs`. Service-level output-device APIs are folder-backed under `src/service_types/output_device/`, where `mod.rs` stays structural, `backend.rs` owns backend naming/status projection, `configuration.rs` owns descriptor configuration and graph-format reset, `lifecycle.rs` owns start/stop calls into the output runtime, `status.rs` owns output-device status snapshots, and `catalog.rs` owns backend/device listing; software output-device block rendering and backend callback pull/reporting behavior now live in `src/service_types/output_render.rs`. Mixer preset discovery/application and rerouting of active playbacks/sources after preset graph replacement now live in `src/service_types/mixer_presets.rs`; service-level mixer graph APIs are folder-backed under `src/service_types/mixer_graph/`, where `mod.rs` stays structural, `configuration.rs` owns full graph import, `snapshot.rs` owns mixer snapshot projection, `tracks.rs` owns track CRUD, `sends.rs` owns send CRUD and validation handoff, and `effects.rs` owns effect CRUD and validation handoff. Service-level dynamic-event APIs are folder-backed under `src/service_types/dynamic_events/`, where `mod.rs` stays structural, `catalog.rs` owns catalog snapshots, registration, unregistering, and dependent cleanup, `handlers.rs` owns handler listing/registration/unregistration, `invocation.rs` owns pending invocation submission/draining, and `dispatch.rs` owns deterministic delivery fan-out. Service-level dynamic-event executor APIs are folder-backed under `src/service_types/dynamic_event_executors/`, where `mod.rs` stays structural, `registration.rs` owns callback registration and handler-existence validation, `unregistration.rs` owns executor removal errors, and `execution.rs` owns delivery dispatch plus per-handler execution report assembly. Sound parameter storage and lookup now live in `src/service_types/parameters.rs`; service-level automation binding/application and automation curve sample calls now live in `src/service_types/automation_timeline.rs`; timeline sequence scheduling, removal, listing, and advancement now live in `src/service_types/timeline_sequences.rs`. Listener and `AudioVolume` registration now live in `src/service_types/acoustics.rs`; static impulse-response lifecycle now lives in `src/service_types/impulse_responses.rs`; ray-tracing convolution status plus ray-traced impulse-response submission/listing/clearing now live in `src/service_types/ray_tracing_convolution.rs`; HRTF profile loading, removal, listing, validation, and HRTF render-state invalidation now live in `src/service_types/hrtf_profiles.rs`. Global volume/default spatial-scale service configuration and direct software `render_mix` now live in `src/service_types/runtime_settings.rs`. This keeps the service root structural instead of owning asset loading, lifecycle, playback controls, playback status reporting, playback validation, source controls, source seek policy, source status reporting, graph mutation, preset replacement, output rendering, event-dispatch, dynamic-event execution, parameter storage, timeline-control, acoustics-state, impulse-response state, ray-tracing convolution state, HRTF profile state, runtime-setting, external-source buffer, manager state, or trait-dispatch behavior directly.

Automation support is folder-backed under `src/automation/`: `binding.rs` validates and normalizes stable timeline binding descriptors through the shared `AnimationTrackPath` parser, `curve.rs` owns curve validation/sampling/interpolation, and `values.rs` provides the shared finite-value guard consumed by service controls and timeline advancement. `src/automation/target/mod.rs` is structural and exposes only the target-application entry point.

`src/automation/target/apply.rs` routes automation targets, clones and validates graph/source/listener/volume descriptors before committing them, and handles synth-parameter mutation; the target child modules for track, source, listener, and volume keep parameter-name mapping separate from service method locking.

Effect target mapping is folder-backed under `src/automation/target/effect/`: `mod.rs` stays structural, `apply.rs` dispatches effect-kind parameter updates, `common.rs` owns enabled/bypass/wet parameters, and the remaining child modules map gain, filter, reverb/convolution, dynamics/limiter, modulation, delay, wave-shaper, and stereo parameters before graph validation commits the change.

Timeline sequence support is folder-backed under `src/timeline/`: `playback.rs` owns scheduled sequence playback state, `schedule.rs` owns sequence insertion/replacement/removal, `validation.rs` owns sequence id, duration, binding, curve, and keyframe-range constraints, and `advance.rs` owns delta validation, loop/completion time resolution, curve sampling, and target application. Service modules and engine state import these direct child owners instead of depending on a flat timeline helper surface.

Descriptor validation is folder-backed under `src/descriptor_validation/`: `source/` owns `AudioSource` descriptor validation, with `mod.rs` exposing only the current source-validation entry points, `tracks.rs` collecting graph-track references and send/output checks, `clip_range.rs` guarding clip-backed start/duration ranges, `input.rs` validating clip/external/synth/silence inputs, `bindings.rs` validating source-to-synth parameter bindings, `spatial.rs` enforcing spatial-setting ranges, and `values.rs` keeping source-local scalar/time guards out of the entry file; `external_source.rs` owns external handle/block validation; `listener.rs` owns listener vector and mixer-target validation; `volume.rs` owns `AudioVolume` gain/filter/shape validation; `hrtf.rs` owns HRTF profile id/sample-rate/kernel validation; and `common.rs` keeps the private finite-vec3 guard shared by listener, source, and volume checks. Service modules and mixer graph import call these direct child modules instead of importing a flat descriptor helper surface. Current source descriptor validation boundary coverage keeps graph-track references, clip ranges, input variants, parameter bindings, spatial settings, and source-local scalar/time values covered through source input, spatial, graph import, and full runtime tests.

Dynamic event support is folder-backed under `src/dynamic_events/`: `catalog.rs` owns event descriptor catalog validation plus event registration/removal, `handlers.rs` owns handler validation and handler list mutation, `invocation.rs` owns invocation validation and pending queue submission, and `dispatch.rs` owns deterministic handler ordering and delivery fan-out. Service modules and mixer graph import call these direct child modules instead of importing a flat dynamic-event helper surface.

Dynamic event ABI projection is folder-backed under `src/dynamic_event_abi/`: `callback.rs` owns the `DefaultSoundManager` ABI callback registration entry point, `executor.rs` adapts callbacks into the existing deterministic executor path, `request.rs` projects `SoundDynamicEventDelivery` into `ZrPluginEventCallbackRequestV1`, `slice.rs` owns borrowed byte-slice construction, and `status.rs` maps ABI status/diagnostic slices into handler failure detail. The neutral runtime-interface crate still owns only generic `ZrPluginEvent*` ABI DTOs; sound-specific projection remains in the Sound runtime plugin.

Mixer graph configuration is folder-backed under `src/mixer_configuration/`: `configure.rs` owns full graph import orchestration and validation order, `sources.rs` rebuilds graph-owned source voices and source IDs, `automation.rs` rebuilds validated and normalized automation binding tables, `dynamic_events.rs` prunes handlers, executors, and pending invocations against the imported event catalog, `timeline.rs` removes scheduled sequences whose bindings no longer exist, and `runtime_state.rs` clears graph-dependent DSP, track, HRTF, meter, and latency runtime state. `src/service_types/mixer_graph/configuration.rs` imports `configure::configure_mixer_graph` directly, so the old flat graph-import helper surface is gone.

Ray-traced convolution provider support is folder-backed under `src/ray_tracing/`: `provider.rs` owns provider-fed impulse-response submission and clearing, `status.rs` owns cached-cell/ray-count status refresh, and `validation.rs` owns ray-tracing status plus provider descriptor validation against current source, listener, and volume state. Service modules import these direct child modules instead of depending on a flat ray-tracing helper file.

## Test Coverage

`src/tests/manifest.rs` keeps static and generated metadata in sync. It checks option keys plus option labels, value types, default values, and required capability gates, runtime module contributions, dependency rows, event catalogs, component descriptors, and verifies that static TOML, runtime descriptor, generated package manifest, and built-in runtime catalog agree on maturity and capability status. `src/tests/runtime_core.rs` verifies that neutral `SoundPluginOptions` values are preserved into `SoundConfig` and that `DefaultSoundManager::with_config(...)` uses those values for runtime mix format, global volume, and spatial scale. `src/tests/optional_feature_manifest.rs` owns the static-to-generated optional feature bundle parity for `sound.timeline_animation_track` and `sound.ray_traced_convolution_reverb`, including dependency rows, runtime/editor module rows, capabilities, default packaging strategies, and enabled-by-default flags. The provider crates add local unit contracts in `zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs` and `zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs` so linked feature registrations keep the same id, display name, owner, dependency, capability, packaging, and module metadata as the Sound owner bundle. The broader runtime test tree covers graph routing, DSP state, spatial/HRTF behavior, ray-traced impulse-response provider input, dynamic events, output-device behavior, presets, source lifecycle, automation, and manifest parity.

`src/tests/dynamic_events.rs` owns dynamic-event boundary coverage: a static folder-backed contract for `src/dynamic_events/`, `src/dynamic_event_abi/`, `src/service_types/dynamic_events/`, `src/service_types/dynamic_event_executors/`, and `src/engine/state/dynamic_events.rs`; registry validation; pending invocation queue behavior; deterministic handler fan-out; executor registration/removal; execution success/failure/missing-executor reports; and ABI callback request/status mapping. This keeps the dynamic-event service root structural while still testing the current local runtime execution path.

Focused validation after adding the dynamic-event folder-backed contract passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-dynamic-events-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events.rs` passed. The first cold-target `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` attempt timed out after 20 minutes during dependency compilation without Rust diagnostics; exact residual processes for that target directory were stopped. The warmed retry passed with 11 dynamic-event tests, 0 failures, and 87 filtered tests. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` also passed; remaining output was limited to existing `zircon_runtime` warnings.

Focused validation after adding optional feature bundle manifest parity passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-optional-feature-manifest`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/manifest.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs zircon_plugins/sound/runtime/src/tests/dynamic_events.rs` passed. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --offline --jobs 1 --message-format short --color never` passed with 4 manifest tests, 0 failures, and 95 filtered tests after the optional feature parser was split out of the oversized manifest test file. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` also passed; remaining output was limited to existing `zircon_runtime` warnings.

`src/tests/playback.rs` now owns the Bevy-inspired source and playback lifecycle regression cases that were previously embedded in the large runtime test aggregate: source speed/mute controls, sink-style source controls, start/duration ranges, cleanup intent, playback presets, invalid initial mix parameters, pause/resume/mute/speed status, seek/range handling, and finished playback reports. Keeping these tests folder-backed makes future playback work independent from mixer graph, DSP, spatial, and manifest coverage.

`src/tests/source_inputs.rs` now owns source-input regression coverage for external audio blocks, invalid or missing external handles, clip/external resampling to mixer rate, and synth-parameter source bindings. This keeps source ingestion coverage separate from playback lifecycle, mixer graph, DSP, and spatial tests.

`src/tests/automation_binding.rs` now owns synth-parameter visibility and automation binding coverage that used to live in the runtime test aggregate: snapshot visibility for bound synth parameters, shared animation-track path normalization, automation value application to synth, track, and effect targets, and typed failures for invalid target paths or missing targets. `src/tests/automation_curve.rs` owns automation curve sampling, keyframe validation, one-shot timeline sequence advancement, and looping timeline behavior, so binding target resolution can evolve independently from curve sampling and timeline scheduling behavior.

`src/tests/runtime_core.rs` now owns runtime-plugin registration and default manager baseline coverage that used to live in the root test module: runtime module/component/option/event contribution, silent render format defaults, and final global-volume gain validation. `src/tests.rs` now remains a navigation and shared-fixture module instead of owning behavioral assertions.

Focused validation after tightening the Timeline binding path contract passed on 2026-05-31 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-timeline-path-contract`. `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime -- --check` passed. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml automation --locked --offline --jobs 1 --color never` passed with 10 automation/Timeline/graph-import tests, 0 failed, and 92 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml graph_config --locked --offline --jobs 1 --color never` passed with 2 graph-import tests, 0 failed, and 100 unrelated tests filtered out. `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` and `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1` passed. Remaining output was limited to existing `zircon_runtime` warnings.

Focused validation after adding Sound optional feature editor-module capability parity passed on 2026-05-31 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-feature-editor-capability`. `rustfmt --edition 2021 --check` passed for the touched Sound feature provider/editor files, Sound runtime feature-manifest file, and runtime catalog/test files. The first provider run intentionally failed before the fix because `sound.timeline_animation_track.editor` had no `editor.feature.sound.timeline_animation_track` capability. After the fix, `cargo test --manifest-path zircon_plugins\sound\features\timeline_animation_track\runtime\Cargo.toml timeline_feature_provider_manifest_matches_sound_owner_contract --locked --offline --jobs 1 --color never` and `cargo test --manifest-path zircon_plugins\sound\features\ray_traced_convolution_reverb\runtime\Cargo.toml ray_traced_feature_provider_manifest_matches_sound_owner_contract --locked --offline --jobs 1 --color never` each passed with 1 test. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml manifest --locked --offline --jobs 1 --color never` passed with 5 manifest tests, including static optional feature parity. `cargo test --manifest-path Cargo.toml -p zircon_runtime builtin_sound_optional_features_declare_editor_capabilities --locked --offline --jobs 1 --color never` passed with 1 focused runtime catalog test. `cargo check` passed for both feature editor crates, and `cargo metadata --locked --offline --no-deps --format-version 1` passed for the Sound runtime plus both feature runtime/editor manifests. Remaining output was limited to existing `zircon_runtime` and `zircon_editor` warnings.

`src/tests/convolution.rs` now owns static convolution and impulse-response lifecycle coverage that used to live in the root test module: master-track static IR processing, static IR cache invalidation when an impulse response is removed, and ray-tracing convolution status validation. Provider-fed ray-traced IR submission and occlusion cases remain in `src/tests/ray_tracing.rs`.

`src/tests/common.rs` owns shared Sound test fixtures after the root test module cleanup: mono clip asset construction, default listener construction, effect descriptor construction, and near-equality sample assertions. `src/tests.rs` is now only the folder-backed test module index plus a re-export of those helpers for child modules.

`src/tests/spatial.rs` now owns HRTF profile behavior plus the remaining spatial source, spatial scale, AudioVolume, and pre-spatial source-send coverage that used to live in the runtime test aggregate. Keeping these tests together makes spatial attenuation, occlusion, HRTF, scale overrides, volume crossfade, and pre-spatial send behavior visible as one sound-domain test boundary.

`src/tests/mixer_graph.rs` now owns mixer graph and routing regression coverage that used to live in the runtime test aggregate: custom track routing through effect chains, track removal rerouting active playbacks, parent/send cycle rejection, send CRUD, solo routing, and sidechain pre/post-effect tap behavior. Keeping these tests together makes graph mutation and render routing failures visible without expanding the root test module.

`src/tests/dsp_state.rs` now owns both stateful DSP regression coverage and the deterministic single-block DSP effect checks that used to live in the runtime test aggregate: bypass/wet-dry behavior, delay, pan/phase, limiter, filter, reverb, waveshaper, flanger, phaser, chorus, state continuity, latency snapshots, parameter validation, and sidechain reference validation. Keeping these checks in one module makes DSP failures local to the effect/state boundary instead of mixing them with source, graph, spatial, or automation tests.

After this boundary extraction, focused validation on 2026-05-26 passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: 13 related tests passed, 0 failed, and 84 unrelated tests were filtered out. The run emitted only existing `zircon_runtime` warnings.

Focused validation after the source-input extraction passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` with the same target directory: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out. The run emitted only existing `zircon_runtime` warnings.

Focused validation after the spatial extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 13 spatial-related tests passed, 0 failed, and 84 unrelated tests were filtered out. One earlier cold-target attempt in the same target directory exited during dependency compilation at `unicode-bidi` with rustc exit code `1073807364` and no Sound diagnostics; the warmed retry completed and is the accepted evidence.

Focused validation after the mixer graph extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 8 mixer graph tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after the DSP extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 14 DSP tests passed, 0 failed, and 83 unrelated tests were filtered out.

Focused validation after replacing `src/engine/dsp/effects.rs` with folder-backed `src/engine/dsp/effects/` chain orchestration, effect-kind dispatch, and sidechain lookup modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-dsp-effect-chain-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after rustfmt import ordering. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` passed with 14 DSP-state tests, 0 failures, and 83 filtered. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 mixer graph tests, 0 failures, and 89 filtered. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after the automation binding extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 4 automation-binding-related tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving sound parameter storage/lookup into `src/service_types/parameters.rs` and timeline sequence operations into `src/service_types/timeline_sequences.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 automation-binding-related tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after the same parameter/timeline extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 automation-curve/timeline tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after replacing the flat `src/automation.rs` helper with folder-backed `src/automation/` binding, curve, value, and target modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 automation-binding/graph-import tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after the same automation helper extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 automation-curve/timeline tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after moving automation target dispatch out of `src/automation/target/mod.rs` and into `src/automation/target/apply.rs` passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-automation-target-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. After an earlier cold `cargo check` attempt timed out after 10 minutes before diagnostics, the warmed `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` rerun passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests, 0 failures, and 93 filtered. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests, 0 failures, and 92 filtered. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing `src/automation/target/effect.rs` with folder-backed `src/automation/target/effect/` effect-kind dispatch and per-effect parameter modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-automation-effect-target-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after applying rustfmt to the new files. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` first exposed that the new internal effect entry was too private for sibling `target/apply.rs`; after narrowing visibility to `crate::automation::target`, the accepted rerun passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests, 0 failures, and 93 filtered. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests, 0 failures, and 92 filtered. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after converting `src/service_types.rs` into folder-backed `src/service_types/mod.rs` and moving concrete manager state into `src/service_types/manager_state.rs` passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-types-root-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` passed with 3 tests, 0 failures, and 94 filtered. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat `src/descriptor_validation.rs` helper with folder-backed `src/descriptor_validation/` descriptor-domain modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input/external-source tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after the same descriptor-validation extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 spatial/listener/volume/HRTF tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same descriptor-validation extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 2 graph-source import/validation tests passed, 0 failed, and 95 unrelated tests were filtered out.

Focused validation after the root runtime-core extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after the convolution extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 5 convolution-related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the common fixture extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml common --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 0 tests matched the `common` filter because it is helper-only, and the crate compiled successfully with 97 tests filtered out.

Focused validation after the render math extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 12 render-related tests passed, 0 failed, and 85 unrelated tests were filtered out.

Focused validation after the source environment extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`: 13 spatial-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the manager playback/source lifecycle extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source lifecycle related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same service lifecycle extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving mixer preset discovery/application and active source/playback rerouting into `src/service_types/mixer_presets.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml presets --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 preset-related tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after the same mixer-preset extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 mixer graph tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after moving external audio source block lifecycle into `src/service_types/external_sources.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving playback settings validation and start/duration range calculation into `src/service_types/playback_validation.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same playback validation extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving playback pause/resume/toggle, gain/speed/seek, and mute controls into `src/service_types/playback_controls.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-control related tests passed, 0 failed, and 84 unrelated tests were filtered out. An earlier attempt with the same command timed out after 10 minutes while dependency compilation was still active under concurrent workspace jobs and emitted no Sound test failure; the longer warmed retry completed successfully.

Focused validation after moving source status snapshots and finished-source draining into `src/service_types/source_status.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source lifecycle related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same source-status extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving source pause/resume/toggle, gain/speed/seek, and mute controls into `src/service_types/source_controls.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source-control related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after moving source seek/cursor repositioning into `src/service_types/source_seek.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback/source-seek related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after the same source-control extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out.

Focused validation after moving playback status snapshots and finished-playback draining into `src/service_types/playback_status.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after moving clip asset-manager access, test clip injection, clip loading, and clip info snapshots into `src/service_types/clip_assets.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after moving the `SoundManager` trait dispatch boundary into `src/service_types/manager_trait.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after the manager output-device extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after moving software output-device block rendering and backend callback pull/reporting into `src/service_types/output_render.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after the manager mixer graph extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 mixer graph tests passed, 0 failed, and 89 unrelated tests were filtered out. Two earlier attempts in the same target directory timed out while compiling `zircon_runtime`; process inspection showed the Sound cargo/rustc jobs were still compiling rather than running a stuck Sound test binary, and the warmed retry completed successfully.

Focused validation after replacing `src/service_types/mixer_graph.rs` with folder-backed `src/service_types/mixer_graph/` configuration, snapshot, track, send, and effect service modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-mixer-graph-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a rustfmt import adjustment. The first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` exposed that the moved service methods were too private for sibling `manager_trait.rs`; the accepted rerun passed after narrowing visibility to `crate::service_types`, with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 mixer graph tests, 0 failed, and 89 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 graph import tests, 0 failed, and 95 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after the manager dynamic-events extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 10 dynamic-event tests passed, 0 failed, and 87 unrelated tests were filtered out.

Focused validation after moving dynamic-event executor registration and execution report assembly out of the broader service surface passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 10 dynamic-event tests passed, 0 failed, and 87 unrelated tests were filtered out.

Focused validation after replacing the flat `src/dynamic_events.rs` helper with folder-backed `src/dynamic_events/` catalog, handler, invocation, and dispatch modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 10 dynamic-event tests passed, 0 failed, and 87 unrelated tests were filtered out.

Focused validation after replacing the flat timeline helper with folder-backed `src/timeline/` playback, schedule, validation, and advance modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 automation-curve/timeline tests passed, 0 failed, and 92 unrelated tests were filtered out. An earlier attempt stopped before Sound tests executed because the active Material property-value reporting session temporarily imported `RenderMaterialPropertyValueState` from `core::framework::render` before that type was exported; the owning session later cleared the external compile gap and the rerun completed.

Focused validation after moving runtime-plugin descriptor, feature-bundle, and registration behavior out of `src/lib.rs` into `src/runtime_plugin/` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 manifest/plugin metadata tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after replacing the flat `src/package.rs` helper with folder-backed `src/package/` attachment, dependency, option, and event-catalog modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 manifest/plugin metadata tests passed, 0 failed, and 94 unrelated tests were filtered out.

Focused validation after replacing the flat `src/mixer_configuration.rs` helper with folder-backed `src/mixer_configuration/` graph-import, source, automation, dynamic-event, timeline, and runtime-state configuration modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 2 graph-import tests passed, 0 failed, and 95 unrelated tests were filtered out. The same slice also passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never`: 8 mixer-graph tests passed, 0 failed, and 89 unrelated tests were filtered out. An earlier validation attempt ran both filtered commands concurrently against the same target directory and timed out after 604 seconds; process inspection afterward showed no remaining Sound target cargo/rustc processes and active compiler work belonged to other Editor, Hub, and Material sessions, so the accepted evidence is the serial rerun.

Focused validation after replacing the flat `src/ray_tracing.rs` helper with folder-backed `src/ray_tracing/` provider, status, and validation modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 ray-tracing/convolution-status tests passed, 0 failed, and 92 unrelated tests were filtered out. The same slice also passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never`: 5 convolution/ray-tracing cache tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after replacing the flat `src/dynamic_event_abi.rs` helper with folder-backed `src/dynamic_event_abi/` callback, executor, request, slice, and status modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 10 dynamic-event and ABI callback tests passed, 0 failed, and 87 unrelated tests were filtered out.

Focused validation after replacing the flat `src/presets.rs` helper with folder-backed `src/presets/` locator, catalog, and built-in graph modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml presets --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 4 preset-related tests passed, 0 failed, and 93 unrelated tests were filtered out. The same slice also passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never`: 8 mixer-graph tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after replacing the flat render helper with folder-backed `src/engine/render/` orchestration, playback, source, routing, runtime-state, and sampling modules passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 12 render-related tests passed, 0 failed, and 85 unrelated tests were filtered out.

Focused validation after replacing the flat source-environment helper with folder-backed `src/engine/source_environment/` listener, spatial, HRTF, volume, convolution, and constants modules passed on 2026-05-29 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 spatial/HRTF/volume tests passed, 0 failed, and 84 unrelated tests were filtered out. The same slice passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never`: 5 ray-tracing/convolution-status tests passed, 0 failed, and 92 unrelated tests were filtered out. The full Sound runtime rerun also passed with the same target directory: 97 runtime tests passed, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and the existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after moving output lifecycle, catalog, descriptor validation, and status diagnostics out of `src/output/mod.rs` passed on 2026-05-29 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out. The same slice also passed the CPAL feature variant `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --features cpal-backend --locked --jobs 1 --message-format short --color never`: 12 output-device and CPAL callback tests passed, 0 failed, and 89 unrelated tests were filtered out.

Focused validation after the CPAL adapter folder extraction passed on 2026-05-29 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --features cpal-backend --locked --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 12 output-device and CPAL callback tests passed, 0 failed, and 89 unrelated tests were filtered out. The same slice also passed the non-CPAL output-device path with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out. The full runtime rerun passed with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never`: 97 runtime tests passed, 0 failed, and doctests had no failures.

Focused validation after replacing the flat DSP executor with folder-backed `src/engine/dsp/` effect-family modules passed on 2026-05-29 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 14 DSP state/effect tests passed, 0 failed, and 83 unrelated tests were filtered out. The same slice also passed `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and the full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never`: 97 runtime tests passed, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and the existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat DSP state helper with folder-backed `src/engine/dsp_state/` effect-key, effect-runtime, track-runtime, delay-line, and history modules passed on 2026-05-29 with `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-dsp-state-boundary`. The same slice passed `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never`; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never`: 14 DSP state/effect tests passed, 0 failed, and 83 unrelated tests were filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never`: 97 runtime tests passed, 0 failed, and doctests had no failures. The first `cargo check` after the move exposed that `SoundDelayLineState` and `SoundHistoryState` were too private for sibling DSP modules; the accepted rerun passed after restoring their crate-internal visibility without adding an external API. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat filter helper with folder-backed `src/engine/filter/` state, apply, coefficient, shelf, and constant modules ran on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed, and `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed after the initial private coefficient re-export was removed from `filter/mod.rs`. The standard Cargo test rerun later stopped before Sound tests because active Texture work introduced `zircon_runtime/src/asset/assets/texture/upload_support.rs:611` type errors, so the accepted Sound-only runtime evidence is the generated Sound test executable from this slice: direct `dsp_state` execution passed 14 tests, 0 failed, 83 filtered, and direct full runtime execution passed 97 tests, 0 failed. Earlier `cargo test dsp_state` attempts timed out under parallel workspace compilation or stopped on the low-space `E:` target before the `D:` target rerun.

Focused validation after replacing the flat HRTF runtime helper with folder-backed `src/engine/hrtf/` key, state, apply, and prune modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. An earlier cold `D:\cargo-targets\zircon-sound-hrtf-boundary` check timed out under parallel workspace compilation before Rust diagnostics. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat occlusion helper with folder-backed `src/engine/occlusion/` query, constants, gain, and ray-traced matching modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a single rustfmt import-line adjustment; `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 ray-tracing/provider tests, 0 failed, and 92 unrelated tests filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after moving `apply_source_environment` orchestration out of `src/engine/source_environment/mod.rs` and into `src/engine/source_environment/apply.rs` passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 ray-tracing/provider tests, 0 failed, and 92 unrelated tests filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. The earlier parallel spatial/ray-tracing attempt timed out during active compilation before Sound test diagnostics; the accepted evidence is the serial rerun. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat source-environment HRTF helper with folder-backed `src/engine/source_environment/hrtf/` loaded-profile dispatch, preview fallback, and tail-query modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; the first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` exposed that the moved loaded/preview entries were too private for sibling `apply.rs`, and the accepted rerun passed after narrowing their visibility to the source-environment boundary. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 ray-tracing/provider tests, 0 failed, and 92 unrelated tests filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat source-environment spatial helper with folder-backed `src/engine/source_environment/spatial/` profile, attenuation, cone, Doppler, and pan modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; the first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` exposed that the moved profile and pan entries were too private for sibling `apply.rs`, and the accepted rerun passed after narrowing their visibility to the source-environment boundary. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 ray-tracing/provider tests, 0 failed, and 92 unrelated tests filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat source-environment volume helper with folder-backed `src/engine/source_environment/volume/` influence selection, shape/crossfade weight, and low-pass filter modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 ray-tracing/provider tests, 0 failed, and 92 unrelated tests filtered out. The full Sound runtime `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings; the parallel focused-test start briefly waited on package and artifact locks before both commands completed.

Focused validation after replacing the flat source descriptor validation helper with folder-backed `src/descriptor_validation/source/` modules passed on 2026-05-29 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed; the first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` exposed that `validate_source_descriptor_for_tracks` was unnecessarily re-exported from `source/mod.rs`, and the accepted rerun passed after tightening it to a private child helper. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 source-input tests, 0 failed, and 93 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 graph-source import tests, 0 failed, and 95 unrelated tests filtered out; and the full Sound runtime command passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat engine validation helper with folder-backed `src/engine/validation/` graph, track, effect, reference, ordering, and value modules passed on 2026-05-29 with `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`. The same slice passed `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never`: 2 graph import tests passed, 0 failed, and 95 unrelated tests were filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never`: 8 mixer graph/routing tests passed, 0 failed, and 89 unrelated tests were filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never`: 14 DSP validation/state tests passed, 0 failed, and 83 unrelated tests were filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and the existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat render-source helper with folder-backed `src/engine/render/source/` orchestration, input, external-block, parameter-binding, and clip-range modules passed on 2026-05-29 with `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-render-source-boundary`. The same slice passed `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never`; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never`: 4 source-input tests passed, 0 failed, and 93 unrelated tests were filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never`: 13 playback-related tests passed, 0 failed, and 84 unrelated tests were filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never`: 13 spatial/HRTF/volume tests passed, 0 failed, and 84 unrelated tests were filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Earlier source-input attempts timed out during active parallel workspace compilation before Sound diagnostics; process inspection found no residual Sound target work before the accepted isolated rerun. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after moving block-level render orchestration out of `src/engine/render/mod.rs` and into `src/engine/render/orchestration.rs` passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-root-orchestration`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` passed with 12 render/DSP/HRTF-tail tests, 0 failed, and 85 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 mixer graph tests, 0 failed, and 89 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after moving explicit source render orchestration out of `src/engine/render/source/mod.rs` and into `src/engine/render/source/orchestration.rs` passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-source-orchestration`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. The first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` exposed that the moved `mix_sources` method was too private for the render root; the accepted rerun passed after narrowing visibility to `crate::engine::render`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 source-input tests, 0 failed, and 93 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 spatial/HRTF/volume tests, 0 failed, and 84 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat render sampling helper with folder-backed `src/engine/render/sampling/` resample-step, source cursor/range position, interpolation, and frame/channel-folding modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 source-input tests, 0 failed, and 93 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` passed with 13 playback tests, 0 failed, and 84 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat render playback helper with folder-backed `src/engine/render/playback/` active-playback routing, clip block sampling, pan/gain projection, and finished-playback reporting modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a rustfmt import-order adjustment. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` passed with 13 playback tests, 0 failed, and 84 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 source-input tests, 0 failed, and 93 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` passed with 12 render/DSP/HRTF-tail tests, 0 failed, and 85 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat service output-device helper with folder-backed `src/service_types/output_device/` backend status, catalog, configuration, lifecycle, and status modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. The first focused `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` attempt timed out while a residual cargo/rustc process was still compiling `zircon_runtime`; process inspection showed the command had not reached the Sound test binary. The warmed rerun passed with 8 output-device tests, 0 failed, and 89 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat service dynamic-event helper with folder-backed `src/service_types/dynamic_events/` catalog, handler, invocation, and dispatch modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` passed with 10 dynamic-event tests, 0 failed, and 87 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat service dynamic-event executor helper with folder-backed `src/service_types/dynamic_event_executors/` registration, unregistration, and execution modules passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with existing `zircon_runtime` warnings only. The first focused `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` attempt stopped during cold compilation before the Sound test binary ran; process inspection showed an unrelated editor cargo job active in a separate target directory. The warmed rerun passed with 11 dynamic-event tests, 0 failed, and 87 unrelated tests filtered out. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 98 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat engine state helper with folder-backed `src/engine/state/` storage, graph, snapshot, dynamic-event executor, playback, and source voice modules passed on 2026-05-29 with `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-engine-state-boundary`. The same slice passed `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never`; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never`: 2 graph import tests passed, 0 failed, and 95 unrelated tests were filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never`: 10 dynamic-event state/executor tests passed, 0 failed, and 87 unrelated tests were filtered out; and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never`: 97 runtime tests passed, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after replacing the flat output lifecycle helper with folder-backed `src/output/lifecycle/` storage, config, start/stop, callback, status, and session modules passed on 2026-05-29 with `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-output-lifecycle-boundary`. The same slice passed `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never`; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never`: 8 output-device tests passed, 0 failed, and 89 unrelated tests were filtered out; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --features cpal-backend --locked --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-output-lifecycle-cpal`: 12 output-device and CPAL callback tests passed, 0 failed, and 89 unrelated tests were filtered out; and the full Sound runtime command passed with 97 runtime tests, 0 failed, and doctests had no failures. Remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings.

Focused validation after the manager automation/timeline extraction was attempted on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`. The command stopped while compiling `zircon_runtime` before Sound tests executed because active UI accessibility work still exposes `append_binding_report_diagnostic` as private while re-exporting/importing it across sibling action modules. The intended follow-up focused commands for this slice are the same `automation_binding` command and `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` once that external compile blocker is cleared.

Focused validation after the manager acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 spatial/HRTF/volume related tests passed, 0 failed, and 84 unrelated tests were filtered out. The first cold attempt timed out during dependency compilation while other workspace validation jobs were active; the warmed retry completed successfully.

Focused validation after moving HRTF profile loading/removal/listing into `src/service_types/hrtf_profiles.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 13 spatial/HRTF related tests passed, 0 failed, and 84 unrelated tests were filtered out.

Focused validation after moving static impulse-response lifecycle into `src/service_types/impulse_responses.rs` and ray-tracing convolution status/provider-fed impulse-response operations into `src/service_types/ray_tracing_convolution.rs` passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 convolution/ray-tracing status tests passed, 0 failed, and 92 unrelated tests were filtered out. An earlier attempt stopped during `zircon_runtime` compilation because concurrent material standard-texture summary work had temporarily left new `RenderMaterialReadinessReport` fields unapplied in a few initializers; that external compile gap was resolved by its owning session before the retry.

Focused validation after the same acoustic-response extraction passed on 2026-05-28 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 ray-tracing impulse-response tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the same acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 convolution and impulse-response related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the same acoustics extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 5 ray-tracing impulse-response related tests passed, 0 failed, and 92 unrelated tests were filtered out.

Focused validation after the manager runtime-settings extraction passed on 2026-05-27 with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` and `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 3 runtime-core tests passed, 0 failed, and 94 unrelated tests were filtered out.

Fresh validation on 2026-05-26 passed the full runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-direct`: 97 runtime tests passed, 0 failed, and doctests had no failures. The command was rerun after the playback and source-input test-boundary extractions with the same 97 passed / 0 failed result, rerun again on 2026-05-27 after the spatial extraction with `CARGO_TARGET_DIR=E:\Git\ZirconEngine\target\codex-sound-spatial`, rerun after the mixer graph extraction, rerun after the DSP extraction, rerun after the automation binding extraction, rerun after the runtime-core/convolution root extraction, rerun after the common fixture extraction, rerun after the render math extraction, and rerun after the source environment extraction with the same target directory: 97 runtime tests passed, 0 failed, and doctests had no failures. The full runtime command was then rerun after the manager playback/source lifecycle extraction, manager output-device extraction, manager mixer graph extraction, manager mixer-preset extraction, manager acoustics extraction, manager runtime-settings extraction, manager trait-dispatch extraction, manager external-source extraction, manager playback-validation extraction, manager playback-control extraction, manager source-status extraction, manager source-control extraction, manager playback-status extraction, manager clip-assets extraction, manager output-render extraction, manager dynamic-event-executor extraction, manager parameter/timeline extraction, manager source-seek extraction, manager HRTF-profile extraction, manager acoustic-response extraction, automation helper folder extraction, descriptor-validation folder extraction, dynamic-event helper folder extraction, timeline folder extraction, runtime-plugin folder extraction, package-manifest folder extraction, mixer-configuration folder extraction, ray-tracing folder extraction, dynamic-event ABI folder extraction, preset catalog folder extraction, render folder extraction, source-environment folder extraction, output root extraction, CPAL adapter folder extraction, DSP folder extraction, and engine-validation folder extraction with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-service-lifecycle`: 97 runtime tests passed, 0 failed, and doctests had no failures. The full runtime command was rerun again after the render-source folder extraction with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-render-source-boundary`: 97 runtime tests passed, 0 failed, and doctests had no failures. The full runtime command was rerun again after the engine-state folder extraction with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-engine-state-boundary`: 97 runtime tests passed, 0 failed, and doctests had no failures. The full runtime command was rerun again after the output-lifecycle folder extraction with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-output-lifecycle-boundary`: 97 runtime tests passed, 0 failed, and doctests had no failures. An intermediate full runtime rerun after the manager dynamic-events extraction had stopped in unrelated active UI accessibility work because `append_binding_report_diagnostic` was private to the `action::result` child module while being re-exported/imported across sibling action modules; the later full Sound runtime reruns now cover the Sound manager service-root split, render-source folder boundary, engine-state folder boundary, and output-lifecycle folder boundary. The earlier app/provider command `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" first_party_sound_provider_preserves_manifest_maturity_and_capability_status -- --nocapture --test-threads=1` passed for the linked first-party provider path and proves Sound maturity, capability status, module, option, and dynamic-event catalog metadata stay preserved through app bootstrap.
