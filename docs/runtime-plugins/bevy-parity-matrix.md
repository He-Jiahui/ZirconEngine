---
related_code:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/job_scheduler.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_builder.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/generated_files.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_app/src/prelude.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/shared.slint
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - .codex/skills/zircon-dev/reporting.md
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/clip_event.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/editor/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/sound/plugin.toml
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/execution.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/registration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/unregistration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/effects.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/snapshot.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/tracks.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/status.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/components.rs
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
  - zircon_plugins/sound/runtime/src/mixer_configuration/mod.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/automation.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/runtime_state.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/timeline.rs
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
  - zircon_plugins/sound/runtime/src/output/software.rs
  - zircon_plugins/sound/runtime/src/output/ring_buffer.rs
  - zircon_plugins/sound/runtime/src/engine/state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/state/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/engine/state/graph.rs
  - zircon_plugins/sound/runtime/src/engine/state/playback.rs
  - zircon_plugins/sound/runtime/src/engine/state/snapshot.rs
  - zircon_plugins/sound/runtime/src/engine/state/source.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/engine/render/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/clip.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/finish.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mixing.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/pan.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/external.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/input.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/parameters.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/range.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/frame.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/interpolation.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/position.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/step.rs
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
  - zircon_plugins/sound/runtime/src/engine/validation/mod.rs
  - zircon_plugins/sound/runtime/src/engine/validation/effect.rs
  - zircon_plugins/sound/runtime/src/engine/validation/graph.rs
  - zircon_plugins/sound/runtime/src/engine/validation/ordering.rs
  - zircon_plugins/sound/runtime/src/engine/validation/references.rs
  - zircon_plugins/sound/runtime/src/engine/validation/track.rs
  - zircon_plugins/sound/runtime/src/engine/validation/values.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/runtime/src/tests.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/tests.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests.rs
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests.rs
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/tests.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/websocket.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/transport.rs
  - zircon_runtime/src/core/framework/net/session.rs
  - zircon_runtime/src/core/framework/net/reliable.rs
  - zircon_runtime/src/core/framework/net/diagnostics.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/module.rs
  - zircon_plugins/particles/runtime/src/package.rs
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/simulation/pool.rs
  - zircon_plugins/particles/runtime/src/simulation/rng.rs
  - zircon_plugins/particles/runtime/src/render/extract.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/render/gpu/layout.rs
  - zircon_plugins/particles/runtime/src/render/gpu/planner.rs
  - zircon_plugins/particles/runtime/src/render/gpu/program.rs
  - zircon_plugins/particles/runtime/src/render/gpu/readback.rs
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
  - zircon_plugins/particles/runtime/src/interop/animation.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/prefab_tools/plugin.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/tilemap_2d/plugin.toml
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/zr_vm_language/plugin.toml
implementation_files:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/pools.rs
  - zircon_runtime/src/core/job_scheduler.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/plugin/runtime_profile.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/plugin/export_profile.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection.rs
  - zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_selection_builder.rs
  - zircon_runtime/src/plugin/export_build_plan/export_build_plan.rs
  - zircon_runtime/src/plugin/export_build_plan/export_materialize_report.rs
  - zircon_runtime/src/plugin/export_build_plan/from_project_manifest.rs
  - zircon_runtime/src/plugin/export_build_plan/generated_files.rs
  - zircon_runtime/src/plugin/export_build_plan/materialize.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor.rs
  - zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_descriptor_builder.rs
  - zircon_runtime/src/plugin/runtime_plugin/package_manifest.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/capability_status.rs
  - zircon_runtime/src/plugin/plugin_maturity.rs
  - zircon_app/src/prelude.rs
  - zircon_app/src/plugins/builder.rs
  - zircon_app/src/plugins/groups.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status_report.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/builtin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_hub/ui/plugins.slint
  - zircon_hub/ui/shared.slint
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - .codex/skills/zircon-dev/validation/SKILL.md
  - .codex/skills/zircon-dev/validation/manual-commands.md
  - .codex/skills/zircon-dev/reporting.md
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/clip_event.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/animation/editor/src/lib.rs
  - zircon_plugins/audio_importer/plugin.toml
  - zircon_plugins/gltf_importer/plugin.toml
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/sound/plugin.toml
  - zircon_runtime/src/core/framework/sound/manager.rs
  - zircon_runtime/src/core/framework/sound/playback.rs
  - zircon_runtime/src/core/framework/sound/output.rs
  - zircon_runtime/src/core/framework/sound/status.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/graph.rs
  - zircon_runtime/src/core/framework/sound/effects.rs
  - zircon_runtime/src/core/framework/sound/acoustics.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/sound/runtime/src/service_types/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_state.rs
  - zircon_plugins/sound/runtime/src/service_types/manager_trait.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/execution.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/registration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/unregistration.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/dispatch.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/handlers.rs
  - zircon_plugins/sound/runtime/src/service_types/dynamic_events/invocation.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/effects.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/sends.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/snapshot.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/tracks.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/mod.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/backend.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/catalog.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/configuration.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/lifecycle.rs
  - zircon_plugins/sound/runtime/src/service_types/output_device/status.rs
  - zircon_plugins/sound/runtime/src/config.rs
  - zircon_plugins/sound/runtime/src/components.rs
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
  - zircon_plugins/sound/runtime/src/mixer_configuration/mod.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/automation.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/configure.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/runtime_state.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/sources.rs
  - zircon_plugins/sound/runtime/src/mixer_configuration/timeline.rs
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
  - zircon_plugins/sound/runtime/src/output/software.rs
  - zircon_plugins/sound/runtime/src/output/ring_buffer.rs
  - zircon_plugins/sound/runtime/src/engine/state/mod.rs
  - zircon_plugins/sound/runtime/src/engine/state/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/engine/state/graph.rs
  - zircon_plugins/sound/runtime/src/engine/state/playback.rs
  - zircon_plugins/sound/runtime/src/engine/state/snapshot.rs
  - zircon_plugins/sound/runtime/src/engine/state/source.rs
  - zircon_plugins/sound/runtime/src/engine/state/storage.rs
  - zircon_plugins/sound/runtime/src/engine/render/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/clip.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/finish.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/mixing.rs
  - zircon_plugins/sound/runtime/src/engine/render/playback/pan.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/external.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/input.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/orchestration.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/parameters.rs
  - zircon_plugins/sound/runtime/src/engine/render/source/range.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/mod.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/frame.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/interpolation.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/position.rs
  - zircon_plugins/sound/runtime/src/engine/render/sampling/step.rs
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
  - zircon_plugins/sound/runtime/src/engine/validation/mod.rs
  - zircon_plugins/sound/runtime/src/engine/validation/effect.rs
  - zircon_plugins/sound/runtime/src/engine/validation/graph.rs
  - zircon_plugins/sound/runtime/src/engine/validation/ordering.rs
  - zircon_plugins/sound/runtime/src/engine/validation/references.rs
  - zircon_plugins/sound/runtime/src/engine/validation/track.rs
  - zircon_plugins/sound/runtime/src/engine/validation/values.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_plugins/sound/editor/src/lib.rs
  - zircon_runtime/src/core/framework/render/sprite/anchor.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/bounds.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/core/framework/render/sprite/rect.rs
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_vertex.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_plugins/navigation/plugin.toml
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_runtime/src/core/framework/navigation/mod.rs
  - zircon_runtime/src/asset/assets/navigation.rs
  - zircon_runtime_interface/src/ui/focus.rs
  - zircon_runtime_interface/src/ui/navigation.rs
  - zircon_runtime_interface/src/ui/surface/focus_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation_state.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/surface/navigation/route.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/net/runtime/src/config.rs
  - zircon_plugins/net/runtime/src/service_types.rs
  - zircon_plugins/net/runtime/src/http.rs
  - zircon_plugins/net/runtime/src/websocket.rs
  - zircon_plugins/net/runtime/src/tests.rs
  - zircon_plugins/net/features/http/runtime/src/feature.rs
  - zircon_plugins/net/features/http/runtime/src/tests.rs
  - zircon_plugins/net/features/websocket/runtime/src/feature.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests.rs
  - zircon_plugins/net/features/rpc/runtime/src/feature.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests.rs
  - zircon_plugins/net/features/replication/runtime/src/feature.rs
  - zircon_plugins/net/features/replication/runtime/src/tests.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/feature.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs
  - zircon_plugins/net/features/content_download/runtime/src/feature.rs
  - zircon_plugins/net/features/content_download/runtime/src/manager.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/manager.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/websocket.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/core/framework/net/sync.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/transport.rs
  - zircon_runtime/src/core/framework/net/session.rs
  - zircon_runtime/src/core/framework/net/reliable.rs
  - zircon_runtime/src/core/framework/net/diagnostics.rs
  - zircon_plugins/obj_importer/plugin.toml
  - zircon_plugins/particles/plugin.toml
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/module.rs
  - zircon_plugins/particles/runtime/src/package.rs
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/component.rs
  - zircon_plugins/particles/runtime/src/service.rs
  - zircon_plugins/particles/runtime/src/simulation/cpu.rs
  - zircon_plugins/particles/runtime/src/simulation/pool.rs
  - zircon_plugins/particles/runtime/src/simulation/rng.rs
  - zircon_plugins/particles/runtime/src/render/extract.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/feature.rs
  - zircon_plugins/particles/runtime/src/render/gpu/backend.rs
  - zircon_plugins/particles/runtime/src/render/gpu/layout.rs
  - zircon_plugins/particles/runtime/src/render/gpu/planner.rs
  - zircon_plugins/particles/runtime/src/render/gpu/program.rs
  - zircon_plugins/particles/runtime/src/render/gpu/readback.rs
  - zircon_plugins/particles/runtime/src/render/gpu/shaders.rs
  - zircon_plugins/particles/runtime/src/render/gpu/transparent.rs
  - zircon_plugins/particles/runtime/src/interop/animation.rs
  - zircon_plugins/particles/runtime/src/interop/physics.rs
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/authoring.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/prefab_tools/plugin.toml
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/shader_wgsl_importer/plugin.toml
  - zircon_plugins/solari/plugin.toml
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/texture/plugin.toml
  - zircon_plugins/texture_importer/plugin.toml
  - zircon_plugins/tilemap_2d/plugin.toml
  - zircon_plugins/ui_document_importer/plugin.toml
  - zircon_plugins/virtual_geometry/plugin.toml
  - zircon_plugins/zr_vm_language/plugin.toml
plan_sources:
  - user: 2026-05-08 实现 ZirconEngine Bevy 级插件完成度里程碑计划
  - user: 2026-05-24 继续完善 ZirconEngine 到 Bevy 完成度的详细计划并引用 Bevy 源码
  - .codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_runtime/src/tests/plugin_extensions/profile_maturity.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract.rs
  - zircon_plugins/sound/runtime/src/tests/output_device.rs
  - zircon_plugins/sound/runtime/src/tests/graph_config.rs
  - zircon_plugins/sound/runtime/src/tests/spatial.rs
  - zircon_plugins/sound/runtime/src/tests/dsp_state.rs
  - zircon_plugins/sound/runtime/src/tests/automation_curve.rs
  - zircon_plugins/sound/runtime/src/tests/dynamic_events.rs
  - zircon_plugins/sound/runtime/src/tests/ray_tracing.rs
  - zircon_runtime/src/graphics/tests/render_product_sprite.rs
  - zircon_runtime/src/ui/tests/focus_navigation.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_hub/src/plugins/catalog.rs
  - zircon_hub/src/app/view_model/plugins.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/tests.rs
  - zircon_plugins/net/features/http/runtime/src/tests.rs
  - zircon_plugins/net/features/websocket/runtime/src/tests.rs
  - zircon_plugins/net/features/rpc/runtime/src/tests.rs
  - zircon_plugins/net/features/replication/runtime/src/tests.rs
  - zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs
  - zircon_plugins/net/features/content_download/runtime/src/tests.rs
  - zircon_plugins/particles/runtime/src/tests.rs
  - zircon_plugins/particles/editor/src/tests.rs
  - cargo test -p zircon_runtime --locked plugin_extensions::profile_maturity -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --test runtime_physics_animation_tick_contract --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 -- --nocapture
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
  - cargo test -p zircon_runtime --locked render_product_sprite -- --nocapture
  - cargo test -p zircon_runtime --lib focus_navigation --locked -- --nocapture
  - cargo test -p zircon_runtime --lib widget_range_navigation --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime --locked -- --nocapture
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_editor --locked -- --nocapture
  - cargo test -p zircon_runtime --lib plugin_extensions::export_build_plan --locked -- --nocapture
  - cargo test -p zircon_editor --lib minimal_host_contract --locked --jobs 1 -- --nocapture
  - cargo test -p zircon_editor --lib pane_presentation --locked --jobs 1 -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml plugins --locked --offline --jobs 1 -- --nocapture
  - .github/workflows/ci.yml
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
  - cargo build --workspace --locked --verbose
  - cargo test --workspace --locked --verbose
  - cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose
  - cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose
  - cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose
  - cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose
  - cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1
  - cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1
  - docs-only M0 review: every Bevy DefaultPlugins row has a Zircon profile/maturity decision
  - docs-only M1 review: app/prelude/state/time/tasks/log/diagnostics foundation rows have source evidence
  - docs-only M3 review: animation completion rows have Bevy source evidence and Zircon owner/test paths
  - docs-only M4 review: audio/sound completion rows have Bevy source evidence, Zircon owner/test paths, and advanced-feature divergence
  - docs-only M5 review: sprite completion rows have Bevy sprite/sprite-render evidence, Zircon owner/test paths, and plugin-identity gap
  - docs-only M6 review: navigation split rows have Bevy input-focus/UI-navigation evidence, Zircon UI focus owner/test paths, and gameplay navigation divergence
  - docs-only M7 review: net/remote rows have Bevy RemotePlugin/BRP evidence, Zircon net owner/test paths, and no-default-listener security posture
  - docs-only M8 review: particles/VFX rows have Bevy default-feature absence evidence, Zircon particles owner/test paths, and advanced optional status
  - docs-only M9 review: export/editor UX rows have Bevy feature/plugin-group evidence, Zircon export/profile/editor/Hub owner paths, and shared-report gates
  - docs-only M10 review: docs/CI rows have Bevy workflow/tool evidence, Zircon workflow/validator evidence, and explicit acceptance gates
doc_type: module-detail
---

# Bevy Parity Matrix

This matrix records the M0/M1 plugin metadata layer. It does not claim feature completion for animation, sound, navigation, networking, or particles; it makes their current maturity and capability status explicit so profiles and export can gate them without parsing generic externalized warnings.

## Metadata Model

`PluginMaturity` is now carried by both `RuntimePluginDescriptor` and `PluginPackageManifest`:

- `Core`: built into the runtime profile spine.
- `Stable`: suitable for stable/default profile membership.
- `Beta`: first-party runtime exists but product coverage is still partial.
- `Experimental`: advanced optional capability or high-cost feature.
- `Externalized`: descriptor exists but no linked/native registration was supplied.
- `Stub`: descriptor is a placeholder only.
- `Deprecated`: not valid for new required profile use.

`CapabilityStatusManifest` records per-capability state with `Complete`, `Partial`, `Stub`, `Externalized`, or `Unsupported`, optional target modes, Bevy source references, and notes. Package TOML files can persist the same metadata under `[[capability_statuses]]`.

## M0 Audit Scope

M0 is a vocabulary and ownership gate. It does not promote a subsystem to complete; it proves every Bevy default/minimal/plugin-family precedent has a Zircon row with an explicit profile role, maturity status, and next milestone.

The dominant reference is Bevy because the target is Bevy-grade plugin completion. The stabilizing reference is the current Zircon runtime/plugin/app/export boundary:

- Bevy `DefaultPlugins` installs a broad ordered default group, while `MinimalPlugins` keeps only task pools, frame count, time, and the schedule runner. See `dev/bevy/crates/bevy_internal/src/default_plugins.rs`.
- Bevy `PluginGroupBuilder` supports deterministic order edits, disable/re-enable, replacement, and missing-anchor tests. See `dev/bevy/crates/bevy_app/src/plugin_group.rs`.
- Bevy cargo profiles separate high-level profiles from collections: `default = 2d + 3d + ui + audio`, and `default_app`, `default_platform`, `common_api`, `2d_api`, `2d_bevy_render`, `3d_api`, and `3d_bevy_render` are separate concepts. See `dev/bevy/Cargo.toml` and `dev/bevy/docs/cargo_features.md`.
- Zircon keeps the reusable contract in `zircon_runtime::plugin` and `zircon_runtime::builtin`, while `zircon_app` supplies linked first-party provider registrations. This is the boundary that prevents `zircon_runtime` from depending on `zircon_plugins`.

M0 uses the same target vocabulary as `RuntimeProfileId`: `minimal`, `client_2d`, `client_3d`, `editor`, `dev`, and `server`. Optional advanced work stays outside the default gate unless a later milestone promotes it with tests.

## Bevy Default Coverage Matrix

| Bevy reference area | Bevy evidence | Zircon profile decision | Current status | Next gate |
|---|---|---|---|---|
| Panic/log/task/time/frame/diagnostics minimal spine | `DefaultPlugins` starts with panic/log/task/frame/time/diagnostics; `MinimalPlugins` keeps task/frame/time/schedule runner. | `minimal` owns lifecycle, tasks, time, frame count, diagnostics; `dev` adds diagnostic cadence and plugin/tooling surfaces. | Core profile contracts are represented in `RuntimeProfileDescriptor`; earlier M1/M2 validation exists but this doc-only pass does not re-run it. | G1 profile contract remains the lower gate for every later default profile. |
| Plugin group composition and editing | `PluginGroupBuilder` has `set`, `disable`, `enable`, `add_before`, `add_after`, and order-anchor behavior. | `zircon_app::plugins` owns app-level group composition; runtime profiles own plugin ids/capabilities. | App profile bootstrap tests already select `MinimalPlugins`/`DevPlugins`; builder order edge tests should be treated as profile infrastructure acceptance. | M1/G1 must keep order edits deterministic before broadening defaults. |
| Window/input/platform/a11y | Bevy default includes input, focus, window, accessibility, winit, and gilrs conditionally. | Covered by the broader Bevy completion roadmap and platform/input/a11y plans, not by this runtime-plugin matrix alone. | Platform/input capability docs and active sessions own this area. | Do not use runtime plugin completion to claim platform/input completion. |
| Asset/scene/common API | Bevy `default_app` and `common_api` separate asset/scene/common data types from renderer implementation. | `client_2d`, `client_3d`, `editor`, and `dev` require asset and scene capability before default render/audio/plugin work is considered complete. | Asset importer packages are present and have partial capability rows; active asset readiness work owns product completion. | M0 matrix must keep asset/importer rows visible; asset completion remains a separate milestone. |
| Rendering / default 2D and 3D | Bevy default composes 2D, 3D, UI, post-process, anti-alias, sprite render, PBR, and related render collections. | `rendering` is stable/default profile infrastructure; advanced VG/HGI/Solari stay opt-in. | Rendering is required by client/editor profiles, while active render parity sessions own product details. | Do not let advanced providers close default render gates. |
| UI runtime | Bevy default includes UI and UI render when `ui` profile is selected. | `ui` remains a built-in/runtime capability and profile member, with UI/Text/Focus/A11y milestones tracking Bevy parity details. | Active UI/Text/A11y plans own detailed widget/focus/a11y acceptance. | Runtime plugin matrix tracks membership only. |
| Audio | Bevy default includes `AudioPlugin`; the plugin owns global volume and spatial scale and registers playback systems. | `sound` is required for client/editor profiles but remains `Beta`/`Partial` until M4 sound completion. | Sound runtime, cpal adapter, importer readiness, and docs are actively changing in related sessions. | M4 is the first subsystem promotion candidate, after shared sound validation clears. |
| Animation | Bevy `AnimationPlugin` registers clip/graph assets and PostUpdate systems; tests cover timeline event triggering. | `animation` is optional until M3 proves clip/graph/events/gltf/editor authoring. | Runtime/editor plugin crates exist; profile optional linking is covered by prior app tests. | M3 must validate runtime semantics, not only registration. |
| Sprite2D | Bevy splits `SpritePlugin` from `SpriteRenderPlugin`; default 2D includes sprite API/render paths. | Zircon needs a dedicated `sprite_2d` first-party plugin instead of overloading `texture` or `tilemap_2d`. | No first-party `RuntimePluginId::Sprite2d` / `zircon_plugins/sprite_2d/runtime` exists yet. | M5 creates the id, crate, descriptor, render integration, and tests. |
| UI navigation / focus | Bevy first-party navigation precedent is input focus and directional UI navigation, not navmesh gameplay AI. | Introduce or track `ui_navigation` separately from gameplay `navigation`. | UI focus/a11y work is active elsewhere; gameplay navmesh runtime exists. | M6 must split semantics before profile promotion. |
| Gameplay navigation | Bevy does not provide first-party navmesh/pathfinding as default parity. | Zircon `navigation` remains `Beta` advanced optional. | Runtime/native Recast provider exists and profile optional linking has prior evidence. | M6 gameplay acceptance covers navmesh/path/agent/provider diagnostics, not default parity. |
| Remote/networking | Bevy `RemotePlugin` installs protocol methods, while HTTP transport is separate. | `net.remote_protocol` belongs to `dev`/`editor`; server opt-in is allowed, default clients must not listen. | Net runtime and HTTP/WebSocket/RPC/content feature crates exist. | M7 must prove no default listener and structured protocol errors. |
| Particles/VFX | Bevy core has no first-party particles crate in default. | Zircon particles stay `Experimental` advanced optional. | Particles runtime/render/simulation crates exist. | M8 cannot block default Bevy parity. |

## Core Foundation Coverage Matrix

This matrix covers the original `app / prelude / state / time / tasks / log / diagnostic` gap separately from first-party runtime plugin completion. The Bevy reference is still `DefaultPlugins` and `MinimalPlugins`, but the Zircon owner is split between `zircon_app::plugins` for composition and `zircon_runtime::core` for reusable runtime services.

| Foundation area | Bevy source evidence | Zircon source evidence | Current status | Next gate |
|---|---|---|---|---|
| `DefaultPlugins` / `MinimalPlugins` / dev profile | Bevy `DefaultPlugins` installs panic/log/task/frame/time/diagnostics/platform/asset/render/UI/audio/state families, while `MinimalPlugins` keeps task/frame/time/schedule runner. | `zircon_app/src/plugins/groups.rs` defines `MinimalPlugins`, `DefaultPlugins`, `DevPlugins`, and `HeadlessPlugins`; `zircon_app/src/entry/engine_entry.rs` selects them from `EntryConfig` and `RuntimeProfileId`. | Implemented as app-level module groups, not as runtime-plugin ids. `DevPlugins` adds `LogDiagnosticsModule`; `HeadlessPlugins` drops graphics. | G1 must keep profile-to-group mapping deterministic and keep minimal free of platform/input/render modules. |
| PluginGroup editing | Bevy `PluginGroupBuilder` supports ordered entries, replacement, disable/enable, add-before/add-after, and explicit missing-anchor failures. | `zircon_app/src/plugins/builder.rs` provides `set`, `disable`, `enable`, `add_before`, `add_after`, duplicate/missing/disabled-anchor errors; `zircon_app/src/plugins/tests.rs` covers these behaviors. | Mostly aligned. One deliberate divergence is string module keys instead of Bevy type ids because Zircon modules expose `module_name()`. | Add CI/documented gate so group ordering regressions block profile broadening. |
| Stable prelude | Bevy `bevy_internal/src/prelude.rs` re-exports app/ECS/input/math/time/transform plus feature-gated subsystems and `DefaultPlugins`/`MinimalPlugins`. | `zircon_runtime/src/prelude.rs` re-exports core runtime, state/time/tasks/diagnostics/log/profile/plugin contracts; `zircon_app/src/prelude.rs` layers entry/plugin-group types on top; `zircon_app/src/tests/prelude.rs` smoke-tests both. | Present and usable. The remaining work is stability policy: avoid dumping experimental subsystem internals into the stable prelude. | M1 docs should define stable-vs-experimental prelude admission rules. |
| Runtime state | Bevy `StatesPlugin` inserts `StateTransition` into the schedule and exposes `State`, `NextState`, and transition events. | `zircon_runtime/src/core/state` exposes `State`, `NextState`, hooks, and transition events; `CoreRuntime` offers `init_state`, `set_next_state`, `apply_state_transition`, and hook registration. | Core FSM exists, but it is runtime-owned rather than Bevy ECS schedule-owned. | G1/G2 must prove state transitions can be driven from the runtime schedule without feature-specific branches. |
| Time and frame count | Bevy `TimePlugin` initializes `Time<Real>`, `Time<Virtual>`, `Time<Fixed>`, update strategy, fixed-loop handling; `FrameCountPlugin` is default/minimal. | `zircon_runtime::core::framework::time` exposes real/virtual/fixed clocks; `CoreRuntime` exposes `tick_time`, `advance_time_by`, and frame diagnostics; `TimeModule` and `FrameCountModule` are in minimal groups. | Core surface exists. Product parity still depends on schedule integration and profile diagnostics. | Time tests should cover pause/speed/max-delta/fixed-step overflow and diagnostic emission at profile boundaries. |
| Task pools / jobs | Bevy `TaskPoolPlugin` creates IO, async compute, and compute task pools, then ticks global pools. | `zircon_runtime/src/core/tasks` has `TaskPools`, `TaskPoolOptions`, thread assignment, reports, and a `JobScheduler` facade backed by the compute pool; `TasksModule` is minimal. | Core abstraction exists and is no longer just a Rayon note. | M1 should freeze task-pool reporting and profile-configurable options before asset/render subsystems depend on it. |
| Log and diagnostics | Bevy `LogPlugin`, `DiagnosticsPlugin`, `FrameTimeDiagnosticsPlugin`, and `LogDiagnosticsPlugin` split process logging, diagnostic storage, frame metrics, and console output. | `zircon_runtime::diagnostic_log`, `DiagnosticStore`, `LogModule`, `DiagnosticsCoreModule`, `FrameCountModule`, and `LogDiagnosticsModule`; `zircon_app` parses `--log-level`, `--log-filter`, `ZIRCON_LOG*`, and `RUST_LOG`. | Default/dev split exists: default has log + core diagnostics; dev adds log diagnostics cadence. | G1 should make dev profile diagnostics explicit in profile docs and ensure default profile does not become noisy by accident. |

## Current First-Party Baseline

| Plugin | Maturity | Capability Status | Profile Role |
|---|---:|---:|---|
| `core/runtime`, `core/tasks`, `core/time`, `core/diagnostics` | Core | Complete/Partial by subsystem | Required by `minimal`; lower gate for all profiles. |
| `asset`, importer packages | Stable/Partial | Importer capabilities Partial | Required by client/editor profiles, but asset-stack product completion is tracked by asset milestones. |
| `scene` | Stable/Partial | Scene capability Partial | Required by client/editor/server profiles; ECS/native systems continue in separate scene milestones. |
| `ui` | Stable/Partial | Runtime UI capability Partial | Required by editor/dev and UI/client flows; UI/Text/Focus/A11y parity is owned by UI milestones. |
| `rendering` | Stable | `runtime.plugin.rendering = Complete` | Required by `client_2d`, `client_3d`, `editor`, and `dev` profiles. |
| `texture` | Stable | `runtime.plugin.texture = Complete` | Default optional baseline for client/editor profiles. |
| `sound` | Beta | `runtime.plugin.sound = Partial` | Required by client/editor profiles, but still needs M4 product completion. |
| `animation` | Beta | `runtime.plugin.animation = Partial`; timeline event capability partial | Optional profile capability until M3 completion. |
| `sprite_2d` | Not present | Unsupported until id/crate exist | Required for future Bevy-grade `client_2d`; planned in M5. |
| `ui_navigation` | Not present as runtime-plugin row | Partial through UI/input/a11y layers | Required for Bevy-style UI navigation parity; planned split in M6. |
| `navigation` | Beta | `runtime.plugin.navigation = Partial` | Advanced optional gameplay navmesh; UI navigation parity remains separate. |
| `net` | Beta | `runtime.plugin.net = Partial` | Dev/server optional baseline; remote protocol completion is M7. |
| `particles` | Experimental | `runtime.plugin.particles = Partial` | Advanced VFX optional; not a Bevy default parity blocker. |
| `physics`, `virtual_geometry`, `hybrid_gi`, `solari` | Experimental | Partial | Optional advanced systems outside Bevy default plugin parity; Solari currently registers only the experimental provider contract and reports unavailable until a raytraced lighting pass lands. |

Asset importer plugins (`gltf_importer`, `obj_importer`, `texture_importer`, `audio_importer`, `shader_wgsl_importer`, `ui_document_importer`) are marked Stable at package level with Partial importer capability status because the importer surfaces exist but broader asset-stack completion is owned by a separate asset milestone.

## RuntimePluginId Coverage Matrix

This table closes the M0 audit for the current `RuntimePluginId` enum in `zircon_runtime/src/builtin/runtime_modules.rs`. External runtime-plugin descriptors come from `zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs`; persisted maturity/status evidence comes from the matching `zircon_plugins/*/plugin.toml` files. `Ui` is the one built-in runtime-domain entry: `zircon_runtime/src/plugin/runtime_profile.rs` treats it as `Core`, while `zircon_runtime/src/builtin/runtime_modules.rs` gates the actual module behind the `plugin-ui` feature.

| RuntimePluginId / key | Source evidence | Current maturity / capability status | Profile role | Bevy parity decision |
|---|---|---|---|---|
| `Ui` / `ui` | Enum/profile built-in domain; no external catalog descriptor. | Core built-in profile entry; module availability still feature-gated by `plugin-ui`. | Required by `client_2d`, `client_3d`, `editor`, and `dev`. | Tracks Bevy UI/focus/a11y membership only; UI/Text/Focus/A11y plans own product parity. |
| `Physics` / `physics` | Built-in catalog plus `zircon_plugins/physics/plugin.toml`. | Experimental; `runtime.plugin.physics` Partial and raycast Partial. | Optional for `server`; dependency for sound ray-traced reverb feature. | Not a Bevy default blocker; promote only through a physics/provider milestone. |
| `Sound` / `sound` | Built-in catalog plus `zircon_plugins/sound/plugin.toml`; Bevy reference `bevy_audio/src/lib.rs`. | Beta; `runtime.plugin.sound` Partial. | Required by client/editor/dev profiles. | M4 audio gate must prove playback, spatial settings, importer flow, and diagnostics before promotion. |
| `Texture` / `texture` | Built-in catalog plus `zircon_plugins/texture/plugin.toml`. | Stable; `runtime.plugin.texture` Complete. | Optional default support for client/editor/dev. | Texture is render/asset support; it does not satisfy the future `sprite_2d` row. |
| `Net` / `net` | Built-in catalog plus `zircon_plugins/net/plugin.toml`; Bevy reference `bevy_remote/src/lib.rs`. | Beta; `runtime.plugin.net` Partial. | Optional/non-required default for `dev` and `server`; optional editor use. | M7 remote/dev protocol gate; default clients must not start listeners. |
| `Navigation` / `navigation` | Built-in catalog plus `zircon_plugins/navigation/plugin.toml`. | Beta; `runtime.plugin.navigation` Partial. | Optional for client3d/editor/dev/server. | Gameplay navmesh/pathfinding is separate from Bevy UI navigation; M6 must keep the split explicit. |
| `Particles` / `particles` | Built-in catalog plus `zircon_plugins/particles/plugin.toml`. | Experimental; `runtime.plugin.particles` Partial. | Optional for client/editor/dev. | Advanced VFX; Bevy default parity does not wait on this plugin. |
| `Animation` / `animation` | Built-in catalog plus `zircon_plugins/animation/plugin.toml`; Bevy reference `bevy_animation/src/lib.rs`. | Beta; runtime plugin Partial and timeline event track Partial. | Optional for client/editor/dev/server. | M3 must cover clip/graph/event semantics and glTF/editor interactions before profile promotion. |
| `Terrain` / `terrain` | Built-in catalog plus `zircon_plugins/terrain/plugin.toml`. | Beta; `runtime.plugin.terrain` Partial. | Catalog target client/editor; not currently in built-in profiles. | Advanced world-authoring feature, outside Bevy default plugin parity. |
| `Tilemap2d` / `tilemap_2d` | Built-in catalog plus `zircon_plugins/tilemap_2d/plugin.toml`. | Beta; `runtime.plugin.tilemap_2d` Partial. | Optional for `client_2d`. | 2D extension only; M5 still needs a dedicated Bevy-style `sprite_2d` plugin. |
| `PrefabTools` / `prefab_tools` | Built-in catalog plus `zircon_plugins/prefab_tools/plugin.toml`. | Beta; `runtime.plugin.prefab_tools` Partial. | Catalog target client/editor; not currently in built-in profiles. | Editor/runtime authoring utility; asset/scene prefab gates own completion. |
| `GltfImporter` / `gltf_importer` | Built-in catalog plus `zircon_plugins/gltf_importer/plugin.toml`. | Stable package; `runtime.asset.importer.model.gltf` Partial. | Asset importer for client/editor flows. | Asset milestone owns importer readiness; M3 consumes it for animation/glTF parity. |
| `ObjImporter` / `obj_importer` | Built-in catalog plus `zircon_plugins/obj_importer/plugin.toml`. | Stable package; `runtime.asset.importer.model.obj` Partial. | Asset importer for client/editor flows. | Asset milestone owns product completion. |
| `TextureImporter` / `texture_importer` | Built-in catalog plus `zircon_plugins/texture_importer/plugin.toml`. | Stable package; `runtime.asset.importer.texture.image` Partial. | Asset importer for client/editor flows. | Asset readiness owns image/container/metadata completion. |
| `AudioImporter` / `audio_importer` | Built-in catalog plus `zircon_plugins/audio_importer/plugin.toml`. | Stable package; `runtime.asset.importer.audio.wav` Partial. | Asset importer for sound-enabled profiles. | M4 audio must prove importer-to-playback integration. |
| `ShaderWgslImporter` / `shader_wgsl_importer` | Built-in catalog plus `zircon_plugins/shader_wgsl_importer/plugin.toml`. | Stable package; `runtime.asset.importer.shader.wgsl` Partial. | Asset importer for render/editor flows. | Render/asset milestones own shader import validation. |
| `UiDocumentImporter` / `ui_document_importer` | Built-in catalog plus `zircon_plugins/ui_document_importer/plugin.toml`. | Stable package; `runtime.asset.importer.ui_document` Partial. | Asset importer for UI/editor flows. | UI document authoring/import remains under UI/Text/A11y milestones. |
| `Rendering` / `rendering` | Built-in catalog plus `zircon_plugins/rendering/plugin.toml`. | Stable; `runtime.plugin.rendering` Complete. | Required by client/editor/dev profiles. | Complete means registration/profile capability closure; active render parity plans own product-level render proof. |
| `VirtualGeometry` / `virtual_geometry` | Built-in catalog plus `zircon_plugins/virtual_geometry/plugin.toml`. | Experimental; plugin Partial and advanced render capability Partial. | Optional for `client_3d` and `dev`. | Advanced render provider; cannot replace default render parity evidence. |
| `HybridGi` / `hybrid_gi` | Built-in catalog plus `zircon_plugins/hybrid_gi/plugin.toml`. | Experimental; plugin Partial and advanced GI capability Partial. | Optional for `client_3d` and `dev`. | Advanced render provider; outside Bevy default gate. |
| `Solari` / `solari` | Built-in catalog plus `zircon_plugins/solari/plugin.toml`. | Experimental; plugin Partial and Solari capability Partial. | Optional for `client_3d` and `dev`. | Experimental provider contract only; realtime pass execution remains a later render milestone. |
| `ZrVmLanguage` / `zr_vm_language` | Built-in catalog plus `zircon_plugins/zr_vm_language/plugin.toml`. | Experimental; runtime plugin Partial and script backend Partial. | Catalog target client/server/editor; not currently in built-in profiles. | Scripting backend parity is independent of Bevy default plugins. |

Package manifests that do not map to a `RuntimePluginId` are tracked separately from default profile closure: `animation_graph`, `timeline_sequence`, `editor_build_export_desktop`, `material_editor`, `native_dynamic_fixture`, `native_window_hosting`, `plugin_sdk_examples`, `runtime_diagnostics`, and `ui_asset_authoring`. Feature bundle ids inside `sound` and `rendering` TOMLs are subfeatures of their owning runtime plugins, not additional runtime-profile rows.

## App Provider Closure

The M2 linked-provider closure lives in `zircon_app`, not in `zircon_runtime`. Runtime profiles and catalog descriptors remain implementation-agnostic, while `zircon_app` can opt into compiled first-party providers through feature-gated dependencies on `zircon_plugins/*/runtime` crates. The `first-party-runtime-plugins` app feature links `sound`, `rendering`, `texture`, `animation`, `net`, and `particles`; `first-party-navigation-runtime-plugin` links `navigation` separately because its runtime crate builds the Recast/Detour native bridge.

Profile bootstrap tests cover the practical Bevy-grade path: `EntryConfig::for_runtime_profile(RuntimeProfileId::Client2d)` projects the profile manifest, the app provider supplies linked registration reports for required sound/rendering and optional texture, and `BuiltinEngineEntry` appends the resulting plugin modules without making `zircon_runtime` depend on any `zircon_plugin_*` crate.

The same app-provider validation now covers app-owned render/platform bootstrap persistence. Headless profile selection stores both the headless platform feature set and the `Headless` render bundle, while the provider-enabled `client_2d` path keeps linked first-party registration closure in `zircon_app`.

## M3 Animation Completion Matrix

M3 is not a registration milestone. Bevy's `AnimationPlugin` registers `AnimationClip` and `AnimationGraph` assets, adds graph threading, transition advance, animation advance, target application, event triggering, and transition expiry into a chained `PostUpdate` system set before transform propagation. Zircon already has an `animation` runtime plugin, scene post-update hook, asset types for clips/graphs/state machines/sequences, sequence and clip-event sampling, graph/state-machine evaluation, and an integration test covering physics/animation tick contracts. The remaining gate is to prove the runtime semantics are complete enough for profile use.

| Capability | Bevy source evidence | Zircon owner / current evidence | M3 completion gate |
|---|---|---|---|
| Clip asset, curve channels, and duration | `AnimationClip` stores `AnimationCurves`, event tracks, and duration in `dev/bevy/crates/bevy_animation/src/lib.rs:105`; `add_curve_to_target` updates target curves and clip duration at `dev/bevy/crates/bevy_animation/src/lib.rs:284`. | `AnimationClipAsset`, `AnimationClipBoneTrackAsset`, `AnimationEventTrackAsset`, and binary fallback live in `zircon_runtime/src/asset/assets/animation.rs`; runtime pose sampling is in `zircon_plugins/animation/runtime/src/manager.rs`. | Keep skeleton clips and property sequences separate, document the target id/bone-name fallback order, and test step/hermite/quaternion sampling, zero-duration clips, non-finite samples, and missing channel reports. |
| Target identity and retargeting | Bevy's `AnimationTargetId` is UUID based and can be assigned independently of player ancestry in `dev/bevy/crates/bevy_animation/src/lib.rs:187`; targeted events use the same id path. | Zircon uses `target_id: Option<String>` on clip bone tracks, event tracks, sequence bindings, and graph mask targets; tests already cover target id before bone/path fallback in `runtime_physics_animation_tick_contract.rs`. | Either promote the string target id convention to a documented stable contract or introduce a normalized target-id type; add duplicate target, missing target, and retargeted hierarchy diagnostics before marking `animation` complete. |
| Graph, blend, additive, and mask behavior | Bevy `AnimationGraph` is a DAG with clip/blend/add nodes and mask groups in `dev/bevy/crates/bevy_animation/src/graph.rs:36`, `:114`, and `:213`. | Zircon graph assets support `Clip`, `Blend`, `Additive`, `Mask`, and `Output` nodes; `DefaultAnimationManager::evaluate_graph` collects active clips and masks; tests cover blend weight, additive mask, and clip target reporting. | Lock graph evaluation order and cycle/missing-node diagnostics; test weight normalization, additive-only mask targets, unloaded clip handles, repeated graph load/unload, and graph/player playback-time persistence. |
| Player update order and transform application | Bevy chains graph threading, transition advance, animation advance, target animation, event triggering, and transition expiry in `AnimationPlugin::build` at `dev/bevy/crates/bevy_animation/src/lib.rs:1227` before `TransformSystems::Propagate`. | `AnimationSceneRuntimeHook` registers `animation.scene.post_update` and runs from `SystemStage::PostUpdate`; `tick_animation_world` advances skeletal clips, sequences, graphs, and state machines. | Record the exact Zircon scene stage order relative to physics and transform propagation; add assertions that animation pose output is applied before dependent render/scene consumers and that disabled playback settings clear stale outputs. |
| Timeline events | Bevy `AnimationClip::add_event`, `add_event_to_target`, and `AnimationEventTrigger` route events to player or target entities; event tests cover multiple events, forward/reverse, and looping behavior in `dev/bevy/crates/bevy_animation/src/lib.rs:1550`. | Zircon `sample_clip_events` emits `AnimationClipEvent` for crossed ranges and looping occurrences; integration tests cover direct player, graph player, state-machine active graph, transition from/to graph, and loop-boundary events. | Add reverse playback and seek/replay semantics, guarantee no double-fire at exact boundaries, and decide whether target-specific events become typed runtime events or remain structured string events. |
| Transitions and state machines | Bevy `AnimationTransitions::play`, `advance_transitions`, and `expire_completed_transitions` fade weights and stop expired layers in `dev/bevy/crates/bevy_animation/src/transition.rs:33`, `:78`, `:111`, and `:147`. | Zircon has `AnimationStateMachineAsset`, `AnimationStateTransitionEvaluation`, state-machine player tick state, and tests for transition blending until duration completes. | Add explicit transition expiry reporting, interrupted-transition behavior, invalid condition diagnostics, and repeated transition stress tests so the state machine can be considered runtime-grade rather than fixture-grade. |
| glTF animation import | Bevy animation is integrated with graph/clip assets and glTF curve loaders through `dev/bevy/crates/bevy_animation/src/gltf_curves.rs` and the Bevy glTF importer path. | Zircon has `gltf_importer` as a first-party importer row and animation asset structures that can represent clips/skeletons/graphs, but this matrix has not seen an accepted glTF animation fixture gate yet. | M3 must include a glTF fixture that imports skeleton target ids, clip duration, translation/rotation/scale channels, and event/extension behavior where supported; importer completion remains owned by asset readiness but animation consumes the fixture. |
| Editor authoring and profile promotion | Bevy exposes animation types through prelude/assets and runtime systems; authoring is not allowed to stand in for runtime behavior. | `zircon_plugins/animation/editor` exists and `plugin.toml` exposes an editor module, while runtime profile metadata keeps `animation` `Beta`/`Partial`. | Editor UI can be promoted only after runtime gates pass. Profile policy may keep animation optional for default client profiles unless a product scenario requires it; if promoted, it must appear as linked/available without externalized required gaps. |

M3 candidate commands:

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --test runtime_physics_animation_tick_contract --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_animation_runtime --lib --locked -- --nocapture`
- `cargo test -p zircon_runtime --locked asset::tests -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`

M3 debug rule: when an animation profile, importer, or scene tick test fails, diagnose the lower shared layer first in this order: animation asset decode and target-id normalization, `AnimationManager` sampling/evaluation, scene hook stage ordering, profile/provider availability. Do not make a profile test pass by weakening required plugin policy or by treating editor authoring as runtime evidence.

## M4 Audio/Sound Completion Matrix

M4 is the default-profile audio milestone. Bevy treats audio as part of the ordinary default stack: `DefaultPlugins` includes `bevy_audio::AudioPlugin` under the `bevy_audio` feature, and `AudioPlugin` wires resources, source playback systems, sink cleanup, output availability, and spatial listener/emitter updates into `PostUpdate`. Zircon already has a much wider sound architecture than Bevy's minimal audio layer: a shared `SoundManager` contract, playback/source/listener/output DTOs, mixer graph/effects/acoustics descriptors, `DefaultSoundManager`, CPAL/software output state, DSP/HRTF/occlusion code, editor authoring package, and focused runtime tests. The M4 gate is therefore not "add sound from zero"; it is to prove Bevy-grade default semantics, diagnostics, and profile promotion while keeping ray/convolution/timeline authoring optional.

| Capability | Bevy source evidence | Zircon owner / current evidence | M4 completion gate |
|---|---|---|---|
| Default profile membership | `DefaultPlugins` includes `bevy_audio::AudioPlugin` at `dev/bevy/crates/bevy_internal/src/default_plugins.rs:78`; Bevy's crate default feature set includes audio in `dev/bevy/Cargo.toml:134`; public docs begin at `https://docs.rs/bevy_audio/latest/bevy_audio/struct.AudioPlugin.html`. | `zircon_plugins/sound/plugin.toml` marks `sound` as `Beta` and `runtime.plugin.sound = partial`; `zircon_app` can link first-party runtime providers with `first-party-runtime-plugins`; runtime profiles already require sound for client profiles. | Client/editor/dev profiles may require sound only when the linked provider is available and reports structured availability. Sound must not be represented as a generic externalized warning in a profile that claims Bevy default parity. |
| Plugin build and schedule | `AudioPlugin::build` inserts `GlobalVolume` and `DefaultSpatialScale`, configures `AudioPlaybackSystems` in `PostUpdate`, runs playback only when `audio_output_available`, orders spatial updates after transform propagation, initializes `AudioOutput`, and calls `add_audio_source::<AudioSource>()` in `dev/bevy/crates/bevy_audio/src/lib.rs:81` and `:108`. | `zircon_plugins/sound/runtime/src/module.rs` registers `SoundModule`, `SoundDriver`, and `DefaultSoundManager`; `zircon_runtime/src/core/framework/sound/manager.rs:21` defines the shared manager API. | Document and test the exact Zircon scene/audio tick point: source/listener component projection, mixer render, finished cleanup, and transform-dependent spatial updates must have deterministic order. Output missing must skip or degrade by structured status, not pretend audio ran successfully. |
| Audio asset and decoding | Bevy `AudioSource` stores bytes, `AudioLoader` handles supported extensions, and `Decodable` provides the decoder hook in `dev/bevy/crates/bevy_audio/src/audio_source.rs:8`, `:39`, and `:83`. | Zircon sound runtime loads `SoundAsset` through `ProjectAssetManager` in `zircon_plugins/sound/runtime/src/service_types/clip_assets.rs`; audio importer readiness is tracked by `zircon_plugins/audio_importer/plugin.toml` and asset readiness plans. | Accept WAV whole-frame validation, duration/frame-count metadata, unsupported-format diagnostics, and importer selection precedence where a diagnostic-only importer cannot hide a real runtime-capable audio importer. OGG/MP3/FLAC can be staged by explicit importer maturity, not implied by Bevy docs. |
| Playback settings and lifecycle | Bevy `PlaybackSettings` covers mode, volume, speed, paused, muted, spatial, spatial scale, start position, and duration in `dev/bevy/crates/bevy_audio/src/audio.rs:35`; `PlaybackMode` has `Once`, `Loop`, `Despawn`, and `Remove` in `audio.rs:11`. | `SoundPlaybackSettings` includes gain, speed, looped, completion action, paused, muted, start/duration, output track, and pan in `zircon_runtime/src/core/framework/sound/playback.rs:14`; `SoundPlaybackCompletionAction` has `None`, `DespawnEntity`, and `RemoveAudioComponents` at `playback.rs:151`. | Prove `ONCE`, `LOOP`, `DESPAWN`, and `REMOVE` equivalents with start/duration clipping, loop-boundary behavior, pause/mute defaults, invalid speed/range diagnostics, and finished-action reporting. Any lifecycle divergence must be named and tested. |
| Sink controls and runtime control API | Bevy `AudioSinkPlayback` exposes volume, speed, play/pause/toggle, seek, stop, empty, mute/unmute, and toggle mute in `dev/bevy/crates/bevy_audio/src/sinks.rs:10`; concrete `AudioSink` and `SpatialAudioSink` wrap the sink at `sinks.rs:139` and `:243`. | `SoundManager` exposes playback/source pause/resume/toggle, gain/speed, seek, mute/unmute, empty/status, and finished drain methods in `zircon_runtime/src/core/framework/sound/manager.rs:38`; `DefaultSoundManager` implements those controls through `zircon_plugins/sound/runtime/src/service_types/playback_controls.rs`, `source_controls.rs`, `playback_status.rs`, and `source_status.rs`. | Add focused tests for idempotent control semantics, invalid handle errors, seek clamping, stop versus natural completion, and parity between direct `play_clip` handles and scene-owned `SoundSourceDescriptor` controls. |
| Global volume and volume math | Bevy `GlobalVolume` is a resource in `dev/bevy/crates/bevy_audio/src/volume.rs:10`; `Volume` supports linear/decibel conversions at `volume.rs:36` with conversion tests. | Zircon exposes `global_volume_gain` and `set_global_volume_gain` through `SoundManager`; mixer output applies `config.master_gain` and clamps samples in `zircon_plugins/sound/runtime/src/engine/render/orchestration.rs`. | Lock the public math contract: non-negative finite gain, decibel/linear conversion helper if the editor exposes dB, sample clamp behavior, and regression tests for zero, unity, high gain, NaN/Inf, and negative input. |
| Spatial audio baseline | Bevy `SpatialListener`, `SpatialScale`, `AudioPlayer`, `SpatialAudioSink`, and emitter/listener update systems are in `dev/bevy/crates/bevy_audio/src/audio.rs:173`, `:205`, `:251`, `dev/bevy/crates/bevy_audio/src/sinks.rs:243`, and `dev/bevy/crates/bevy_audio/src/audio_output.rs:341`. | Zircon has `SoundSourceDescriptor`, `SoundSpatialSourceSettings`, `SoundListenerDescriptor`, and HRTF/occlusion runtime code in `zircon_runtime/src/core/framework/sound/components.rs:13`, `:89`, `:128`, plus folder-backed `zircon_plugins/sound/runtime/src/engine/hrtf/` and `engine/occlusion/`. | Baseline Bevy parity is simple source/listener spatial update with default scale and deterministic attenuation. HRTF, Doppler, occlusion, convolution, and ray-traced impulse responses are valuable advanced capabilities but must stay optional or separately gated until their own tests pass. |
| Output/backend degradation | Bevy `AudioOutput` stores an optional device and logs no-device state; playback systems run behind `audio_output_available` in `dev/bevy/crates/bevy_audio/src/audio_output.rs:84` and `:336`. | Zircon `SoundOutputDeviceDescriptor`, `SoundOutputDeviceInfo`, `SoundOutputLatencyStatus`, and `SoundOutputDeviceStatus` live in `zircon_runtime/src/core/framework/sound/output.rs:42`, `:54`, `:82`, and `:97`; runtime output lifecycle lives in folder-backed `zircon_plugins/sound/runtime/src/output/lifecycle/` modules for storage, config, start/stop, callback accounting, status projection, and backend-session state. Backend/device listing lives in `output/catalog.rs`, descriptor validation in `output/descriptor_validation.rs`, latency/status diagnostic helpers in `output/status.rs`, and concrete software/CPAL paths in `output/software.rs` plus `output/cpal/`. | Backend unavailable must return structured `BackendUnavailable`/status diagnostics, keep software-null deterministic for CI, expose device/latency rows for editor tooling, and avoid panic or silent success when no hardware device is available. |
| Cleanup and finished reporting | Bevy inserts `PlaybackDespawnMarker`/remove markers and `cleanup_finished_audio` removes or despawns when sinks are empty in `dev/bevy/crates/bevy_audio/src/audio_output.rs:37` and `:284`. | Zircon has `SoundPlaybackFinished`, `SoundSourceFinished`, completion actions, and `drain_finished_playbacks`/`drain_finished_sources` in `zircon_runtime/src/core/framework/sound/playback.rs` and `manager.rs`. | Test voice cleanup, finished drain stability, double-drain behavior, entity/component cleanup integration, and missing-clip completion. A profile cannot count sound complete while finished playback leaks runtime state. |
| Advanced features outside default blocker | Bevy's default audio layer is playback, sinks, volume, simple spatial, and output; it does not require a mixer console, timeline authoring, HRTF database, or ray-traced convolution. | Zircon sound already has mixer graph, DSP descriptors, automation, dynamic events, timeline feature, ray-tracing/convolution descriptors, editor live output, and acoustic debug UI. | Keep timeline animation track, ray-traced convolution reverb, geometry-backed occlusion, HRTF database interpolation, live-output editor commands, and dynamic event ABI in optional feature lanes. They improve Zircon beyond Bevy baseline, but cannot substitute for default playback/import/output/profile gates. |

M4 candidate commands:

- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --features cpal-backend --locked --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_event_abi --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/sound/editor/Cargo.toml live_output --locked --offline --jobs 1 --message-format short --color never`
- `cargo test --manifest-path zircon_plugins/audio_importer/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`

M4 debug rule: when a sound profile/importer/playback/output test fails, diagnose the lowest shared layer first in this order: `SoundAsset` import and duration/frame metadata, `SoundPlaybackSettings` and folder-backed `descriptor_validation/source/` source descriptor validation, `SoundManager` handle/lifecycle semantics, mixer render and finished queues, output backend status, then app profile/provider availability. Do not make M4 pass by weakening required sound profile policy, by treating editor mixer authoring as runtime playback evidence, or by making ray/convolution optional features satisfy default audio gates.

## M5 Sprite2D Completion Matrix

M5 is a plugin identity and 2D render-product milestone. Bevy keeps sprite API and sprite rendering separate: `SpritePlugin` owns atlas registration, bounds, text/picking hooks, and sprite components, while `SpriteRenderPlugin` owns render-world extraction, slice computation, sprite image bind groups, batching, transparent queueing, and the 2D render plugin stack. Zircon already has runtime sprite DTOs, scene `Sprite2dComponent`, a simple sprite renderer, phase queue tests, and texture fallback stats, but it has no first-party `RuntimePluginId::Sprite2d` or `zircon_plugins/sprite_2d/runtime`. The M5 gate is therefore to promote sprite from "render framework capability" to a named first-party runtime plugin without overloading `texture`, `tilemap_2d`, or `rendering`.

| Capability | Bevy source evidence | Zircon owner / current evidence | M5 completion gate |
|---|---|---|---|
| Dedicated plugin identity | Bevy has `SpritePlugin` for API/bounds in `dev/bevy/crates/bevy_sprite/src/lib.rs:68` and `SpriteRenderPlugin` for renderer wiring in `dev/bevy/crates/bevy_sprite_render/src/lib.rs:54`; public docs expose sprite as a first-party module through `https://docs.rs/bevy/latest/bevy/sprite/`. | `RuntimePluginId` currently includes `Texture`, `Tilemap2d`, and `Rendering` but no `Sprite2d` in `zircon_runtime/src/builtin/runtime_modules.rs:36`; no `zircon_plugins/sprite_2d` directory exists. | Add `RuntimePluginId::Sprite2d`, catalog descriptor, package manifest, runtime crate, and profile rows. `texture` remains asset processing, `tilemap_2d` remains tilemap authoring, and `rendering` remains broad render feature ownership. |
| Sprite component API | Bevy `Sprite` carries image, optional atlas, color, flip flags, custom size, rect, and `SpriteImageMode` in `dev/bevy/crates/bevy_sprite/src/sprite.rs:19`; `Anchor` supplies normalized pivots in `sprite.rs:257`. | Zircon has `Sprite2dComponent` with image, material, atlas region, rect, flip, anchor, custom size, color, z order, and alpha mode in `zircon_runtime/src/scene/components/render2d/sprite.rs`; render DTOs live under `zircon_runtime/src/core/framework/render/sprite`. | Define the plugin-owned component/resource surface and serialization schema: image handle, optional material, atlas rect/index convention, rect clipping, flip, anchor convention, custom size, color, z/sort order, layer mask, and alpha mode. Anchor coordinate differences from Bevy must be documented and tested. |
| Bounds and culling | Bevy `SpritePlugin` runs `calculate_bounds_2d` and `calculate_bounds_2d_sprite_mesh` in `PostUpdate` under `VisibilitySystems::CalculateBounds` in `dev/bevy/crates/bevy_sprite/src/lib.rs:77` and `:118`; tests cover image sprites, custom size, custom rect, and anchor behavior. | Zircon has `RenderSpriteBounds`, `RenderSpriteSnapshot`, scene extract, and render-product sprite tests, but no standalone sprite plugin gate for culling/bounds semantics. | Add bounds tests for image size, custom size, rect, atlas region, zero/negative/non-finite sizes, anchors, render layers, disabled/hidden sprites, and stable 2D culling before profile promotion. |
| 9-slice and tiling | Bevy `SpriteImageMode` supports `Auto`, `Scale`, `Sliced(TextureSlicer)`, and `Tiled` in `dev/bevy/crates/bevy_sprite/src/sprite.rs:168`; `TextureSlicer` and `SliceScaleMode` implement 9-slice and tile behavior in `dev/bevy/crates/bevy_sprite/src/texture_slice/slicer.rs:13`; render-side `ComputedTextureSlices` recomputes on image or sprite changes in `dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs:9`. | Zircon `RenderSpriteSnapshot` currently carries `atlas_region` and `rect`; the renderer builds one quad in `zircon_runtime/src/graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs`. | M5 can stage 9-slice/tiling after the basic quad path, but the plan must name the gap: either implement slice descriptors and multi-quad extraction, or mark 9-slice/tile as `Partial` with diagnostics. Do not claim Bevy sprite parity from single-quad rendering alone. |
| Extract and render queue | Bevy `SpriteRenderPlugin` registers required render-world sync, initializes sprite resources, extracts sprites in `ExtractSchedule`, queues sprites in `RenderSystems::Queue`, prepares image/view bind groups, and sorts 2D phases in `dev/bevy/crates/bevy_sprite_render/src/lib.rs:73` and `:110`. | Zircon has `SpriteExtract`, `build_sprite_phase_queue`, `SpritePhaseInput`, `RenderPhaseMeshSource::SpriteIndex`, and `SpriteRenderer::record` under `zircon_runtime/src/core/framework/render` and `zircon_runtime/src/graphics/scene/scene_renderer/sprite`. Existing `render_product_sprite` tests cover phase order and fallback stats. | The future `sprite_2d` plugin must own extraction into `RenderSpriteSnapshot`, render-layer filtering, 2D phase queueing, texture/material readiness, batching/readiness diagnostics, and submit stats. It must not require particles or tilemap to be enabled. |
| Texture atlas and rect readiness | Bevy combines atlas rect and per-sprite rect during extraction in `dev/bevy/crates/bevy_sprite_render/src/render/mod.rs:345`; it relies on `TextureAtlasPlugin` if not already added. | Zircon texture processing is a separate stable `texture` plugin; texture readiness and compressed upload support are active in asset-readiness work. Sprite DTOs can already carry `RenderSpriteAtlasRegion` and `RenderSpriteRect`. | Sprite2D depends on texture/image readiness but does not own texture import. M5 acceptance requires atlas rect validation, missing texture fallback, image readiness diagnostics, and a clear dependency on `texture`/texture importer capabilities where needed. |
| Sprite mesh and material path | Bevy also has `SpriteMesh`, `SpriteAlphaMode`, `Mesh2dRenderPlugin`, `ColorMaterialPlugin`, and `SpriteMeshPlugin` in `dev/bevy/crates/bevy_sprite/src/sprite_mesh.rs:13`, `dev/bevy/crates/bevy_sprite_render/src/mesh2d/mesh.rs:57`, and `mesh2d/color_material.rs:12`. | Zircon has material alpha mode and optional material handle on sprite snapshots, but broad material/PBR work is owned by the active rendering session. | Split basic sprite quads from advanced materialized `SpriteMesh`: M5 baseline requires texture tint, alpha mode, sort order, and fallback material. Material graph/PBR integration can stay behind rendering/material milestones until validated. |
| Tilemap separation | Bevy sprite render composes `TilemapChunkPlugin` alongside sprite render in `dev/bevy/crates/bevy_sprite_render/src/lib.rs:73`, but sprite API remains separate. | Zircon has `tilemap_2d` as a beta/partial package with runtime/editor modules in `zircon_plugins/tilemap_2d/plugin.toml`; it should not satisfy the sprite gap. | Keep `tilemap_2d` optional. Sprite2D may share atlas/texture/render infrastructure, but default 2D presentation must not require tilemap authoring or tilemap runtime packages. |
| Profile promotion | Bevy default feature composition includes `2d` and sprite-related crates; `DefaultPlugins` installs the broad default group. | Zircon `client_2d` currently requires `sound` and `rendering`, makes `texture` optional, and has no `sprite_2d` default plugin in `zircon_runtime/src/plugin/runtime_profile.rs:456`. | `client_2d` must include `sprite_2d` as required once plugin/provider coverage exists. `client_3d` can include it as common optional/default depending on profile policy, but missing `sprite_2d` must not be hidden by `rendering` or `texture`. |

M5 candidate commands:

- `cargo test -p zircon_runtime --locked render_product_sprite --jobs 1 --message-format short --color never -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked --jobs 1 --message-format short --color never -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap --message-format short --color never -- --nocapture --test-threads=1`
- After the crate exists: `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_sprite_2d_runtime --locked --jobs 1 --message-format short --color never -- --nocapture`

M5 debug rule: when a sprite profile/render test fails, diagnose lower layers in this order: texture/image readiness and atlas rect metadata, `Sprite2dComponent` to `RenderSpriteSnapshot` projection, bounds/culling and render-layer filtering, `SpriteExtract` phase queue ordering, texture/material fallback readiness, then app profile/provider availability. Do not make M5 pass by marking `texture`, `tilemap_2d`, or broad `rendering` as the sprite plugin, and do not count particle sprite rendering as `Sprite2d` evidence.

## M6 Navigation Split Completion Matrix

M6 is a semantic split milestone. Bevy's first-party navigation evidence for default parity lives in UI/input-focus navigation, not gameplay navmesh/pathfinding. `bevy_input_focus` owns `InputFocus`, focus-visible state, `FocusedInput`, focus-change events, tab navigation, and directional/manual navigation; `bevy_ui` adds automatic directional navigation over computed UI positions. Zircon already has runtime UI focus/navigation contracts and tests, while `RuntimePluginId::Navigation` and `zircon_plugins/navigation` are gameplay navmesh/pathfinding. The M6 gate is to make `ui_navigation` a UI capability/profile gate and keep gameplay `navigation` optional beta/advanced.

| Capability | Bevy source evidence | Zircon owner / current evidence | M6 completion gate |
|---|---|---|---|
| Capability split and naming | Bevy input-focus crate docs list `InputFocus`, `FocusGained`/`FocusLost`, `FocusedInput`, and tab/directional navigation in `dev/bevy/crates/bevy_input_focus/src/lib.rs:12`; public docs begin at `https://docs.rs/bevy/latest/bevy/input_focus/`. | `zircon_plugins/navigation/plugin.toml:12` marks `runtime.plugin.navigation` as partial gameplay navigation, while UI navigation contracts live in `zircon_runtime_interface/src/ui/navigation.rs` and UI focus contracts live in `zircon_runtime_interface/src/ui/focus.rs`. | Add or document explicit `ui_navigation` capability/profile membership under UI/default profiles. Do not let `RuntimePluginId::Navigation` satisfy UI focus parity. |
| Focus state, events, and focused input | Bevy defines `InputFocus` in `bevy_input_focus/src/lib.rs:104`, `InputFocusVisible` at `:177`, `FocusedInput` at `:189`, `InputFocusPlugin` at `:260`, and focused input dispatch at `:331`. | Zircon defines `UiFocusVisibleReason`, `UiFocusVisible`, `UiFocusChangeReason`, `UiFocusChangeEvent`, `UiFocusedInputKind`, and `UiFocusedInput` in `zircon_runtime_interface/src/ui/focus.rs:7`; `UiFocusState` in `zircon_runtime_interface/src/ui/surface/focus_state.rs:14`; `UiSurface::focus_node` in `zircon_runtime/src/ui/surface/focus.rs:16`; and focused-input route recording in `zircon_runtime/src/ui/surface/input/dispatch.rs:1029`. | Pin focus gained/lost, focus-visible reasons, autofocus, invalid owner cleanup, hidden/disabled cleanup, focused input bubbling/route recording, and render-only dirty behavior. |
| Tab navigation and modal groups | Bevy defines `TabIndex` in `bevy_input_focus/src/tab_navigation.rs:63`, `TabGroup` at `:72`, `NavAction` at `:105`, `TabNavigation::navigate` at `:178`, `TabNavigationPlugin` at `:375`, and Tab key handling at `:424`; public docs begin at `https://docs.rs/bevy/latest/bevy/input_focus/tab_navigation/`. | Zircon defines `UiTabIndex` in `zircon_runtime_interface/src/ui/navigation.rs:16`, `UiNavigationGroup` at `:48`, and `UiNavigationEventKind::{Next, Previous, Home, End}` in `zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs:4`; tests cover `tab_navigation_uses_index_order_and_modal_group_trap` and `tab_navigation_crosses_non_modal_groups_by_group_order` in `zircon_runtime/src/ui/tests/focus_navigation.rs:276` and `:315`. | Tab/Shift-Tab, First/Last/Home/End policy, group order, modal trap, wrap/no-wrap, and disabled/non-focusable filtering must all be explicit tests before promotion. |
| Directional and automatic navigation | Bevy `DirectionalNavigationPlugin` initializes map/config resources in `bevy_input_focus/src/directional_navigation.rs:74`, `DirectionalNavigationMap` is at `:254`, `DirectionalNavigation::navigate` is at `:410`, and automatic edge generation is at `:518`; Bevy UI `AutoDirectionalNavigation` is at `bevy_ui/src/auto_directional_navigation.rs:106`, `AutoDirectionalNavigator` at `:118`, with a z-index agnostic warning at `:52`. Public docs begin at `https://docs.rs/bevy/latest/bevy/input_focus/directional_navigation/` and `https://docs.rs/bevy/latest/bevy/ui/auto_directional_navigation/`. | Zircon defines `UiDirectionalNavigation` in `zircon_runtime_interface/src/ui/navigation.rs:59`, `UiDirectionalNavigationTarget::{Auto, Node, Group, Blocked}` at `:68`, and dispatches navigation through `UiNavigationDispatcher` in `zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs:18`. Tests cover `directional_navigation_honors_manual_overrides_and_blocked_edges` and `modal_directional_navigation_rejects_manual_targets_outside_modal_group` in `zircon_runtime/src/ui/tests/focus_navigation.rs:330` and `:352`. | Implement/test nearest-neighbor by arranged frame, manual overrides before auto, blocked edges, modal boundary rejection, surface/layer/z-order awareness, and keyboard/gamepad directional mapping. Zircon may intentionally improve on Bevy's current z-index agnostic automatic navigation, but that divergence must be tested. |
| Input bridge and widget/a11y boundary | Bevy `InputDispatchPlugin` dispatches focused keyboard, gamepad, and mouse-wheel input in `bevy_input_focus/src/lib.rs:279`; `bevy_ui` pointer focus uses `FocusPolicy` in `bevy_ui/src/focus.rs:109` and `ui_focus_system` at `:149`. The input-focus crate docs say it provides no widget integration in `bevy_input_focus/src/lib.rs:18`. | Zircon routes UI navigation through `UiNavigationRoute` in `zircon_runtime_interface/src/ui/surface/navigation/route.rs:8`; input dispatch maps keyboard, text, IME, navigation, and pointer events to focused input kinds in `zircon_runtime/src/ui/surface/input/dispatch.rs:1029`; tests cover `pointer_and_navigation_focus_sources_update_visible_reason`, `text_and_ime_inputs_record_focused_input_routes`, and `range_home_and_end_navigation_use_authored_min_max_aliases` in `focus_navigation.rs:41`, `:115`, and `widget_range_navigation.rs:19`. | UI navigation consumes neutral input events without duplicating platform input ownership. Widgets and a11y output must receive routed focus/navigation events through the same focused-input route. |
| Gameplay navigation stays optional | Bevy first-party navigation evidence above is UI focus/navigation; there is no Bevy default gameplay navmesh plugin in this parity lane. | Zircon `RuntimePluginId::Navigation` exists in `zircon_runtime/src/builtin/runtime_modules.rs:39`; `client_3d`, `editor`, `dev`, and `server` profiles keep it optional in `zircon_runtime/src/plugin/runtime_profile.rs:483`, `:507`, `:523`, and `:542`; the runtime plugin descriptor is in `zircon_plugins/navigation/runtime/src/lib.rs:23`, and the shared gameplay `NavigationManager` trait is in `zircon_runtime/src/core/framework/navigation/mod.rs:661`. | Keep gameplay `navigation` optional beta/advanced with Recast/native provider, navmesh asset validation, bake/query diagnostics, agents, obstacles, offmesh links, and editor gizmos. Passing UI navigation must not promote the gameplay navigation plugin. |

M6 candidate commands:

- `cargo test -p zircon_runtime --lib focus_navigation --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib widget_range_navigation --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_navigation_runtime --locked -- --nocapture`

M6 debug rule: when a UI navigation test fails, diagnose lower layers in this order: focus target validity and disabled/hidden owner gates, navigation contract extraction, tab/group ordering, directional manual/blocked edge resolution, arranged-frame nearest-neighbor calculation, focused-input route/a11y output, then profile/provider availability. When gameplay navigation tests fail, stay in navmesh asset/settings/native provider paths; do not fix UI focus by changing the gameplay `navigation` plugin.

## M7 Net / Remote Completion Matrix

M7 is a security and layering milestone. Bevy Remote Protocol (BRP) is developer tooling for inspecting and mutating an app over JSON-RPC; `RemotePlugin` installs method resources and request processing but explicitly does not start transports. The HTTP listener is a separate `RemoteHttpPlugin`, and the root Bevy feature is opt-in. Zircon already has a broad `net` runtime plugin, base `NetManager`, HTTP/WebSocket backends, RPC/session, replication, reliable UDP, and content download feature crates. M7 must carve out a Bevy-grade `remote_protocol` developer capability without treating multiplayer transports, replication, or content download as default profile requirements and without opening listeners in published clients.

| Capability | Bevy source evidence | Zircon owner / current evidence | M7 completion gate |
|---|---|---|---|
| Protocol separated from transport | Bevy crate docs state `RemotePlugin` sets up BRP without starting transports in `dev/bevy/crates/bevy_remote/src/lib.rs:1`; `RemoteHttpPlugin` is the second plugin that enables HTTP communication in `lib.rs:5`. The root Bevy feature is opt-in in `dev/bevy/Cargo.toml:389`. | Zircon `net` is `Beta`/`Partial` in `zircon_plugins/net/plugin.toml:1` and exposes optional feature bundles for HTTP, WebSocket, RPC, replication, reliable UDP, and content download. Profiles keep `Net` optional/default-disabled for editor/dev/server in `zircon_runtime/src/plugin/runtime_profile.rs:509`, `:521`, and `:539`. | Add or document `net.remote_protocol` as a dev/editor capability separate from listener transports. Default clients must not open a port; server/dev/editor may enable it only by explicit profile/manifest selection. |
| Method registry and custom methods | Bevy `RemotePlugin` stores method handlers at `bevy_remote/src/lib.rs:572`, adds custom methods with `with_method_main` / `with_method_render` at `:589` and `:601`, registers systems during `Plugin::build` at `:800`, and exposes runtime mutation through `RemoteMethods::insert` at `:967`. | Zircon has `RpcDescriptor` and `RpcDirection` in `zircon_runtime/src/core/framework/net/rpc.rs:28`, a `NetManager` trait with transport primitives in `zircon_runtime/src/core/framework/net/manager.rs:8`, and RPC feature registration/tests under `zircon_plugins/net/features/rpc/runtime/src/feature.rs:8` and `tests.rs:17`. | Define a remote method registry that supports stable names, duplicate handling, typed schema metadata where available, main/runtime versus render/editor domains, and runtime method list/discovery. |
| JSON-RPC request/response/error shape | Bevy docs define BRP as JSON-RPC 2.0 in `bevy_remote/src/lib.rs:8`, require `id` and `method` at `:28`, describe `result`/`error` responses at `:49`, and document error `code`/`message`/`data` at `:93`. `BrpError` is defined at `:1198`; JSON-RPC-style error codes are in `error_codes` at `:1281`. | Zircon `NetError` is in `zircon_runtime/src/core/framework/net/error.rs:6`; HTTP request/response descriptors are in `zircon_runtime/src/core/framework/net/http.rs:15` and `:57`; RPC feature tests cover schema/handler/missing-handler failures in `zircon_plugins/net/features/rpc/runtime/src/tests.rs:385` and `:424`. | Lock a BRP-like request envelope: required method/id, optional params, batch policy, structured error codes, parse errors, invalid params, method-not-found, handler failure, response id preservation, and size limits. |
| Built-in developer methods | Bevy built-ins include world component/resource get/list/query/mutate/spawn/despawn/reparent/event/message methods, registry schema, schedule list/graph, watch methods, and `rpc.discover` in `bevy_remote/src/builtin_methods.rs:42` through `:105`; examples use `world.query` and `world.write_message` in `examples/remote/client.rs:60` and `:140`. | Zircon currently exposes broad net primitives but does not yet have a documented Bevy-like world/query/diagnostics remote method set in the parity matrix. Runtime diagnostics/profile availability are already structured in the plugin/profile layer. | Stage methods in tiers: read-only world/query/diagnostics first, then controlled mutation/message methods behind explicit dev/editor permission, then watch/streaming. No mutation endpoint should be accepted in published clients by default. |
| Discovery and schema | Bevy `rpc.discover` follows OpenRPC service discovery in `bevy_remote/src/lib.rs:481`; `OpenRpcDocument`, `ServerObject`, and `MethodObject` are defined in `schemas/open_rpc.rs:14`, `:55`, and `:71`; JSON schema export types are in `schemas/json_schema.rs:224`. | Zircon has descriptor, manifest, capability, profile, and event catalog metadata; `NetRuntimePlugin` registers module/options/event catalogs in `zircon_plugins/net/runtime/src/lib.rs:50`; plugin and feature package manifests already advertise capabilities in `plugin.toml`. | Remote discovery must expose method list, capability/profile/maturity notes, target support, schema metadata where available, and server connection details only when a transport is actually enabled. |
| HTTP listener and transport security | Bevy `RemoteHttpPlugin` binds `DEFAULT_ADDR` 127.0.0.1 and ports in `bevy_remote/src/http.rs:52` and `:60`; it starts the HTTP server in `Startup` at `:135` and accepts TCP listeners in `server_main` / `listen` at `:252` and `:266`. The server example adds `RemotePlugin` and `RemoteHttpPlugin` explicitly in `examples/remote/server.rs:18` and `:19`. | Zircon `NetManager` supports `listen_http`, `send_http_request`, `listen_websocket`, `connect_websocket`, TCP/UDP primitives, diagnostics, and events in `zircon_runtime/src/core/framework/net/manager.rs:8`. Base net tests require the HTTP/WebSocket feature for real backends in `zircon_plugins/net/runtime/src/tests.rs:220` and `:278`; feature tests cover security policy and certificate pin behavior in HTTP tests `:74` / `:107` and WebSocket tests `:155` / `:180`. | Transport enabling must require explicit bind address/port, loopback default, no wildcard bind without warning/fatal policy, request size/body limits, security policy, and provider-unavailable diagnostics. |
| Watch/streaming and batching | Bevy registers watching methods for `world.get_components+watch` and `world.list_components+watch` in `bevy_remote/src/lib.rs:723` and `:728`; `RemoteWatchingRequests` is tracked at `:991`; HTTP processing handles batch request shape in `bevy_remote/src/http.rs:302`. | Zircon WebSocket loopback and frame budget tests live in `zircon_plugins/net/runtime/src/tests.rs:244`; WebSocket feature tests cover real handshake and policy in `zircon_plugins/net/features/websocket/runtime/src/tests.rs:37` and `:77`. | Watch/streaming should start after read-only request/response semantics: per-client budget, cancellation, bounded queues, stale entity/resource behavior, and WebSocket transport gating. |
| Advanced game networking remains optional | Bevy Remote is not a replication/multiplayer default. It is a feature-gated developer protocol in `dev/bevy/Cargo.toml:389`; remote examples are under the Remote Protocol category at `:4844`. | Zircon has replication tests in `zircon_plugins/net/features/replication/runtime/src/tests.rs:36`, reliable UDP tests in `zircon_plugins/net/features/reliable_udp/runtime/src/tests.rs:36`, RPC/session tests in `rpc/runtime/src/tests.rs:40`, and content download tests in `content_download/runtime/src/tests.rs:44`. | Keep replication, reliable UDP, gameplay RPC, WebSocket game sessions, and content download as optional feature gates. They can reuse `net`, but they do not satisfy BRP-style dev remote parity and should not enter default client profiles. |

M7 candidate commands:

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_http_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_websocket_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_rpc_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_replication_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_reliable_udp_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_content_download_runtime --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`

M7 debug rule: when a remote/profile test fails, diagnose in this order: protocol envelope parsing and JSON-RPC error mapping, method registry/discovery, world/query/diagnostics method permissions, request/body/batch size limits, transport provider availability, bind address/port policy, profile/export selection, then advanced replication/RPC/game transport features. Do not make a published client pass by opening a remote listener, and do not make Bevy remote parity pass by counting gameplay replication or content download as developer remote protocol.

## M8 Particles / VFX Completion Matrix

M8 is an advanced optional milestone, not a default parity blocker. The local Bevy tree has no `bevy_particles` crate under `dev/bevy/crates`, no `particles` feature row in `dev/bevy/Cargo.toml` or `dev/bevy/docs/cargo_features.md`, and `DefaultPlugins` contains render, sprite, PBR, post-process, and pipeline building blocks but no particle plugin. Bevy still gives useful render-stage evidence for particle-like systems: TAA docs require either particle motion vectors or rendering particles after TAA, and core pipeline schedules expose early/final post-process sets. Zircon already has an experimental `particles` package with runtime/editor modules, CPU simulation, GPU planning/readback, transparent render contracts, physics/animation optional interop, editor authoring, and tests. The M8 gate is therefore honest classification and optional VFX completion, not promotion into default client profiles.

| Capability | Bevy source evidence | Zircon owner / current evidence | M8 completion gate |
|---|---|---|---|
| Default parity classification | Bevy default is `["2d", "3d", "ui", "audio"]` in `dev/bevy/Cargo.toml:134`; 2D/3D feature collections pull sprite/render/PBR/post-process primitives in `Cargo.toml:214`, `:217`, `:227`, and `:240`, but the particle search has no matching first-party crate or feature row. `DefaultPlugins` lists render/post-process/sprite/PBR plugins in `dev/bevy/crates/bevy_internal/src/default_plugins.rs:43`, `:61`, `:65`, `:67`, and `:77`. | `zircon_plugins/particles/plugin.toml:1` declares `particles`; `:5` marks maturity `experimental`; `:9` marks `runtime.plugin.particles` partial and explicitly says it is not a Bevy default parity blocker. | Keep particles experimental/optional until runtime, render, and editor gates are all explicit. Default `client_2d` / `client_3d` may list particles as optional, but no default profile may require it for Bevy parity. |
| Profile and provider identity | Bevy has no first-party particle plugin identity; public feature docs begin at `https://docs.rs/crate/bevy/latest/features` and the local feature table has sprite/render/PBR but no particles. | Zircon keeps `RuntimePluginId::Particles` optional in client/editor/dev profiles in `zircon_runtime/src/plugin/runtime_profile.rs:462`, `:484`, `:508`, and `:524`; app bootstrap can link optional `ParticlesModule` through first-party registrations in `zircon_app/src/entry/tests/profile_bootstrap.rs:199`, `:204`, `:239`, and `:259`; runtime module resolver recognizes the id in `zircon_runtime/src/builtin/runtime_modules.rs:1080`. | Provider reports must surface particles as linked/native/externalized optional with structured diagnostics. Missing particles must remain warning-only unless a manifest explicitly makes it required. |
| Asset and component schema | Bevy sprite/render/material/PBR are reusable building blocks, not a particle asset schema: feature docs list `bevy_sprite`, `bevy_sprite_render`, `bevy_render`, and `bevy_pbr` in `docs/cargo_features.md:91`, `:95`, `:100`, and `:101`. | Zircon owns particle asset data in `zircon_plugins/particles/runtime/src/asset.rs:8` through `:328`, component identity in `component.rs:7` and `:23`, and editor CPU sprite template authoring in `editor/src/authoring.rs:13`, `:25`, and `:71`. Editor tests pin template shape in `editor/src/tests.rs:184`. | Validate emitter ids, scalar/vector/color ranges, bursts, lifetime, max particles, coordinate space, material/texture metadata, serialization shape, and editor CPU sprite template defaults before treating authoring as usable. |
| CPU simulation determinism | Bevy has no first-party particle simulator. Its closest default foundations are ECS/scheduling/time; particle-specific render guidance appears only as TAA/post-process constraints. | Zircon CPU simulation seeds instances from system seed and handle in `simulation/cpu.rs:43`, resets deterministically at `:80`, spawns due particles at `:205`, updates lifetime/death at `:280`, and validates non-finite asset values at `:344`. Tests cover deterministic seed at `tests.rs:97`, lifetime/reuse at `:125`, preview controls at `:151`, invalid settings at `:368`, and invalid bursts/bindings at `:384`. | Add/maintain boundary tests for fixed timestep accumulation, zero/negative/non-finite values, disabled emitters, burst-at-boundary semantics, long-running spawn drift, max-particle saturation, and deterministic replay after rewind. |
| Render extraction and transparency | Bevy TAA warns that particle effects need motion vectors or must render after TAA in `bevy_anti_alias/src/taa/mod.rs:106` through `:110`; core pipeline stages define early/final post-process in `bevy_core_pipeline/src/schedule.rs:37`, `:38`, `:76`, and `:77`. | Zircon sorts extracted particle sprites in `render/extract.rs:10` and `:16`, registers transparent render feature passes in `render/feature.rs:8` through `:54`, and declares transparent executor resources in `render/executors.rs:106` and `:163`. Tests cover extract sort at `tests.rs:171`, material/texture/rotation metadata at `:200`, resource contracts at `:498`, and transparent render plan at `:534`. | Pin extract order, transparent sorting, camera fallback, missing material/texture fallback, render-graph resource contracts, TAA/post-process placement policy, and no dependency on Sprite2D plugin promotion. |
| GPU/VFX path and fallback | Bevy provides render pipelines/post-process building blocks but no default particle GPU simulation plugin. | Zircon compiles GPU layout up to `PARTICLE_GPU_MAX_PARTICLES` in `render/gpu/layout.rs:3` and `:82`, builds frame params in `render/gpu/planner.rs:129`, compiles GPU programs and fallback diagnostics in `render/gpu/program.rs:15`, `:135`, `:140`, `:147`, and `:158`, records backend/transparent/readback paths in `render/gpu/backend.rs:67`, `:295`, `:360`, and `:386`, and decodes readback in `render/gpu/readback.rs:26`. Tests cover CPU fallback at `tests.rs:410`, GPU extract at `:473`, readback/parity at `:571`, neutral feedback at `:599`, frame planner at `:634`, and capacity clamp diagnostics at `:675`. | Keep CPU fallback authoritative when GPU/provider/layout support is unavailable. GPU completion requires capacity diagnostics, unsupported-module fallback, WGSL parse coverage, readback/parity reporting, transparent pass integration, and provider-unavailable behavior. |
| Optional physics and animation interop | Bevy has separate animation/render foundations, but no default particle-physics or particle-animation integration lane. | Zircon interop contracts live in `interop/physics.rs:4` and `interop/animation.rs:7`; manager diagnostics emit no-op messages when missing capabilities in `service.rs:307` and `:328`. Tests cover physics no-op/enabled behavior at `tests.rs:243`, late capability enablement at `:282`, animation event gating at `:306`, and helper missing-capability reporting at `:695`. | `runtime.feature.particles.physics` and `runtime.feature.particles.animation_control` must remain optional feature dependencies. Missing features produce diagnostics/no-op behavior rather than default-profile failures. |
| Editor authoring and preview | Bevy has no editor in this parity lane, so editor surface is Zircon value-add only. | Zircon editor registers authoring and preview surfaces in `editor/src/authoring.rs:25` through `:48`, component drawers at `:52`, operations for create/add/open/validate/preview at `:71` through `:82`, and CPU sprite template creation at `:175`. Tests cover authoring extension registration at `editor/src/tests.rs:15`, operation/menu/template ids at `:102`, UI templates at `:120`, preview controls at `:160`, and CPU sprite defaults at `:184`. | Editor completion cannot substitute for runtime evidence. It must expose validation diagnostics, disabled capability states, preview play/pause/stop/rewind/warmup, template defaults, and authoring operations without making published clients depend on editor-only crates. |

M8 candidate commands:

- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_runtime --locked -- --nocapture`
- `cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_particles_editor --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`

M8 debug rule: when a particle test fails, diagnose in this order: asset/component schema and numeric validation, seeded CPU simulation, lifetime/spawn/budget behavior, render extraction and transparent sorting, texture/material readiness, GPU provider/layout/fallback/readback, optional physics/animation capability gates, profile/provider availability, then editor authoring. Do not make M8 pass by promoting particles into default profiles or by counting Bevy render/sprite/PBR/post-process infrastructure as first-party particle evidence.

## M9 Export / Editor UX Completion Matrix

M9 is an observability and productization milestone. Bevy core does not ship an editor/export pipeline, so the Bevy evidence is not a direct UI clone. The relevant Bevy contract is the user-facing selection model: Cargo feature profiles, `DefaultPlugins` / `MinimalPlugins`, `PluginGroupBuilder` edit semantics, no-default/no-renderer/plugin-group examples, and dev-only feature warnings. Zircon's equivalent surface is export/profile/plugin-manager/Hub UX. The gate is that every visible plugin/export decision must come from the same structured profile, availability, and build-plan data, not from duplicated UI strings or warning parsing.

| Capability | Bevy source evidence | Zircon owner / current evidence | M9 completion gate |
|---|---|---|---|
| Selection vocabulary | Bevy names public feature collections in `dev/bevy/Cargo.toml:134`, `:137`, `:140`, `:143`, and `:153`; local feature docs describe default, 2D, 3D, dev, asset processor, remote, dynamic linking, and file watcher rows in `dev/bevy/docs/cargo_features.md:24`, `:25`, `:26`, `:39`, `:65`, `:94`, `:122`, and `:128`. | Zircon names runtime/export selection through `RuntimeProfileDescriptor` in `zircon_runtime/src/plugin/runtime_profile.rs:233`, `ExportProfile` in `zircon_runtime/src/plugin/export_profile.rs:116`, and `ProjectPluginManifest` / `ProjectPluginSelection` in `zircon_runtime/src/plugin/project_plugin_manifest/project_plugin_manifest.rs:6` and `project_plugin_selection.rs:10`. | UI and docs must use the same names for runtime profile, target mode, target platform, packaging strategy, maturity, and required/optional membership. No panel may invent a second vocabulary for the same state. |
| Default/minimal/dev disclosure | Bevy `DefaultPlugins` documents feature-controlled composition in `dev/bevy/crates/bevy_internal/src/default_plugins.rs:105`; `MinimalPlugins` is the bare-bones group in `:148` and `:156`; the `dev` feature warns it is for development and not published apps in `dev/bevy/Cargo.toml:153` and `dev/bevy/docs/cargo_features.md:39`. | Zircon app entry selects profiles through `EntryConfig::for_runtime_profile` / `with_runtime_profile` in `zircon_app/src/entry/entry_config.rs:40` and `:44`; app provider registration projects first-party runtime plugins in `zircon_app/src/entry/first_party_runtime_plugins.rs:12`; profile bootstrap tests check linked and optional provider reports in `zircon_app/src/entry/tests/profile_bootstrap.rs:143`, `:174`, `:199`, `:278`, and `:292`. | Hub/editor/export UX must distinguish `minimal`, `client_2d`, `client_3d`, `editor`, `dev`, and `server`; dev-only capabilities such as remote/debug/hot reload must be visible as dev/editor-only and must not silently enter published export profiles. |
| Plugin group editing semantics | Bevy `PluginGroupBuilder` supports `set`, `try_set`, `add_before`, `add_after`, `enable`, and `disable` in `dev/bevy/crates/bevy_app/src/plugin_group.rs:313`, `:325`, `:388`, `:438`, `:488`, and `:502`; missing plugin anchors produce explicit errors in tests around `:663` and `:712`. | Zircon editor plugin status exposes packaging, runtime/editor capabilities, optional features, target modes, and diagnostics in `editor_plugin_status.rs:4`; built-in status computes blocked dependency maps in `status/builtin.rs:46` and `:340`; editor tests pin required built-in plugin disable blocking in `minimal_host_contract.rs:562` and target/packaging editing in `:619`. | Plugin Manager must show required vs optional, dependency gates, blocked reason, target support, packaging choices, and disabled/unload/hot-reload actions. Required built-ins remain non-disableable, and failed edit anchors must be reported as structured diagnostics. |
| Export profile and packaging plan | Bevy uses features and examples for selection/cropping: `examples/app/empty.rs:6` creates an app without defaults, `empty_defaults.rs:6` uses `DefaultPlugins`, `no_renderer.rs:13` disables renderer backends for CI/headless scenarios, and `plugin_group.rs:29` shows group disabling/editing. | Zircon export planning starts from `ExportProfile` and `ExportPackagingStrategy` in `export_profile.rs:109` and `:116`; `from_project_manifest.rs:16` builds the plan, `:221` rejects unsupported native dynamic targets, and `:289` resolves runtime profile; `generated_files.rs:13` and `materialize.rs:11` materialize the output. | Export UI must read `ExportBuildPlan`, not bespoke status text. It must show profile, platform, target mode, strategies, generated files, linked runtime crates, native dynamic packages, and fatal diagnostics; fatal plan diagnostics block packaging. |
| Provider availability and fatal diagnostics | Bevy plugin lifecycle and uniqueness are explicit in `dev/bevy/crates/bevy_app/src/plugin.rs:57`, `:61`, `:68`, `:74`, and `:87`; plugin-group edits produce concrete missing/duplicate-style errors rather than vague warnings. | Zircon availability buckets live in `RuntimePluginAvailabilityReport` fields `linked`, `native_dynamic`, `externalized_missing`, and `missing_required` in `runtime_profile.rs:48`; query/diagnostic helpers are in `:82`, `:126`, `:407`, and provider bucketing in `:424`, `:449`, `:489`, and `:498`; `ExportBuildPlan::has_fatal_diagnostics` is in `export_build_plan.rs:103`. | Every UI surface must expose linked, native dynamic, externalized missing, missing required, maturity/stub/blocked state, counts, and entry details from the same report. Missing required entries are fatal, optional missing entries remain warnings. |
| Native dynamic packaging | Bevy `dynamic_linking` is a development compile-time feature in `dev/bevy/Cargo.toml:279` and `dev/bevy/docs/cargo_features.md:122`; it is not runtime plugin packaging. | Zircon has export packaging `NativeDynamic` in `export_profile.rs:109`; native-aware editor status maps default packaging in `status/native.rs:23`, `:130`, and `:230`; export preparation/probe happens in `export_build/manager.rs:47`, `:115`, `:190`, and `:282`; materialization reports missing native artifacts in `materialize.rs:154`. | UX must clearly separate Bevy-style dev dynamic linking from Zircon export native dynamic plugins. Unsupported target/platform, duplicate package directory, missing artifact, and probe/load failures must be visible as build/export diagnostics. |
| Plugin Manager status | Bevy plugin docs describe plugins as app extensions, and plugin-group examples show users how to enable, disable, and order those extensions. Public docs begin at `https://bevy.org/learn/quick-start/getting-started/plugins/`. | Zircon `EditorPluginStatusReport` is in `editor_plugin_status_report.rs:4`; built-in/native reports merge package manifest, feature dependency, capabilities, packaging, and target modes in `status/builtin.rs:19`, `:150`, `:174`, `:248`, and `status/native.rs:31`, `:62`, `:113`; pane presentation pins visible actions and diagnostics in `pane_presentation.rs:604` through `:649`. | Plugin Manager completion requires visible maturity, profile membership, target support, runtime/editor capabilities, optional feature dependency state, default/current packaging, diagnostics, primary action, packaging action, target mode action, unload, and hot reload. |
| Hub project/engine plugin list | Bevy users see feature/profile decisions in `Cargo.toml` and feature docs before compiling. | Zircon Hub discovers engine/project plugin manifests in `zircon_hub/src/plugins/catalog.rs:18`, `:49`, `:103`, `:151`, and reads maturity/default packaging at `:172`; the Hub view model projects maturity, packaging, and module count in `zircon_hub/src/app/view_model/plugins.rs:10`, `:32`, `:33`, and `:34`; Slint views expose row metadata in `zircon_hub/ui/plugins.slint:11` and `zircon_hub/ui/shared.slint:110`. | Hub may summarize, but it cannot diverge from runtime/export state. It must either show the same availability/export buckets or clearly link to the editor/export report that owns them. |
| Build/export progress and cancellation | Bevy does not provide this pipeline; the relevant contrast is that `dev` features are explicitly not for published apps and no-renderer examples support non-visual validation. | Zircon export manager reports stages for resolving plan, native package prep, materialize, cargo build, probe, diagnostics, completion, cancellation, and progress in `export_build/manager.rs:81`, `:103`, `:115`, `:130`, `:158`, `:190`, `:212`, and `:221`; build/export pane payloads expose profile, platform, target mode, strategies, enabled plugins, linked crates, native packages, generated files, diagnostics, and fatal state in `pane_payload_builders/build_export.rs:17` through `:27`. | Export UX must show stage progress, generated files, native prep/probe, skipped cargo, diagnostic output, fatal state, and cancellation cleanup. It must not rely on a single "ready" label when structured diagnostics exist. |

M9 candidate commands:

- `cargo test -p zircon_runtime --lib plugin_extensions::export_build_plan --locked -- --nocapture`
- `cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1`
- `cargo test -p zircon_editor --lib minimal_host_contract --locked --jobs 1 -- --nocapture`
- `cargo test -p zircon_editor --lib pane_presentation --locked --jobs 1 -- --nocapture`
- `cargo test --manifest-path zircon_hub/Cargo.toml plugins --locked --offline --jobs 1 -- --nocapture`

M9 debug rule: when an export/editor UX test fails, diagnose in this order: `RuntimeProfileDescriptor` / `ProjectPluginManifest` / `ExportProfile` source data, `ExportBuildPlan` provider/packaging/platform fatal diagnostics, `RuntimePluginAvailabilityReport` buckets, editor status projection, native dynamic preparation/probe, pane payload conversion, then Hub presentation. Do not fix failures by duplicating plugin state in UI code or parsing warning strings.

## M10 Docs / CI Gates Completion Matrix

M10 turns the roadmap into a regression-resistant maintenance contract. Bevy's strongest precedent is not a single command; it is a layered validation system: CI runs build/test/lint/doc/compile/example checks, docs deployment builds all-feature rustdoc, and feature/example documentation has explicit "missing update" checks. Zircon's equivalent gate must keep profile/plugin docs, CI commands, local validator usage, export-platform contracts, and milestone evidence synchronized.

| Capability | Bevy source evidence | Zircon owner / current evidence | M10 completion gate |
|---|---|---|---|
| Layered CI command model | Bevy `.github/workflows/ci.yml:58` runs `cargo run -p ci -- test`, `:97` runs lints, `:165` runs compile checks, `:420` runs doc checks; `dev/bevy/tools/ci/src/ci.rs:75` through `:92` expands the default command list into format, clippy, tests, integration tests, doc checks, compile-fail, benches, and examples. | Zircon `.github/workflows/ci.yml:49` builds the workspace, `:52` tests it, `:55` checks plugin workspace targets, `:58` builds plugins, `:61` tests plugins, and `:104` runs export platform policy across the platform matrix. Local validation is centralized by `.codex/skills/zircon-dev/scripts/validate-matrix.ps1:576` and runs Cargo build/test at `:615` and `:621`. | M10 must preserve a named validation ladder: docs/static checks, focused crate tests, workspace build/test, plugin workspace check/build/test, export-platform matrix, and final validator run. A milestone cannot be promoted from a smoke test alone. |
| Documentation build and doc tests | Bevy `tools/ci/src/commands/doc.rs:8` aliases doc-test and doc-check; `doc_test.rs:19` runs workspace doc tests; `doc_check.rs:17` builds all-feature private-item docs with `RUSTDOCFLAGS=-D warnings`. Docs deployment builds all-feature rustdoc in `.github/workflows/docs.yml:68` through `:75`. | Zircon code-facing docs use machine-readable frontmatter, required by `.codex/skills/zircon-project-skills/code-module-docs-maintenance/required-doc-header-format.md`; current profile/plugin docs already record related code, implementation files, plan sources, tests, and validation notes. | Every touched subsystem must have a doc owner with `related_code`, `implementation_files`, `plan_sources`, and `tests`. M10 should add a future rustdoc/doc-test lane only after current warning volume is known, but no milestone may omit docs for changed runtime/plugin/export/editor contracts. |
| Feature/profile documentation synchronization | Bevy checks missing example metadata in `.github/workflows/ci.yml:431` and missing feature docs in `:469`; `:482` runs `cargo run -p build-templated-pages -- check-missing features`, and `:484` updates feature docs, then `git diff --quiet HEAD --` fails if docs are stale. | Zircon profile/plugin source of truth is split across `RuntimeProfileDescriptor`, `RuntimePluginDescriptor::builtin_catalog`, `zircon_plugins/*/plugin.toml`, `docs/runtime-plugins/profile-selection.md`, and this matrix. Current docs-only reviews already record M0 through M9 coverage and candidate commands. | Add or maintain a profile/catalog docs-sync gate: every `RuntimePluginId`, first-party plugin TOML capability/maturity row, profile default/optional membership, and export availability bucket must have a row in profile-selection or the parity matrix. Stale docs fail before profile promotion. |
| Formatting and prose hygiene | Bevy CI runs `cargo fmt --all -- --check` via `format.rs:13`, Clippy with `-Dwarnings` via `clippy.rs:17`, Taplo in `.github/workflows/ci.yml:359`, typos at `:377`, and Markdown lint at `:331` through `:344`. | Zircon currently has scoped `git diff --check` evidence in docs, Rust validation through Cargo, and no dedicated markdown/toml/typo CI lane in the visible workflow. | M10 acceptance should at least require scoped `git diff --check`, placeholder scans, and command/evidence recording. Adding Taplo/Markdown/typo lanes is a follow-on CI hardening task, not a prerequisite for plugin parity docs to remain useful. |
| Example and template validation | Bevy compiles examples with `tools/ci/src/commands/example_check.rs:15`; the separate example-run workflow executes selected examples with `CI_TESTING_CONFIG` in `.github/workflows/example-run.yml:55`, `:135`, and `:204`. | Zircon's analogous surfaces are app examples, export host generation, project templates, Hub plugin catalog fixtures, and editor pane payload tests. M9 already mapped export/editor UX to runtime/export/editor/Hub tests. | M10 must keep examples/templates tied to the feature they demonstrate: project templates choose explicit runtime profiles, generated export hosts carry profile ids, and plugin-manager/Hub examples cannot rely on stale display strings. |
| Cross-platform/export contract | Bevy CI checks no-std and wasm compile lanes in `.github/workflows/ci.yml:196`, `:201`, `:232`, `:263`, `:294`, and `:324`. | Zircon CI runs `export-platform-contract` for `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless` in `.github/workflows/ci.yml`; the `headless` lane pins the server/native runner policy separately from Linux desktop export. | Any profile/export/plugin change must declare which platforms it affects and keep `platform_target_policy_matches_host_resource_and_plugin_strategy` green across the platform matrix. Native dynamic/export-host work cannot be promoted from desktop-only evidence. |
| Dependency and security lanes | Bevy dependency workflow runs `cargo deny check advisories`, `bans`, `licenses`, and `sources` in `.github/workflows/dependencies.yml:38`, `:52`, `:66`, and `:80`; security static analysis is separate in `.github/workflows/security-static-analysis.yml`. | Zircon visible CI does not yet expose a `cargo-deny`/CodeQL-equivalent gate. Lockfile changes currently show up through `--locked` Cargo commands and manual review. | M10 should mark dependency/security automation as a CI-hardening follow-up. Until then, any lockfile or new crate change must explicitly record why it is in scope and must pass locked workspace/plugin validation after the lockfile is intentionally updated. |
| Acceptance evidence and reporting | Bevy CI comment workflows surface missing feature/example artifacts and failure comments in `.github/workflows/ci-comment-failures.yml:86`, `:142`, and `:155`. | Zircon reporting rules require exact command scope, `--locked` status, workspace-vs-crate validation scope, CI parity comparison, and remaining risks in `.codex/skills/zircon-dev/reporting.md`. | Every milestone completion note must record exact commands, package/workspace scope, `--locked` use, skipped blockers, active-session constraints, and remaining gaps. "No Cargo was run" is acceptable for docs-only continuations only when stated explicitly. |

M10 candidate commands:

- `git diff --check -- docs/runtime-plugins/bevy-parity-matrix.md docs/runtime-plugins/profile-selection.md ".codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md"`
- Placeholder scan across the parity matrix, profile-selection guide, and main milestone plan using the repository's standard placeholder-pattern set.
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime`
- `cargo build --workspace --locked --verbose`
- `cargo test --workspace --locked --verbose`
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose`
- `cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`
- `cargo test --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose`
- `cargo test -p zircon_runtime platform_target_policy_matches_host_resource_and_plugin_strategy --locked --verbose` with `ZR_EXPORT_CONTRACT_PLATFORM` set to every platform in the CI matrix.

M10 debug rule: when a docs/CI gate fails, diagnose in this order: stale source-of-truth metadata (`RuntimePluginId`, plugin TOML, profile descriptor), stale docs header or parity row, focused profile/export test failure, plugin workspace failure, export platform policy failure, then full workspace CI failure. Do not weaken CI or delete docs rows to make the gate pass; update the source or the documented contract.

## Cross-Milestone Promotion Rollup

This rollup is the operational view of the M0-M10 matrices. It exists so future implementation work promotes capabilities in dependency order instead of advancing upper UX or optional features before the profile/provider foundation is proven.

| Promotion layer | Milestones | Dependency rule | Required evidence |
|---|---|---|---|
| Foundation closure | M0, M1, M2 | No subsystem may be marked default-profile complete until capability classification, profile/maturity policy, and provider-chain availability are all documented and test-covered. | `RuntimePluginId` coverage, plugin TOML maturity/status coverage, `RuntimePluginAvailabilityReport` buckets, app profile bootstrap, export availability, and no required `Externalized` / `Stub` in stable/default profiles. |
| Default-facing feature promotion | M3, M4, M5 | Animation, sound, and sprite work must prove lower asset/import/runtime/scene/render support before entering default-facing profiles. | Bevy source evidence, Zircon runtime tests, profile/provider tests, import/asset readiness, render/scene ordering where relevant, and explicit advanced-feature divergence. |
| Optional advanced lanes | M6, M7, M8 | Split Bevy parity capabilities from Zircon advanced capabilities before promoting either. | UI navigation separate from gameplay navmesh, developer remote separate from gameplay networking, particles classified as experimental optional, and profile gates reflecting those splits. |
| Productization | M9 | Editor/export/Hub surfaces may summarize plugin state only after runtime/export reports are the source of truth. | `ExportBuildPlan`, `RuntimePluginAvailabilityReport`, `EditorPluginStatusReport`, pane payloads, Hub catalog/view model, native dynamic packaging diagnostics, and no warning-string parsing. |
| Regression protection | M10 | A capability is not durable until docs, CI command shape, and acceptance evidence are recorded. | Machine-readable doc headers, candidate commands, debug order, scoped docs checks, focused tests, plugin workspace checks, export-platform matrix, workspace build/test, and validator/CI evidence. |

Promotion stops at the first failing lower layer. If a M9 or M10 check fails because M1/M2 profile/provider state is stale, the fix belongs in the profile/provider contract first, not in editor/export presentation. If a default-facing feature fails because importer, asset, scene, render, or provider readiness is incomplete, the feature remains partial even if an editor or demo path can display it.

## Bevy References

The metadata intentionally points to local Bevy source files rather than copying API shapes:

- `dev/bevy/crates/bevy_internal/src/default_plugins.rs` for default/minimal plugin ordering.
- `dev/bevy/crates/bevy_app/src/plugin.rs` and `dev/bevy/crates/bevy_app/src/plugin_group.rs` for plugin lifecycle, uniqueness, builder editing semantics, and order/error tests. Public plugin docs begin at `https://bevy.org/learn/quick-start/getting-started/plugins/`.
- `dev/bevy/Cargo.toml` and `dev/bevy/docs/cargo_features.md` for profile/collection vocabulary, dev-only warnings, dynamic linking, asset processor, remote, and file watcher features.
- `dev/bevy/.github/workflows/ci.yml`, `docs.yml`, `dependencies.yml`, `example-run.yml`, and `ci-comment-failures.yml`, plus `dev/bevy/tools/ci/src/ci.rs` and `dev/bevy/tools/ci/src/commands/*`, for layered validation, docs build, missing feature/example docs checks, example execution, dependency/security lanes, and PR feedback artifacts.
- `dev/bevy/examples/app/empty.rs`, `empty_defaults.rs`, `no_renderer.rs`, `plugin_group.rs`, and `plugin.rs` for no-default, default, no-renderer, plugin-group editing, and plugin authoring examples.
- `dev/bevy/crates/bevy_audio/src/lib.rs`, `audio.rs`, `sinks.rs`, `volume.rs`, `audio_source.rs`, and `audio_output.rs` for audio plugin setup, playback settings/modes, sink controls, volume math, decoding, output availability, cleanup, and spatial update precedent; public API docs begin at `https://docs.rs/bevy_audio/latest/bevy_audio/`.
- `dev/bevy/crates/bevy_animation/src/lib.rs`, `dev/bevy/crates/bevy_animation/src/graph.rs`, `dev/bevy/crates/bevy_animation/src/transition.rs`, and `dev/bevy/crates/bevy_animation/src/animation_event.rs` for animation clip/graph/transition/event precedent.
- `dev/bevy/crates/bevy_sprite/src/lib.rs`, `sprite.rs`, `sprite_mesh.rs`, and `texture_slice/slicer.rs` plus `dev/bevy/crates/bevy_sprite_render/src/lib.rs`, `render/mod.rs`, `texture_slice/computed_slices.rs`, `mesh2d/mesh.rs`, and `mesh2d/color_material.rs` for sprite API/render split, bounds, slice/tile behavior, extraction, queueing, image bind groups, 2D mesh/material, and tilemap composition.
- `dev/bevy/crates/bevy_input_focus/src/lib.rs`, `tab_navigation.rs`, and `directional_navigation.rs`; `dev/bevy/crates/bevy_ui/src/auto_directional_navigation.rs` and `focus.rs`; and `dev/bevy/examples/ui/navigation/directional_navigation.rs` for UI focus, focused input dispatch, tab navigation, directional/manual navigation, automatic UI directional navigation, pointer focus policy, and keyboard/gamepad navigation examples. Public docs begin at `https://docs.rs/bevy/latest/bevy/input_focus/`, `https://docs.rs/bevy/latest/bevy/input_focus/tab_navigation/`, `https://docs.rs/bevy/latest/bevy/input_focus/directional_navigation/`, and `https://docs.rs/bevy/latest/bevy/ui/auto_directional_navigation/`.
- `dev/bevy/crates/bevy_remote/src/lib.rs`, `builtin_methods.rs`, `http.rs`, `schemas/open_rpc.rs`, and `schemas/json_schema.rs`, plus `dev/bevy/examples/remote/server.rs`, `client.rs`, and `integration_test.rs`, for Bevy Remote Protocol JSON-RPC envelope, method registry, built-in world/resource/schedule/schema methods, OpenRPC discovery, HTTP transport separation, loopback defaults, and example request flows. Public docs begin at `https://docs.rs/bevy_remote/latest/bevy_remote/`.

Navigation and particles diverge deliberately: Bevy first-party navigation in the referenced plan is UI focus/directional navigation, while Zircon's current `navigation` plugin is gameplay navmesh/pathfinding. Bevy core has no first-party particles crate or feature row in the local `dev/bevy` tree; Bevy render/sprite/PBR/post-process sources are particle-adjacent infrastructure, not a first-party particle plugin. Zircon particles stay experimental optional. Public Bevy feature docs begin at `https://docs.rs/crate/bevy/latest/features`.

## Source Traceability Acceptance Rules

This matrix is accepted only when every parity claim can be traced from a local Bevy source anchor to a Zircon owner path and a validation artifact. A row may stay documentation-only, but it must say so in `Validation` and must not promote a plugin from `Partial` to `Complete` without implementation evidence.

| Route | Bevy anchors | Zircon owner path | Acceptance rule |
|---|---|---|---|
| App shell and profiles | `dev/bevy/Cargo.toml`, `docs/cargo_features.md`, `crates/bevy_internal/src/default_plugins.rs`, `crates/bevy_app/src/plugin.rs`, `crates/bevy_app/src/plugin_group.rs`, `crates/bevy_internal/src/prelude.rs` | `zircon_app/src/plugins/*`, `zircon_app/src/prelude.rs`, `zircon_runtime/src/prelude.rs`, `zircon_runtime/src/plugin/*` | Profile membership, stable prelude, plugin group editing, and maturity policy must be reviewed together. A default profile claim is invalid if required plugins can still fall through to unstructured externalized warnings. |
| Core runtime foundation | `bevy_state/src/app.rs`, `bevy_time/src/lib.rs`, `bevy_app/src/task_pool_plugin.rs`, `bevy_log/src/lib.rs`, `bevy_diagnostic/src/lib.rs`, `frame_time_diagnostics_plugin.rs`, `log_diagnostics_plugin.rs` | `zircon_runtime/src/core/state`, `zircon_runtime/src/core/framework/time`, `zircon_runtime/src/core/tasks`, `zircon_runtime/src/core/diagnostics`, `zircon_runtime/src/diagnostic_log`, `zircon_runtime/src/core/runtime/runtime.rs` | `State`, `Time`, task pools, logging, diagnostics, and dev profile behavior count as Bevy-level only when they have prelude policy, schedule/tick semantics, diagnostics shape, and profile split evidence. |
| Provider closure | `DefaultPlugins` direct plugin entries and `PluginGroupBuilder` error/order semantics | `zircon_runtime/src/builtin/runtime_modules.rs`, `zircon_runtime/src/plugin/runtime_profile.rs`, `zircon_app/src/entry/*`, first-party provider registration sites | A plugin report must classify each plugin as available, linked, native dynamic, externalized missing, stub, blocked, or missing required. Editor/export/Hub cannot repair missing provider truth with presentation logic. |
| Animation | `bevy_animation/src/lib.rs`, `graph.rs`, `transition.rs`, `animation_event.rs` | `zircon_plugins/animation/runtime`, animation graph/editor owners, profile provider tests | Completion requires clip/target id/graph/transition/event/glTF/schedule evidence. Registration alone is partial. |
| Sound | `bevy_audio/src/lib.rs`, `audio.rs`, `audio_output.rs`, `audio_source.rs`, `sinks.rs`, `volume.rs` | `zircon_plugins/sound/runtime`, audio importer, output backend, mixer tests, profile/export reports | Completion requires asset import, playback settings, global volume, sink controls, spatial update, cleanup, and output-unavailable diagnostics. Advanced DSP is optional and does not replace baseline evidence. |
| Sprite2D | `bevy_sprite/src/lib.rs`, `sprite.rs`, `sprite_mesh.rs`, `texture_slice/slicer.rs`, `bevy_sprite_render/src/lib.rs`, `render/mod.rs`, `texture_slice/computed_slices.rs` | Zircon sprite/render runtime owners, render snapshot/extract/queue tests, future `zircon_plugins/sprite_2d/runtime` | Completion requires a dedicated `Sprite2d` plugin identity, profile gate, image/atlas/custom-rect/custom-size/slice/tile behavior, extraction, queueing, and render ordering evidence. |
| Navigation | `bevy_input_focus/src/lib.rs`, `tab_navigation.rs`, `directional_navigation.rs`, `bevy_ui/src/auto_directional_navigation.rs`, `examples/ui/navigation/directional_navigation.rs` | Zircon UI focus/navigation/a11y owners and gameplay `zircon_plugins/navigation/runtime` | UI focus/tab/directional navigation is the Bevy parity lane. Navmesh/pathfinding is a separate advanced lane and cannot satisfy this row. |
| Remote / net | `bevy_remote/src/lib.rs`, `builtin_methods.rs`, `http.rs`, `schemas/open_rpc.rs`, `schemas/json_schema.rs`, `examples/remote/*` | `zircon_plugins/net/runtime`, RPC/HTTP/WebSocket/replication/reliable/content feature gates, dev/editor/server profiles | Developer remote protocol must be opt-in and must not open default client/server listeners. Gameplay networking is an advanced lane with separate maturity/profile gates. |
| Particles / VFX | `dev/bevy/crates` has no `bevy_particles` crate; default feature rows do not include particles | `zircon_plugins/particles/plugin.toml`, `zircon_plugins/particles/runtime`, VFX/render interop/editor tests | Particles cannot block Bevy default parity. Zircon may promote it only as experimental optional with deterministic simulation, render integration, diagnostics, and feature dependency evidence. |
| Export/editor/docs/CI | Bevy app examples, public plugin guide, `.github/workflows/ci.yml`, `.github/workflows/docs.yml`, `tools/ci/src/ci.rs`, `tools/ci/src/commands/*` | `ExportBuildPlan`, `ProjectPluginManifest`, `EditorPluginStatusReport`, Hub catalog/view-model, `.github/workflows/ci.yml`, `validate-matrix.ps1`, docs owners | Product surfaces and docs/CI may only consume the shared report/profile model. A promoted row needs exact commands, doc owner, debug order, and recorded evidence. |

## Validation Package Ladder

Use this ladder to classify validation evidence in this document. A row can cite a lower rung while it is being planned, but promotion from `Partial` to `Complete` requires the highest rung that matches the row's runtime surface.

| Rung | Scope | Accepts | Does not accept |
|---|---|---|---|
| V0 Docs-only | Source reading, matrix edits, owner mapping, placeholder scan, `git diff --check`. | Planning rows, source traceability, intentional divergence records, candidate commands. | Any claim that code builds, tests pass, runtime behavior works, or a plugin is complete. |
| V1 Static contract | Schema/manifest/catalog/profile rows, non-Cargo scans, focused source inspection. | M0 inventory, docs-sync design, maturity/status/profile consistency when no code changed. | Runtime behavior, linked provider behavior, app bootstrap, export materialization, rendering/audio/input behavior. |
| V2 Focused runtime tests | Single crate or focused package tests such as runtime profile, export build plan, sound/animation/sprite/net/particles unit tests. | A narrow implementation slice in one owner crate. | Default profile promotion, cross-crate provider closure, editor/export/Hub productization. |
| V3 App/profile/export integration | `zircon_app` profile bootstrap, runtime provider registration, export build plan/materialize report, native dynamic classification. | M1/M2 profile/provider closure and M9 lower data contracts. | Plugin workspace-wide readiness or full workspace health. |
| V4 Plugin workspace | `cargo check/build/test --manifest-path zircon_plugins/Cargo.toml --workspace --locked ...` and feature-specific plugin lanes. | First-party plugin runtime promotion for animation/sound/navigation/net/particles and future sprite2d. | Export platform policy, Hub/editor UX, full root workspace claims. |
| V5 Export/platform matrix | `ZR_EXPORT_CONTRACT_PLATFORM` matrix and native/linked/platform diagnostics. | Packaging/profile/platform claims for export and server/client targets. | Full CI equivalence, docs freshness, all workspace packages. |
| V6 Workspace/validator/CI | Root workspace build/test, validator script, or CI job evidence with exact platform and command. | M10 regression guard and final milestone promotion. | Claims outside the command scope, stale log reuse, or unchecked warnings treated as accepted. |

Validation recording rules:
- Every validation note must include the rung, exact command or static check, platform, scope, and result.
- `V0` and `V1` entries must explicitly say no Cargo command was run.
- When a higher rung is blocked by another active session or shared target contention, record the blocker and keep the row unpromoted.
- If a higher rung fails because a lower rung is stale, fix the lower source-of-truth first: plugin id/catalog/TOML/profile descriptor before app/export/editor presentation.

## Critical-Path Rejection Rules

These rules reject common false-positive parity claims. They are intentionally negative because the matrix is used to decide when a plugin or profile can be promoted.

| Claim pattern | Reject because | Required replacement evidence |
|---|---|---|
| "A first-party plugin exists, so the profile is complete." | Bevy default plugins are direct app plugins with runtime behavior, not descriptors alone. | Provider-chain evidence plus focused runtime tests and profile/bootstrap availability. |
| "A warning says the runtime is externalized, so tooling can show it as optional." | Tooling would be parsing a diagnostic string instead of source-of-truth state. | `RuntimePluginAvailabilityReport` category plus profile required/optional membership. |
| "Gameplay navigation closes Bevy navigation." | Bevy evidence here is input focus, tab navigation, directional navigation, and UI auto navigation. | Separate `ui_navigation` evidence and optional gameplay navmesh/pathfinding evidence. |
| "Remote networking closes Bevy Remote." | Bevy Remote Protocol is a developer JSON-RPC method registry; HTTP transport is a separate opt-in plugin. | Method envelope, registry/schema/world/schedule methods, no-default-listener tests, and dev/editor profile gate. |
| "Particles close a Bevy default gap." | Bevy core has no first-party particles crate or default feature row. | Experimental optional particle evidence with profile gates and explicit non-blocking status. |
| "Advanced renderer output closes default sprite/PBR/presentation gaps." | Bevy default render evidence covers render/image/mesh/camera/light/core pipeline/post-process/AA/sprite/UI/PBR before experimental advanced render. | Default renderer family tests, render product readiness, presentation target evidence, and diagnostics bridge. |
| "Docs-only source review proves current checkout behavior." | V0/V1 validation does not execute runtime/build/test paths. | V2-V6 evidence matching the row surface, with exact command and platform. |
| "Editor/Hub displays a green status, so the runtime is ready." | Product surfaces may lag or duplicate state. | Shared runtime/export report evidence and no second status model in UI layers. |

Decision records:
- D1: Bevy parity means user-facing profile/plugin/capability completion, not API name cloning.
- D2: `zircon_runtime` owns descriptors and reports, while concrete first-party providers remain outside runtime core.
- D3: Default-facing promotion starts only after M0-M2 foundation closure.
- D4: Sound is the preferred first proof case for provider-chain-to-profile-to-export because Bevy audio has a compact baseline and Zircon sound already has a real runtime.
- D5: Particles, gameplay navigation, replication/reliable UDP, and advanced rendering are optional advanced lanes unless a future Bevy source baseline changes.

## Evidence Record Requirements

Every new validation entry in this matrix must be useful to a future worker who did not see the terminal output. A valid evidence record contains these fields in prose or table form:

| Field | Required content |
|---|---|
| Date and validation rung | Calendar date, V0-V6 rung, and whether the pass was docs-only or runtime/build/test evidence. |
| Scope | Milestone, plugin/profile/export/editor/Hub surface, package or workspace boundary, and exact owner docs touched. |
| Bevy evidence | Local Bevy source anchors and public docs links used for the parity claim. |
| Zircon evidence | Zircon source paths, tests, manifests, profile rows, reports, or docs that were inspected or changed. |
| Command evidence | Exact command or static check, platform, `--locked` usage, feature flags, offline/target-dir notes, and result. V0/V1 records must state that no Cargo command was run. |
| Promotion decision | `Unpromoted`, `Partial`, `Complete`, or `Deferred`, with the reason tied to a lower gate or validation rung. |
| Remaining gap | The next concrete lower-layer fix, validation command, or docs-sync item. |

Promotion labels:
- `Unpromoted` means the row is planning evidence only, usually V0/V1.
- `Partial` means at least one focused behavior has current evidence but required completion gates remain open.
- `Complete` means the row reached the highest required validation rung for its runtime surface and no stop condition remains.
- `Deferred` means promotion waits on a named active session, target contention, platform lane, or external owner.

Evidence records must not rely on hidden terminal output. When a command is important, summarize the pass/fail count or failure class in the validation note so the matrix remains self-contained.

## First Wave Work-Order Acceptance

This table mirrors the first execution wave in `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`.
It defines the minimum evidence needed before a work-order row can change a matrix status.

| Work order | Required validation rung | Matrix acceptance note |
|---|---|---|
| W0.1 M0 catalog freeze | V0 static review now; V1 docs-sync script design when implemented. | Every Bevy default/minimal/profile capability, `RuntimePluginId`, and first-party package has one row with Bevy anchor, Zircon owner, maturity/capability decision, and next gate. |
| W1.1 M1 profile contract refresh | V2 focused profile tests for code changes; V0/V1 for docs-only reconciliation. | Profile rows may change only when profile semantics, required/optional membership, target mode, and `Externalized`/`Stub` policy are explicit. |
| W1.2 M1 core foundation prelude gates | V2 focused app/runtime tests; V0 for docs-only source review. | App/prelude/core foundation rows need current evidence for stable prelude, group mutation, state/time/tasks/log/diagnostics, and dev/default log split. |
| W2.1 M2 availability report contract | V3 app/profile/export integration evidence for implementation; V0 if only documenting. | Rows must cite structured `RuntimePluginAvailabilityReport` buckets and reject warning-string parsing as evidence. |
| W2.2 M2 provider chain negative cases | V3 negative-case tests. | Missing required first-party plugins are fatal; optional missing plugins stay structured diagnostics; categories are distinguishable and deduplicated by `RuntimePluginId`. |
| W4.1 M4 sound proof case | V4 plugin/profile/export evidence after W2 acceptance. | Sound is the first default-facing proof candidate, not full default-profile completion; optional timeline/HRTF/ray-traced reverb features stay separate. |
| W10.1 M10 docs-sync check design | V0 design now; V1/V2 scripted/static validation after coordination with the active M10 docs/CI owner. | The matrix may record the docs-sync gate shape before automation, but CI protection requires a real script or workflow evidence record. |

Foundation rule: W0.1, W1.1, W1.2, W2.1, and W2.2 are blockers for default-profile completion claims.
W4.1 is a proof candidate, not a substitute for completing animation/sprite/UI/render/asset breadth.
W10.1 may start as docs-only design, but promotion to CI gate requires current command evidence.

## Foundation Packet Evidence Map

These evidence rows bind the W0-W2 execution packets to matrix updates. A packet may add source review notes before code changes, but the matrix status changes only at the validation rung named here.

| Packet | Evidence that must be recorded | Status-change rule |
|---|---|---|
| W0.1 catalog freeze | Count of Bevy default/minimal/dev capabilities reviewed, count of `RuntimePluginId` rows, count of first-party plugin packages, explicit rows for package-only/tooling/editor/importer coverage, and source anchors for intentional divergences. | May move inventory rows from unclassified to classified. Must not mark runtime behavior complete. |
| W1.1 profile contract refresh | Profile membership table for `minimal/client_2d/client_3d/editor/dev/server`, required vs optional decisions, target mode compatibility, maturity floor, and externalized/stub policy. | May promote profile contract rows only with V2 focused profile/group tests or an equivalent current command record. |
| W1.2 core foundation prelude gates | Stable prelude exports, plugin-group mutation behavior, state/time/task/log/diagnostic owner paths, dev/default diagnostic split, and named app/runtime tests. | May promote app/core foundation rows only when prelude and core runtime behavior both have current V2 evidence. |
| W2.1 availability report contract | Structured category evidence for available, linked, native dynamic, externalized missing, stub, blocked, and missing required entries; public query helpers used by app/export/editor/Hub surfaces. | May promote availability rows only with V3 app/profile/export evidence and no warning-string parsing. |
| W2.2 provider chain negative cases | Negative-case list for linked present/absent, native dynamic present/unsupported, externalized missing, target/maturity block, stub, optional missing, and missing required. | May promote provider-chain closure only when missing required is fatal, optional missing is structured diagnostic, and duplicate diagnostics are deduplicated by `RuntimePluginId`. |

Packet sequencing is strict: W0.1 precedes W1/W2 status promotion; W1.1 and W1.2 must both be accepted before W2 promotion; W2.1 precedes W2.2 assertions. Later subsystem rows may cite packet text as planning evidence, but not as runtime/build/test evidence.

## Default Subsystem Packet Acceptance

These rows mirror the S3-S9 packet board in `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`. They are acceptance constraints for subsystem rows that want to move from planning/partial evidence to promoted default-facing status.

| Packet | Minimum promotion evidence | Matrix rejection rule |
|---|---|---|
| S3 M3 animation | V4 plugin/profile evidence for clip asset, target id, graph/blend/transition, event timing, glTF fixture, scene-stage ordering, and provider/profile registration. | Reject promotion if animation only proves a manager exists, if target ids are unstable, or if graph/transition/event edge cases are not covered. |
| S4 M4 sound | V4 plugin/profile/export evidence for playback, importer, output backend fallback, sink cleanup, global/spatial controls, and profile/provider visibility. | Reject promotion when optional timeline/HRTF/occlusion/ray-traced features are used instead of Bevy-baseline audio behavior. |
| S5 M5 sprite2d | V4 plugin/render/profile evidence for dedicated Sprite2D identity, component schema, atlas/slice/tile readiness, bounds/culling, extraction/queueing, and profile membership. | Reject promotion if `texture`, `tilemap_2d`, generic rendering, or particle sprites substitute for first-party `sprite_2d` identity. |
| S6 M6 UI navigation | V4 UI runtime/profile evidence for focus, focused input dispatch, tab/group traversal, directional navigation, auto navigation, hidden/disabled cleanup, and modal/group negative cases. | Reject promotion if gameplay navigation/pathfinding is counted as UI focus parity or if a11y/input routing is only documented without tests. |
| S7 M7 remote/net | V4 plugin/profile/dev-protocol evidence for explicit dev/editor/server profile membership, method registry/schema, request/response/error envelope, and transport opt-in. | Reject promotion if default client profiles open remote transports or if gameplay networking/replication/content download is treated as Bevy Remote Protocol parity. |
| S8 M8 particles/VFX | V4 optional advanced evidence for deterministic simulation, CPU/GPU fallback, render ordering, profile diagnostics, and editor authoring when claimed. | Reject any claim that particles are a Bevy default blocker or that Bevy render/sprite/PBR/post-process infrastructure proves particles complete. |
| S9 M9 export/editor UX | V5 export/editor/Hub evidence that all surfaces consume the same `RuntimeProfileDescriptor` -> `ExportBuildPlan` -> `RuntimePluginAvailabilityReport` -> status projection chain. | Reject promotion if UI owns a separate availability model, parses warning strings, or hides target/maturity/profile blocked reasons. |

Default-facing rule: S3-S7 and S9 need accepted W0-W2 foundation evidence before they can promote default-facing rows. S8 is optional advanced regardless of how complete its runtime becomes. Rows owned by active subsystem sessions may receive V0 planning notes here, but implementation evidence belongs to the owning session's validation record.

## Packet Promotion Evidence Checklist

Use this checklist when adding a validation note that changes a row status. It aligns this matrix with the packet promotion runbook in `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`.

| Evidence stage | Required record | Allowed matrix change | Rejection condition |
|---|---|---|---|
| P0 Claim intake | Packet id, target row, active-session scan result, Bevy source anchors, Zircon owner paths, and W0-W2 gate status. | No status promotion; may mark row `Deferred` if an owner session blocks implementation. | Missing packet id, missing owner path, or conflict with an active owner session. |
| P1 V0/V1 source/docs | Static source review, docs changed, static script design if any, and explicit "No Cargo command was run" when applicable. | May classify rows or keep `Partial`; cannot mark runtime behavior `Complete`. | Stale logs, hidden terminal output, or "Bevy does this" without source anchors. |
| P2 V2 focused evidence | Focused crate/unit command with platform, flags, pass/fail count, and first failing lower layer. | May promote foundation-only rows when the packet requires no higher integration evidence. | Any lower foundation failure, missing negative case, or no named command. |
| P3 V3 integration evidence | App/profile/export command evidence, structured availability buckets, fatal/optional provider behavior, and report API consumers. | May promote W2/provider-chain rows and unblock default-facing packet promotion. | Warning-string parsing, duplicated diagnostics, or missing required plugin not fatal. |
| P4 V4 subsystem evidence | Plugin/runtime/profile/export tests, fixture coverage, edge cases, optional/default split, and profile membership. | May promote S3-S7 subsystem rows or S8 optional-advanced rows inside their classification. | Optional advanced feature used as Bevy baseline evidence, missing fixture/negative case, or owner-session evidence absent. |
| P5 V5 product surface evidence | Export/editor/Hub surfaces consuming the same report chain with maturity, target support, blocked reason, and profile membership visible. | May promote productization rows; cannot upgrade runtime rows alone. | Separate UI availability model, warning parsing, hidden blocked reason, or presentation-only evidence. |
| P6 V6 release/CI evidence | Workspace/plugin/export/docs-sync or CI evidence with exact job, date, command scope, and failure count. | May mark release acceptance for rows whose highest required rung is satisfied. | Docs/profile/catalog/CI mismatch or missing current command evidence. |

Promotion notes must name the highest required rung for the row, not just the rung reached in the current session. When current evidence is lower than required, record `Partial`, `Deferred`, or `Unpromoted` and name the next lower-layer fix.

## Current Packet Status Snapshot

This snapshot mirrors `Current Packet Status Ledger` in `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`. It is a roadmap status record, not fresh runtime validation.

| Packet | Matrix status today | Next evidence before promotion |
|---|---|---|
| W0.1 catalog freeze | `Partial`, V0 source/documentation classification. | Inventory proof across Bevy default/minimal/dev capabilities, `RuntimePluginId`, first-party packages, matrix rows, and owner paths. |
| W1.1 profile contract refresh | `Partial`, V0/V1 profile planning. | V2 profile/plugin-group tests for ids, membership, target modes, maturity floors, and externalized/stub policy. |
| W1.2 core foundation prelude gates | `Partial`, V0 app/core review. | V2 focused evidence for app/runtime preludes, plugin groups, state, time, task pools, diagnostics, and dev/default log split. |
| W2.1 availability report contract | `Partial` until current V3 evidence is recorded. | App/profile/export integration proving structured availability categories are consumed instead of warning strings. |
| W2.2 provider chain negative cases | `Partial` until current V3 negative evidence is recorded. | Negative cases for linked/native/externalized/blocked/stub/optional-missing/fatal-required-missing behavior. |
| S3 animation | V0 source review; not default-profile promoted. | V4 plugin/profile evidence for clip, target id, graph, transition, event, fixture, and scene-stage behavior. |
| S4 sound | V0 source review and active owner work; proof candidate only. | V4 playback/import/output/cleanup/profile evidence after W2 closure and sound owner handoff. |
| S5 sprite2d | V0 source review; dedicated identity required. | V4 evidence for `RuntimePluginId::Sprite2d`, package/runtime owner, schema, atlas/slice readiness, render queueing, and profile gate. |
| S6 UI navigation | V0 source review; gameplay navigation split accepted. | V4 UI focus/navigation/a11y/input evidence with hidden/disabled/modal/group negative cases. |
| S7 remote/net | V0 source review; remote separated from gameplay networking. | V4 dev-protocol evidence for method registry/schema, BRP envelope, transport opt-in, and profile membership. |
| S8 particles/VFX | V0 source review; optional advanced classification. | V4 optional-advanced evidence only; no default-profile promotion effect. |
| S9 export/editor UX | V0 source review; productization gate. | V5 evidence that export/editor/Hub surfaces consume the shared profile/export/availability/status chain. |
| W10.1 docs-sync/CI | V0 design gate with active docs/CI owner. | V1/V2 scripted sync evidence, then V6 workspace/plugin/export/docs/CI evidence for release acceptance. |

Rows in this snapshot must not be converted to `Complete` by editing text alone. A promotion requires a new validation entry with current command evidence at the row's required rung.

## W10.1 Docs-Sync Static Check Contract

W10.1 defines the static contract that a future docs-sync checker must enforce before M10 can become a real CI gate. This section is design evidence only until a script, validator lane, or CI job runs it.

| Contract area | Required comparison | Acceptable V0/V1 result | Promotion blocker |
|---|---|---|---|
| Runtime id coverage | Compare `RuntimePluginId` and built-in descriptor keys against the RuntimePluginId Coverage Matrix. | Every id has one matrix row with maturity, capability status, profile role, Bevy decision, and next gate. | Missing/duplicate id row, stale package key, or id represented only by a human-readable diagnostic. |
| First-party package coverage | Compare `zircon_plugins/*/plugin.toml` packages against runtime-id rows and package-only/tooling/importer classifications. | Each package is mapped to a runtime id, package-only note, tooling/editor/importer note, or owned feature bundle. | Unclassified package, stale maturity/capability row, or package counted as the wrong default capability. |
| Profile membership docs | Compare `RuntimeProfileDescriptor` required/default/optional membership against `profile-selection.md` and matrix profile rows. | Every profile documents target mode, required capabilities, optional plugins, maturity floor, externalized/stub policy, and target restrictions. | Required `Externalized`/`Stub` or missing-provider state accepted without fatal behavior, or optional failure lacking structured diagnostics. |
| Provider/export categories | Compare `RuntimePluginAvailabilityReport`, `RuntimeModuleLoadReport`, and `ExportBuildPlan` categories against M2/M9/M10 docs. | Linked, native dynamic, externalized missing, stub, blocked, missing required, and optional warning paths are documented as structured report data. | UI/export/Hub row parses warning strings, hides fatal required-provider state, or uses presentation data as source of truth. |
| Evidence ledger fields | Compare validation notes and packet status rows against the Evidence Record Requirements and Packet Promotion Evidence Checklist. | Every row-status change names packet id, validation rung, exact static check or command, platform when relevant, pass/fail count or failure class, and remaining gap. | Docs-only text marks runtime behavior complete, stale logs are reused, or the first failing lower layer is missing. |
| CI ladder shape | Compare M10 candidate commands, visible Zircon CI, and `validate-matrix.ps1` against Bevy's layered CI precedent. | Docs/static, focused profile/export, app/provider, plugin workspace, export-platform, workspace build/test, and validator/CI gates remain named. | Release acceptance omits a gate class or claims cross-platform/export readiness from desktop-only evidence. |

Future checker shape:
- A V1 checker may be static-only and parse Markdown/TOML/source lists without compiling Rust.
- A V2 checker may combine static docs-sync output with focused profile/catalog tests when metadata changes.
- A V6 release gate requires docs-sync evidence plus workspace, plugin workspace, export-platform, and validator or CI evidence.

This contract does not validate runtime behavior, linked provider execution, editor/Hub presentation, or full workspace health by itself. It only prevents the docs, profile/catalog metadata, and release-evidence ledger from drifting apart.

## Owner Handoff Acceptance

This matrix mirrors the `Owner Handoff Matrix` in `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`. It defines when evidence from another owner session is acceptable for changing this parity matrix.

| Owner lane | Acceptable handoff evidence | Matrix action |
|---|---|---|
| Foundation app/prelude/state/time/tasks/log/diagnostics | Bevy source anchors, Zircon owner paths, V2 focused app/runtime commands, pass/fail count, and profile-selection update. | May promote foundation rows only inside app/core scope. |
| Profile/provider/report foundation | V2/V3 profile/provider command evidence, structured availability buckets, app/export consumers, and no warning-string parsing. | May unblock W2 and default-facing packet promotion. |
| Animation | V4 animation runtime/profile evidence covering clip, target id, graph, transition, events, fixture, and scene stage order. | May update S3 rows; cannot promote default profile without W0-W2 accepted. |
| Sound | Active sound session handoff with V4 playback/import/output/cleanup/profile evidence. | May update S4 rows after provider-chain closure; optional advanced sound remains separate. |
| Sprite/render/asset | Coordinated render/asset handoff with dedicated sprite2d identity and V4 render/profile evidence. | May update S5 rows; generic render/texture evidence remains insufficient. |
| UI navigation/input/a11y | Active UI/Input owner handoff with V4 focus/tab/directional/a11y/input routing evidence and negative cases. | May update S6 rows; gameplay navigation remains separate. |
| Remote/net | V4 dev-protocol evidence for method registry/schema, BRP envelope, transport opt-in, and dev/editor/server profile membership. | May update S7 rows; gameplay networking features stay optional advanced. |
| Particles/VFX | V4 optional-advanced evidence for VFX runtime/editor claims. | May update S8 optional rows; no default-profile effect. |
| Export/editor/Hub | Active product-surface handoff proving export/editor/Hub consumes the shared report chain. | May update S9 productization rows only. |
| Docs/CI | V1/V2 docs-sync script evidence or V6 workspace/plugin/export/docs/CI evidence with exact job/command scope. | May update M10 or release acceptance rows. |

Handoff rejection rule: if a handoff lacks command scope, date, platform, pass/fail count, Bevy anchors, Zircon owner paths, or the first failing lower layer, this matrix may record it as context but must not use it for promotion.

## M0 Acceptance Gates

M0 is accepted when the following static checks are true:

- Every Bevy `DefaultPlugins` family has a Zircon row in this document.
- Every current first-party `RuntimePluginId` has a maturity/capability decision, including the built-in-domain `Ui` variant that does not have an external package descriptor.
- Every `zircon_plugins/*/plugin.toml` package that does not map to `RuntimePluginId` is explicitly classified as package/tooling/editor coverage rather than default profile closure.
- Every app/core foundation row (`PluginGroup`, prelude, state, time, tasks, log, diagnostics, dev profile) has a Bevy source reference and a Zircon owner path.
- Every default-profile required capability has either an accepted implementation status or a named follow-up milestone.
- Every intentional divergence is written down: gameplay navigation is not UI navigation, particles are not a Bevy default blocker, and advanced render providers cannot replace default render product evidence.
- No subsystem is promoted from `Partial` to `Complete` by this document alone.

## Validation

The profile/maturity unit tests cover manifest serde roundtrip, descriptor-to-manifest projection, built-in catalog classification, profile ids, deterministic profile manifests, required-plugin availability gates, optional-plugin warning buckets, and provider-aware linked/native registration buckets.

Review-fix coverage now also pins exact profile-manifest loading for `minimal`, provider reports not bypassing `Stub` or below-minimum maturity gates, and built-in importer/physics capability statuses matching the persisted plugin TOML capability metadata.

Latest attempted validation in this session:

- 2026-05-29 Sound HRTF boundary continuation: focused Sound runtime validation replaced the flat loaded-HRTF runtime helper with folder-backed `zircon_plugins/sound/runtime/src/engine/hrtf/` modules for render-state keys, FIR tail state, loaded-profile convolution, and stale-state pruning. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary Sound evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound occlusion boundary continuation: focused Sound runtime validation replaced the flat occlusion helper with folder-backed `zircon_plugins/sound/runtime/src/engine/occlusion/` modules for query DTOs, fallback gain constants, the render-facing gain entry point, and ray-traced descriptor specificity matching. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a single rustfmt import-line adjustment. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary Sound evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound source descriptor validation boundary continuation: focused Sound runtime validation replaced the former flat source descriptor helper with folder-backed `zircon_plugins/sound/runtime/src/descriptor_validation/source/` modules for graph-track references, clip ranges, inputs, parameter bindings, spatial settings, and source-local scalar/time guards. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary` after tightening an unnecessary internal re-export. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary Sound evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound filter boundary continuation: focused Sound runtime validation replaced the flat filter helper with folder-backed `zircon_plugins/sound/runtime/src/engine/filter/` modules for state, block application, coefficients, shelf formulas, and constants. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary` after removing an internal private coefficient re-export. Direct execution of the generated Sound runtime test binary passed `dsp_state` with 14 tests and 0 failures, and passed the full Sound runtime binary with 97 tests and 0 failures. A later standard Cargo test rerun was blocked before Sound tests by active Texture work in `zircon_runtime/src/asset/assets/texture/upload_support.rs:611`; this records module-boundary Sound evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound DSP boundary continuation: focused Sound runtime validation passed after replacing the flat DSP executor with folder-backed `zircon_plugins/sound/runtime/src/engine/dsp/` modules. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` passed with 14 DSP tests, 0 failures, and 83 filtered. The full Sound runtime command `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml --locked --offline --jobs 1 --message-format short --color never` passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound DSP effect-chain boundary continuation: focused Sound runtime validation replaced `zircon_plugins/sound/runtime/src/engine/dsp/effects.rs` with folder-backed `zircon_plugins/sound/runtime/src/engine/dsp/effects/` modules for chain orchestration, effect-kind dispatch, and sidechain lookup. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after rustfmt import ordering. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-dsp-effect-chain-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` passed with 14 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound engine-validation boundary continuation: focused Sound runtime validation passed after replacing the flat engine validation helper with folder-backed `zircon_plugins/sound/runtime/src/engine/validation/` modules. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` passed with 14 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound render-source boundary continuation: focused Sound runtime validation passed after replacing the flat render-source helper with folder-backed `zircon_plugins/sound/runtime/src/engine/render/source/` modules for orchestration, source input mixing, external provider blocks, parameter bindings, and clip ranges. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-render-source-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. Earlier source-input attempts timed out during parallel workspace compilation before Sound diagnostics; the accepted evidence is the isolated serial rerun. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound source-environment apply boundary continuation: focused Sound runtime validation moved `apply_source_environment` orchestration from the source-environment root module into `zircon_plugins/sound/runtime/src/engine/source_environment/apply.rs`, leaving `mod.rs` structural. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound source-environment HRTF boundary continuation: focused Sound runtime validation replaced the flat source-environment HRTF helper with folder-backed `zircon_plugins/sound/runtime/src/engine/source_environment/hrtf/` modules for loaded-profile dispatch, preview fallback, and tail queries. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary` after narrowing moved entry visibility to the source-environment boundary. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound source-environment spatial boundary continuation: focused Sound runtime validation replaced the flat source-environment spatial helper with folder-backed `zircon_plugins/sound/runtime/src/engine/source_environment/spatial/` modules for profile composition, attenuation, cone gain, Doppler preview gain, and source pan. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary` after narrowing moved entry visibility to the source-environment boundary. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound source-environment volume boundary continuation: focused Sound runtime validation replaced the flat source-environment volume helper with folder-backed `zircon_plugins/sound/runtime/src/engine/source_environment/volume/` modules for AudioVolume influence selection, sphere/box shape and crossfade weight, and low-pass filter behavior. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-filter-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml ray_tracing --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound automation target apply boundary continuation: focused Sound runtime validation passed after moving automation target dispatch into `zircon_plugins/sound/runtime/src/automation/target/apply.rs`, leaving `automation/target/mod.rs` structural. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. After an earlier cold check timed out after 10 minutes before diagnostics, `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-automation-target-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound automation effect target boundary continuation: focused Sound runtime validation replaced `zircon_plugins/sound/runtime/src/automation/target/effect.rs` with folder-backed `zircon_plugins/sound/runtime/src/automation/target/effect/` modules for effect-kind dispatch, common enabled/bypass/wet parameters, and per-effect gain/filter/reverb/dynamics/modulation/delay/shaper/stereo parameter mapping. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after applying rustfmt to the new files. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` first exposed that the new effect target entry was too private for sibling `target/apply.rs`; after narrowing visibility to `crate::automation::target`, the accepted rerun passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-automation-effect-target-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_binding --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml automation_curve --locked --offline --jobs 1 --message-format short --color never` passed with 5 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound service-types root boundary continuation: focused Sound runtime validation passed after converting `zircon_plugins/sound/runtime/src/service_types.rs` into folder-backed `zircon_plugins/sound/runtime/src/service_types/mod.rs` and moving concrete manager state into `service_types/manager_state.rs`. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-types-root-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml runtime_core --locked --offline --jobs 1 --message-format short --color never` passed with 3 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound service mixer-graph boundary continuation: focused Sound runtime validation replaced `zircon_plugins/sound/runtime/src/service_types/mixer_graph.rs` with folder-backed `zircon_plugins/sound/runtime/src/service_types/mixer_graph/` modules for full graph import, snapshot, track CRUD, send CRUD, and effect CRUD. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a rustfmt import adjustment. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` first exposed that the moved service methods were too private for sibling `manager_trait.rs`; after narrowing visibility to `crate::service_types`, the accepted rerun passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-mixer-graph-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound render-source orchestration boundary continuation: focused Sound runtime validation moved source buffer orchestration, source-environment delegation, sends, and finish reporting out of `zircon_plugins/sound/runtime/src/engine/render/source/mod.rs` and into `zircon_plugins/sound/runtime/src/engine/render/source/orchestration.rs`, leaving the source root structural. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` first exposed that the moved `mix_sources` method was too private for the render root; after narrowing visibility to `crate::engine::render`, the accepted rerun passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-source-orchestration` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml spatial --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound render root orchestration boundary continuation: focused Sound runtime validation moved block-level `render_mix` orchestration, graph validation, track buffer flow, sidechain taps, DSP/meter application, and master gain clamping out of `zircon_plugins/sound/runtime/src/engine/render/mod.rs` and into `zircon_plugins/sound/runtime/src/engine/render/orchestration.rs`, leaving the render root structural. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-root-orchestration` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` passed with 12 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml mixer_graph --locked --offline --jobs 1 --message-format short --color never` passed with 8 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound render sampling boundary continuation: focused Sound runtime validation replaced the flat render sampling helper with folder-backed `zircon_plugins/sound/runtime/src/engine/render/sampling/` modules for resample step calculation, source cursor/range position, interpolation, and frame/channel folding. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-sampling-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound render playback boundary continuation: focused Sound runtime validation replaced the flat render playback helper with folder-backed `zircon_plugins/sound/runtime/src/engine/render/playback/` modules for active-playback routing, clip block sampling, pan/gain projection, and finished-playback event reporting. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed after a rustfmt import-order adjustment. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-render-playback-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml playback --locked --offline --jobs 1 --message-format short --color never` passed with 13 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml source_inputs --locked --offline --jobs 1 --message-format short --color never` passed with 4 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml render --locked --offline --jobs 1 --message-format short --color never` passed with 12 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures; remaining output was limited to existing `zircon_runtime` warnings and existing non-CPAL `ring_buffer` dead-code warnings. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound engine-state boundary continuation: focused Sound runtime validation passed after replacing the flat engine state helper with folder-backed `zircon_plugins/sound/runtime/src/engine/state/` modules for storage, graph mutation, snapshot projection, dynamic-event executor state, playback records, and source voice records. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-engine-state-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml graph_config --locked --offline --jobs 1 --message-format short --color never` passed with 2 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` passed with 10 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound output-lifecycle boundary continuation: focused Sound runtime validation passed after replacing the flat output lifecycle helper with folder-backed `zircon_plugins/sound/runtime/src/output/lifecycle/` modules for storage, config, start/stop dispatch, callback accounting, status projection, and backend-session state. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-output-lifecycle-boundary`. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` passed with 8 tests and 0 failures; `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --features cpal-backend --locked --jobs 1 --message-format short --color never` passed with 12 tests and 0 failures using `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-output-lifecycle-cpal`. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound service output-device boundary continuation: focused Sound runtime validation passed after replacing the flat service output-device helper with folder-backed `zircon_plugins/sound/runtime/src/service_types/output_device/` modules for backend status, catalog listing, descriptor configuration, start/stop lifecycle, and status projection. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-output-device-boundary` and existing `zircon_runtime` warnings only. The first `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml output_device --locked --offline --jobs 1 --message-format short --color never` attempt timed out while cargo/rustc was still compiling `zircon_runtime`; the warmed rerun passed with 8 output-device tests, 0 failures, and 89 filtered tests. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound service dynamic-events boundary continuation: focused Sound runtime validation passed after replacing the flat service dynamic-event helper with folder-backed `zircon_plugins/sound/runtime/src/service_types/dynamic_events/` modules for catalog snapshot/registration, handler registration, pending invocation queueing, and deterministic dispatch fan-out. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-events-boundary` and existing `zircon_runtime` warnings only. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` passed with 10 dynamic-event tests, 0 failures, and 87 filtered tests. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-30 Sound service dynamic-event executors boundary continuation: focused Sound runtime validation passed after replacing the flat service dynamic-event executor helper with folder-backed `zircon_plugins/sound/runtime/src/service_types/dynamic_event_executors/` modules for executor registration, unregistration, and execution report assembly. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=D:\cargo-targets\zircon-sound-service-dynamic-event-executors-boundary` and existing `zircon_runtime` warnings only. The first `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dynamic_events --locked --offline --jobs 1 --message-format short --color never` attempt stopped during cold compilation before the Sound test binary ran; process inspection showed an unrelated editor cargo job active in a separate target directory. The warmed rerun passed with 11 dynamic-event tests, 0 failures, and 87 filtered tests. The full Sound runtime command passed with 98 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-29 Sound DSP-state boundary continuation: focused Sound runtime validation passed after replacing the flat DSP state helper with folder-backed `zircon_plugins/sound/runtime/src/engine/dsp_state/` modules for effect keys, effect runtime fields, track runtime fields, delay-line state, and cross-block history buffers. `cargo fmt --manifest-path zircon_plugins/sound/runtime/Cargo.toml -- --check` passed. `cargo check --manifest-path zircon_plugins/sound/runtime/Cargo.toml --tests --locked --offline --jobs 1 --message-format short --color never` passed with `CARGO_TARGET_DIR=E:\cargo-targets\zircon-sound-dsp-state-boundary` after an initial private-type visibility error was corrected inside the same boundary. `cargo test --manifest-path zircon_plugins/sound/runtime/Cargo.toml dsp_state --locked --offline --jobs 1 --message-format short --color never` passed with 14 tests and 0 failures. The full Sound runtime command passed with 97 runtime tests, 0 failures, and doctests with no failures. This records module-boundary evidence only and does not promote the default-profile Sound row by itself.
- 2026-05-25 owner-handoff-matrix continuation: V0 docs-only. No Cargo command was run. Static planning update added `Owner Handoff Matrix` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `Owner Handoff Acceptance` to this matrix. This pass records cross-session owner lanes, Bevy anchors, Zircon source-of-truth paths, acceptable handoff artifacts, and rejection rules; handoffs without current command scope and pass/fail evidence remain context only.
- 2026-05-25 packet-status-ledger continuation: V0 docs-only. No Cargo command was run. Static planning update added `Current Packet Status Ledger` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `Current Packet Status Snapshot` to this matrix. This pass records conservative current status and next evidence for W0.1/W1.1/W1.2/W2.1/W2.2/S3/S4/S5/S6/S7/S8/S9/W10.1; no row is promoted by this docs-only snapshot.
- 2026-05-25 packet-promotion-runbook continuation: V0 docs-only. No Cargo command was run. Static planning update added `Packet Promotion Runbook` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `Packet Promotion Evidence Checklist` to this matrix. This pass records P0-P6 evidence stages, allowed matrix changes, downgrade/rejection triggers, and required promotion fields so future W/S packet claims cannot use docs-only analysis, UI presentation, or optional advanced behavior as runtime/default completion evidence.
- 2026-05-25 default-subsystem-packet continuation: V0 docs-only. No Cargo command was run. Static planning update added `Default-Facing Subsystem Packet Board` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `Default Subsystem Packet Acceptance` to this matrix. This pass records S3 animation, S4 sound, S5 sprite2d, S6 UI navigation, S7 remote/net, S8 particles, and S9 export/editor UX as second-wave packets whose promotion depends on accepted W0-W2 foundation evidence and owner-session validation; particles remain optional advanced and do not block Bevy default parity.
- 2026-05-25 foundation-packet continuation: V0 docs-only. No Cargo command was run. Static planning update added `Foundation Execution Packets` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `Foundation Packet Evidence Map` to this matrix. This pass records W0.1/W1.1/W1.2/W2.1/W2.2 packet inputs, candidate validation commands, docs owners, packet sequencing, and stop conditions; runtime/profile/export promotion remains blocked until the named V2/V3 command evidence exists.
- 2026-05-25 first-wave work-order continuation: V0 docs-only. No Cargo command was run. Static planning update added `First Execution Wave Work-Order Queue` to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and `First Wave Work-Order Acceptance` to this matrix. This pass records W0.1/W1.1/W1.2/W2.1/W2.2 as serial foundation blockers, W4.1 sound as the first default-facing proof candidate after provider-chain closure, and W10.1 docs-sync as a coordinated design gate rather than current CI evidence.
- 2026-05-25 M10 docs/CI static gate continuation: V0 docs-only. No Cargo command was run. Static review rechecked the existing M10 Docs / CI Gates matrix and the main milestone plan against Bevy `tools/ci` command expansion, Bevy `doc_check` / `example_check` command shapes, and Zircon `.github/workflows/ci.yml` workspace/plugin/export gates. `git diff --check -- docs/runtime-plugins/bevy-parity-matrix.md docs/runtime-plugins/profile-selection.md ".codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md"` passed with CRLF warnings only. A strict unresolved-marker scan found no matches in the runtime-plugin docs and no matches in this milestone plan; matches reported elsewhere under `.codex/plans` were unrelated editor/runtime-operation plans.
- 2026-05-25 handoff-template continuation: V0 docs-only. No Cargo command was run. Static planning update added executable handoff and promotion submission rules to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`, and added evidence-record requirements to this matrix. This pass records the required fields for future work orders and validation notes: milestone identity, Bevy anchors, Zircon owner paths, lower gates, implementation package, validation package, docs package, stop condition, promotion decision, and remaining gap.
- 2026-05-25 critical-path continuation: V0 docs-only. No Cargo command was run. Static planning update added critical-path and parallel-lane sequencing plus risk/decision records to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`, and added critical-path rejection rules to this matrix. This pass records M0-M2 as the serial foundation path, sound as the first provider/profile/export proof candidate, optional advanced lanes as non-default blockers, and rejection rules for warning-string parsing, gameplay-navigation substitution, particles-as-default, advanced-render substitution, and UI-only status claims.
- 2026-05-25 implementation package continuation: V0 docs-only. No Cargo command was run. Static planning update added a M0-M10 implementation-package ledger to `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md` and a validation-package ladder to this matrix. This records how future work must classify evidence from docs-only through focused runtime tests, app/profile/export integration, plugin workspace validation, export platform matrix, and full workspace/validator/CI evidence before promoting a row.
- 2026-05-25 source traceability continuation: no Cargo command was run. Static source review covered Bevy `Cargo.toml` profiles, `cargo_features.md`, `DefaultPlugins`, `Plugin` lifecycle, `PluginGroupBuilder`, prelude, state/time/task/log/diagnostics plugins, animation/audio/sprite/input-focus/remote source anchors, CI workflow/tooling files, and the absence of a first-party particles crate under `dev/bevy/crates`. This pass added source-to-Zircon acceptance rules so future implementation slices must name the Bevy anchor, Zircon owner path, and validation artifact before promoting a row.
- 2026-05-25 M0 doc-only continuation: no Cargo command was run. Static review updated this document against `.codex/plans/ZirconEngine Bevy 级插件完成度里程碑计划.md`, Bevy `DefaultPlugins`, `PluginGroupBuilder`, feature docs, and current Zircon profile/plugin source paths. Acceptance remains documentation-level until the declared M0/G0 checks are scripted or manually reviewed against every catalog entry.
- 2026-05-25 M0 RuntimePluginId coverage continuation: no Cargo command was run. Static source review covered `RuntimePluginId`, `RuntimeProfileDescriptor`, `RuntimePluginDescriptor::builtin_catalog()`, persisted `zircon_plugins/*/plugin.toml` maturity/status rows, and Bevy `DefaultPlugins`/`PluginGroupBuilder` evidence. The resulting table now gives every enum variant either a catalog/package row or the explicit built-in-domain `Ui` treatment.
- 2026-05-25 M1 core foundation continuation: no Cargo command was run. Static source review covered Bevy `DefaultPlugins`, `MinimalPlugins`, `PluginGroupBuilder`, `TimePlugin`, `TaskPoolPlugin`, `StatesPlugin`, `LogPlugin`, `DiagnosticsPlugin`, `FrameTimeDiagnosticsPlugin`, and Zircon `zircon_app::plugins`, app/runtime preludes, `CoreRuntime` state/time/task/diagnostic APIs, and core module descriptors. The resulting table separates app/core foundation parity from runtime-plugin parity.
- 2026-05-25 M3 animation continuation: no Cargo command was run. Static source review covered Bevy `AnimationPlugin`, `AnimationClip`, `AnimationTargetId`, `AnimationGraph`, `AnimationTransitions`, `AnimationEventTrigger`, Bevy event tests, and Zircon `zircon_plugins/animation/runtime` manager/scene-hook/sequence/clip-event source plus `runtime_physics_animation_tick_contract.rs`. The resulting M3 matrix distinguishes existing runtime coverage from the remaining profile-promotion gates.
- 2026-05-25 M4 audio continuation: no Cargo command was run. Static source review covered Bevy `AudioPlugin`, `PlaybackSettings`, `PlaybackMode`, `AudioSinkPlayback`, `GlobalVolume`, `Volume`, `AudioSource`, `AudioLoader`, `Decodable`, `AudioOutput`, `cleanup_finished_audio`, and spatial emitter/listener update systems, plus Zircon `SoundManager`, `SoundPlaybackSettings`, `SoundSourceDescriptor`, `SoundOutputDeviceStatus`, `DefaultSoundManager`, output backend state, mixer render, HRTF/occlusion code, and sound runtime tests. The resulting M4 matrix separates default-profile audio gates from advanced sound features such as timeline automation, HRTF database work, and ray-traced convolution.
- 2026-05-25 M5 sprite continuation: no Cargo command was run. Static source review covered Bevy `SpritePlugin`, `Sprite`, `SpriteImageMode`, `Anchor`, `TextureSlicer`, `SpriteRenderPlugin`, `ExtractedSprite`, `ComputedTextureSlices`, `queue_sprites`, `prepare_sprite_image_bind_groups`, `Mesh2dRenderPlugin`, `ColorMaterialPlugin`, and `SpriteMesh`, plus Zircon `RenderSpriteSnapshot`, `Sprite2dComponent`, `SpriteExtract`, phase queueing, simple sprite renderer, and `render_product_sprite` tests. The resulting M5 matrix records that Zircon has sprite render infrastructure but still lacks a dedicated first-party `sprite_2d` plugin identity and profile gate.
- 2026-05-25 M6 navigation continuation: no Cargo command was run. Static source review covered Bevy `InputFocus`, `InputFocusVisible`, `FocusedInput`, `InputFocusPlugin`, `InputDispatchPlugin`, `TabIndex`, `TabGroup`, `TabNavigation`, `DirectionalNavigationMap`, `DirectionalNavigation`, `AutoDirectionalNavigation`, `AutoDirectionalNavigator`, `FocusPolicy`, and the directional navigation example, plus Zircon `UiFocusState`, `UiFocusVisible`, `UiFocusedInput`, `UiNavigationContract`, `UiTabIndex`, `UiNavigationGroup`, `UiDirectionalNavigation`, `UiNavigationDispatcher`, `UiSurface` focus routing, focused input dispatch, `focus_navigation` / `widget_range_navigation` tests, and gameplay `navigation` plugin/runtime manager evidence. The resulting M6 matrix records that Bevy parity navigation is UI focus/navigation, while Zircon gameplay `navigation` remains optional beta/advanced.
- 2026-05-25 M7 net/remote continuation: no Cargo command was run. Static source review covered Bevy `RemotePlugin`, `RemoteMethods`, custom method registration, BRP request/response/error envelope, built-in `world.*` / `registry.schema` / `schedule.*` / `rpc.discover` methods, `RemoteHttpPlugin`, OpenRPC/JSON schema export, and remote examples, plus Zircon `RuntimePluginId::Net`, `NetRuntimePlugin`, `NetManager`, HTTP/WebSocket route/listener/request APIs, base net tests, HTTP/WebSocket/RPC/replication/reliable UDP/content-download feature manifests and tests, and editor/dev/server profile membership. The resulting M7 matrix records that Bevy parity is a feature-gated developer remote protocol with explicit transport opt-in, while Zircon game networking features remain optional advanced lanes.
- 2026-05-25 M8 particles/VFX continuation: no Cargo command was run. Static source review covered Bevy `DefaultPlugins`, `Cargo.toml` default/2D/3D/render/sprite/PBR feature groups, `docs/cargo_features.md`, the absence of a `bevy_particles` crate under `dev/bevy/crates`, TAA particle-motion-vector notes, core post-process scheduling, plus Zircon `particles` plugin TOML, runtime asset/component/service/simulation/render/GPU/interop files, editor authoring, runtime/editor tests, and profile/provider evidence. The resulting M8 matrix records particles as advanced optional VFX, not a Bevy default parity blocker.
- 2026-05-25 M9 export/editor UX continuation: no Cargo command was run. Static source review covered Bevy Cargo feature profiles, `DefaultPlugins` / `MinimalPlugins` feature control, `Plugin` / `PluginGroupBuilder` edit semantics, no-default/no-renderer/plugin-group/plugin examples, public plugin docs, plus Zircon `ExportProfile`, `ProjectPluginManifest`, `ExportBuildPlan`, `RuntimePluginAvailabilityReport`, app provider registration, editor plugin status/export build manager, pane payloads, and Hub plugin catalog evidence. The resulting M9 matrix records export/editor UX as a structured-report productization gate, not a Bevy editor clone.
- 2026-05-25 M10 docs/CI continuation: no Cargo command was run. Static source review covered Bevy CI workflow jobs, Bevy `tools/ci` command expansion, docs deployment, missing feature/example docs checks, dependency/security workflows, example-run workflow, CI failure comments, plus Zircon `.github/workflows/ci.yml`, `validate-matrix.ps1`, validation skill docs, reporting rules, and existing runtime-plugin docs. The resulting M10 matrix records docs/CI as a layered acceptance gate: docs ownership, profile/catalog synchronization, workspace/plugin/export validation, and evidence recording.

- `cargo test -p zircon_runtime --lib plugin_extensions::profile_maturity --locked -- --nocapture` passed: 8 profile/maturity tests, 0 failures.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed: 1 manifest/catalog matching test, 0 failures.

Latest M2 scoped validation:

- `cargo check -p zircon_runtime --lib --locked` passed with warning-only output.
- `rustfmt --edition 2021 --check <scoped profile/maturity files>` passed after formatting scoped files.
- `git diff --check -- <scoped profile/maturity files and docs>` passed with line-ending normalization warnings only.
- `cargo test -p zircon_runtime --lib plugin_extensions::manifest_contributions::builtin_runtime_catalog_entries_have_matching_plugin_manifests_and_workspace_members --locked -- --nocapture` passed: 1 manifest/catalog matching test, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" entry_config_can_select_headless_render_profile_bundle -- --nocapture --test-threads=1` passed: 1 test, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --features "plugin-ui,first-party-runtime-plugins" profile_bootstrap -- --nocapture --test-threads=1` passed: 15 tests, 0 failures.
- `CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 profile_bootstrap -- --nocapture --test-threads=1` passed: 13 tests, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" profile_bootstrap --message-format short -- --nocapture --test-threads=1` passed on Windows: 18 tests, 0 failures.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=C:\Users\HeJiahui\AppData\Local\Temp\opencode\zircon-profile-provider-target cargo test -p zircon_app --locked --offline --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed on Windows: 1 test, 0 failures. This relies on the root lockfile resolving `gpu-allocator v0.28.0` to `windows 0.62.2`, matching `wgpu-hal v29.0.3`, while leaving Slint/`zircon_hub`'s `accesskit_windows v0.30.0` on `windows 0.61.3`.
- `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/opencode/zircon-profile-provider-target cargo test -p zircon_app --locked --jobs 1 --no-default-features --features "plugin-ui,first-party-runtime-plugins,first-party-navigation-runtime-plugin" runtime_profile_bootstrap_can_link_navigation_when_native_provider_feature_is_enabled --message-format short -- --nocapture --test-threads=1` passed in WSL/Linux: 1 test, 0 failures.
