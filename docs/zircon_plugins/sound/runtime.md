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
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/gain.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/mute.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/pause.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/speed.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/state_access.rs
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
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/effect.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/synth_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/track.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/missing_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unknown_source.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unsupported_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/bound_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/step_clamping.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/scheduling_validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/wraparound.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_once.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/validation.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/common/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/common/assets.rs
  - zircon_plugins/sound/runtime/src/tests/common/effects.rs
  - zircon_plugins/sound/runtime/src/tests/common/listener.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/ray_status.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/static_ir.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/capture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/detail.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/delivery.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/ordering.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/queue.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/event_cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/ownership.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/event_unregister.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/graph_reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/descriptors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/executor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/missing_handler.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/calls.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/outcomes.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/executors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/schema.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/time.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unknown_event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/event_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/executor_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/state.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/source.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/wiring.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/chain_controls.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/dynamics.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/chorus.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/flanger.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/phaser.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/shaper.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/stereo.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/high_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/low_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/shelf.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/latency.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/flanger_history.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/phaser_phase.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/support.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/parameters.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/import.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/validation.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/event_catalogs.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/line.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/capability_statuses.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/maturity.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/keys.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/parser.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/projection.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/section.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/state.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/types.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/backends.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/disabled.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/windows_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/configured_pull.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/stopped_callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/unsupported_backend.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/validation.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/effect_chain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/track_removal.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/removal_errors.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/upsert_snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/cycles.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/taps.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/solo.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/support.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/playback/completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/gain_mute.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/initial_state.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/speed_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/transport.rs
  - zircon_plugins/sound/runtime/src/tests/playback/range.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/presets.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/completed.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/stopped.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_range.rs
  - zircon_plugins/sound/runtime/src/tests/presets.rs
  - zircon_plugins/sound/runtime/src/tests/presets/catalog_apply.rs
  - zircon_plugins/sound/runtime/src/tests/presets/reroute.rs
  - zircon_plugins/sound/runtime/src/tests/presets/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/cache.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/provider_status.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/cell_key.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/ray_count.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/source.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_normalization.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/manager_projection.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/option_values.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/support.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/manager_projection.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/option_values.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/support.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/manager_projection.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/option_values.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options/support.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/global_volume.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/components.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/modules.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/components.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/modules.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/components.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/modules.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration/options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/render_defaults.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/kernels.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/tail_state.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/parameter_playback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/ear_delay.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/fallback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/tail.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/validation.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/listener.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/default.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/source_override.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/sends.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/support.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/volumes.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/clearing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/missing_blocks.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/validation.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_routing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/parameter_bindings.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/resampling.rs
  - zircon_runtime/src/core/framework/sound/channel_layout.rs
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
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/gain.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/mute.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/pause.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/seek.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/speed.rs
  - zircon_plugins/sound/runtime/src/service_types/playback_controls/state_access.rs
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
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/effect.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/synth_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/track.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/missing_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unknown_source.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unsupported_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/bound_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/step_clamping.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/scheduling_validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/wraparound.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_once.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/validation.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/common/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/common/assets.rs
  - zircon_plugins/sound/runtime/src/tests/common/effects.rs
  - zircon_plugins/sound/runtime/src/tests/common/listener.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/ray_status.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/static_ir.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/capture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/detail.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/delivery.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/ordering.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/queue.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/event_cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/ownership.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/event_unregister.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/graph_reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/descriptors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/executor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/missing_handler.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/calls.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/outcomes.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/executors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/schema.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/time.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unknown_event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/event_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/executor_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/state.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/source.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/wiring.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/chain_controls.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/dynamics.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/chorus.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/flanger.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/phaser.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/shaper.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/stereo.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/high_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/low_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/shelf.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/latency.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/flanger_history.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/phaser_phase.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/support.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/parameters.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/import.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/validation.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/event_catalogs.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/line.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/capability_statuses.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/maturity.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/keys.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/parser.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/projection.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/section.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/state.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/types.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/backends.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/disabled.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/windows_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/configured_pull.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/stopped_callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/unsupported_backend.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/validation.rs
  - zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs
  - zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/effect_chain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/track_removal.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/removal_errors.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/upsert_snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/cycles.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/taps.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/solo.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/support.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/playback/completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/gain_mute.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/initial_state.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/speed_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/transport.rs
  - zircon_plugins/sound/runtime/src/tests/playback/range.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/presets.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/completed.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/stopped.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_range.rs
  - zircon_plugins/sound/runtime/src/tests/presets.rs
  - zircon_plugins/sound/runtime/src/tests/presets/catalog_apply.rs
  - zircon_plugins/sound/runtime/src/tests/presets/reroute.rs
  - zircon_plugins/sound/runtime/src/tests/presets/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/cache.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/provider_status.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/cell_key.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/ray_count.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/source.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_normalization.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/global_volume.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/render_defaults.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/kernels.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/tail_state.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/parameter_playback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/ear_delay.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/fallback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/tail.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/validation.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/listener.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/default.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/source_override.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/sends.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/support.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/volumes.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/clearing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/missing_blocks.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/validation.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_routing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/parameter_bindings.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/resampling.rs
plan_sources:
  - user: 2026-05-25 继续完善插件工作流以及sound插件作为样例完善
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/Sound 插件核心完善计划.md
tests:
  - zircon_plugins/sound/runtime/src/tests.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/effect.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/synth_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/apply/track.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/missing_binding.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/path.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unknown_source.rs
  - zircon_plugins/sound/runtime/src/tests/automation_binding/validation/unsupported_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/bound_parameter.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/sampling/step_clamping.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/scheduling_validation.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_loop/wraparound.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/timeline_once.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve/validation.rs
  - zircon_plugins/sound/runtime/src/tests/common.rs
  - zircon_plugins/sound/runtime/src/tests/common/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/common/assets.rs
  - zircon_plugins/sound/runtime/src/tests/common/effects.rs
  - zircon_plugins/sound/runtime/src/tests/common/listener.rs
  - zircon_plugins/sound/runtime/src/tests/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/ray_status.rs
  - zircon_plugins/sound/runtime/src/tests/convolution/static_ir.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/import.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/capture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/callbacks/failure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/detail.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/delivery.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/ordering.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/queue.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout/support/registration/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/event_cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/ownership.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/event_unregister.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/graph_reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/descriptors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/executor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/missing_handler.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/success.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/calls.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/outcomes.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/executors.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/fixture.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/support/submission.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/drain.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/schema.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/ids.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/invocation.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/support/registration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/time.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unknown_event.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation/unregistration.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/event_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/executor_services.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files/state.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/assertions.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/retired.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support/source.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/wiring.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/chain_controls.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/dynamics.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/chorus.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/flanger.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation/phaser.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/shaper.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/stereo.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/high_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/low_pass.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/filter/shelf.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/latency.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/convolution.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/delay.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/flanger_history.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation/phaser_phase.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/reverb.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/support.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/parameters.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state/validation/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/manifest.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/dependencies.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/event_catalogs.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/line.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/capability_statuses.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/maturity.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/keys.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/parser.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/projection.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/options/state.rs
  - zircon_plugins/sound/runtime/src/tests/manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/dependency.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/feature.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/module.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/section.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/state.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/types.rs
  - zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/values.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/backends.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/catalog/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/devices.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/disabled.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/cpal/windows_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/configured_pull.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/lifecycle/reconfigure.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/stopped_callback.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/software_null/recovery/unsupported_backend.rs
  - zircon_plugins/sound/runtime/src/tests/output_device/validation.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/effect_chain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/routing/track_removal.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/removal_errors.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/routing.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/crud/upsert_snapshot.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sends/cycles.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/compressor.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain/taps.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/solo.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/support.rs
  - zircon_plugins/sound/runtime/src/tests/mixer_graph/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback.rs
  - zircon_plugins/sound/runtime/src/tests/playback/completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/gain_mute.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/initial_state.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/speed_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/controls/transport.rs
  - zircon_plugins/sound/runtime/src/tests/playback/range.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/presets.rs
  - zircon_plugins/sound/runtime/src/tests/playback/settings/validation.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/completed.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_completion/stopped.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_controls/runtime.rs
  - zircon_plugins/sound/runtime/src/tests/playback/source_range.rs
  - zircon_plugins/sound/runtime/src/tests/presets.rs
  - zircon_plugins/sound/runtime/src/tests/presets/catalog_apply.rs
  - zircon_plugins/sound/runtime/src/tests/presets/reroute.rs
  - zircon_plugins/sound/runtime/src/tests/presets/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/cache.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/provider_status.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/cell_key.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/descriptor.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/occlusion.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/ray_count.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing/validation/source.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_normalization.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/config_options.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/global_volume.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/registration.rs
  - zircon_plugins/sound/runtime/src/tests/runtime_core/render_defaults.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/catalog.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/kernels.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded/tail_state.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/parameter_playback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/ear_delay.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview/fallback.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/tail.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/hrtf/validation.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/listener.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/default.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/scale/source_override.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/sends.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/support.rs
  - zircon_plugins/sound/runtime/src/tests/spatial/volumes.rs
  - 2026-06-01: cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime sound_plugin_registration_contributes_runtime_module_components_options_and_events --locked --jobs 1 --message-format short --color never (passed after event catalog assertion update)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/mod.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs (2026-06-04 dynamic-events test boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events.rs zircon_plugins/sound/runtime/src/tests/dynamic_events docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-events test boundary split: passed with expected LF-to-CRLF warning)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/retired.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/service_files.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/support.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure/wiring.rs (2026-06-04 dynamic-event structure sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/structure.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/structure docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event structure sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-04 dynamic-event structure sub-boundary split: timed out after ten minutes without Rust diagnostics; no Sound cargo/rustc process remained afterward, while other-session editor/runtime Cargo lanes were active)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/catalog.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/registry/validation.rs (2026-06-04 dynamic-event registry sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/registry.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/registry docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event registry sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-04 dynamic-event registry sub-boundary split: not run because separate Hub/runtime Cargo/rustc lanes were active at 2026-06-04 23:04 +08:00; focused dynamic-events Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/fanout.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch/handlers.rs (2026-06-04 dynamic-event dispatch sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/dispatch docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event dispatch sub-boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-04 dynamic-event dispatch sub-boundary split: not run because separate editor/runtime cargo/rustc lanes were active at 2026-06-04 22:01 +08:00; focused dynamic-events Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/failure.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/success.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/abi/support.rs (2026-06-04 dynamic-event ABI sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/abi.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/abi docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event ABI sub-boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-04 dynamic-event ABI sub-boundary split: not run because separate Hub/runtime/editor cargo/rustc lanes were active at 2026-06-04 22:07 +08:00; focused dynamic-events Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/registration.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report.rs (2026-06-04 dynamic-event execution sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/execution.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event execution sub-boundary split: passed with expected LF-to-CRLF warnings)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/event_unregister.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/graph_reconfigure.rs (2026-06-04 dynamic-event execution cleanup sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 dynamic-event execution cleanup sub-boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-04 dynamic-event execution cleanup sub-boundary split: not run because separate Hub/runtime/editor cargo/rustc lanes were active at 2026-06-04 23:09 +08:00; focused dynamic-events Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/descriptors.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/cleanup/support/registration/executor.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/calls.rs zircon_plugins/sound/runtime/src/tests/dynamic_events/execution/report/drain.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/dependency.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/feature.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/module.rs (2026-06-05 dynamic-event cleanup registration support split plus compile repair: passed)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-05 dynamic-event cleanup registration support split plus compile repair: first run failed on optional-feature pending helper visibility and missing report trait imports; rerun passed with existing warnings)
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never (2026-06-05 dynamic-event cleanup registration support split: not run because Hub Cargo/rustc lanes were active at 2026-06-05 04:55 +08:00)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/manifest.rs zircon_plugins/sound/runtime/src/tests/manifest/contributions.rs zircon_plugins/sound/runtime/src/tests/manifest/metadata.rs zircon_plugins/sound/runtime/src/tests/manifest/options.rs zircon_plugins/sound/runtime/src/tests/manifest/support.rs zircon_plugins/sound/runtime/src/tests/manifest/support/contributions.rs zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs zircon_plugins/sound/runtime/src/tests/manifest/support/values.rs (2026-06-04 manifest support boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/manifest.rs zircon_plugins/sound/runtime/src/tests/manifest docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 manifest support boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-test-split-0604 --message-format short --color never (planned for manifest test boundary split after active Cargo lanes quiet down)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules.rs zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/line.rs zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules/state.rs (2026-06-04 manifest module-contribution support sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules.rs zircon_plugins/sound/runtime/src/tests/manifest/support/contributions/modules docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 manifest module-contribution support sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-contribution-modules-split-0604 --message-format short --color never (2026-06-04 manifest module-contribution support sub-boundary split: not run because separate editor/runtime Cargo/rustc lanes were active at 2026-06-04 22:39 +08:00; focused manifest Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/maturity.rs zircon_plugins/sound/runtime/src/tests/manifest/support/metadata/capability_statuses.rs (2026-06-04 manifest metadata support sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/manifest/support/metadata.rs zircon_plugins/sound/runtime/src/tests/manifest/support/metadata docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 manifest metadata support sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-metadata-split-0604 --message-format short --color never (2026-06-04 manifest metadata support sub-boundary split: attempted and timed out after 10 minutes without Rust diagnostics; target-specific Cargo/rustc processes were stopped at 2026-06-04 22:58 +08:00; focused manifest Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options/keys.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options/parser.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options/projection.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options/state.rs (2026-06-04 manifest option support sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/manifest/support/options.rs zircon_plugins/sound/runtime/src/tests/manifest/support/options docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 manifest option support sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-test-split-0604 --message-format short --color never (2026-06-04 manifest option support sub-boundary split: not run because a separate Hub cargo/rustc lane was active at 2026-06-04 22:20 +08:00; focused manifest Cargo validation remains pending)
  - rustfmt --check zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/parity.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/runtime.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/types.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/values.rs (2026-06-04 optional-feature manifest test boundary split: passed after rustfmt import-order correction)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 optional-feature manifest test boundary split: passed with expected LF-to-CRLF warnings; new child files covered by rustfmt, trailing-whitespace, and conflict-marker scans)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-optional-feature-manifest-test-split-0604 --message-format short --color never (planned for optional-feature manifest test boundary split after active Cargo lanes quiet down)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/section.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/state.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/dependency.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/feature.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/line/module.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/types.rs (2026-06-04 optional-feature parser support deep sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 optional-feature parser support deep sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-optional-feature-parser-split-0604 --message-format short --color never (2026-06-04 optional-feature parser support deep sub-boundary split: not run because separate render/editor/Hub Cargo/rustc lanes were active at 2026-06-04 22:33 +08:00; focused optional-feature parser Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/dependency.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/feature.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending/module.rs (2026-06-04 optional-feature pending-finalizer sub-boundary split: passed after rustfmt normalized import wrapping)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support/parser/pending docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 optional-feature pending-finalizer sub-boundary split: passed with expected LF-to-CRLF warning)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-optional-feature-parser-split-0604 --message-format short --color never (2026-06-04 optional-feature pending-finalizer sub-boundary split: not run because separate runtime/editor/Hub Cargo/rustc lanes were active at 2026-06-04 23:31 +08:00; focused optional-feature parser Cargo validation remains pending)
  - rustfmt --check zircon_plugins/sound/runtime/src/tests/dsp_state.rs zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic.rs zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs zircon_plugins/sound/runtime/src/tests/dsp_state/latency.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful.rs zircon_plugins/sound/runtime/src/tests/dsp_state/support.rs zircon_plugins/sound/runtime/src/tests/dsp_state/validation.rs (2026-06-04 DSP-state test boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dsp_state.rs zircon_plugins/sound/runtime/src/tests/dsp_state docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 DSP-state test boundary split: passed with expected LF-to-CRLF warnings; new child files covered by rustfmt, trailing-whitespace, and conflict-marker scans)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never (planned for DSP-state test boundary split after active Cargo lanes quiet down)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs zircon_plugins/sound/runtime/src/tests/dsp_state/filter/high_pass.rs zircon_plugins/sound/runtime/src/tests/dsp_state/filter/low_pass.rs zircon_plugins/sound/runtime/src/tests/dsp_state/filter/shelf.rs (2026-06-04 DSP filter sub-boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/dsp_state/filter.rs zircon_plugins/sound/runtime/src/tests/dsp_state/filter docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 DSP filter sub-boundary split: passed with expected LF-to-CRLF warnings)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never (2026-06-04 DSP filter sub-boundary split: not run because separate Hub/runtime/editor Cargo/rustc lanes were active at 2026-06-04 23:16 +08:00; focused DSP-state Cargo validation remains pending)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dsp_state/stateful.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/compressor.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/convolution.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/delay.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/reverb.rs (2026-06-04 DSP stateful sub-boundary split: passed)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/spatial.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf.rs zircon_plugins/sound/runtime/src/tests/spatial/listener.rs zircon_plugins/sound/runtime/src/tests/spatial/scale.rs zircon_plugins/sound/runtime/src/tests/spatial/sends.rs zircon_plugins/sound/runtime/src/tests/spatial/support.rs zircon_plugins/sound/runtime/src/tests/spatial/volumes.rs (2026-06-04 spatial-acoustics test boundary split: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests/spatial.rs zircon_plugins/sound/runtime/src/tests/spatial docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 spatial-acoustics test boundary split: passed with expected LF-to-CRLF warnings; new child files covered by rustfmt, trailing-whitespace, and conflict-marker scans)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-spatial-test-split-0604 --message-format short --color never (planned for spatial-acoustics test boundary split after active Cargo lanes quiet down)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/spatial/hrtf.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/catalog.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/parameter_playback.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/tail.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/validation.rs (2026-06-04 HRTF test sub-boundary split: passed)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/graph_config.rs zircon_plugins/sound/runtime/src/tests/mixer_graph.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/routing.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/sends.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/sidechain.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/solo.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/support.rs zircon_plugins/sound/runtime/src/tests/mixer_graph/validation.rs (2026-06-04 mixer-graph test boundary split and graph-config trait-import repair: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/graph_config.rs zircon_plugins/sound/runtime/src/tests/mixer_graph.rs zircon_plugins/sound/runtime/src/tests/mixer_graph docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md (2026-06-04 mixer-graph test boundary split and graph-config trait-import repair: passed with expected LF-to-CRLF warnings; new child files covered by rustfmt, trailing-whitespace, and conflict-marker scans)
  - cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never (2026-06-04 mixer-graph test boundary split: first run exposed stale `graph_config.rs` `SoundManager` import after the manager capability split; accepted rerun passed after importing `SoundMixerGraphManager`, `SoundMixRenderManager`, and `SoundAutomationTimelineManager`)
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never mixer_graph (2026-06-04 mixer-graph test boundary split: passed with 8 tests, 0 failed, and 95 filtered; the earlier unqualified `cargo test ... mixer_graph` attempt timed out while detached Sound cargo/rustc processes were still compiling `zircon_runtime`, not while running a Sound test binary)
  - rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/manifest/support.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs (2026-06-04 M6 root-gate sound test-support repair: passed)
  - git diff --check -- zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/manifest/support.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest/support.rs docs/zircon_plugins/sound/runtime.md .codex/sessions/20260530-2250-asset-material-mesh-flow.md (2026-06-04 M6 root-gate sound test-support repair: passed with expected LF-to-CRLF warnings)
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --lib --locked --no-run --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0604-post-hub-fingerprint --message-format short --color never (2026-06-04 M6 root-gate sound manifest/test-support repair: passed; log .codex/tmp/sound_runtime_lib_test_compile_after_manifest_support_wrapper_20260604.log)
  - zircon_plugins/sound/runtime/src/tests/source_inputs.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/clearing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/missing_blocks.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_lifecycle/validation.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/external_routing.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/parameter_bindings.rs
  - zircon_plugins/sound/runtime/src/tests/source_inputs/resampling.rs
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

The provider format contract now carries `SoundChannelLayout` alongside `channel_count`. `SoundConfig::from_plugin_options(...)` normalizes a zero channel count to mono and repairs layout/count mismatches to the derived layout for the accepted count. `DefaultSoundManager::with_config(...)`, preset graph construction, direct render blocks, output-device initial state, descriptor reconfiguration, backend status projection, and device picker rows preserve that layout so mono, stereo, 5.1, 7.1, and discrete formats are visible to editor meters, export planning, and backend selection instead of being inferred later from a bare channel count.

Package manifest contributions are folder-backed under `src/package/`: `dependencies.rs` owns required and optional plugin dependencies, `options.rs` owns Sound option rows and capability gating, `events.rs` owns the versioned dynamic-event catalog namespace, and `attach.rs` owns the package-manifest merge order for dependency, option, event-catalog, and component contributions. Internal callers use these direct child modules while `src/lib.rs` keeps the curated public helper exports for external consumers.

Built-in mixer preset catalog construction is folder-backed under `src/presets/`: `locators.rs` owns stable preset locator constants, `catalog.rs` assembles the exposed preset descriptors, `default.rs` constructs the default master-only graph, `music_sfx.rs` constructs the Music/SFX/Ambience bus graph, and `spatial_room.rs` adds the Room Reverb return and sends. `src/service_types/mixer_presets.rs` imports the catalog directly and owns live state replacement/rerouting.

`SoundRuntimePlugin` registers:

- `SoundModule`, `SoundDriver`, and `DefaultSoundManager`.
- `AudioSource`, `AudioListener`, and `AudioVolume` component descriptors.
- Sound plugin options such as backend, sample rate, channel count/layout, block size, global volume, spatial scale, HRTF, convolution, ray tracing, timeline integration, and dynamic-event enablement.
- The concrete versioned `sound.dynamic_events` event catalog: `sound.dynamic_events.impact`, `sound.dynamic_events.marker`, and `sound.dynamic_events.ambient_stinger`, with package-prefixed payload schemas `sound.dynamic.impact.v1`, `sound.dynamic.marker.v1`, and `sound.dynamic.ambient_stinger.v1`.

Runtime audio behavior remains in this crate. The runtime framework layer only owns DTOs, handles, and traits; it does not implement mixing, DSP, output callbacks, or Sound-specific editor behavior.

Engine-owned live state is folder-backed under `src/engine/state/`: `storage.rs` owns `SoundEngineState` and its constructor, `dynamic_events.rs` owns executor callback wrappers and executor keys, `playback.rs` owns loaded clip and active playback runtime records, `source.rs` owns explicit source voice cursors and pending finish state, `graph.rs` owns track add/remove mutations that must revalidate graph shape and reroute active outputs, and `snapshot.rs` owns mixer snapshot projection. `mod.rs` only wires and re-exports these state concepts for the rest of the engine.

Render mixing is folder-backed under `src/engine/render/`: `mod.rs` is structural and only declares the render child modules; `orchestration.rs` owns the `SoundEngineState::render_mix` orchestration loop, graph validation, track buffer flow, sidechain taps, DSP application, meters, and final master gain; `playback/` owns active clip playback, with structural `mod.rs`, `mixing.rs` for active-playback routing and track-buffer dispatch, `clip.rs` for clip block sampling, `pan.rs` for per-channel pan/gain projection, and `finish.rs` for finished-playback event reporting; `source/` owns explicit `AudioSource` rendering, with `mod.rs` kept structural, `orchestration.rs` coordinating source buffers, environment delegation, sends, and finish reporting, `input.rs` mixing clip/external/synth source inputs, `external.rs` resampling provider blocks, `parameters.rs` resolving source parameter bindings, and `range.rs` calculating clip-backed source frame ranges; `sampling/` owns shared resampling and channel projection, with structural `mod.rs`, `step.rs` for resample step calculation, `position.rs` for source cursor/range advancement, `interpolation.rs` for frame interpolation, and `frame.rs` for source-frame sampling plus output-channel folding; `routing.rs` owns solo-track direct-input checks and buffer accumulation; and `runtime_state.rs` owns stale track/effect/HRTF state pruning plus latency-frame estimation. Loaded HRTF runtime state is folder-backed under `src/engine/hrtf/`: `mod.rs` exposes only loaded-profile application, render-state keys, render-state storage, and pruning; `key.rs` owns `(source, listener, profile)` identity, `state.rs` owns FIR tail history retention and tail checks, `apply.rs` owns loaded-profile convolution and history sampling, and `prune.rs` owns stale loaded-profile state retention. `src/engine/math.rs` owns the internal 3D vector helpers used by spatial attenuation, HRTF preview, cone, pan, doppler, and volume calculations.

DSP execution is folder-backed under `src/engine/dsp/`: `mod.rs` exposes only the render-facing entry points, `effects/` owns track effect-chain execution with structural `mod.rs`, `chain.rs` for enabled-effect iteration and wet/dry mixing, `apply.rs` for effect-kind dispatch into DSP families, and `sidechain.rs` for compressor sidechain tap lookup; `controls.rs` applies per-track delay/pan/gain controls, `meter.rs` calculates peaks and RMS values, and `delay.rs`, `reverb.rs`, `modulation.rs`, `dynamics.rs`, `stereo.rs`, `gain.rs`, and `shaper.rs` own the corresponding effect families. Stateful DSP data is folder-backed under `src/engine/dsp_state/`: `mod.rs` exposes the narrow runtime-state surface, `effect_key.rs` owns the track/effect lookup key, `effect_runtime.rs` owns per-effect delay/reverb/convolution/modulation/filter/compressor fields, `track_runtime.rs` owns per-track control-delay state, `delay_line.rs` owns circular delay-line samples and cursors, and `history.rs` owns cross-block history sampling/retention. Filter math is folder-backed under `src/engine/filter/`: `mod.rs` exposes only `apply_biquad_filter_block` and `SoundBiquadFilterState`, `state.rs` owns per-effect/per-channel direct-form history, `apply.rs` runs block filtering, `coefficients.rs` maps `SoundFilterEffect` modes to normalized low-pass/high-pass/band-pass/notch biquad coefficients with cutoff and Q clamps, `shelf.rs` owns low-shelf/high-shelf coefficient formulas, and `constants.rs` keeps cutoff, Q, shelf slope, and gain clamps private to the filter boundary.

Source environment processing is folder-backed under `src/engine/source_environment/` after dry source input generation. `mod.rs` is structural and exposes only the narrow `apply_source_environment`, active-listener, and HRTF-tail entry points; `apply.rs` coordinates gain, pan, volume, HRTF, and convolution effects; `listener.rs` owns active listener selection; `spatial/` owns source spatial profile composition, with `profile.rs` combining blend, occlusion, listener-right pan, and child gains, `attenuation.rs` owning attenuation curves, `cone.rs` owning source-cone gain, `doppler.rs` owning preview Doppler gain, and `pan.rs` owning stereo source pan application; `hrtf/` owns source-environment HRTF dispatch, with `mod.rs` exposing only the local HRTF entries, `loaded.rs` resolving active listener profiles into loaded-profile convolution state, `preview.rs` owning deterministic ear-distance gain and delay fallback, and `tail.rs` checking pending loaded-profile FIR tails; `volume/` owns `AudioVolume` policy, with `mod.rs` exposing only the local volume entries, `influence.rs` selecting the strongest priority/id influence and projecting gain, `weight.rs` calculating shape distance and crossfade weight, and `filter.rs` applying volume low-pass blocks; `convolution.rs` owns source and volume convolution sends; and `constants.rs` keeps the shared preview constants out of behavior files. `src/engine/occlusion/` owns the occlusion query DTO, deterministic fallback gain, provider-fed gain entry point, and ray-traced descriptor specificity matching, so spatial source processing can ask one narrow runtime boundary for occlusion attenuation without owning provider cache policy. `src/engine/render/source/orchestration.rs` delegates this responsibility with cloned frame-state snapshots, keeping per-block orchestration separate from the source render root and source-environment math/policy.

Engine validation is folder-backed under `src/engine/validation/`: `mod.rs` exposes the stable `validate_graph`, `validate_effect`, and `track_render_order` entry points; `graph.rs` owns whole-graph validation flow; `track.rs` owns track-control and send checks; `effect.rs` owns effect parameter constraints; `references.rs` owns sidechain track reference policy; `ordering.rs` owns deterministic render dependency sorting; and `values.rs` keeps shared finite/range guards private to validation.

Concrete output support is folder-backed under `src/output/`: `mod.rs` is a structural entry, `catalog.rs` composes backend capability rows and device picker rows, and `descriptor_validation.rs` owns backend-neutral descriptor validation plus backend availability checks, including channel layout/count consistency. `lifecycle/` owns output-device runtime state and device transitions: `mod.rs` exposes only the lifecycle boundary, `storage.rs` stores `SoundOutputDeviceRuntimeState`, `config.rs` owns reconfiguration and backend-session clearing, `start_stop.rs` owns software/CPAL start and stop paths, `callback.rs` owns rendered-block counters, callback accounting, and unavailable-backend error recording, `status.rs` projects lifecycle state into `SoundOutputDeviceStatus`, and `session.rs` owns the backend-session enum. The sibling `output/status.rs` remains responsible for latency estimation and status diagnostic de-duplication, `software.rs` owns deterministic software output rows and supported layout capability rows, `ring_buffer.rs` owns the bounded realtime FIFO, and `cpal/` owns the optional platform adapter behind `cpal-backend`. Inside `cpal/`, `mod.rs` is structural, `capability.rs` owns backend capability and feature availability rows including supported channel layouts, `device.rs` owns picker row enumeration and derives layout from the reported device format when available, `selection.rs` owns CPAL device and stream-config selection, `session.rs` owns `CpalOutputSession`, `device_thread.rs` and `producer_thread.rs` own the two runtime threads, `callback.rs` owns realtime output draining, `shared_state.rs` owns queue/counter/error state, and `error.rs` maps backend-unavailable details.

`src/service_types/mod.rs` remains the public `DefaultSoundManager` service boundary and only owns child-module wiring plus the curated `DefaultSoundManager`/`SoundDriver` export. `src/service_types/manager_state.rs` owns `SoundDriver`, shared manager state fields, construction, and shared config snapshots. `src/service_types/manager_trait.rs` is now a structural trait-dispatch root: it declares the child capability delegate modules and implements the composed `SoundManager` marker for `DefaultSoundManager`; `manager_trait/{backend,output_device,runtime_settings,playback,mixer_graph,source,automation_timeline,dynamic_events,acoustics,render}.rs` implement the corresponding framework capability traits and forward into the focused service modules. Clip asset-manager resolution, test clip injection, clip loading, and clip info snapshots now live in `src/service_types/clip_assets.rs`. Playback creation and stopped-playback completion events now live in `src/service_types/playback.rs`; `src/service_types/playback_controls.rs` is now a structural playback-control root whose child modules own pause/resume/toggle, gain validation/mutation, speed validation/mutation, range-aware seek, mute/unmute/toggle, and shared active-playback state access. Playback empty checks, playback status snapshots, and finished playback draining now live in `src/service_types/playback_status.rs`; playback settings validation, speed validation, and start/duration range calculation now live in `src/service_types/playback_validation.rs`. Source creation/update/removal and stopped-source completion events now live in `src/service_types/sources.rs`; source pause/resume/toggle, gain/speed, and mute controls now live in `src/service_types/source_controls.rs`; source seek/cursor repositioning now lives in `src/service_types/source_seek.rs`; source empty checks, source status snapshots, range/cursor reporting, and finished source draining now live in `src/service_types/source_status.rs`. External audio source block submission and clearing now live in `src/service_types/external_sources.rs`. Service-level output-device APIs are folder-backed under `src/service_types/output_device/`, where `mod.rs` stays structural, `backend.rs` owns backend naming/status projection, `configuration.rs` owns descriptor configuration and graph-format reset, `lifecycle.rs` owns start/stop calls into the output runtime, `status.rs` owns output-device status snapshots, and `catalog.rs` owns backend/device listing; software output-device block rendering and backend callback pull/reporting behavior now live in `src/service_types/output_render.rs`. Mixer preset discovery/application and rerouting of active playbacks/sources after preset graph replacement now live in `src/service_types/mixer_presets.rs`; service-level mixer graph APIs are folder-backed under `src/service_types/mixer_graph/`, where `mod.rs` stays structural, `configuration.rs` owns full graph import, `snapshot.rs` owns mixer snapshot projection, `tracks.rs` owns track CRUD, `sends.rs` owns send CRUD and validation handoff, and `effects.rs` owns effect CRUD and validation handoff. Service-level dynamic-event APIs are folder-backed under `src/service_types/dynamic_events/`, where `mod.rs` stays structural, `catalog.rs` owns catalog snapshots, registration, unregistering, and dependent cleanup, `handlers.rs` owns handler listing/registration/unregistration, `invocation.rs` owns pending invocation submission/draining, and `dispatch.rs` owns deterministic delivery fan-out. Service-level dynamic-event executor APIs are folder-backed under `src/service_types/dynamic_event_executors/`, where `mod.rs` stays structural, `registration.rs` owns callback registration and handler-existence validation, `unregistration.rs` owns executor removal errors, and `execution.rs` owns delivery dispatch plus per-handler execution report assembly. Sound parameter storage and lookup now live in `src/service_types/parameters.rs`; service-level automation binding/application and automation curve sample calls now live in `src/service_types/automation_timeline.rs`; timeline sequence scheduling, removal, listing, and advancement now live in `src/service_types/timeline_sequences.rs`. Listener and `AudioVolume` registration now live in `src/service_types/acoustics.rs`; static impulse-response lifecycle now lives in `src/service_types/impulse_responses.rs`; ray-tracing convolution status plus ray-traced impulse-response submission/listing/clearing now live in `src/service_types/ray_tracing_convolution.rs`; HRTF profile loading, removal, listing, validation, and HRTF render-state invalidation now live in `src/service_types/hrtf_profiles.rs`. Global volume/default spatial-scale service configuration and direct software `render_mix` now live in `src/service_types/runtime_settings.rs`. This keeps the service root structural instead of owning asset loading, lifecycle, playback controls, playback status reporting, playback validation, source controls, source seek policy, source status reporting, graph mutation, preset replacement, output rendering, event-dispatch, dynamic-event execution, parameter storage, timeline-control, acoustics-state, impulse-response state, ray-tracing convolution state, HRTF profile state, runtime-setting, external-source buffer, manager state, or trait-dispatch behavior directly.

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

`src/tests/manifest.rs` keeps static and generated metadata in sync. It checks option keys plus option labels, value types, default values, and required capability gates, runtime module contributions, dependency rows, event catalogs, component descriptors, and verifies that static TOML, runtime descriptor, generated package manifest, and built-in runtime catalog agree on maturity and capability status. The manifest support root is now structural: `manifest/support/contributions.rs` aggregates parsed contribution rows; `support/contributions/dependencies.rs` owns dependency TOML parsing; `support/contributions/event_catalogs.rs` owns event-catalog TOML parsing; `support/contributions/modules.rs` is now the structural module-contribution parser entry, `support/contributions/modules/state.rs` owns `[[modules]]` scanner state and pending module-row finalization, `support/contributions/modules/line.rs` owns module field parsing; `metadata.rs` is now the structural metadata parser entry, `metadata/maturity.rs` owns the static maturity field parse, `metadata/capability_statuses.rs` owns `[[capability_statuses]]` scanner state and row finalization; `options.rs` is now a structural option-support entry; `options/keys.rs` owns option-key extraction; `options/parser.rs` owns `[[options]]` TOML scanning; `options/state.rs` owns pending option row assembly; `options/projection.rs` owns tuple projection; and `values.rs` owns shared scalar/list/module-kind conversion. `src/tests/runtime_core.rs` verifies that neutral `SoundPluginOptions` values are preserved into `SoundConfig`, that mismatched or zero channel-count inputs normalize to coherent layout metadata, and that `DefaultSoundManager::with_config(...)` uses those values for runtime mix format, global volume, and spatial scale. `src/tests/optional_feature_manifest.rs` is now a structural entry for static-to-generated optional feature bundle parity; `optional_feature_manifest/parity.rs` owns the assertion for `sound.timeline_animation_track` and `sound.ray_traced_convolution_reverb`, while `optional_feature_manifest/support/parser.rs` is now a structural static-TOML parser entry, `support/parser/state.rs` owns scanner state transitions and pending-row flush order, `support/parser/section.rs` owns optional-feature table-header classification, `support/parser/line.rs` is the structural field-parser entry, `support/parser/line/{feature,dependency,module}.rs` own per-section key/value parsing, `support/parser/pending.rs` is the structural pending-finalizer entry, `support/parser/pending/{dependency,module,feature}.rs` own dependency-row, module-row, and feature-row finalization, `support/runtime.rs` owns runtime signature projection, `support/types.rs` owns comparison DTOs, and `support/values.rs` owns scalar/list/module-kind conversion. The parity covers dependency rows, runtime/editor module rows, capabilities, default packaging strategies, and enabled-by-default flags. The provider crates add local unit contracts in `zircon_plugins/sound/features/timeline_animation_track/runtime/src/lib.rs` and `zircon_plugins/sound/features/ray_traced_convolution_reverb/runtime/src/lib.rs` so linked feature registrations keep the same id, display name, owner, dependency, capability, packaging, and module metadata as the Sound owner bundle. The broader runtime test tree covers graph routing, DSP state, spatial/HRTF behavior, ray-traced impulse-response provider input, dynamic events, output-device behavior, channel layout propagation through backend capabilities, descriptors, status, and rendered blocks, presets, source lifecycle, automation, and manifest parity.

`src/tests/output_device.rs` is now the structural output-device test entry. `output_device/catalog.rs` is now the structural backend/device catalog entry, `output_device/catalog/backends.rs` owns deterministic backend capability coverage, `output_device/catalog/devices.rs` owns software picker descriptor and status projection coverage; `output_device/cpal.rs` is now a structural CPAL entry, with `output_device/cpal/disabled.rs` owning no-feature unavailable/recovery assertions, `cpal/catalog.rs` owning feature-enabled backend listing, `cpal/devices.rs` owning feature-enabled device enumeration, and `cpal/windows_lifecycle.rs` owning the Windows start/stop contract while preserving the existing `#[cfg]` gates; `output_device/software_null.rs` is now a structural software-null entry, with `output_device/software_null/callback.rs` owning callback reporting and `output_device/software_null/recovery.rs` as the structural recovery entry, `recovery/stopped_callback.rs` owning stopped callback errors, and `recovery/unsupported_backend.rs` owning unsupported backend status plus recovery back to software-null; `output_device/lifecycle.rs` is now the structural software-test lifecycle entry, `lifecycle/configured_pull.rs` owns configure/start/render/status counter behavior, `lifecycle/reconfigure.rs` owns reconfigure stop/reset and runtime format propagation; and `output_device/validation.rs` owns invalid descriptor and stopped-pull errors. This keeps backend catalog, platform adapter, callback, lifecycle, and descriptor-validation assertions aligned with the production `output`, `output/cpal`, `service_types/output_device`, and `service_types/output_render` boundaries.

`src/tests/dynamic_events.rs` has been retired in favor of `src/tests/dynamic_events/mod.rs` plus folder-backed event infrastructure tests. `dynamic_events/structure.rs` is now a structural entry: `structure/wiring.rs` owns root and production module-wiring assertions, `structure/service_files.rs` is structural over `service_files/{event_services,executor_services,state}.rs` for dynamic-event service files, executor service files, and engine state ownership, `structure/retired.rs` keeps the old flat dynamic-event files retired, and `structure/support.rs` is structural over `support/{source,assertions,retired}.rs` for source-root lookup, source-content assertions, and retired flat-file path construction. `dynamic_events/registry.rs` is now a structural entry: `registry/catalog.rs` is structural over `catalog/{registration,snapshot,drain,support}.rs` for descriptor acceptance, mixer snapshot projection, pending invocation draining, and the shared impact-event fixture, while `registry/catalog/support.rs` is structural over `support/{ids,registration,invocation}.rs` for shared impact-event constants, descriptor registration, and impact invocation construction. `registry/validation.rs` is structural over `validation/{descriptor,unknown_event,time,schema,unregistration,support}.rs` for invalid descriptor, unknown event, non-finite time, schema mismatch, repeated unregister, and shared marker-event fixture coverage. `dynamic_events/dispatch.rs` is now a structural entry: `dispatch/fanout.rs` is structural over `fanout/{ordering,delivery,queue,support}.rs` for deterministic handler ordering, cloned invocation delivery, post-dispatch queue draining, and the shared weapon-fire fixture, while `dispatch/handlers.rs` is itself structural over `handlers/ownership.rs` for event-ownership validation, `handlers/unregistration.rs` for handler removal and missing-handler errors, `handlers/event_cleanup.rs` for event unregister cleanup of handlers and queued dispatches, and `handlers/support.rs` is structural over `support/{ids,registration,submission}.rs` for shared ambient-stinger constants, registration helpers, and queued ambient invocation submission. `execution/report/support.rs` is structural over `support/{ids,registration,executors,submission,fixture}.rs` for shared event ids, event/handler registration, executor callback recording, pending invocation submission, and report fixture assembly. `dynamic_events/abi.rs` is now a structural entry: `abi/success.rs` owns successful ABI request projection, `abi/failure.rs` owns ABI failure-detail status mapping, `abi/support.rs` is structural over `support/{detail,registration,callbacks}.rs`, and `abi/support/callbacks.rs` is structural over `callbacks/{capture,failure}.rs` for successful ABI request assertion and failure-detail callback mapping. Runtime behavior coverage remains split across `registry/{catalog,validation}.rs`, `dispatch/{fanout,handlers}.rs`, `execution/{registration,report,cleanup}.rs`, and `abi/{success,failure,support}.rs`, so event catalog registration, validation, handler delivery, executor reporting/cleanup, and ABI callback projection can grow independently.

`dispatch/fanout/support.rs` is structural over `support/{ids,registration,invocation,fixture}.rs` for shared event constants, event and handler registration, invocation construction, and fanout fixture assembly.

`dispatch/handlers/support.rs` is structural over `support/{ids,registration,submission}.rs` for shared ambient-stinger constants, event and handler registration, and pending ambient invocation submission.

The root Sound test module imports the split manager capability traits that `DefaultSoundManager` implements, so folder-backed child tests can call source, playback, output-device, mixer-graph, render, dynamic-event, backend, runtime-setting, automation-timeline, and acoustics methods through normal trait resolution after `use super::*`. The manifest and optional-feature manifest support roots expose parent-scope wrapper functions for static TOML parsing and runtime signature projection instead of re-exporting private child helpers directly; child parser/runtime modules stay encapsulated while sibling parity tests keep one shared support entry point.

Focused static validation for the output-device test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/output_device.rs` and every child file under `src/tests/output_device/`; `git diff --check -- src/tests/output_device.rs src/tests/output_device docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with expected LF-to-CRLF warnings; trailing-whitespace and conflict-marker scans returned empty; and the line-count audit left the root facade at 5 lines with children between 39 and 124 lines. The later software-null sub-boundary split kept `output_device/software_null.rs` structural at 2 lines and moved the two assertions into `software_null/callback.rs` (39 lines) and `software_null/recovery.rs` (70 lines). The later CPAL sub-boundary split kept `output_device/cpal.rs` structural at 4 lines and moved its feature-gated assertions into `cpal/disabled.rs` (44 lines), `cpal/catalog.rs` (14 lines), `cpal/devices.rs` (23 lines), and `cpal/windows_lifecycle.rs` (38 lines). The 2026-06-05 lifecycle sub-boundary split kept `output_device/lifecycle.rs` structural at 2 lines and moved the two lifecycle assertions into `lifecycle/configured_pull.rs` (31 lines) and `lifecycle/reconfigure.rs` (31 lines), with the test-marker count remaining 2 for the lifecycle subtree. The 2026-06-05 catalog sub-boundary split kept `output_device/catalog.rs` structural at 2 lines and moved the two catalog assertions into `catalog/backends.rs` (20 lines) and `catalog/devices.rs` (32 lines); `rustfmt --edition 2021 --check` over those three files passed; `git diff --check -- src/tests/output_device/catalog.rs src/tests/output_device/catalog docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the catalog subtree test-marker count remained 2; the full output-device test-marker count remained 11; and both new catalog child files appear three times in this document's code lists. A low-concurrency `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device-test-split-0604 --message-format short --color never` attempt on 2026-06-04 timed out after ten minutes without Rust diagnostics; the Sound cargo/rustc child processes from that attempt were stopped, and only other-session Hub Cargo/rustc lanes remained active. Compile/test acceptance is still unproven; retry the same `cargo check`, then `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device-test-split-0604 --message-format short --color never` if the check passes.

The 2026-06-05 software-null recovery sub-boundary split kept `output_device/software_null/recovery.rs` structural at 2 lines and split recovery assertions into `recovery/stopped_callback.rs` at 10 lines and `recovery/unsupported_backend.rs` at 65 lines; focused static checks passed for the recovery subtree, trailing-whitespace and conflict-marker scans returned empty, the recovery subtree now has 2 test markers, the full output-device tree now has 12 test markers, and each new recovery child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; retry `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device-test-split-0604 --message-format short --color never`, then run `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-output-device-test-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the manifest support split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, `git diff --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/manifest.rs`, every child file under `src/tests/manifest/`, this document, and the active session note. The M6 root gate later proved the root support facade also needed parent-scope wrappers rather than private child-helper re-exports, mirroring the optional-feature manifest support facade. After that repair, focused lib-test compile validation passed with `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --lib --locked --no-run --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0604-post-hub-fingerprint --message-format short --color never`; the log is `.codex/tmp/sound_runtime_lib_test_compile_after_manifest_support_wrapper_20260604.log`.

Focused static validation for the manifest module-contribution support sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/manifest/support/contributions/modules.rs` and `support/contributions/modules/{line,state}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; and the line-count audit left `modules.rs` at 11 lines, `modules/line.rs` at 49 lines, and `modules/state.rs` at 57 lines. Focused Cargo validation remains pending because separate editor/runtime Cargo/rustc lanes were active at 2026-06-04 22:39 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-contribution-modules-split-0604 --message-format short --color never`, followed by focused manifest tests if the check passes.

Focused static validation for the manifest metadata support sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/manifest/support/metadata.rs` and `support/metadata/{maturity,capability_statuses}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; and the line-count audit left `metadata.rs` at 15 lines, `metadata/maturity.rs` at 15 lines, and `metadata/capability_statuses.rs` at 74 lines. Focused Cargo validation remains pending: `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-manifest-metadata-split-0604 --message-format short --color never` was attempted on 2026-06-04 and timed out after 10 minutes without Rust diagnostics; target-specific Cargo/rustc processes were stopped at 2026-06-04 22:58 +08:00. Retry the same command when the machine is quiet, then run focused manifest tests if the check passes.

Focused static validation for the optional-feature manifest parser support split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/optional_feature_manifest/support/parser.rs`, `support/parser/{section,state,line,pending}.rs`, `support/parser/line/{feature,dependency,module}.rs`, and `support/types.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; and the line-count audit left `parser.rs` at 15 lines, `parser/line.rs` at 6 lines, `parser/state.rs` at 112 lines, `parser/section.rs` at 23 lines, `parser/line/dependency.rs` at 25 lines, `parser/line/feature.rs` at 50 lines, `parser/line/module.rs` at 49 lines, and `parser/pending.rs` at 77 lines. Focused Cargo validation remains pending because separate render/editor/Hub Cargo/rustc lanes were active at 2026-06-04 22:33 +08:00; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-optional-feature-parser-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml optional_feature_manifest --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-optional-feature-parser-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the optional-feature manifest pending-finalizer sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `support/parser/pending.rs` and `support/parser/pending/{dependency,feature,module}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; and the line-count audit left `pending.rs` at 6 lines, `pending/dependency.rs` at 24 lines, `pending/feature.rs` at 28 lines, and `pending/module.rs` at 27 lines. The 2026-06-05 compile repair keeps pending child helpers parser-scoped with `pub(in super::super)` so `state.rs` can use the structural pending facade without crate-wide helper exposure. Focused Cargo validation for optional-feature tests remains pending because separate runtime/editor/Hub Cargo/rustc lanes were active at 2026-06-04 23:31 +08:00; retry the same optional-feature parser `cargo check`, then run the focused optional-feature manifest tests if the check passes.

Focused static validation after adding channel layout propagation to Sound framework/runtime contracts passed on 2026-06-04 with rustfmt and diff/conflict-marker checks only. Focused Cargo validation is intentionally pending while concurrent runtime/Hub Cargo lanes are active; rerun `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-channel-layout-0604 --message-format short --color never`, `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-channel-layout-0604 --message-format short --color never`, and the framework `cargo test -p zircon_runtime --lib sound --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-framework-contract --message-format short --color never` once the build machine is quiet.

Focused static validation after splitting service playback controls into folder-backed pause, gain, speed, seek, mute, and shared active-playback access modules passed on 2026-06-04 with `rustfmt --edition 2021 --check`, `git diff --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over the touched playback-control Rust files plus this document and the active session note. Focused Cargo validation is intentionally pending while concurrent Cargo/rustc lanes are active; rerun `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-controls-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-controls-split-0604 --message-format short --color never` once the build machine is quiet.

`src/tests/dynamic_events/mod.rs` is the structural dynamic-event test entry. `dynamic_events/structure.rs` owns the static folder-backed contract for `src/dynamic_events/`, `src/dynamic_event_abi/`, `src/service_types/dynamic_events/`, `src/service_types/dynamic_event_executors/`, and `src/engine/state/dynamic_events.rs`; `structure/service_files.rs` is structural over `service_files/{event_services,executor_services,state}.rs`; `registry.rs` is now the structural registry coverage entry; `registry/catalog.rs` is structural over `catalog/{registration,snapshot,drain,support}.rs`; `registry/validation.rs` owns invalid descriptor, unknown event, invalid time, schema mismatch, and unregister rejection coverage; `registry/validation/support.rs` is structural over marker ids, descriptor registration, and invocation construction; `dispatch.rs` is the structural dispatch coverage entry; `dispatch/fanout.rs` is structural over `fanout/{ordering,delivery,queue,support}.rs`; `dispatch/handlers.rs` owns handler ownership validation and unregister cleanup; `execution.rs` is now the structural execution coverage entry; `execution/registration.rs` is structural over `registration/{missing_handler,success,support}.rs` for missing-handler rejection, successful registered-handler executor registration, and the shared registered-handler fixture; `execution/report.rs` owns success/failure/missing-executor report ordering; `execution/cleanup.rs` is now the structural cleanup coverage entry; `cleanup/event_unregister.rs` owns executor cleanup after event unregistering; `cleanup/graph_reconfigure.rs` owns executor cleanup after graph reconfiguration; `cleanup/support.rs` is structural over cleanup fixtures, registration helpers, invocation submission, and skipped-executor assertions; `cleanup/support/registration.rs` is structural over descriptor registration and executor attachment helpers; `abi.rs` is now the structural ABI coverage entry; `abi/success.rs` owns ABI callback request projection; `abi/failure.rs` owns ABI status-detail mapping; `abi/support.rs` is structural over shared ABI detail, registration, and callbacks; and `abi/support/callbacks.rs` is structural over capture and failure callback fixtures. This keeps the dynamic-event service root and the dynamic-event test root structural while still testing the current local runtime execution path.

Within the fanout branch, `dispatch/fanout/support.rs` is structural over ids, registration, invocation, and fixture modules so adding another shared fanout helper does not turn the support root back into mixed setup code.

Focused static validation for the dynamic-event structure sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/dynamic_events/structure.rs` and `structure/{retired,service_files,support,wiring}.rs`; trailing-whitespace and conflict-marker scans returned empty; the line-count audit left `structure/service_files.rs` at 56 lines before this latest split. The 2026-06-05 service-files sub-boundary keeps `structure/service_files.rs` structural at 3 lines, with `service_files/event_services.rs` owning dynamic-event service file checks, `service_files/executor_services.rs` owning executor service file checks, and `service_files/state.rs` owning engine state ownership checks. Focused static checks passed for the latest service-files subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, 3 service-files test markers, 5 structure test markers, 27 dynamic-event test markers, line counts `service_files.rs` 3, `service_files/event_services.rs` 31, `service_files/executor_services.rs` 30, `service_files/state.rs` 12, and each new service-files child path appearing three times in this document's code lists. The 2026-06-05 structure-support sub-boundary keeps `structure/support.rs` structural at 7 lines, with `support/source.rs` owning source-root lookup and source reads, `support/assertions.rs` owning structural-module and source-fragment assertions, and `support/retired.rs` owning retired flat-file path construction. Focused static checks passed for the latest structure-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 7, `support/assertions.rs` 26, `support/retired.rs` 10, `support/source.rs` 10, 5 structure test markers, 28 dynamic-event test markers, and each new structure-support child path appearing three times in this document's code lists.

Focused static validation for the dynamic-event registry sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/dynamic_events/registry.rs` and `dynamic_events/registry/{catalog,validation}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the two moved registry tests in `registry/catalog.rs` and `registry/validation.rs`; and the line-count audit left `registry.rs` at 2 lines, `registry/catalog.rs` at 32 lines, and `registry/validation.rs` at 64 lines. The 2026-06-05 registry-catalog sub-boundary keeps `registry/catalog.rs` structural at 4 lines, with `catalog/registration.rs` owning descriptor acceptance and catalog metadata, `catalog/snapshot.rs` owning mixer snapshot projection, `catalog/drain.rs` owning pending invocation draining, and `catalog/support.rs` now structural over `support/{ids,registration,invocation}.rs`. Focused static checks passed for the catalog subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, 3 catalog test markers, 25 dynamic-event test markers, line counts `catalog.rs` 4, `catalog/registration.rs` 17, `catalog/snapshot.rs` 16, `catalog/drain.rs` 15, and each catalog child path appearing three times in this document's code lists. The 2026-06-05 registry-catalog support sub-boundary keeps `catalog/support.rs` structural at 6 lines, with `support/ids.rs` owning shared impact-event constants, `support/registration.rs` owning descriptor registration, and `support/invocation.rs` owning impact invocation construction. Focused static checks passed for the latest catalog-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 6, `support/ids.rs` 2, `support/invocation.rs` 13, `support/registration.rs` 13, 3 catalog test markers, 28 dynamic-event test markers, and each new catalog-support child path appearing three times in this document's code lists. The 2026-06-05 registry-validation sub-boundary keeps `registry/validation.rs` structural, with `validation/descriptor.rs` owning invalid descriptor rejection, `validation/unknown_event.rs` owning unknown invocation events, `validation/time.rs` owning non-finite invocation time, `validation/schema.rs` owning payload schema mismatch, `validation/unregistration.rs` owning repeated unregister errors, and `validation/support.rs` now structural over marker ids, descriptor registration, and invocation construction. Focused static checks passed for the latest validation subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, 5 validation test markers, 21 dynamic-event test markers, and each new validation child path appearing three times in this document's code lists. The 2026-06-05 registry-validation support sub-boundary keeps `validation/support.rs` structural at 6 lines, with `support/ids.rs` owning shared marker-event constants, `support/registration.rs` owning marker descriptor registration, and `support/invocation.rs` owning marker invocation construction. Focused static checks passed for the latest validation-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths with the expected LF-to-CRLF warning, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 6, `support/ids.rs` 2, `support/invocation.rs` 13, `support/registration.rs` 13, 5 validation test markers, 28 dynamic-event test markers, and each new validation-support child path appearing three times in this document's code lists. Focused Cargo validation remains pending because active Cargo/rustc lanes were still running at 2026-06-05 04:01 +08:00.

The 2026-06-05 fanout sub-boundary keeps `dispatch/fanout.rs` structural at 4 lines, with `fanout/ordering.rs` owning deterministic handler ordering, `fanout/delivery.rs` owning cloned invocation delivery, `fanout/queue.rs` owning post-dispatch queue draining, and `fanout/support.rs` now staying structural over `support/{ids,registration,invocation,fixture}.rs`. The fanout-support sub-boundary keeps `support.rs` structural at 6 lines, with `support/ids.rs` owning shared event constants, `support/registration.rs` structural over `registration/{event,handlers}.rs`, `support/invocation.rs` owning weapon-fire invocation construction, and `support/fixture.rs` owning fanout fixture assembly. Focused static checks passed for the latest fanout-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 6, `support/fixture.rs` 20, `support/ids.rs` 2, `support/invocation.rs` 13, `support/registration.rs` 5, `support/registration/event.rs` 13, `support/registration/handlers.rs` 33, 3 fanout test markers, 28 dynamic-event test markers, and each new fanout-registration child path appearing three times in this document's code lists. Focused Cargo validation remains pending because active Cargo/rustc lanes were still running at 2026-06-05 04:27 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never`, followed by focused dynamic-event tests if the check passes.

Focused static validation for the dynamic-event execution cleanup sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/dynamic_events/execution/cleanup.rs` and `execution/cleanup/{event_unregister,graph_reconfigure}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the two moved cleanup tests in `cleanup/event_unregister.rs` and `cleanup/graph_reconfigure.rs`; and the line-count audit left `cleanup.rs` at 2 lines, `cleanup/event_unregister.rs` at 44 lines, and `cleanup/graph_reconfigure.rs` at 44 lines. The 2026-06-05 cleanup-support sub-boundary keeps `cleanup.rs` structural and moves the duplicated cleanup fixture into `cleanup/support.rs`, with `support/fixture.rs` owning the cleanup event identities, `support/registration.rs` structural over `registration/{descriptors,executor}.rs`, `support/submission.rs` owning invocation submission, and `support/assertions.rs` owning the skipped-missing-executor report assertion. Focused static checks passed for the latest cleanup-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths with the expected LF-to-CRLF warning, empty trailing-whitespace and conflict-marker scans, line counts `cleanup.rs` 3, `cleanup/event_unregister.rs` 20, `cleanup/graph_reconfigure.rs` 22, `cleanup/support.rs` 11, `cleanup/support/assertions.rs` 10, `cleanup/support/fixture.rs` 56, `cleanup/support/registration.rs` 5, `cleanup/support/registration/descriptors.rs` 31, `cleanup/support/registration/executor.rs` 15, `cleanup/support/submission.rs` 15, 2 cleanup test markers, 7 execution test markers, 28 dynamic-event test markers, and each new cleanup-support/cleanup-registration child path appearing three times in this document's code lists. Focused Cargo validation advanced on 2026-06-05: the first `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never` run failed on two earlier split compile issues, optional-feature pending helper visibility and missing `SoundDynamicEventManager` imports in report call/drain tests; after repairing those support boundaries, the same command passed with existing warnings. Focused `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dynamic-events-test-split-0604 --message-format short --color never` remains pending because active Hub Cargo/rustc lanes were running at 2026-06-05 04:55 +08:00.

`src/tests/dsp_state.rs` is now the structural DSP-state test entry. `dsp_state/deterministic.rs` is now the structural one-block effect determinism entry; `deterministic/chain_controls.rs` owns bypass and wet/dry checks, `deterministic/delay.rs` owns delay output, `deterministic/dynamics.rs` owns limiter clipping, `deterministic/stereo.rs` owns pan/phase projection, `deterministic/filter.rs` owns low-pass one-block output, `deterministic/reverb.rs` owns reverb output, `deterministic/shaper.rs` owns waveshaper output, `deterministic/modulation.rs` is now the structural modulation-effect entry, `deterministic/modulation/flanger.rs` owns flanger output, `deterministic/modulation/phaser.rs` owns phaser output, and `deterministic/modulation/chorus.rs` owns chorus output. `filter.rs` is now the structural filter-state entry; `filter/low_pass.rs` owns low-pass continuity, `filter/high_pass.rs` owns high-pass DC rejection, and `filter/shelf.rs` owns shelf gain coverage; `stateful.rs` is now the structural cross-block state entry; `stateful/delay.rs` owns delay-line tail retention, `stateful/convolution.rs` owns static impulse-response tail retention, `stateful/reverb.rs` owns reverb tail retention, `stateful/modulation.rs` is now the structural modulation-state entry, `stateful/modulation/flanger_history.rs` owns flanger delay history, `stateful/modulation/phaser_phase.rs` owns phaser LFO phase continuity, and `stateful/compressor.rs` owns compressor release-envelope continuity; `latency.rs` owns mixer latency projection; `validation.rs` is now the structural validation entry; `validation/parameters.rs` owns invalid effect-parameter rejection; `validation/sidechain.rs` owns compressor sidechain reference and cycle validation; and `support.rs` owns the shared render helper. This keeps audio-effect regression coverage aligned with the production `engine/dsp`, `engine/dsp_state`, and `engine/filter` boundaries.

Focused static validation for the DSP filter sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/dsp_state/filter.rs` and `src/tests/dsp_state/filter/{high_pass,low_pass,shelf}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the three moved filter tests in `filter/high_pass.rs`, `filter/low_pass.rs`, and `filter/shelf.rs`; and the line-count audit left `filter.rs` at 3 lines, `filter/high_pass.rs` at 26 lines, `filter/low_pass.rs` at 26 lines, and `filter/shelf.rs` at 27 lines. Focused Cargo validation remains pending because separate Hub/runtime/editor Cargo/rustc lanes were active at 2026-06-04 23:16 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`, followed by focused DSP-state tests if the check passes.

Focused static validation for the DSP validation sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/dsp_state/validation.rs` plus `src/tests/dsp_state/validation/{parameters,sidechain}.rs`. Focused Cargo validation remains intentionally pending while concurrent Cargo/rustc lanes are active; the existing DSP-state check/test priority command remains `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`.

Focused static validation for the DSP deterministic sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/dsp_state/deterministic.rs` plus `src/tests/dsp_state/deterministic/{chain_controls,delay,dynamics,filter,modulation,reverb,shaper,stereo}.rs`. The 2026-06-05 deterministic modulation sub-boundary split kept `src/tests/dsp_state/deterministic/modulation.rs` structural at 3 lines and moved the three modulation assertions into `modulation/flanger.rs`, `modulation/phaser.rs`, and `modulation/chorus.rs`, each at 18 lines with one test marker; `rustfmt --edition 2021 --check` over the four modulation files passed; `git diff --check -- zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation.rs zircon_plugins/sound/runtime/src/tests/dsp_state/deterministic/modulation docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the modulation subtree test-marker count remained 3; the full DSP-state tree test-marker count remained 23; and all three new modulation child files appear three times in this document's code lists. Focused Cargo validation remains intentionally pending while concurrent Cargo/rustc lanes are active; the existing DSP-state check/test priority command remains `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`.

Focused static validation for the DSP stateful sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/dsp_state/stateful.rs` plus `src/tests/dsp_state/stateful/{compressor,convolution,delay,modulation,reverb}.rs`. The 2026-06-05 stateful modulation sub-boundary split kept `src/tests/dsp_state/stateful/modulation.rs` structural at 2 lines and moved the two modulation-state assertions into `modulation/flanger_history.rs` (22 lines) and `modulation/phaser_phase.rs` (23 lines); `rustfmt --edition 2021 --check` over the three stateful modulation files passed; `git diff --check -- zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation.rs zircon_plugins/sound/runtime/src/tests/dsp_state/stateful/modulation docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the stateful modulation subtree test-marker count remained 2; the full DSP-state tree test-marker count remained 23; and both new stateful modulation child files appear three times in this document's code lists. Focused Cargo validation remains intentionally pending while concurrent Cargo/rustc lanes are active; the existing DSP-state check/test priority command remains `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-dsp-state-test-split-0604 --message-format short --color never`.

`src/tests/spatial.rs` is now the structural spatial-acoustics test entry. `spatial/hrtf.rs` is now the structural HRTF coverage entry; `spatial/hrtf/catalog.rs` owns profile load/list/remove ordering, `loaded.rs` is now the structural loaded-profile entry, `loaded/kernels.rs` owns deterministic loaded-kernel rendering, `loaded/tail_state.rs` owns FIR state across blocks, `tail.rs` owns long-tail emission after source completion, `parameter_playback.rs` owns parameter-driven playing state, `preview.rs` is now the structural preview-profile entry, `preview/fallback.rs` owns missing-profile preview fallback, `preview/ear_delay.rs` owns lateral-source ear-delay behavior, and `validation.rs` owns typed invalid-profile and missing-remove errors. `scale.rs` is now the structural spatial-scale entry; `scale/default.rs` owns default spatial scale, listener/source distance, and invalid default scale validation; `scale/source_override.rs` owns source-level spatial-scale overrides and invalid per-source scale validation. `listener.rs` owns attenuation, pan, and occlusion against the active listener; `volumes.rs` owns AudioVolume priority and crossfade behavior; `sends.rs` owns pre-spatial source sends; and `support.rs` owns shared HRTF test descriptors. This keeps HRTF, listener attenuation, volume influence, spatial-scale policy, and pre-spatial routing coverage aligned with the production `engine/hrtf` and `engine/source_environment` boundaries.

Focused static validation for the spatial-acoustics test split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, `git diff --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/spatial.rs`, every child file under `src/tests/spatial/`, this document, and the active session note. The later HRTF sub-boundary split also passed `rustfmt --edition 2021 --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/spatial/hrtf.rs` plus `src/tests/spatial/hrtf/{catalog,loaded,parameter_playback,preview,tail,validation}.rs`. The 2026-06-05 spatial-scale sub-boundary split passed focused static validation over `src/tests/spatial/scale.rs` and `src/tests/spatial/scale/{default,source_override}.rs`; the test-marker audit retained the two spatial-scale tests, and the line-count audit left the scale entry at 2 lines with child tests at 41 and 29 lines. The 2026-06-05 HRTF preview sub-boundary split kept `src/tests/spatial/hrtf/preview.rs` structural at 2 lines and moved the two preview assertions into `preview/ear_delay.rs` (29 lines) and `preview/fallback.rs` (22 lines); `rustfmt --edition 2021 --check` over the three preview files passed; `git diff --check -- zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/preview docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the preview subtree test-marker count remained 2; the full spatial tree test-marker count remained 13; and both new preview child files appear three times in this document's code lists. The 2026-06-05 HRTF loaded-profile sub-boundary split kept `src/tests/spatial/hrtf/loaded.rs` structural at 2 lines and moved the two loaded-profile assertions into `loaded/kernels.rs` (25 lines) and `loaded/tail_state.rs` (22 lines); `rustfmt --edition 2021 --check` over the three loaded files passed; `git diff --check -- zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded.rs zircon_plugins/sound/runtime/src/tests/spatial/hrtf/loaded docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the loaded subtree test-marker count remained 2; the full spatial tree test-marker count remained 13; and both new loaded child files appear three times in this document's code lists. Focused Cargo validation is intentionally pending while concurrent Cargo/rustc lanes are active; rerun `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-spatial-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-spatial-test-split-0604 --message-format short --color never` once the build machine is quiet.

`src/tests/graph_config.rs` is now the structural mixer graph import test entry. `graph_config/import.rs` owns successful imported source, track, automation binding, path normalization, render, and automation application behavior; `graph_config/validation.rs` owns duplicate source-id and invalid automation binding rejection. This keeps whole-graph import coverage aligned with the production `mixer_configuration` and `service_types/mixer_graph/configuration` boundaries.

Focused static validation for the graph-config test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/graph_config.rs` and every child file under `src/tests/graph_config/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker count remained 2; and the line-count audit left the root facade at 2 lines with children between 36 and 42 lines. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-graph-config-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-graph-config-test-split-0604 --message-format short --color never` if the check passes.

`src/tests/mixer_graph.rs` is now the structural mixer-graph test entry. `mixer_graph/routing.rs` owns custom track routing through effect chains plus active-playback rerouting when a track is removed; `sends.rs` is now the structural send coverage entry; `sends/crud.rs` is the structural send CRUD entry, `sends/crud/upsert_snapshot.rs` owns repeated send upsert and snapshot gain projection, `sends/crud/routing.rs` owns send gain routing into the target bus, and `sends/crud/removal_errors.rs` owns send removal plus missing send/target errors; `sends/cycles.rs` owns send-cycle rejection; `solo.rs` owns solo/mute routing semantics; `sidechain.rs` owns compressor sidechain ducking and pre/post-effect taps; `support.rs` owns the shared sidechain tap render helper; and `validation.rs` owns parent-cycle and missing-track rejection. This keeps audio mixer graph coverage aligned with the production `service_types/mixer_graph`, `engine/render/routing`, and `engine/validation` boundaries.

Focused static validation for the mixer-graph send sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests/mixer_graph/sends.rs` plus `src/tests/mixer_graph/sends/{crud,cycles}.rs`. The 2026-06-05 send CRUD sub-boundary split kept `src/tests/mixer_graph/sends/crud.rs` structural at 3 lines and split send CRUD assertions into `sends/crud/upsert_snapshot.rs` at 42 lines, `sends/crud/routing.rs` at 35 lines, and `sends/crud/removal_errors.rs` at 41 lines; `rustfmt --edition 2021 --check` passed for those four files, `git diff --check` passed with the expected LF-to-CRLF warning on this runtime document, trailing-whitespace and conflict-marker scans returned empty, the send CRUD subtree now has 3 test markers, the full send subtree now has 4 test markers, the full mixer-graph tree now has 10 test markers, and each new child path appears three times in this document's code lists. Focused Cargo validation remains intentionally pending while concurrent Cargo/rustc lanes are active; the existing mixer-graph check/test priority command remains `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never mixer_graph`.

Focused validation for the mixer-graph test split passed on 2026-06-04 with `rustfmt --edition 2021 --check`, `git diff --check`, trailing-whitespace scanning, conflict-marker scanning, and line-count auditing over `src/tests.rs`, `src/tests/graph_config.rs`, `src/tests/mixer_graph.rs`, every child file under `src/tests/mixer_graph/`, this document, and the active session note. The first focused `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never` exposed a stale local `SoundManager` import in `graph_config.rs`; the file now imports the focused mixer-graph, render, and automation timeline traits used by its method calls. The accepted rerun passed with existing `zircon_runtime` warnings and two Sound test warnings. The first unqualified `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never` timed out while detached Sound cargo/rustc processes were still compiling `zircon_runtime`; the captured rerun `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-mixer-graph-test-split-0604 --message-format short --color never mixer_graph` passed with 8 tests, 0 failures, and 95 filtered.

2026-06-04 dynamic-events test boundary split replaced the 729-line `src/tests/dynamic_events.rs` file with a folder-backed `src/tests/dynamic_events/{mod,structure,registry,dispatch,execution,abi}.rs` tree. Later sub-boundary splits reduced `execution.rs` to a structural entry plus `execution/{registration,report,cleanup}.rs`, reduced `execution/cleanup.rs` to a structural entry plus `cleanup/{event_unregister,graph_reconfigure}.rs`, reduced `dispatch.rs` to a structural entry plus `dispatch/{fanout,handlers}.rs`, and reduced `abi.rs` to a structural entry plus `abi/{success,failure,support}.rs`, preserving the existing executor registration, execution-report, cleanup, handler fan-out, handler lifecycle, ABI request projection, and ABI failure-detail assertion bodies while separating static structure checks, registry validation, dispatch fan-out, handler unregister cleanup, executor registration, execution reports, event-unregister executor cleanup, graph-reconfigure executor cleanup, ABI success projection, ABI failure mapping, and shared ABI callback fixtures. The 2026-06-05 execution-report sub-boundary now keeps `execution/report.rs` structural at 4 lines, with `report/outcomes.rs` owning ordered success/failure/missing-executor status assertions, `report/calls.rs` owning executor-call recording, `report/drain.rs` owning post-report queue draining, and `report/support.rs` owning the shared weapon-fire report fixture. Focused static checks passed for the report subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, 3 report test markers, 15 dynamic-event test markers, and each new report child path appearing three times in this document's code lists. The 2026-06-05 handler-lifecycle sub-boundary keeps `dispatch/handlers.rs` structural at 4 lines, with `handlers/ownership.rs` owning registered-event validation, `handlers/unregistration.rs` owning handler removal and missing-handler errors, `handlers/event_cleanup.rs` owning event unregister cleanup of handler state and queued deliveries, and `handlers/support.rs` now structural over shared ambient-stinger helpers. Focused static checks passed for the handler subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, 3 handler test markers, 17 dynamic-event test markers, and each handler-lifecycle child path appearing three times in this document's code lists. The 2026-06-05 handler support sub-boundary keeps `dispatch/handlers/support.rs` structural at 6 lines, with `support/ids.rs` owning shared ambient constants, `support/registration.rs` owning event and handler registration, and `support/submission.rs` owning queued ambient invocation submission. Focused static checks passed for the latest handler-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 6, `support/ids.rs` 2, `support/registration.rs` 25, `support/submission.rs` 15, 3 handler test markers, 28 dynamic-event test markers, and each new handler-support child path appearing three times in this document's code lists. The 2026-06-05 execution-registration sub-boundary keeps `execution/registration.rs` structural at 3 lines, with `registration/missing_handler.rs` owning missing-handler rejection, `registration/success.rs` owning successful registered-handler executor registration, and `registration/support.rs` now structural over `support/{ids,registration}.rs` for registered-handler ids plus event/handler registration. Focused static checks passed for the latest registration-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths with the expected LF-to-CRLF warning, empty trailing-whitespace and conflict-marker scans, line counts `registration.rs` 3, `registration/missing_handler.rs` 13, `registration/success.rs` 13, `registration/support.rs` 5, `registration/support/ids.rs` 4, `registration/support/registration.rs` 22, 2 registration test markers, 7 execution test markers, 28 dynamic-event test markers, and each new registration-support child path appearing three times in this document's code lists. Focused Cargo validation remains pending because active Cargo/rustc lanes were still running at 2026-06-05 04:08 +08:00. The 2026-06-05 execution-report support sub-boundary keeps `execution/report/support.rs` structural at 7 lines, with `support/ids.rs` owning shared event constants, `support/registration.rs` owning event and handler registration, `support/executors.rs` owning executor callback registration and call recording, `support/submission.rs` owning pending invocation submission, and `support/fixture.rs` owning report fixture assembly. Focused static checks passed for the latest report-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 7, `support/executors.rs` 26, `support/fixture.rs` 22, `support/ids.rs` 2, `support/registration.rs` 31, `support/submission.rs` 15, 3 report test markers, 7 execution test markers, 28 dynamic-event test markers, and each new report-support child path appearing three times in this document's code lists. The 2026-06-05 ABI support sub-boundary keeps `abi/support.rs` structural at 6 lines and `abi/support/callbacks.rs` structural at 5 lines, with `support/detail.rs` owning shared ABI ids and failure detail bytes, `support/registration.rs` owning event and handler registration, `callbacks/capture.rs` owning successful ABI request assertion, and `callbacks/failure.rs` owning failure-detail callback mapping. Focused static checks passed for the latest ABI-support subtree with `rustfmt --edition 2021 --check`, `git diff --check` over the touched Rust/docs/session paths, empty trailing-whitespace and conflict-marker scans, line counts `support.rs` 6, `support/callbacks.rs` 5, `callbacks/capture.rs` 31, `callbacks/failure.rs` 19, `support/detail.rs` 8, `support/registration.rs` 26, 2 ABI test markers, 28 dynamic-event test markers, and each new ABI-support child path appearing three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running.

Focused validation after adding the dynamic-event folder-backed contract passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-dynamic-events-boundary`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests/dynamic_events.rs` passed. The first cold-target `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` attempt timed out after 20 minutes during dependency compilation without Rust diagnostics; exact residual processes for that target directory were stopped. The warmed retry passed with 11 dynamic-event tests, 0 failures, and 87 filtered tests. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` also passed; remaining output was limited to existing `zircon_runtime` warnings.

Focused validation after adding optional feature bundle manifest parity passed on 2026-05-30 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-optional-feature-manifest`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` and `rustfmt --edition 2021 --check zircon_plugins/sound/runtime/src/tests.rs zircon_plugins/sound/runtime/src/tests/manifest.rs zircon_plugins/sound/runtime/src/tests/optional_feature_manifest.rs zircon_plugins/sound/runtime/src/tests/dynamic_events.rs` passed. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml manifest --locked --offline --jobs 1 --message-format short --color never` passed with 4 manifest tests, 0 failures, and 95 filtered tests after the optional feature parser was split out of the oversized manifest test file. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` also passed; remaining output was limited to existing `zircon_runtime` warnings.

`src/tests/playback.rs` now owns the Bevy-inspired source and playback lifecycle regression cases that were previously embedded in the large runtime test aggregate: source speed/mute controls, sink-style source controls, start/duration ranges, cleanup intent, playback presets, invalid initial mix parameters, pause/resume/mute/speed status, seek/range handling, and finished playback reports. Keeping these tests folder-backed makes future playback work independent from mixer graph, DSP, spatial, and manifest coverage.

`src/tests/presets.rs` is now the structural mixer-preset test entry. `presets/catalog_apply.rs` owns built-in preset discovery and successful spatial-room application; `reroute.rs` owns source and active-playback rerouting after graph replacement removes a temporary track; and `validation.rs` owns typed unknown-locator rejection. This keeps preset catalog/application coverage aligned with `src/presets/` and `src/service_types/mixer_presets.rs` instead of mixing catalog, graph replacement, render, and error assertions in one file.

Focused static validation for the preset test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/presets.rs` and every child file under `src/tests/presets/`; trailing-whitespace scanning over the same Rust files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with expected LF-to-CRLF warnings on tracked files; the test marker count remained 3; and the line-count audit left the root facade at 3 lines with children between 15 and 47 lines. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml presets --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-presets-test-split-0604 --message-format short --color never` if the check passes.

`src/tests/source_inputs.rs` remains the source-input regression boundary, but behavioral coverage is now folder-backed: `source_inputs/external_routing.rs` owns external audio block routing, completion, and finish-report projection; `external_lifecycle.rs` is now the structural external-source lifecycle entry, `external_lifecycle/validation.rs` owns invalid handles, invalid blocks, and invalid external source descriptors, `external_lifecycle/clearing.rs` owns unknown-source and successful external-source clearing, `external_lifecycle/missing_blocks.rs` owns cleared-source missing-block silence; `resampling.rs` owns clip and external input resampling to the mixer rate; and `parameter_bindings.rs` owns synth-parameter-backed source gain plus invalid binding/default validation. This keeps source ingestion coverage separate from playback lifecycle, mixer graph, DSP, and spatial tests.

Initial focused static validation for the source-input test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/source_inputs.rs` and every child file under `src/tests/source_inputs/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker count remained 4 at that split point; and the line-count audit left the root facade at 4 lines with children between 35 and 73 lines. The 2026-06-05 external-source lifecycle sub-boundary split kept `src/tests/source_inputs/external_lifecycle.rs` structural at 3 lines and split lifecycle assertions into `external_lifecycle/validation.rs` at 51 lines, `external_lifecycle/clearing.rs` at 21 lines, and `external_lifecycle/missing_blocks.rs` at 24 lines; `rustfmt --edition 2021 --check` passed for those four files, `git diff --check` passed with the expected LF-to-CRLF warning on this runtime document, trailing-whitespace and conflict-marker scans returned empty, the external lifecycle subtree now has 3 test markers, the full source-input tree now has 6 test markers, and each new child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-source-inputs-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-source-inputs-test-split-0604 --message-format short --color never` if the check passes.

`src/tests/automation_binding.rs` remains the synth-parameter visibility and automation binding test boundary, but behavioral coverage is now folder-backed: `automation_binding/snapshot.rs` owns snapshot visibility for bound synth parameters and dynamic-event emptiness; `path.rs` owns shared animation-track path normalization; `apply.rs` is the structural automation-value application entry, with `apply/synth_parameter.rs` owning synth parameter value application and render projection, `apply/track.rs` owning track gain application, and `apply/effect.rs` owning effect wet automation; and `validation.rs` is the structural typed-failure entry, with `validation/missing_binding.rs` owning missing binding application errors, `validation/path.rs` owning blank and malformed timeline track paths, `validation/unsupported_parameter.rs` owning unsupported parameter application errors, and `validation/unknown_source.rs` owning missing source target reports. `src/tests/automation_curve.rs` remains the automation curve and timeline sequence test boundary, but behavioral coverage is now folder-backed: `automation_curve/sampling.rs` is the structural curve-sampling entry, with `sampling/bound_parameter.rs` owning linear and smooth-step sampling against bound synth parameters and `sampling/step_clamping.rs` owning step interpolation plus endpoint clamping; `validation.rs` owns empty, unsorted, and non-finite keyframe rejection; `timeline_once.rs` owns one-shot timeline advancement and completion cleanup; and `timeline_loop.rs` is the structural looping sequence entry, with `timeline_loop/scheduling_validation.rs` owning empty-track and unknown-binding scheduling errors and `timeline_loop/wraparound.rs` owning looping wraparound plus live-sequence retention. This keeps binding target resolution independent from curve sampling and timeline scheduling behavior.

Focused static validation for the automation-binding test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/automation_binding.rs` and every child file under `src/tests/automation_binding/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker count remained 4 at that split point; and the line-count audit left the root facade at 4 lines with children between 19 and 62 lines. The 2026-06-05 automation apply sub-boundary split kept `src/tests/automation_binding/apply.rs` structural at 3 lines and split value-application assertions into `apply/synth_parameter.rs` at 26 lines, `apply/track.rs` at 25 lines, and `apply/effect.rs` at 49 lines; `rustfmt --edition 2021 --check` passed for those four files, `git diff --check` passed with the expected LF-to-CRLF warning on this runtime document, trailing-whitespace and conflict-marker scans returned empty, the apply subtree now has 3 test markers, the full automation-binding tree then had 6 test markers, and each new child path appears three times in this document's code lists. The 2026-06-05 automation validation sub-boundary split kept `src/tests/automation_binding/validation.rs` structural at 4 lines and split typed-failure assertions into `validation/missing_binding.rs` at 12 lines, `validation/path.rs` at 25 lines, `validation/unsupported_parameter.rs` at 19 lines, and `validation/unknown_source.rs` at 20 lines; focused static checks passed for the validation subtree, trailing-whitespace and conflict-marker scans returned empty, the validation subtree now has 4 test markers, the full automation-binding tree now has 9 test markers, and each new validation child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-binding-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-binding-test-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the automation-curve test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/automation_curve.rs` and every child file under `src/tests/automation_curve/`; `git diff --check -- src/tests/automation_curve.rs src/tests/automation_curve docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with expected LF-to-CRLF warnings; trailing-whitespace and conflict-marker scans returned empty; the test marker count remained 5 at that split point; and the line-count audit left the root facade at 4 lines with children between 36 and 71 lines. The 2026-06-05 timeline-loop sub-boundary split kept `src/tests/automation_curve/timeline_loop.rs` structural at 2 lines and split looping assertions into `timeline_loop/scheduling_validation.rs` at 32 lines and `timeline_loop/wraparound.rs` at 35 lines; focused static checks passed for the timeline-loop subtree, trailing-whitespace and conflict-marker scans returned empty, the timeline-loop subtree now has 2 test markers, the full automation-curve tree now has 6 test markers, and each new timeline-loop child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-curve-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-automation-curve-test-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the automation-curve sampling sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/automation_curve/sampling.rs` and `sampling/{bound_parameter,step_clamping}.rs`; test-marker auditing found the two moved sampling tests in the two child files; trailing-whitespace scanning returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; and the line-count audit left `sampling.rs` at 2 lines, `sampling/bound_parameter.rs` at 35 lines, and `sampling/step_clamping.rs` at 37 lines. Focused Cargo validation remains pending because separate editor/Hub/runtime Cargo/rustc lanes were active at 2026-06-04 23:41 +08:00; retry the same automation-curve `cargo check`, then run focused automation-curve tests if the check passes.

`src/tests/runtime_core.rs` remains the runtime-plugin registration and default-manager baseline test boundary, but behavioral coverage is now folder-backed: `runtime_core/registration.rs` is the structural registration entry, with `registration/modules.rs` owning runtime module and target-mode contribution checks, `registration/components.rs` owning runtime/package component descriptors, `registration/options.rs` owning extension/package option rows, `registration/dependencies.rs` owning optional timeline dependency projection, and `registration/dynamic_events.rs` owning concrete dynamic-event catalog contribution checks; `render_defaults.rs` owns silent render defaults; `global_volume.rs` owns final global-volume gain scaling and invalid value rejection; `config_options.rs` is the structural plugin-option preservation entry, with `config_options/option_values.rs` owning neutral `SoundPluginOptions` to `SoundConfig` value preservation, `config_options/manager_projection.rs` owning manager and render-mix projection of those values, and `config_options/support.rs` owning the shared cinematic fixture; and `config_normalization.rs` owns zero channel-count and layout mismatch normalization through output-device status and render output. `src/tests.rs` now remains a navigation and shared-fixture module instead of owning behavioral assertions.

Focused static validation for the runtime-core test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/runtime_core.rs` and every child file under `src/tests/runtime_core/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker count remained 5 at that split point; and the line-count audit left the root facade at 5 lines with children between 9 and 95 lines. The 2026-06-05 runtime-core registration sub-boundary split kept `src/tests/runtime_core/registration.rs` structural at 5 lines and split registration assertions into `registration/modules.rs` at 18 lines, `registration/components.rs` at 21 lines, `registration/options.rs` at 40 lines, `registration/dependencies.rs` at 10 lines, and `registration/dynamic_events.rs` at 26 lines; focused static checks passed for the registration subtree, trailing-whitespace and conflict-marker scans returned empty, the registration subtree now has 5 test markers, the full runtime-core tree then had 9 test markers, and each new registration child path appears three times in this document's code lists. The 2026-06-05 runtime-core config-options sub-boundary split kept `src/tests/runtime_core/config_options.rs` structural at 3 lines and split plugin-option assertions into `config_options/option_values.rs` at 25 lines, `config_options/manager_projection.rs` at 13 lines, and `config_options/support.rs` at 30 lines; focused static checks passed for the config-options subtree, trailing-whitespace and conflict-marker scans returned empty, the config-options subtree now has 2 test markers, the full runtime-core tree now has 10 test markers, and each new config-options child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-runtime-core-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-runtime-core-test-split-0604 --message-format short --color never` if the check passes.

2026-06-01 M6 plugin-workspace validation exposed that `sound_plugin_registration_contributes_runtime_module_components_options_and_events` still expected the old placeholder event catalog. The production event catalog was already concrete, so the test now asserts the three stable event ids and payload schemas listed above. The focused rerun `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sound_runtime sound_plugin_registration_contributes_runtime_module_components_options_and_events --locked --jobs 1 --message-format short --color never` passed with 1 test and 0 failures.

Focused validation after tightening the Timeline binding path contract passed on 2026-05-31 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-timeline-path-contract`. `cargo fmt --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_sound_runtime -- --check` passed. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml automation --locked --offline --jobs 1 --color never` passed with 10 automation/Timeline/graph-import tests, 0 failed, and 92 unrelated tests filtered out. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml graph_config --locked --offline --jobs 1 --color never` passed with 2 graph-import tests, 0 failed, and 100 unrelated tests filtered out. `cargo check --manifest-path zircon_plugins\sound\runtime\Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` and `cargo metadata --manifest-path zircon_plugins\sound\runtime\Cargo.toml --locked --offline --no-deps --format-version 1` passed. Remaining output was limited to existing `zircon_runtime` warnings.

Focused validation after adding Sound optional feature editor-module capability parity passed on 2026-05-31 with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-feature-editor-capability`. `rustfmt --edition 2021 --check` passed for the touched Sound feature provider/editor files, Sound runtime feature-manifest file, and runtime catalog/test files. The first provider run intentionally failed before the fix because `sound.timeline_animation_track.editor` had no `editor.feature.sound.timeline_animation_track` capability. After the fix, `cargo test --manifest-path zircon_plugins\sound\features\timeline_animation_track\runtime\Cargo.toml timeline_feature_provider_manifest_matches_sound_owner_contract --locked --offline --jobs 1 --color never` and `cargo test --manifest-path zircon_plugins\sound\features\ray_traced_convolution_reverb\runtime\Cargo.toml ray_traced_feature_provider_manifest_matches_sound_owner_contract --locked --offline --jobs 1 --color never` each passed with 1 test. `cargo test --manifest-path zircon_plugins\sound\runtime\Cargo.toml manifest --locked --offline --jobs 1 --color never` passed with 5 manifest tests, including static optional feature parity. `cargo test --manifest-path Cargo.toml -p zircon_runtime builtin_sound_optional_features_declare_editor_capabilities --locked --offline --jobs 1 --color never` passed with 1 focused runtime catalog test. `cargo check` passed for both feature editor crates, and `cargo metadata --locked --offline --no-deps --format-version 1` passed for the Sound runtime plus both feature runtime/editor manifests. Remaining output was limited to existing `zircon_runtime` and `zircon_editor` warnings.

`src/tests/convolution.rs` remains the static convolution and impulse-response lifecycle test boundary, but behavioral coverage is now folder-backed: `convolution/static_ir.rs` owns master-track static IR processing; `lifecycle.rs` owns static IR cache invalidation and unknown-IR removal errors; and `ray_status.rs` owns ray-tracing convolution status visibility plus invalid ray-count rejection. `src/tests/ray_tracing.rs` remains the provider-fed ray-traced impulse-response test boundary, but behavioral coverage is now folder-backed: `ray_tracing/provider_status.rs` owns provider submission, status projection, cached descriptor listing, and convolution output; `occlusion.rs` owns ray-traced occlusion gain overriding the static fallback; `cache.rs` owns clearing provider-fed impulse responses plus static IR invalidation; and `validation.rs` is now a structural provider-validation entry. `validation/descriptor.rs` owns the shared valid provider descriptor fixture, while `validation/cell_key.rs`, `validation/source.rs`, `validation/ray_count.rs`, and `validation/occlusion.rs` isolate empty cell keys, missing source references, zero-ray reports, and invalid occlusion gain.

Focused static validation for the convolution test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/convolution.rs` and every child file under `src/tests/convolution/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker count remained 3; and the line-count audit left the root facade at 3 lines with children between 29 and 41 lines. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-convolution-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml convolution --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-convolution-test-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the ray-tracing test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/ray_tracing.rs` and every child file under `src/tests/ray_tracing/`; trailing-whitespace scanning over the same Rust files returned empty; the test marker audit found 7 focused ray-tracing tests after the provider-validation test was split by error category; and the line-count audit left the root facade at 4 lines, the validation facade at 5 lines, validation children between 15 and 18 lines, and other first-level children between 31 and 54 lines. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-ray-tracing-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-ray-tracing-test-split-0604 --message-format short --color never` if the check passes. The 2026-06-04 provider-validation sub-boundary split specifically reduced `src/tests/ray_tracing/validation.rs` to structural module wiring and moved empty cell key, missing source, zero rays, and invalid occlusion assertions into dedicated child tests while keeping the same assertion targets.

`src/tests/common.rs` is now the structural shared-fixture entry. `common/assets.rs` owns mono clip asset construction and explicit sample-rate clip construction; `listener.rs` owns default listener descriptors; `effects.rs` owns effect descriptor construction; and `assertions.rs` owns near-equality sample assertions. The root re-exports the helper functions for sibling test modules, while the child helpers remain scoped to `crate::tests` rather than becoming public runtime API.

Focused static validation for the shared common fixture split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/common.rs` and every child file under `src/tests/common/`; trailing-whitespace scanning over the same Rust files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with expected LF-to-CRLF warnings on tracked files; and the line-count audit left the root facade at 9 lines with children between 7 and 19 lines. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-common-fixtures-split-0604 --message-format short --color never`.

`src/tests/spatial.rs` remains the spatial sound-domain test boundary, but the behavioral coverage is now folder-backed: `spatial/hrtf.rs` stays structural over HRTF catalog, loaded-kernel, tail, parameter-playback, preview, and validation modules; `spatial/hrtf/loaded.rs` stays structural over `loaded/{kernels,tail_state}.rs`; `spatial/hrtf/preview.rs` stays structural over `preview/{ear_delay,fallback}.rs`; `spatial/scale.rs` stays structural over `scale/default.rs` and `scale/source_override.rs`; `spatial/listener.rs` covers attenuation/pan/occlusion, `spatial/volumes.rs` covers AudioVolume influence, and `spatial/sends.rs` covers pre-spatial sends. Keeping the root, loaded, and preview files structural prevents future 3D audio and acoustics cases from mixing unrelated assertions in one file.

`src/tests/mixer_graph.rs` remains the mixer graph and routing regression boundary, but behavioral coverage is now folder-backed: `mixer_graph/routing.rs` stays structural over route behavior modules, `mixer_graph/routing/effect_chain.rs` covers effect-chain routing, `mixer_graph/routing/track_removal.rs` covers active-playback reroutes after track removal, `mixer_graph/sends.rs` stays structural over send CRUD/routing and cycle modules, `mixer_graph/sends/crud.rs` stays structural over send CRUD behavior modules, `mixer_graph/sends/crud/upsert_snapshot.rs` covers send upsert and snapshot gain projection, `mixer_graph/sends/crud/routing.rs` covers send gain routing, `mixer_graph/sends/crud/removal_errors.rs` covers send removal and missing target reports, `mixer_graph/sends/cycles.rs` covers send cycles, `mixer_graph/solo.rs` covers solo routing, `mixer_graph/sidechain.rs` stays structural over sidechain behavior modules, `mixer_graph/sidechain/compressor.rs` covers compressor ducking from another track, `mixer_graph/sidechain/taps.rs` covers pre-effect and post-effect tap selection, and `mixer_graph/validation.rs` covers parent-cycle and missing-track rejection. Keeping the root, route, sends, CRUD, and sidechain files structural prevents future bus, return, sidechain, track-removal, send policy, and mixer policy tests from mixing unrelated assertions in one file.

Focused static validation for the mixer-graph routing sub-boundary split on 2026-06-05 kept `src/tests/mixer_graph/routing.rs` structural at 2 lines and moved the two routing assertions into `routing/effect_chain.rs` (37 lines) and `routing/track_removal.rs` (31 lines). `rustfmt --edition 2021 --check` over those three files passed; `git diff --check -- src/tests/mixer_graph/routing.rs src/tests/mixer_graph/routing docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; and the test-marker audit found one test in each child file and none in the route facade. Focused Cargo validation remains pending while other Cargo/rustc lanes are active; rerun the mixer-graph `cargo check` and focused `cargo test --lib mixer_graph` commands from the 2026-06-04 mixer-graph validation record when the machine is quiet.

Focused static validation for the mixer-graph sidechain sub-boundary split on 2026-06-05 kept `src/tests/mixer_graph/sidechain.rs` structural at 2 lines and moved the two sidechain assertions into `sidechain/compressor.rs` (58 lines) and `sidechain/taps.rs` (9 lines). `rustfmt --edition 2021 --check` over those three files passed; `git diff --check -- src/tests/mixer_graph/sidechain.rs src/tests/mixer_graph/sidechain docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with the expected LF-to-CRLF warning on this document; trailing-whitespace and conflict-marker scans returned empty; the test-marker audit found one test in each child file and none in the sidechain facade; and both new sidechain child files appear three times in this document's code lists. Focused Cargo validation remains pending while other Cargo/rustc lanes are active; rerun the mixer-graph `cargo check` and focused `cargo test --lib mixer_graph` commands from the 2026-06-04 mixer-graph validation record when the machine is quiet.

`src/tests/playback.rs` remains the Bevy-inspired source and playback lifecycle regression boundary, but behavioral coverage is now folder-backed: `playback/source_controls.rs` is the structural source-control entry, `source_controls/descriptor.rs` covers source descriptor speed/mute settings, `source_controls/runtime.rs` covers sink-style source pause/resume/gain/mute/speed controls, `playback/source_range.rs` covers source start/duration range validation and looping cursor behavior, `playback/source_completion.rs` is the structural source-cleanup event entry, `source_completion/completed.rs` covers rendered source completion cleanup intent, `source_completion/stopped.rs` covers stopped source cleanup intent for clip and external inputs, `playback/settings.rs` is the structural playback-settings entry, `settings/presets.rs` covers Bevy-style playback presets and fluent builder overrides, `settings/validation.rs` covers invalid initial gain and pan parameters, `playback/controls.rs` is the structural sink-control entry, `controls/initial_state.rs` covers paused/muted initial status and cursor behavior, `controls/transport.rs` covers pause/resume/toggle state changes, `controls/gain_mute.rs` covers gain and mute render/status changes, `controls/speed_completion.rs` covers speed validation plus completion-after-control errors, `playback/range.rs` covers playback start/duration/seek/loop range handling, and `playback/completion.rs` covers finished and stopped playback reports. Keeping the root and controls files structural prevents future sink, transport, range, completion, and low-level source-control assertions from accumulating in one mixed playback test file.

`src/tests/dsp_state.rs` now owns both stateful DSP regression coverage and the deterministic single-block DSP effect checks that used to live in the runtime test aggregate: bypass/wet-dry behavior, delay, pan/phase, limiter, filter, reverb, waveshaper, flanger, phaser, chorus, state continuity, latency snapshots, parameter validation, and sidechain reference validation. One-block deterministic effect coverage is now subfolder-backed under `dsp_state/deterministic/`, with modulation effects further isolated under `dsp_state/deterministic/modulation/{flanger,phaser,chorus}.rs`; validation coverage is now subfolder-backed under `dsp_state/validation/`; and cross-block state continuity is now subfolder-backed under `dsp_state/stateful/`, with modulation state further isolated under `dsp_state/stateful/modulation/{flanger_history,phaser_phase}.rs`, so future delay-line, convolution, reverb, modulation, dynamics, parameter validation, sidechain, or additional effect-history assertions can grow without turning `deterministic.rs`, `deterministic/modulation.rs`, `validation.rs`, `stateful.rs`, or `stateful/modulation.rs` into mixed files.

Focused static validation for the playback test split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/playback.rs` and every child file under `src/tests/playback/`; `git diff --check -- src/tests/playback.rs src/tests/playback docs/zircon_plugins/sound/runtime.md .codex/sessions/20260603-2304-plugin-ecosystem-continuation.md` passed with expected LF-to-CRLF warnings; trailing-whitespace and conflict-marker scans returned empty; and the line-count audit left the root facade at 7 lines with children between 49 and 86 lines. The 2026-06-05 playback controls sub-boundary split kept `src/tests/playback/controls.rs` structural at 4 lines and split sink-control assertions into `controls/initial_state.rs` at 30 lines, `controls/transport.rs` at 39 lines, `controls/gain_mute.rs` at 35 lines, and `controls/speed_completion.rs` at 33 lines; `rustfmt --edition 2021 --check` passed for those five files, `git diff --check` passed with the expected LF-to-CRLF warning on this runtime document, trailing-whitespace and conflict-marker scans returned empty, the controls subtree now has 4 test markers, the full playback tree now has 13 test markers, and each new child path appears three times in this document's code lists. Focused Cargo validation remains pending while other active Cargo/rustc lanes are running; the intended commands are `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-test-split-0604 --message-format short --color never`, followed by `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-test-split-0604 --message-format short --color never` if the check passes.

Focused static validation for the playback source-completion sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/playback/source_completion.rs` and `source_completion/{completed,stopped}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the two moved source-completion tests in `source_completion/completed.rs` and `source_completion/stopped.rs`; and the line-count audit left `source_completion.rs` at 2 lines, `source_completion/completed.rs` at 32 lines, and `source_completion/stopped.rs` at 45 lines. Focused Cargo validation remains pending because separate editor/runtime/Hub Cargo/rustc lanes were active at 2026-06-04 23:20 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-test-split-0604 --message-format short --color never`, followed by focused playback tests if the check passes.

Focused static validation for the playback source-controls sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/playback/source_controls.rs` and `source_controls/{descriptor,runtime}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the two moved source-control tests in `source_controls/descriptor.rs` and `source_controls/runtime.rs`; and the line-count audit left `source_controls.rs` at 2 lines, `source_controls/descriptor.rs` at 24 lines, and `source_controls/runtime.rs` at 50 lines. Focused Cargo validation remains pending because separate runtime/render Cargo/rustc lanes were active at 2026-06-04 23:25 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-test-split-0604 --message-format short --color never`, followed by focused playback tests if the check passes.

Focused static validation for the playback settings sub-boundary split passed on 2026-06-04 with `rustfmt --edition 2021 --check` over `src/tests/playback/settings.rs` and `settings/{presets,validation}.rs`; trailing-whitespace scanning over the same Rust/docs/session files returned empty; conflict-marker scanning returned empty; `git diff --check` passed with the expected LF-to-CRLF warning on this document; the test-marker audit found the two moved playback-settings tests in `settings/presets.rs` and `settings/validation.rs`; and the line-count audit left `settings.rs` at 2 lines, `settings/presets.rs` at 50 lines, and `settings/validation.rs` at 21 lines. Focused Cargo validation remains pending because separate editor/runtime Cargo/rustc lanes were active at 2026-06-04 23:47 +08:00; the intended command is `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-sound-playback-test-split-0604 --message-format short --color never`, followed by focused playback tests if the check passes.

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
