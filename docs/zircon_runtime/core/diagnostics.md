---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/frame_diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/core/runtime/diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/camera.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/visibility.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/hzb.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light_grid.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/effect_stack.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/material.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/sprite.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/ui.rs
  - zircon_runtime/src/core/runtime/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/animation.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/frame_diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/core/runtime/diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/capability.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/history.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/anti_alias.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/virtual_geometry.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/hybrid_gi.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/advanced_provider.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/solari.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/camera.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/visibility.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/hzb.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light_grid.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/effect_stack.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/material.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/light.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/mesh_queue.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/gpu_scene.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/sprite.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product/ui.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/animation.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/history/copy_history_textures.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/graphics/types/viewport_frame.rs
  - zircon_runtime/src/graphics/types/viewport_render_output_target.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_transform.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/compiled_scene_outputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_runtime_outputs/store_last_runtime_outputs.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/diagnostics.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/state/runtime_inner.rs
  - zircon_runtime/src/animation/scene_hook/diagnostics.rs
  - zircon_runtime/src/dynamic_api/session/scene_asset_reload_diagnostics.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
plan_sources:
  - user: 2026-05-22 continue M10 render diagnostics and profiling bridge checklist
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render
  - user: 2026-06-17 bind HZB executor-owned external buffers for render plan 01
  - user: 2026-05-16 continue Bevy-style runtime Time diagnostics integration
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/zircon_runtime/graphics/render-product-submit.md
  - dev/bevy/crates/bevy_render/src/diagnostic/mod.rs
  - dev/bevy/crates/bevy_render/src/diagnostic/internal.rs
  - dev/bevy/docs/profiling.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/motion_vector.rs
  - zircon_runtime/src/tests/runtime_diagnostics/motion_vector.rs::runtime_diagnostics_reports_motion_vector_camera_and_mesh_draw_eligibility
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::history_copy_report_counts_copied_slots_from_slot_flags
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::camera_target_writeback_report_separates_copy_and_conversion_debug_markers
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_texture_conversion_writeback_marker
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_texture_direct_graph_import_readiness
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_camera_stack_suppressed_target_output
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_capture_source_report
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_gpu_scene_upload_stats
  - zircon_runtime/src/core/framework/render/capture.rs::tests::captured_frame_new_defaults_to_primary_framework_offscreen_source
  - zircon_runtime/src/core/framework/render/capture.rs::tests::texture_capture_report_distinguishes_direct_import_and_conversion_sources
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::graph_execution_coverage_report_preserves_neutral_counts
  - zircon_runtime/src/core/framework/render/backend_types.rs::tests::graph_stage_execution_report_preserves_neutral_counts
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/graphics/tests/render_profiling.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_transient_allocation_bytes
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_graph_execution_coverage
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_graph_stage_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_tracks_compute_dispatch_metadata
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_preserves_resource_binding_report
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_preserves_renderer_stage_metadata
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_counts_renderer_stage_order_violations
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_preserves_history_copy_report
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_creates_dense_transients_and_skips_sparse_reservations
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_fails_unbound_required_external_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs::tests::hzb_external_fallback_buffers_satisfy_materialization_report
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs::tests::hzb_occlusion_cull_declares_execution_owned_external_buffers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_oldest_entries_to_budget
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::graph_execution_coverage_report_counts_missing_unexpected_and_duplicate_passes
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_hits_on_identical_key
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_misses_on_feature_set_change
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_misses_on_viewport_resize
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_invalidates_on_pipeline_revision_bump
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_evicts_least_recently_used_entry
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_reports_lookup_status
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::headless_wgpu_server_falls_back_async_compute_passes_to_graphics
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked
  - cargo test -p zircon_runtime --lib execution_record_tracks_compute_dispatch_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_execute_output_target_writeback.rs::tests::output_target_writeback_report_maps_ready_and_blocked_plans
  - cargo test -p zircon_runtime --lib time --locked
  - cargo check -p zircon_runtime --profile profiling --features profiling --locked
  - cargo test -p zircon_runtime --lib core::runtime::tests:: --locked --jobs 1 --target-dir D:\cargo-targets\zircon-core-runtime-registry-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-menu-normalization-0605 --message-format short --color never (2026-06-05 devtools service snapshot registry-key name adaptation: passed with existing warnings)
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::render_framework_stats_report_scene_camera_ordering_metadata
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_surface_offscreen_submit_and_capture_survive_unbind_noop
  - zircon_runtime/src/graphics/tests/surface_targets.rs::graphics_camera_target_headless_size_controls_offscreen_capture_size
doc_type: module-detail
---

# Core Runtime Diagnostics

`zircon_runtime::core::diagnostics` remains the curated public facade for the read-only diagnostic snapshot surface. The physical owner is now `zircon_runtime::core::runtime::diagnostics`, because the store, devtools projection, profiling controls, and render-stat projection are runtime observability behavior. The store contracts already existed as plain data structures; the current Bevy-parity slice makes `CoreRuntime` own one `DiagnosticStore` so frame and system metrics can accumulate in the same runtime instance that owns lifecycle, time, task pools, and services.

## Reference Evidence

Bevy's `FrameTimeDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/frame_time_diagnostics_plugin.rs` registers `frame_time`, `fps`, and `frame_count` diagnostics from `Time<Real>` plus `FrameCount`. Bevy's `LogDiagnosticsPlugin` in `dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs` consumes the diagnostics store as a reporting layer rather than owning frame timing itself.

Zircon mirrors the ownership split: `CoreRuntime` records diagnostic measurements, while log/dev tooling can read snapshots later through `collect_runtime_diagnostics`.

## Ownership Boundary

- `DiagnosticStore` owns bounded series history, current values, smoothing, min/max, units, and subsystem tags.
- `CoreRuntimeInner` owns one `DiagnosticStore` per runtime instance.
- `CoreRuntime` and `CoreHandle` expose `record_diagnostic`, `diagnostic_store`, and `diagnostic_store_snapshot`.
- `CoreHandle::advance_time_by(...)` records Bevy-style time measurements after advancing runtime clocks.
- `collect_runtime_diagnostics` starts with the runtime-owned store and then overlays derived render, physics, animation, and profiling diagnostics.
- Runtime subsystem producers may write namespaced count rows through `CoreHandle::record_diagnostic(...)`; Runtime 07's animation scene hook uses this for `animation.scene.*` rows, and the dynamic session scene-asset reload integration uses this for `scene.asset_reload.*` rows, without making `core::diagnostics` own either subsystem's semantics.
- `collect_runtime_devtools_snapshot` projects runtime services and dependencies as copied values; service names come from the runtime registry map key, and service dependencies come from runtime-internal canonical registry names. Both remain copied strings in the tooling snapshot so the devtools DTO does not depend on the internal entry storage shape.
- `diagnostic_log::format_diagnostic_store_snapshot(...)` and `write_diagnostic_store_snapshot(...)` turn store snapshots into process-log lines for dev-profile diagnostics.

The diagnostics store is not a global singleton. This keeps tests, runtime preview sessions, editor-host runtimes, and future export hosts isolated from each other.

## Animation Scene Diagnostics

`AnimationSceneFrameDiagnostics` is owned by `animation/scene_hook/diagnostics.rs` and records count-only rows for `animation.scene.scanned_entities`, sequence/clip/graph/state-machine sampling, `animation.scene.output_poses`, `animation.scene.applied_transforms`, `animation.scene.published_events`, and `animation.scene.state_transitions`. These rows are Runtime 07 evidence inputs for animation scene-hook frame cost, not a new core diagnostics domain model. The current slice status is `animation_scene_frame_diagnostics_static_passed_cargo_deferred`; package Cargo validation stays with the broader Runtime 07 extract/ecs_query/profiling/FPS gates.

## Scene Asset Reload Diagnostics

`scene_asset_reload_diagnostics.rs` is owned by `dynamic_api/session/` because it projects the project session's reload-frame report, not a core diagnostics behavior. It records count rows for `scene.asset_reload.events_drained`, `scheduled`, `skipped`, skip reasons, `superseded_pending`, `applied`, `failed`, `stale`, and `pending`, plus a bool row for `scene.asset_reload.receiver_disconnected`. These rows let runtime diagnostics, logs, and editor tooling observe scene hot-reload activity without reading queue internals or treating reload failures as frame-aborting errors.

## Frame Diagnostics Contract

Runtime 15 F14 diagnostics normalization adds `FrameDiagnostics` and `FrameDiagnosticsStatus` under `core/runtime/diagnostics/frame_diagnostics.rs`. The trait is intentionally small: each frame diagnostics object reports a stable domain string, whether that subdomain is available for the current frame, and an optional error message. `RuntimeRenderDiagnostics`, `RuntimePhysicsDiagnostics`, and `RuntimeAnimationDiagnostics` implement it with the `render`, `physics`, and `animation` domains, while `RuntimeDiagnosticsSnapshot::frame_diagnostics_statuses()` returns those three status rows as a subdomain composition.

This contract does not replace `DiagnosticStore` paths. Count, bool, byte, and timing evidence still flows through `DiagnosticStore`; the frame status trait is the common naming/status layer that prevents parallel `*Diagnostics` wrappers from growing unrelated availability conventions. The ECS side also implements the same trait on `EcsFramePerformanceDiagnostics` with the `scene.ecs` domain, and `World` now stores that object directly instead of wrapping it in `WorldEcsFramePerformanceDiagnostics`.

Status: `runtime_15_diagnostics_frame_trait_wrapper_removed_coremin_check_passed`. Guards: `runtime_snapshot_frame_diagnostics_statuses_preserve_subdomains`, `ecs_frame_performance_diagnostics_uses_scene_ecs_frame_domain`, and `runtime_15_diagnostics_use_frame_trait_without_world_wrapper`.

## Render Diagnostics Bridge

M10.8 keeps render diagnostics on the same runtime-owned snapshot boundary. Bevy's `RenderDiagnosticsPlugin` records CPU/GPU pass elapsed time, pipeline statistics, and buffer-backed scalar diagnostics, then syncs finished rows into `DiagnosticsStore`. Zircon is not at that parity level yet: `RuntimeRenderDiagnostics` currently wraps a queried `RenderStats` snapshot, and `collect_runtime_diagnostics(...)` records submit/product counters into the store. The capability rows use `render.capability.*` bool/count paths for queue class count, surface/offscreen support, async queues, cache/storage/readback/indirect support, raytracing/resource-indexing capabilities, anti-alias feature support, max MSAA samples, VG/HGI backend gates, and the M8 `render.capability.neural_compute_supported` / `render.capability.sparse_texture_supported` slots. History rows use `render.history.*` paths for current/previous handle presence, previous-frame usability, aggregate invalidation, target/render dimensions (`target_width`, `target_height`, `render_width`, `render_height`), and one-hot invalidation reasons including `render_size_changed`; history-copy rows use `render.history.copy.*` for requested copy count, actual copied count, history target presence, history-copy debug marker emission, target extent, and slot-level copied booleans including `exposure_copied`. Camera scheduling rows use `render.camera.scheduled_count` and `render.camera.order_ambiguity_count` from the submitted `RenderViewExtract.scene_camera_order_report`; frames without that report produce zeroes instead of inventing renderer-private camera state. Camera target-resolution rows use `render.camera.target.*` bool/count paths for target family, primary viewport dimensions, resolved target dimensions, effective view dimensions, and dynamic-resolution-scaled render dimensions. Camera target graph-import rows use `render.camera.target.graph_import.*` for one-hot not-requested, pending, direct-import-ready, direct-imported, conversion-writeback-required, suppressed-by-camera-stack, or blocked status plus direct-import, conversion-writeback, blocked, width, and height counts from `RenderStats.last_camera_target_graph_import`; readiness rows keep direct-import count at zero, executed sRGB texture target imports publish `direct_imported` plus a nonzero direct-import count, and suppressed rows prove a texture target was withheld because the selected child was not the stack-terminal final-output owner. Camera target-writeback rows use `render.camera.target.writeback.*` for one-hot skipped, pending, ready, suppressed-by-camera-stack, skipped-direct-import, copied, converted, or blocked status plus copy/convert counts, texture-writeback debug marker emission, conversion debug marker emission, and target extent from `RenderStats.last_camera_target_writeback`; the skipped-direct-import row proves graph execution wrote the prepared target directly and no output-target copy/conversion was submitted. Capture-source rows use `render.capture.*` for one-hot none, framework-offscreen, texture direct graph import, texture conversion writeback, or texture copy writeback status plus output width and height from `RenderStats.last_capture_report`. These rows are execution/count evidence, not direct access to prepared WGPU texture handles. The render graph rows now include the legacy `render.last_graph_executed_pass_count` plus stable `render.graph.*` count paths for planned pass count, culled passes, queue fallbacks, resource lifetimes, sparse texture reservation lifetimes, planned resource accesses, planned dependencies, dense transient texture slots, sparse texture reservation slots, transient buffer slots, executed passes, executed resource accesses, executed dependencies, execution resource binding counts, execution coverage counts, and stage-summary counts under `render.graph.execution.*`, pass-level debug marker coverage, concrete compute dispatch count, aggregate compute dispatch group volume, compute storage-write resource count, and executed family counts for AA, VG, HGI, particles, transparent, and async-compute passes. The same graph bridge records `bytes` rows for dense transient texture/buffer reservations, total dense transient reservation pressure, and sparse virtual texture reservation footprint. Post-process graph rows use `render.post_process.graph.*` for node count, skipped node count, executed node count, and final composite presence; effect-stack rows remain under `render.post_process.effect_stack.*`; LUT renderer readiness rows now use `render.post_process.lut.request_count`, `ready_count`, and `fallback_count`. Motion-vector rows expose camera readiness and mesh draw-level previous/missing transform eligibility under `render.post_process.motion_vector.*` and `render.mesh.queue.*`; object-history match/miss diagnostic series were removed with the CPU viewport object-history path. GPUScene rows expose primitive/instance counts, dirty entry count, uploaded bytes, direct queue-write upload policy, allocator free spans, primitive/instance upload range counts, and now own the rolled previous-transform source used by temporal object velocity. Mesh queue indirect planning rows expose `render.mesh.queue.indirect_batch_count`, `indirect_batched_draw_count`, `indirect_fallback_draw_count`, and `indirect_args_count` as submit-side batch telemetry rather than GPU timing. Advanced-slot rows now mirror AA fallback state and PP-M4 normalization state under `render.anti_alias.*`, including `render.anti_alias.normalization.graph_sample_count`, `taa_msaa_conflict`, and `terminal_slot`; GPU particle counters under `render.particle.gpu.*`; VG counters/source/debug/readback rows under `render.virtual_geometry.*`; HGI probe/cache/scene/voxel rows under `render.hybrid_gi.*`; provider availability/report rows under `render.advanced_provider.*`; and Solari requested/status/degradation rows under `render.solari.*`.

That narrower bridge is still the correct consumer boundary. Runtime diagnostics panels, diagnostic log schedules, overlays, and editor tooling should consume `RuntimeDiagnosticsSnapshot` or `DiagnosticStoreSnapshot` instead of querying renderer-private state. The same bridge also mirrors material, light, mesh queue, sprite, UI, post-process effect-stack, and LUT texture fallback readiness rows. Mesh queue rows include shadow-caster and alpha-mask shadow-caster queue counts as submit-level preparation diagnostics; they do not claim pass timing or shadow-map GPU execution totals. Promotion beyond the current bridge still requires adding stable diagnostic paths for pass-level CPU timing, backend-gated GPU timing, pipeline/cache status, present/capture failures, render-asset residency, and mesh allocator memory. Profiling artifacts can support this evidence, but they do not replace store-backed diagnostics.

Runtime 07 `render_product_diagnostics_owner_split_static_passed_cargo_deferred` keeps the render product bridge folder-backed. `render_stats_store/product.rs` is now only the product-family dispatcher, while `render_stats_store/product/{camera,visibility,hzb,light_grid,effect_stack,material,light,mesh_queue,gpu_scene,sprite,ui}.rs` own concrete diagnostic projections; the status-table shorthand is `render_stats_store/product/{camera,mesh_queue,gpu_scene}.rs`. The guard `runtime_07_render_product_diagnostics_owner_split_keeps_families_folder_backed` records this as a structure slice with `expected_source_file_count = 38`, `hotspot_guard_anchor_count = 25`, `doc_anchor_count = 29`, `large_file_hotspot_count = 39`, and `runtime-other = 15`; package-level Runtime 07 Cargo validation remains deferred with the broader extract/ecs_query/profiling/FPS gates.

The 2026-06-17 editor UI popup-row adornment validation exposed a product-family import boundary issue before the editor crate was checked. Because concrete product diagnostic modules live under `render_stats_store/product/`, their `super::record_*` helper imports resolve through `product.rs`; that dispatcher now imports `record_bool`, `record_bytes`, and `record_count` from `render_stats_store.rs` so child modules can continue using the local parent as the shared helper owner. `cargo fmt -p zircon_runtime -p zircon_editor`, `cargo fmt -p zircon_runtime --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed after the support fix, with only existing warning noise.

Motion-vector object diagnostics are draw-eligibility evidence, not a public draw list. GPUScene records the shader-visible previous transform by rolling current instance transforms after successful submissions; `build_mesh_draws` then marks motion-vector-eligible mesh draws as having or missing previous object transforms. `DiagnosticStore` mirrors those aggregate counts as `render.mesh.queue.{previous,missing}_motion_vector_transform_draw_count`, so tooling can distinguish first-frame/new-object gaps from a broken graph executor without accessing WGPU buffers or per-draw renderer internals.

RenderDoc markers are explicitly debugging evidence, not profiling evidence. Bevy's profiling docs route GPU performance investigation through Tracy RenderQueue or vendor profilers, while RenderDoc remains a capture/debug tool. Zircon records `render.graph.debug_marker_count` from `RenderStats.last_graph_executed_debug_markers` only to prove graph pass marker coverage; it is not a timing metric and should stay separate from future GPU timestamp or pipeline-statistics rows.

Compute dispatch diagnostics are likewise execution evidence, not timing or backend object exposure. `RenderGraphExecutionRecord` collects `RenderGraphComputeDispatchRecord` rows from the graph GPU context after SSAO or clustered-lighting executors launch their WGPU compute pass bodies. Runtime diagnostics only mirror three numeric aggregates: `render.graph.compute_dispatch_count`, `render.graph.compute_dispatch_group_count`, and `render.graph.compute_storage_write_resource_count`. This lets tools see whether graph-declared compute work actually ran, including graphics-queue fallback cases, while the framework surface remains neutral.

Sparse texture diagnostics are resource-reservation evidence, not proof of a sparse residency implementation. `CompiledRenderGraphStats.sparse_texture_lifetime_count` and `CompiledRenderGraphTransientAllocationPlan.sparse_texture_slot_count` flow through `RenderStats.last_graph_sparse_texture_lifetime_count` / `last_graph_sparse_texture_slot_count` and into `DiagnosticStore` as `render.graph.sparse_texture_lifetime_count` / `render.graph.sparse_texture_slot_count`. These rows show that graph validation preserved sparse virtual texture reservations and kept them out of dense transient aliasing; page tables, tile uploads, residency eviction, and WGPU sparse objects remain future renderer/provider work.

Transient allocation byte diagnostics are planning evidence, not allocator ownership evidence. `CompiledRenderGraphTransientAllocationPlan` derives byte totals from RHI-neutral buffer sizes and texture descriptors, then `update_base_stats(...)` copies them into `RenderStats.last_graph_transient_texture_bytes_reserved`, `last_graph_transient_buffer_bytes_reserved`, `last_graph_transient_dense_bytes_reserved`, and `last_graph_sparse_texture_virtual_bytes`. `DiagnosticStore` mirrors those as `render.graph.transient_texture_bytes_reserved`, `render.graph.transient_buffer_bytes_reserved`, `render.graph.transient_dense_bytes_reserved`, and `render.graph.sparse_texture_virtual_bytes` with unit `bytes`.

Compiled graph cache diagnostics are submission compile evidence, not GPU execution evidence. `CompiledGraphCache` lives on `RenderFrameworkState`; `compile_submission_pipeline(...)` records hit, miss, eviction, and live-entry counts while reusing `Arc<CompiledRenderPipeline>` for stable pipeline revision/options/capability/frame fingerprints. The cache lookup also reports per-call hit/miss status internally so debug builds can re-check `extract_compile_fingerprint(...)` on hit without adding another diagnostic row. `update_base_stats(...)` copies the aggregate counters into `RenderStats.last_graph_compiled_cache_*`, and `DiagnosticStore` mirrors them as `render.graph.compiled_cache.hit_count`, `miss_count`, `eviction_count`, and `entry_count`.

Graph execution resource diagnostics are binding-count evidence, not renderer object exposure. `RenderGraphExecutionResources::resource_report()` counts texture views, external texture views, renderer-owned dense transient textures, buffers, and total bound resources after frame target import, optional history import, and transient materialization. `RenderGraphExecutionRecord` carries that neutral `RenderGraphExecutionResourceReport` into `RenderStats.last_graph_execution_resource_report`, and `DiagnosticStore` mirrors it as `render.graph.execution.texture_view_count`, `external_texture_view_count`, `owned_texture_count`, `buffer_count`, and `bound_resource_count`. These rows prove the frame registry had concrete bindings available for graph executors; they do not expose WGPU handles, allocator residency, sparse page tables, or pass timing.

Graph materialization diagnostics are live-lifetime completeness evidence. `RenderGraphExecutionResources::validate_materialized_graph_resources(...)` compares compiled graph lifetimes against the execution resource table after transient materialization and before executor dispatch. Typed texture and buffer misses are hard execution errors; sparse texture reservations are counted without dense backing; external lifetimes are counted as required/bound/missing report rows and carry a `RenderGraphExternalResourceBinding` that decides whether a miss is report-only or fatal. Required external misses fail the submit before diagnostics publish a successful-frame row, while report-only externals continue to expose imported frame-target/history evidence without making those resources mandatory. The same audit rejects logical texture or buffer bindings already present in the execution table when their names are absent from the compiled live lifetime set, so culled frame/history/plugin/fallback resources cannot survive as stale pre-bound rows. HZB occlusion now binds its required executor-owned external buffers before this audit, so its phase-local indirect/compaction/replay/stats names can contribute bound external evidence instead of only missing rows. `RenderStats.last_graph_materialization_report` mirrors required, bound, missing, missing typed, texture, buffer, external, stale binding, and sparse reservation counts under `render.graph.materialization.*` when validation succeeds.

Graph execution alias diagnostics are logical-to-physical binding evidence, not WGPU object exposure. `RenderGraphExecutionResources::resource_alias_report()` snapshots owned-backed graph resources before the frame returns them to `TransientResourcePool`; `RenderGraphExecutionRecord` stores the full `RenderGraphExecutionAliasReport`, and `RenderStats.last_graph_execution_alias_report` keeps those rows available to query/capture consumers. The full buffer rows can include execution-owned external aliases such as HZB indirect-args and stats backing labels, while `DiagnosticStore` records only `render.graph.execution.alias.{texture,buffer}_logical_count`, `{texture,buffer}_alias_count`, and `{texture,buffer}_backing_count`. This proves transient slot sharing, SSR mip aliases, and HZB external buffer binding without creating one path per resource name.

Graph execution profile diagnostics are CPU command-recording evidence, not GPU timestamp evidence. `execute_graph_stage(...)` measures the wall-clock CPU span around each executor call and records pass name, executor id, and elapsed microseconds in `RenderGraphExecutionProfileReport`. Runtime diagnostics project only `render.graph.execution.profile.pass_count`, `cpu_elapsed_total_us`, and `cpu_elapsed_max_us` with unit `microseconds`. RenderDoc marker matching, GPU timestamps, and pipeline statistics remain future profiling work; these rows only show that the runtime now has a stable per-pass CPU profile report available after a successful submit.

Transient pool diagnostics are physical WGPU pool evidence, not graph planning evidence. `TransientResourcePool` reports texture/buffer creation, reuse, retained entry counts, stale evictions, budget evictions, retained bytes, and configured budgets through `RenderGraphTransientPoolReport`. `DiagnosticStore` mirrors those rows under `render.graph.execution.transient_pool.*`, with retained/budget byte paths using unit `bytes` and count paths using unit `count`. This is the runtime-side companion to `render.graph.transient_*_bytes_reserved`: the graph rows describe the compiled allocation plan, while the execution pool rows describe what the cross-frame descriptor buckets retained or evicted after a submitted frame.

Graph execution coverage diagnostics are planned-vs-executed evidence, not a public pass trace. `update_base_stats(...)` compares the compiled graph's non-culled pass names with the renderer's executed pass record, then stores `RenderGraphExecutionCoverageReport` on `RenderStats.last_graph_execution_coverage_report`. `DiagnosticStore` mirrors planned live pass count, executed pass count, matched planned pass count, missing planned pass count, unexpected executed pass count, and duplicate executed pass count under `render.graph.execution.coverage.*`. `render_framework_stats_report_graph_execution_coverage` proves the submitted default Forward+ WGPU frame reports complete live-pass parity before diagnostics project those counters. These rows make graph cutover gaps visible without storing pass-name lists, executor handles, or GPU timing in diagnostics.

Graph stage execution diagnostics are stage-coverage evidence, not a public graph enum contract. `RenderGraphExecutionRecord` summarizes the internal executed-pass stage sequence into `RenderGraphStageExecutionReport`, then `RenderStats.last_graph_stage_execution_report` exposes staged pass count, unstaged pass count, unique stage count, stage transition count, and backward stage-order violation count. `DiagnosticStore` mirrors those counts as `render.graph.execution.stage.staged_pass_count`, `unstaged_pass_count`, `unique_stage_count`, `transition_count`, and `order_violation_count`. `render_framework_stats_report_graph_stage_execution` proves the submitted default Forward+ WGPU frame reports the same aggregate stage summary as the compiled live pass order before diagnostics project those counters. These rows prove how much of the executed pass sequence carried stage metadata and whether its stage sequence moved backward; they do not expose exact per-pass stage enums, queue internals, or GPU timing.

History copy diagnostics are copy-count and marker evidence, not texture ownership exposure. `copy_history_textures(...)` still performs the private WGPU copies/rolls for scene color, global illumination, ambient occlusion, screen-space reflection history, and exposure history when the matching feature path requests them and a history target exists. It now returns `RenderHistoryCopyReport`; `RenderGraphExecutionRecord` carries that report into `RenderStats.last_frame_history_copy_report`, and diagnostics mirror it as `render.history.copy.history_target_present`, `debug_marker_emitted`, `requested_count`, `copied_count`, `target_width`, `target_height`, and slot-level copied rows including `render.history.copy.exposure_copied`. Requested count can be nonzero while copied count is zero when no history target exists, which keeps first-frame or invalidated-history behavior visible without leaking history textures or buffers.

## Time Diagnostics

Each nonzero time advance records:

- `time.frame_time` in milliseconds,
- `time.fps` in hertz,
- `time.frame_count` in frames,
- `time.fixed_steps` in fixed-step count for that outer update.

`time.frame_count` and `time.fixed_steps` are still recorded on zero-delta updates. `time.frame_time` and `time.fps` are skipped for zero deltas, matching Bevy's guard against dividing by zero.

## Test Coverage

`zircon_runtime/src/tests/time.rs` verifies that advancing runtime time records the expected frame time, FPS, frame count, and fixed-step measurements, and that `collect_runtime_diagnostics` includes those runtime-owned values.

`zircon_runtime/src/diagnostic_log/diagnostics.rs` verifies stable formatting for current, smoothed, min, and max diagnostic values. `zircon_runtime/src/tests/prelude.rs` continues to verify the public diagnostic store, snapshot, and diagnostic-log formatting helpers through the stable runtime prelude.

2026-06-12 runtime 02 M2.2 owner migration evidence:

- `diagnostics/` moved from the `core` root to `core/runtime/diagnostics/`.
- `core/mod.rs` keeps `pub use runtime::diagnostics;` as the curated public facade, while `core/runtime/mod.rs` declares `pub mod diagnostics;`.
- `runtime_absorption::root_entries::core_root_reexports_runtime_diagnostics_without_root_directory` guards that the retired root directory is absent and the runtime owner exists.
- `runtime_absorption::root_entries::core_module_tree_matches_decided_spine_shape` guards that `core/` contains only `framework`, `manager`, `math`, `resource`, `runtime`, and `mod.rs`.
- Scoped verification passed with `rustfmt --edition 2021 --check`, the standalone `root_entries.rs` harness, and `zircon_runtime` / `zircon_app` core-min `cargo check`.

2026-06-14 render-main-chain object velocity diagnostics evidence:

- `zircon_runtime/src/tests/runtime_diagnostics/motion_vector.rs::runtime_diagnostics_reports_motion_vector_camera_and_mesh_draw_eligibility` verifies that `collect_runtime_diagnostics(...)` mirrors camera motion-vector readiness and mesh draw-level previous/missing motion-vector transform eligibility into stable diagnostic rows after CPU object-history diagnostics were removed.
- First scoped `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-cpu-history-cut-0614 --message-format short --color never`: PASS with the repository warning set for the CPU object-history hard cut. Final rerun after docs/formatting is currently blocked before render diagnostics by unrelated UI match drift in `zircon_runtime/src/ui/surface/surface/default_interactions.rs`.

2026-06-12 GPUScene upload diagnostics evidence:

- `cargo test -p zircon_runtime --lib render_product_diagnostics_record_gpu_scene_upload_stats --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`: PASS, proving GPUScene primitive/instance counts, dirty entry count, uploaded bytes, direct queue-write upload path, allocator free spans, and upload range counts project from `RenderStats` into `DiagnosticStore`.
- `render_product_diagnostics_record_mesh_indirect_batch_stats` now covers the four `render.mesh.queue.indirect_*` rows in source, and the GS-M4 replay slice adds source tests for phase-local WGPU indirect args buffers plus `multi_draw_indexed_indirect` replay. Fresh filtered `cargo test -p zircon_runtime --lib render_gpu_scene_indirect_batcher --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain` attempts after the GS-M4 batch-planning slice timed out while compiling the lib-test binary after 120 seconds and 300 seconds; `cargo test -p zircon_runtime --lib mesh_indirect_draw_execution_uses_wgpu_indirect_args_buffer --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` also timed out after 304 seconds after the replay slice. These diagnostics/replay source tests still need a completed lib-test run.
- `zircon_runtime/src/tests/runtime_diagnostics/support.rs` now registers the fake render diagnostics module as `crate::graphics::GRAPHICS_MODULE_NAME`, matching the canonical owner encoded in `GraphicsModule.Manager.RenderFramework`. The full `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` aggregation rerun is pending because the current lib-test target is blocked by active plugin-session compile errors in `extension_registry_bridge.rs` and `runtime_extension_registry.rs`, not by GPUScene diagnostics code.

2026-06-05 M5 runtime service dependency-name shape evidence:

- `cargo test -p zircon_runtime --lib core::runtime::tests:: --locked --jobs 1 --target-dir D:\cargo-targets\zircon-core-runtime-registry-cache-0605 --message-format short --color never -- --test-threads=1 --nocapture`: PASS, 18 runtime core tests passed. The lib-test compile covers `collect_runtime_devtools_snapshot` after `ServiceEntry.dependencies` moved from descriptor-shaped `DependencySpec` values to canonical `RegistryName` slices projected as copied strings in the devtools snapshot.

2026-05-26 M10W evidence:

- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked runtime_diagnostics --jobs 1 --message-format short --color never`: PASS, 2 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo test -p zircon_runtime --locked diagnostic_store --jobs 1 --message-format short --color never`: PASS, 5 matching lib tests passed.
- `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-m10w-assets-pbr-gate cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never`: PASS with 7 existing warnings.

2026-06-02 render-main-chain LUT diagnostics evidence:

- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving `render.post_process.lut.request_count`, `ready_count`, and `fallback_count` are projected from `RenderStats` into `DiagnosticStore`.

2026-06-03 render-main-chain compute dispatch diagnostics evidence:

- `cargo test -p zircon_runtime --lib execution_record_tracks_compute_dispatch_metadata --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving `RenderGraphExecutionRecord` preserves compute dispatch metadata and aggregates dispatch group volume plus storage-write resources.
- `cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving a compute-declared graph pass that falls back to the graphics queue still reports concrete clustered-lighting dispatch evidence.
- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never`: PASS, proving the compute dispatch, dispatch group, and storage-write resource count rows are projected from `RenderStats` into `DiagnosticStore`.

2026-06-03 render-main-chain history-size diagnostics evidence:

- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never`: PASS with existing warnings, proving the `RenderStats` / `FrameHistoryStatus` / `render.history.*` row additions type-check through the runtime lib.
- `cargo test -p zircon_runtime --lib render_framework_invalidates_history_when_dynamic_render_size_changes --locked --jobs 1 --target-dir F:\cargo-targets\zircon-render-main-chain-history --message-format short --color never`: BLOCKED before test execution by unrelated `zircon_runtime/src/tests/plugin_extensions/static_manifest_contracts/*` lib-test compile errors (`E0364` private re-exports and `E0282` inference errors).

2026-06-04 render-main-chain transient allocation byte diagnostics evidence:

- `cargo test -p zircon_runtime --lib render_framework_stats_report_transient_allocation_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`: focused validation target for `RenderStats` transient byte projection.
- `cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`: focused validation target for byte-unit `DiagnosticStore` rows.

2026-06-17 RG-M4 alias/profile diagnostics evidence:

- `RenderStats` now carries `last_graph_execution_alias_report` and `last_graph_execution_profile_report`; `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` has fixture data and assertions for alias count rows plus CPU profile count/total/max microsecond rows.
- `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-alias-profile-0617`: first cold-target attempt timed out after 124 seconds; warmed rerun passed in 301.3 seconds with existing warnings only. Focused runtime diagnostics tests remain deferred for the implementation-first phase.

2026-06-17 graph materialization diagnostics evidence:

- `RenderStats` now carries `last_graph_materialization_report`; `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` has fixture data and assertions for required/bound/missing resource counts, missing typed resource count, texture/buffer/external splits, and sparse reservation count rows.
- `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-materialization-0617`: first cold-target attempt timed out after 244 seconds; warmed rerun passed in 188 seconds with existing warnings only. Focused runtime diagnostics tests remain deferred for the implementation-first phase.

2026-06-18 stale materialization binding diagnostics evidence:

- `RenderGraphMaterializationReport` now carries `stale_texture_binding_count` and `stale_buffer_binding_count`, `render_stats_store::graph` projects aggregate and per-kind stale binding rows, and the runtime diagnostics fixture/assertions cover the zero-count diagnostic rows.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-stale-lifetime-validation-0618 --message-format short --color never`: passed in 421.59 seconds with the existing 141-warning set. The first focused stale-binding `cargo test` filter timed out after 904 seconds during lib-test compilation; `cargo test -p zircon_runtime --lib materialization_validation_rejects_stale --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-stale-lifetime-validation-0618 --message-format short --color never` then completed in 8m06s with the existing 52-warning set. Direct full-path exact execution of the compiled `zircon_runtime-d071a300da0585cb.exe` passed both stale texture and stale buffer materialization validation tests in 1.41s and 1.43s, and no stale-lifetime Cargo/rustc/rustdoc processes remained afterward.

2026-06-17 HZB external materialization diagnostics evidence:

- HZB executor-owned external buffers are now bound into `RenderGraphExecutionResources` before the materialization report is generated; alias rows can show their logical-to-backing buffer mapping, while diagnostics still project only low-cardinality aggregate counts.
- `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-materialization-0617`: first cold-target attempt timed out after 364 seconds without a Rust diagnostic; warmed rerun passed in 73.5 seconds with existing warnings only. Focused tests remain deferred for the implementation-first phase.

2026-06-17 required External binding diagnostics evidence:

- `RenderGraphExternalResourceBinding` now distinguishes report-only external imports from required texture/buffer imports. HZB occlusion declares its executor-owned external names as required buffers, and materialization validation returns a hard error if such a buffer is not bound before executor dispatch.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never`: passed with the existing warning set. Focused required-external/materialization/HZB descriptor tests remain deferred for the implementation-first phase.
