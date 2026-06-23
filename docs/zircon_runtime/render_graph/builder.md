---
related_code:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/dump.rs
  - zircon_runtime/src/render_graph/error.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_history_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_plugin_graph_resources.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
implementation_files:
  - zircon_runtime/src/render_graph/builder.rs
  - zircon_runtime/src/render_graph/dump.rs
  - zircon_runtime/src/render_graph/error.rs
  - zircon_runtime/src/render_graph/graph.rs
  - zircon_runtime/src/render_graph/mod.rs
  - zircon_runtime/src/render_graph/types.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/rhi/descriptors.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/render_feature_pass_descriptor.rs
  - zircon_runtime/src/graphics/feature/render_feature_pass_descriptor/new.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/graph_resources.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/mesh.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_geometry.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/deferred_lighting.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/compute_workload.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/hzb_occlusion.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/deferred.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/resource_lookup.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/surface.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/computed_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/post_process.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_plugins/particles/runtime/src/render/executors.rs
  - zircon_plugins/particles/runtime/src/render/gpu/runtime_owner.rs
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/graph.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/render_framework_state/render_framework_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/register_pipeline_asset/register_pipeline_asset.rs
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/record_capture.rs
  - zircon_runtime/src/render_graph/tests/resources.rs
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - .codex/plans/Zircon SRPRHI 渲染管线补全计划.md
  - user: 2026-06-02 PLEASE IMPLEMENT THIS PLAN - ZirconEngine WGPU 渲染主链闭环计划
  - user: 2026-06-17 bind HZB executor-owned external buffers for render plan 01
  - user: 2026-06-17 implement WGPU-to-render pipeline design from docs/plans/zircon_runtime/render, feature-first with tests deferred
  - user: 2026-06-17 continue Plan 01 required external texture import and materialization modularization
tests:
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_duplicate_resource_names
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_tracks_transient_lifetimes_and_resource_edges
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_transient_read_without_producer
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_write_after_write_without_dependency
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_records_attachment_clear_load_store_ops
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_records_storage_writes_without_attachment_ops
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_transient_attachment_load_without_producer
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_rejects_read_after_discarded_transient_attachment_store
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_culls_unused_resource_writer_but_keeps_external_output_chain
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_culls_passes_unreachable_from_present_root
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_non_root_external_write_is_culled
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_readback_marked_buffer_keeps_producer_alive
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_persistent_texture_keeps_producer_alive
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_side_effect_pass_survives_culling
  - zircon_runtime/src/render_graph/tests/culling.rs::render_graph_missing_cull_root_is_compile_error
  - zircon_runtime/src/render_graph/tests/resources.rs::render_graph_dump_lists_pass_order_resources_and_culled
  - zircon_runtime/src/render_graph/tests/ordering.rs::compile_exposes_inferred_resource_dependencies_on_compiled_passes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::metadata_context_resolves_pass_resource_handles
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::resource_registry_validates_declaration_kind_before_name_lookup
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::compile_options_fallback_async_compute_passes_to_graphics_queue
  - zircon_runtime/src/graphics/tests/pipeline_compile.rs::pipeline_compile_rejects_storage_write_mode_on_read_access
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_compute_workload_metadata
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_preserves_sparse_texture_reservations_without_dense_transient_slot
  - zircon_runtime/src/render_graph/tests/resources.rs::graph_transient_allocation_plan_reports_slot_reserved_bytes
  - zircon_runtime/src/graphics/tests/render_framework_graph_stats.rs::render_framework_stats_report_transient_allocation_bytes
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_preserves_compute_workload_from_feature_descriptor
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_rejects_compute_workload_on_non_compute_queue
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_audits_planned_compute_workloads_against_dispatches
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs::tests::execution_record_flags_compute_workload_label_workgroup_and_extent_mismatches
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs::tests::rg_resource_resolver_requires_pass_declared_access_before_physical_texture_lookup
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context.rs::tests::resolver_backed_name_access_ignores_stale_context_resource_rows
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_compatible_transient_texture_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization.rs::tests::materialization_receives_incompatible_texture_resources_in_separate_graph_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs::tests::materialization_aliases_transient_buffer_slots
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_reuses_entries_across_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_stale_entries_after_keep_frames
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/transient_resource_pool.rs::tests::transient_resource_pool_evicts_oldest_entries_to_budget
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs::headless_wgpu_server_falls_back_async_compute_passes_to_graphics
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_hits_on_identical_key
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_misses_on_feature_set_change
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_misses_on_viewport_resize
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_invalidates_on_pipeline_revision_bump
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_evicts_least_recently_used_entry
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::compiled_render_pipeline_cache_reports_lookup_status
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs::tests::render_graph_compile_frame_fingerprint_tracks_compile_extract_inputs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs::runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_reports_unbound_compiled_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_reports_unbound_external_lifetimes_without_failing
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_fails_unbound_required_external_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_fails_unbound_required_external_texture
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_texture_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization_validation.rs::tests::materialization_validation_rejects_stale_buffer_binding_outside_live_lifetimes
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_execution_owned_graph_resources.rs::tests::hzb_external_fallback_buffers_satisfy_materialization_report
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/hzb.rs::tests::hzb_occlusion_cull_declares_execution_owned_external_buffers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_describes_hzb_as_half_power_of_two_mip_chain
  - zircon_plugins/particles/runtime/src/render/runtime_prepare.rs::tests::particle_runtime_prepare_registration_id_is_stable
  - zircon_plugins/particles/runtime/src/tests/manager_resolution.rs::particles_runtime_plugin_module_and_runtime_prepare_share_manager
  - zircon_plugins/particles/runtime/src/tests/gpu.rs::particle_gpu_runtime_owner_executes_backend_and_exposes_active_buffers
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_preserves_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs::tests::compile_rejects_conflicting_required_external_texture_and_buffer_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_forward_plus_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/shadow_atlas_required_external_tests.rs::compile_deferred_preserves_shadow_atlas_required_external_texture_binding
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_imports_only_live_compiled_frame_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_rebinds_live_final_aliases_to_imported_texture_target
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/bind_frame_graph_resources.rs::tests::frame_binder_leaves_advanced_transients_for_materialization
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-alias-profile-0617
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-materialization-0617
  - cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-materialization-0617
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-required-external-texture-0617 --message-format short --color never
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617 --message-format short --color never
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_graph --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib graph_records_storage_writes_without_attachment_ops --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compile_options_fallback_async_compute_passes_to_graphics_queue --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib pipeline_compile_rejects_storage_write_mode_on_read_access --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - cargo test -p zircon_runtime --lib compute_workload --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo test -p zircon_runtime --lib headless_wgpu_server_falls_back_async_compute_passes_to_graphics --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo test -p zircon_runtime --lib runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
doc_type: module-detail
---

# RenderGraph Builder

`RenderGraphBuilder` is the runtime frame-graph authoring surface used by SRP compilation and renderer execution. It owns graph texture and buffer handles, imported external targets, pass dependencies, pass resource accesses, attachment load/store operations, culling decisions, queue lane assignment, resource declaration rows, and resource lifetime metadata.

## Resource Ownership

Every graph resource name is unique within a compiled frame graph. Duplicate names across graph textures, graph buffers, and external imports are rejected before pass ordering is derived. This keeps RenderDoc labels, lifetime spans, transient aliasing, and future history slot names unambiguous.

The RG-M1 handle table uses `RgTextureHandle`, `RgBufferHandle`, `ExternalResource`, and the sum type `RenderGraphResource`. `RenderGraphBuilder::create_texture(...)` and `create_buffer(...)` allocate stable logical handles before any physical WGPU resource exists, while `import_external_resource(...)` registers imported roots. `CompiledRenderGraph::resource_declarations()` preserves every declared resource row, including resources whose only writers were later culled, and `RenderGraphResourceLifetime.resource` carries the same logical handle for every live resource interval. `CompiledRenderGraph::resource_declaration(...)`, `resource_declaration_by_name(...)`, `resource_lifetime(...)`, and `resource_lifetime_by_name(...)` are the narrow lookup surface for resolver work: declarations answer "was this graph resource authored?", while lifetimes answer "did it survive culling and need execution-time backing?". This follows the Unreal RDG split between logical handles and later pooled resource resolution without exposing WGPU objects through the graph API.

The 2026-06-17 public export follow-up keeps `RenderGraphExternalResourceBinding`, `RenderGraphExternalResourceRequirement`, and `RenderGraphExternalResourceType` available from `zircon_runtime::render_graph`. Their definitions remain in `render_graph/types.rs`, but `render_graph/mod.rs` re-exports them beside the other graph authoring DTOs so SRP feature descriptors and pipeline compilation use the public graph boundary instead of reaching into private module ownership. Validation used `cargo fmt -p zircon_runtime -p zircon_editor --check` and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`; the first editor check window timed out after the export fix without a new Rust diagnostic, leftover validation processes were stopped, and the longer rerun passed with the existing warning set.

Compiled pass resource access rows remain the current execution bridge: they preserve stable resource names, resource kind, access mode, and attachment ops so existing executors can continue querying execution-record counters by name. `RgResourceResolver` is now attached to real `RenderPassExecutionContext` instances during staged execution; it binds the compiled graph plus current pass id so executors can resolve pass-declared `RenderGraphResource` handles back to declaration rows and live lifetimes. `RenderPassExecutionContext::attachment_ops_for_write(...)`, `reads_texture(...)`, and `reads_transient_texture(...)` now prefer that resolver when present and fall back to the stored pass-resource rows only for manually built metadata contexts. `RenderGraphExecutionResources::require_texture_view_for_declaration(...)` and `require_buffer_for_declaration(...)` consume declaration rows and validate texture/buffer kind before name lookup. Product postprocess resource checks now ask the resolver for pass-scoped read declarations before requiring texture views.

The WGPU execution resource table still exposes logical names to executors, but it now keeps a separate logical-name to physical-backing map. `CompiledRenderGraph::transient_allocation_plan()` partitions dense transient lifetimes by descriptor bucket before interval coloring: textures key by WGPU-relevant shape, format, dimension, residency, and usage, while buffers key by size and usage. Slot indices are therefore bucket-local and dump slot rows carry `bucket_key_hash`. Execution materialization groups by `(bucket_key_hash, slot)`, so bucket-local slot `0` in two descriptor buckets creates two physical WGPU backings and two alias labels. Compatible non-overlapping lifetimes in the same bucket can still share one owned WGPU backing while preserving the existing name-based executor API, and execution validates descriptor compatibility defensively before binding a shared backing.

Execution validation treats `resource_lifetimes()` as the live backing contract, not `resource_declarations()`. After frame/history/plugin binders and transient materialization run, any logical texture view or buffer binding whose name is not in the compiled lifetime set is reported as a stale binding and rejected before pass execution. This keeps declarations for culled resources useful for diagnostics and dumps without allowing old pre-bound WGPU rows to authorize execution of resources that the graph compiler removed.

External resources are imported logical resources, not automatically safe WGPU backings. Reads from external resources do not require an in-graph producer, while transient reads still require an ordered producer. Culling liveness comes from `RenderGraphResourceUsageFlags` (`present`, `readback`, or `persistent`) and pass-level roots, so non-present external scratch resources can be culled when they do not feed a root.

`RenderGraphExternalResourceBinding` now carries the execution-side contract for imported resources separately from culling intent. The default binding is `report_only`, preserving frame targets, history imports, and optional external consumers that should appear in materialization reports without failing the frame. Optional typed imports use `report_only_texture()` or `report_only_buffer()`: their type is propagated through `RenderGraphResourceDeclaration` / `RenderGraphResourceLifetime`, but missing bindings remain diagnostic evidence. Required typed imports use `required_buffer()` or `required_texture()`: their type and requirement are propagated through the same rows, and `RenderPipelineAsset::compile(...)` rejects one external name being declared as incompatible external types. Feature descriptors expose typed optional helpers (`read_external_texture(...)`, `read_external_buffer(...)`, `write_external_texture(...)`, `write_external_buffer(...)`, `write_storage_external_texture(...)`, `write_storage_external_buffer(...)`, `write_external_texture_with_ops(...)`) beside the required helpers (`read_required_external_buffer(...)`, `write_required_external_buffer(...)`, `read_required_external_texture(...)`, `write_required_external_texture(...)`, `write_required_external_texture_with_ops(...)`, and `write_required_storage_external_texture(...)`); the SRP compile-side merge and conflict checks live in `render_pipeline_asset/graph_resources.rs`. The first production required consumers are HZB occlusion's required external buffers and the shadow atlas required external texture. Optional typed production consumers include built-in frame target/history/post-process externals and first-party particles/Hybrid GI/Virtual Geometry plugin resources. Renderer-owned frame targets and final aliases are bound by `bind_frame_graph_resources(...)` only when their lifetimes remain live in the compiled graph. Built-in history externals are bound by `bind_history_graph_resources(...)`, which imports only enabled history textures/buffers that remain live in the compiled graph. `shadow-atlas` writes `SHADOW_ATLAS` as a required external texture with attachment ops, while Forward+ and Deferred receiver passes read the same external so the compiled graph lifetime carries `required_texture()`. Post-process exposure remains a normal compiled dependency: only passes that actually sample or dispatch from `EXPOSURE_CURRENT` declare the buffer read, so split DoF, motion-blur, and blur passes can stay before `post.exposure.resolve` while scene-composite, color-LUT bake, and uber keep their real exposure ordering edge.

## Pass Validation

The builder validates explicit dependency cycles, transient read-before-produce, and write-after-write hazards without an ordering dependency. Read-after-write dependencies are inferred after manual dependency order is known, so passes can be authored in declaration order without losing deterministic execution.

Compiled graph metadata exposes each pass resource access by stable resource name and kind. Resource lifetimes include the original descriptor, first pass index, last pass index, and whether the resource was imported.

## Culling Roots

Pass culling is rooted in explicit graph intent, not resource kind shortcuts. `RenderGraphResourceUsageFlags` marks resources as `present`, `readback`, or `persistent`; writing any of those resources makes the writer a culling root, and the builder walks backwards through reads and declared dependencies to keep only the producers needed by those roots. `import_external_resource(...)` remains the presentation-target convenience path and defaults to `present`; callers that need an external scratch/readback target without presentation semantics use `import_external_resource_with_usage(...)`.

`mark_readback(...)` can promote any declared graph resource into a readback root, while `mark_persistent(...)` promotes a transient texture into a history/persistent root. Passes with `has_side_effects` or `allow_culling == false` also remain roots even when they do not write a resource. A graph with passes but no present, readback, persistent, or pass-level root now fails with `MissingCullRoot` instead of silently keeping no-write or external-write passes alive. `RenderGraphResourceDeclaration`, `RenderGraphResourceLifetime`, and graph dump resource rows preserve the usage flags so diagnostics can show why a resource anchored culling.

## Attachment Operations

Texture and external target writes carry explicit attachment operations through `RenderGraphAttachmentOps`. The default transient texture write is `Clear + Store`, matching the safe first-use behavior expected by depth prepass, gbuffer, opaque scene color, and scratch targets. External writes default to `Load + Store`, because imported targets may already contain swapchain, UI, or previous composition contents. Callers can use `write_texture_with_ops(...)` or `write_external_with_ops(...)` when a pass needs explicit `Load`, `Clear`, `Store`, or `Discard` semantics.

Validation is intentionally conservative:

- a transient attachment cannot use `Load` before any ordered producer has stored content,
- a transient attachment stored with `Discard` cannot be read or loaded by a later pass,
- read-after-write dependencies are still inferred only after manual dependency order is known,
- write-after-write still requires an explicit ordering dependency.

Compiled pass metadata preserves the attachment ops beside the resource access. SceneRenderer executors can therefore derive WGPU `LoadOp` / `StoreOp` decisions from graph data instead of hard-coding each pass forever.

## Storage Writes

Compute and non-attachment passes use storage write metadata instead of pretending their outputs are render attachments. `RenderFeatureResourceWriteMode` splits `Attachment` writes from `Storage` writes at the SRP descriptor layer. `RenderGraphBuilder::write_storage_texture(...)` and `write_storage_external(...)` record ordinary write edges with no `RenderGraphAttachmentOps`, so dependency ordering, culling, queue counts, resource lifetimes, and debug evidence still work while load/store validation stays scoped to attachment writes.

The SRP compiler preserves that split when it lowers `RenderFeaturePassDescriptor` resources into the graph. Attachment writes still default to clear/store for first transient writes and load/store for later or external writes. Storage writes reject attachment ops and read resources reject storage write mode. The built-in `ao.ssao-evaluate` pass now writes `ambient-occlusion` through `write_storage_external(...)`, keeping its async-compute declaration and graphics-queue fallback evidence without assigning invalid attachment load/store semantics to the SSAO output.

## Sparse Texture Reservations

Sparse texture resources use the ordinary `TextureDesc` shape plus `TextureResidency::SparseReserved`. RenderGraph preserves that descriptor in `RenderGraphResourceLifetime`, exposes `RenderGraphResourceLifetime::is_sparse_reserved_texture()`, and counts such lifetimes in `CompiledRenderGraphStats.sparse_texture_lifetime_count`. This is a reservation contract only: no page table, residency allocator, tile upload path, WGPU sparse object, or runtime provider state is implied by the graph descriptor.

Transient allocation planning keeps sparse reservations out of the dense transient texture aliasing pool. `CompiledRenderGraphTransientAllocationPlan.texture_slot_count` still counts only dense transient texture slots, while `sparse_texture_slot_count` records the number of sparse transient reservations that a later residency manager must own explicitly. That keeps virtual texture resources visible to graph validation and diagnostics without pretending they can alias with ordinary render targets or scratch textures.

The allocation plan is byte-aware and descriptor-bucket-aware. Each dense transient allocation records its descriptor-derived `size_bytes` plus `bucket_key_hash`; each texture or buffer slot reservation records the maximum byte requirement for one bucket-local slot; and the plan exposes `dense_texture_bytes_reserved`, `dense_buffer_bytes_reserved`, and `total_dense_bytes_reserved()`. `slot_bytes_for_bucket(...)` returns a single bucket reservation, while `slot_bytes(...)` is an aggregate compatibility helper for older callers that look only at `(kind, slot)`. Sparse reservations remain excluded from dense slots but still contribute to `sparse_texture_virtual_bytes`, giving residency planning a virtual footprint without claiming dense backing memory. The size estimate uses the same RHI-neutral `BufferDesc.size_bytes` and `TextureDesc::checked_storage_size_bytes()` inputs as the headless WGPU transient allocator stats.

`update_base_stats(...)` copies those byte totals into `RenderStats` as `last_graph_transient_texture_bytes_reserved`, `last_graph_transient_buffer_bytes_reserved`, `last_graph_transient_dense_bytes_reserved`, and `last_graph_sparse_texture_virtual_bytes`. Runtime diagnostics mirror them with `bytes` units under `render.graph.transient_texture_bytes_reserved`, `render.graph.transient_buffer_bytes_reserved`, `render.graph.transient_dense_bytes_reserved`, and `render.graph.sparse_texture_virtual_bytes`. These rows remain graph planning evidence; the execution resource report is the WGPU-side evidence for how many physical owned textures, external views, and buffers were actually materialized after descriptor compatibility checks.

## Compute Workload Metadata

Compute workload metadata is a planned graph contract, not a backend object. `RenderGraphComputeWorkload` carries a neutral pipeline label, non-zero workgroup size, and dispatch extent (`Viewport`, `ClusterGrid`, or `Fixed`). SRP feature descriptors attach it with `with_compute_workload(...)`; `RenderPipelineAsset::compile(...)` validates that the pass still declares `QueueLane::AsyncCompute`, then copies the workload onto `CompiledRenderPass.compute_workload`.

This gives pipeline activation, graph review, and plan-vs-execution diagnostics a stable place to see expected compute work before `SceneRenderer` resolves concrete WGPU resources. Actual execution evidence remains separate in `RenderGraphComputeDispatchRecord`, which is produced by renderer executors only after they record a concrete compute pass. Dispatch records now carry the renderer-private pipeline label, workgroup size, dispatch group count, and storage-write resource names; `RenderGraphExecutionRecord` derives expected dispatch groups from the compiled workload (`Viewport`, `ClusterGrid`, or `Fixed`) plus the current frame dispatch context, then compares pipeline label, workgroup size, and dispatch groups against those concrete records. The audit stores planned and actual dispatch groups so mismatched extents can be diagnosed without reading renderer-private WGPU objects, and it counts matched, missing, mismatched, and unexpected compute work.

The audit stays backend-neutral. `RenderStats` exposes only counts such as `last_graph_compute_planned_workload_count`, `last_graph_compute_matched_workload_count`, `last_graph_compute_missing_dispatch_count`, `last_graph_compute_workload_mismatch_count`, and `last_graph_compute_unexpected_dispatch_count`; `DiagnosticStore` mirrors them under `render.graph.compute_*_workload_count`, `render.graph.compute_missing_dispatch_count`, `render.graph.compute_workload_mismatch_count`, and `render.graph.compute_unexpected_dispatch_count`. WGPU pipelines, bind groups, buffers, and texture handles remain renderer-private.

## Graph Dump

`CompiledRenderGraph::dump()` produces a WGPU-free `RenderGraphDump` artifact from the compiled graph. The dump keeps structured rows for pass order, pass id, declared/effective queue, queue fallback, culled state, culling flags, executor id, inferred dependencies, pass resource IO, compute workload metadata, resource declarations, usage flags, live lifetimes, transient slot assignment, descriptor bucket hash, resource byte size, and bucket-local reserved bytes. `RenderGraphDump::to_text()` turns the same data into a stable line-oriented format for capture diagnostics and RenderDoc marker comparison.

Captured runtime frames now carry the same text through `CapturedFrame.graph_dump`. During capture submission, `record_capture(...)` serializes `context.compiled_pipeline().graph.dump().to_text()` beside the RGBA payload and `RenderCaptureReport`, so tooling that queries the last captured frame can inspect the exact graph that produced that frame without reaching into renderer-private WGPU resources.

## Execution Alias And CPU Profile Reports

RDG aliasing now has a runtime evidence path in addition to the compile-time transient slot dump. `RenderGraphExecutionResources::resource_alias_report()` snapshots owned-backed logical texture and buffer names after WGPU materialization and before the frame releases those owned backings into `TransientResourcePool`. The report distinguishes logical resource names from physical backing labels, including transient slot labels, SSR mip aliases, and HZB executor-owned external buffer bindings. External frame-target imports stay out of the alias rows because the execution table only owns their cloned texture views, not their source texture identity.

`RenderGraphExecutionRecord` owns the alias report and a per-pass `RenderGraphExecutionProfileReport`. `execute_graph_stage(...)` records one CPU span around each executor call and stores pass name, executor id, and elapsed microseconds. The profile is intentionally CPU command-recording evidence, not GPU timestamp evidence. `RenderStats` carries the full rows for query/capture consumers, while `DiagnosticStore` records only low-cardinality counters: alias logical/aliased/backing counts for textures and buffers, plus profile pass count, total CPU microseconds, and max CPU microseconds.

## Execution Materialization Validation

Compiled graph lifetimes now have an execution-side completeness audit. After graph-lifetime-aware frame resource imports, optional history imports, transient slots, pool backings, and SSR mip aliases are materialized, `RenderGraphExecutionResources::validate_materialized_graph_resources(...)` walks `CompiledRenderGraph::resource_lifetimes()` and verifies that every live typed texture lifetime has a texture view and every live typed buffer lifetime has a buffer. Sparse texture reservations are counted but intentionally stay unbacked by dense WGPU resources.

The audit returns a neutral `RenderGraphMaterializationReport`, stores it on `RenderGraphExecutionRecord`, and copies it into `RenderStats.last_graph_materialization_report`. Missing dense typed texture/buffer bindings are hard errors before pass executors run. External lifetime reporting now has two buckets: required external coverage (`required_external_count`, `bound_required_external_count`, `missing_required_external_count`) and report-only coverage (`report_only_external_count`, `bound_report_only_external_count`, `missing_report_only_external_count`). Aggregate external helpers still expose total bound/missing external coverage for diagnostics, but `required_external_count` no longer includes imported frame targets, optional history slots, or optional plugin resources. Missing `required_buffer` or `required_texture` lifetimes fail before executor dispatch; report-only unknown/texture/buffer external lifetimes remain diagnostic evidence. This lets HZB external buffers hard-fail when the execution-owned bridge is absent without making imported frame targets, optional history slots, or optional plugin resources mandatory.

The HZB occlusion external-buffer subset now binds before that audit. `render_compiled_scene(...)` calls `bind_execution_owned_graph_resources(...)` after transient materialization and before validation; that helper maps live HZB required external names to the current phase-local indirect args, compaction metadata, compact replay outputs, draw-count, and stats buffers, with minimum fallback buffers for zero-candidate frames. Renderer-owned frame resources are bound earlier through `bind_frame_graph_resources(...)`, which imports only live compiled lifetimes for fixed frame targets, final aliases, `LIGHT_LIST`, and `SHADOW_ATLAS`; the shadow atlas required texture maps to the persistent `ShadowAtlasResources::atlas_view()` when atlas resources are available. Built-in TAA, SSR, HZB, Hybrid GI history, and exposure history resources are bound through `bind_history_graph_resources(...)` only when their frame flags are enabled and their external lifetimes are live. Plugin external buffers are bound through `bind_plugin_graph_resources(...)`: runtime-prepare collectors can register actual per-frame WGPU buffers, and the binder falls back to deterministic `:plugin-external-fallback` backings for first-party names when no real binding was registered. Virtual Geometry now registers `virtual-geometry-feedback` from prepared page-request sidebands when such feedback exists. Particles now register `particles.runtime-prepare` with the shared plugin manager; concrete GPU instances execute through `ParticleGpuRuntimeOwner` and bind real `ParticleGpuBackend` buffers for `particles.gpu.*`, while frames without concrete GPU state can still bind neutral `ParticleExtract.gpu_frame` summary buffers before falling back to deterministic materialization buffers.

Pass-level physical lookup is now starting to move through `RgResourceResolver`. The resolver proves that the compiled pass declared the requested resource name and access kind before `RenderPassExecutionContext::require_texture_view_by_name(...)` resolves a WGPU view from the execution resource table, and product post-process required-resource validation/input selection now uses that resolver-backed path. `RenderPassExecutionContext::with_gpu(...)` also propagates the resolver into the GPU execution context, so depth-prepass writes, deferred G-buffer writes/reads, deferred lighting graph texture/buffer accesses, shadow-atlas writes, HZB previous-history reads, mesh-stage color/depth attachments, mesh light-grid reads, object and particle velocity attachments, TAA reactive-mask mesh attachments, sprite/preview-sky/UI/overlay surface bridge lookups, particle transparent bridge lookups, SSR-specific resolve/reflection-pyramid/coarse-pyramid/specular-occlusion bridge lookups, and root postprocess stack/effect/compute/terminal bridge lookups validate against the compiled pass before touching the WGPU resource table. The deferred bridge methods are split into `render_pass_execution_context/gpu/deferred.rs`, the surface bridge methods are split into `render_pass_execution_context/gpu/surface.rs`, particle bridge methods remain in `render_pass_execution_context/gpu/particle.rs`, and postprocess bridge methods are split across `gpu/post_process.rs`, `gpu/post_process/effects.rs`, `computed_resources.rs`, `temporal.rs`, `terminal.rs`, and `screen_space_reflection.rs`. The shared resolver-backed WGPU lookup helpers live in `gpu/resource_lookup.rs`. Mesh light-grid buffers use optional resolver-backed reads because Forward mesh descriptors declare them and Deferred transparent mesh intentionally does not; shared post-process texture inputs use the same optional helper shape to preserve fallback textures for passes that do not declare a slot while still rejecting declared kind mismatches. HZB history uses a declared-optional helper so the pass still proves its compiled read declaration when first-frame history is physically absent. The particles plugin executor now delegates texture-view access to the runtime particle bridge instead of prechecking `gpu.resources` directly. SSR passes now declare `light-list` as an external buffer read before recording the shared post-process bind group. Direct GPU executor lookup is now confined to `gpu/resource_lookup.rs` helper fallback/internal calls, and raw `RenderGraphExecutionResources` texture/buffer lookup methods are narrowed to graph-execution scope.

The 2026-06-24 resolver suppression cleanup keeps that production split explicit. `RgResourceResolver` no longer exposes a production physical direct-lookup surface just to support focused tests: `with_physical(...)`, the resolver-owned texture/buffer lookup helpers, `physical_resources(...)`, and the handle-level physical require helper are `#[cfg(test)]`. Production code still receives the same declaration/access proof from the resolver, then performs WGPU lookup through the execution context and `gpu/resource_lookup.rs`. Status anchor: `render_plan01_rg_resource_resolver_physical_test_surface_static_passed_cargo_deferred_active_lanes`.

The same follow-up removes dead-code suppression from `RenderPassExecutionContext`. Context construction helpers remain available for graph execution focused tests and plugin test contexts without a suppression attribute, `require_buffer_by_name(...)` is live through deferred/postprocess/SSR buffer lookups, and `uses_queue_fallback()` stays the public queue-fallback observation helper. Status anchor: `render_plan01_execution_context_suppression_cleanup_static_passed_cargo_deferred_active_lanes`.

`RenderGraphExecutionResources::owned_texture_desc(...)` is likewise production-live and no longer carries dead-code suppression. GI history copy and graph-owned texture readback paths use it to inspect execution-owned texture descriptors before copy/readback decisions. Status anchor: `render_plan01_execution_resources_owned_texture_desc_suppression_static_passed_cargo_deferred_active_lanes`.

`RenderGraphExecutionRecord` now follows the same split. Legacy pass push helpers, test readback setters, and raw staged/resource/dependency/compute inspection accessors are test-only instead of being held open with dead-code suppression, while `executed_post_process_nodes(...)` and `executed_stage_count(...)` stay production-live for stats and product graph observation. Status anchor: `render_plan01_execution_record_observation_suppression_cleanup_static_passed_cargo_deferred_active_lanes`.

## Compiled Graph Cache

`CompiledGraphCache` caches `Arc<CompiledRenderPipeline>` by pipeline handle, pipeline revision, shader quality tier, compile options, backend capability fingerprint, and frame compile fingerprint. The frame fingerprint covers the compile-affecting extract shape currently read by pipeline lowering: core pipeline, view size, render size, camera HDR, camera MSAA, and particle-sprite pass presence. This keeps the cache owner in `graphics::pipeline` / `graphics::runtime::render_framework` while the RenderGraph contract remains WGPU-free.

`compile_submission_pipeline(...)` now routes submission compilation through `get_or_compile_with_status(...)`. Cache misses still run `RenderPipelineAsset::compile_with_options(...)` and capability validation; hits reuse the existing compiled pipeline and skip descriptor lowering, pass dependency derivation, culling, and transient allocation planning for that key. Hit lookups return an explicit status and re-run `extract_compile_fingerprint(...)` in debug builds to catch future compile input drift before a stale graph can hide it. Registering or reloading a pipeline invalidates entries for the affected handle, and reload also bumps the pipeline asset revision so stale compiled graphs cannot survive asset mutation.

`RenderStats` exposes compiled-cache hit, miss, eviction, and live-entry counts as `last_graph_compiled_cache_*`, and runtime diagnostics mirror them under `render.graph.compiled_cache.*`. These rows are hot-path compile evidence, not GPU execution evidence.

## History Preparation

History resources must use distinct names for previous, current, and output slots before they are represented in the graph. The unique-name rule prevents a feature from accidentally declaring one physical graph resource as both imported history input and writable history output. The later scene renderer registry should map those slots onto concrete backing textures after resize, camera-cut, and motion-validity checks.

Focused RenderGraph validation on 2026-06-02 passed with 22 tests, 0 failures, using `CARGO_TARGET_DIR=E:\cargo-targets\zircon-render-main-chain`. The latest validation included attachment clear/load/store metadata, load-before-producer rejection, and read-after-discard rejection.

The 2026-06-03 M8 storage-write slice used the same target dir. `graph_records_storage_writes_without_attachment_ops`, `compile_options_fallback_async_compute_passes_to_graphics_queue`, and `pipeline_compile_rejects_storage_write_mode_on_read_access` passed, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only.

The 2026-06-03 M8 workload-audit slice reused `E:\cargo-targets\zircon-render-main-chain`. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only. `cargo test -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --color never compute_workload` passed 5 filtered tests, covering graph metadata preservation, pipeline compile validation, and execution-record workload audit status. `headless_wgpu_server_falls_back_async_compute_passes_to_graphics` and `runtime_diagnostics_combines_core_render_contract_and_missing_externalized_plugins` also passed, proving the matched workload count reaches `RenderStats` and runtime diagnostics. The follow-up dispatch-extent audit extends the execution-record tests so viewport, cluster-grid, and fixed dispatch plans preserve planned/actual dispatch-group evidence and report `DispatchExtentMismatch` when a renderer records the wrong group count.

The 2026-06-04 byte-aware transient allocation slice extended `CompiledRenderGraphTransientAllocationPlan` with per-resource byte size, per-slot reserved byte size, dense texture/buffer byte totals, and sparse virtual texture bytes. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only. Focused `cargo test -p zircon_runtime --lib render_graph::tests::resources::graph_transient_allocation_plan_reports_slot_reserved_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture` initially timed out while the Windows lib-test binary was compiling and linking; after the compile lane drained and produced `zircon_runtime-b34ee8d8fc52f1fd.exe`, the warmed rerun passed 1 test, 0 failed, 2680 filtered, with existing warnings only.

The follow-up diagnostics bridge preserves those planned byte totals through `RenderStats` and `DiagnosticStore` without exposing backend allocations. Focused validation target: `cargo test -p zircon_runtime --lib render_framework_stats_report_transient_allocation_bytes --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never -- --test-threads=1 --nocapture`.

The 2026-06-12 WGPU materialization slice consumes the neutral transient allocation plan inside `RenderGraphExecutionResources`: compatible dense texture slots share one WGPU texture backing, and buffer slots share one WGPU buffer backing with max size and unioned usage. `SceneRendererCore` now owns `TransientResourcePool`, so submitted frame resources are released into descriptor-keyed texture/buffer pools and reused on later frames. The 2026-06-18 descriptor-bucket follow-up moved incompatible texture/buffer separation into `CompiledRenderGraph::transient_allocation_plan()` itself by adding bucket hashes to allocations and slot reservations, then the execution materialization follow-up switched WGPU grouping and alias labels to `(bucket_key_hash, slot)`. Execution fallback for incompatible texture descriptors is now defensive rather than the normal allocation contract, and alias reports can distinguish two descriptor buckets that both use bucket-local slot zero. `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never` passed with existing warnings only for the original slice. Focused lib-test commands were blocked before running the filtered tests by unrelated `zircon_runtime` lib-test compile errors in `zircon_runtime/src/ui/tests/runtime_input_manager.rs` and `zircon_runtime/src/ui/tests/style_mapping.rs`; an earlier materialization attempt was also blocked by dirty `zircon_runtime/src/scene/tests/ecs_schedule.rs` test source.

The 2026-06-17 materialization validation slice adds `RenderGraphMaterializationReport` and validates live typed texture/buffer lifetimes before graph pass execution. It also exposes external lifetime counts as report/diagnostic evidence while leaving executor-owned external buffer hard cutover for a later graph model slice. The 2026-06-18 external coverage report follow-up splits hard-required and report-only external counts in that report and in diagnostics, preserving aggregate external bound/missing helpers while making `required_external_count` truthful. The 2026-06-18 stale-binding follow-up also rejects texture/buffer rows pre-bound under names absent from the compiled lifetime set and projects `stale_*_binding_count` diagnostics. `rustfmt --edition 2021` passed over touched Rust files; `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-materialization-0617` timed out once at 244 seconds during cold-target compilation, then passed on the warmed rerun in 188 seconds with the existing warning set. The focused `cargo test -p zircon_runtime --lib materialization_validation_reports_required_and_report_only_external_coverage_separately --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-external-coverage-report-0618 --message-format short --color never` passed on 2026-06-18 after clearing stale test-crate import and mesh snapshot projection drift. The scoped stale-binding check `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-stale-lifetime-validation-0618 --message-format short --color never` passed in 421.59 seconds. The direct `cargo test ... materialization_validation_rejects_stale` filter previously timed out after 904 seconds during lib-test compilation; the follow-up `cargo test ... --no-run` on the same target dir completed in 8m06s with the existing 52-warning set, and direct execution of `zircon_runtime-d071a300da0585cb.exe` passed the exact full-path stale texture and stale buffer filters in 1.41s and 1.43s respectively.

The 2026-06-17 HZB executor-owned external binding slice adds the execution-side bridge for the HZB occlusion external buffers described above. `rustfmt --edition 2021` passed over touched Rust files; `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-materialization-0617` timed out once at 364 seconds during cold-target compilation, then passed on the warmed rerun in 73.5 seconds with the existing warning set. The focused fallback materialization test was authored and left for the deferred test phase.

The 2026-06-17 required External binding contract slice adds `RenderGraphExternalResourceBinding` to graph declarations/lifetimes and propagates it through feature descriptors and pipeline lowering. Required HZB buffers now fail validation when unbound, while report-only externals preserve the existing imported target/history behavior. `rustfmt --edition 2021` passed over touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-external-binding-contract-0617 --message-format short --color never` passed with the existing warning set. Focused tests for required external validation, HZB descriptor binding, and HZB pipeline lifetime metadata are authored but deferred for the implementation-first phase.

The 2026-06-17 typed optional External ownership slice adds report-only texture and buffer bindings for optional external descriptors. Pipeline lowering preserves `report_only_texture()` and `report_only_buffer()` in compiled lifetimes, rejects same-name optional texture/buffer conflicts, and materialization validation reports missing typed optional resources without failing the dense materialization gate. `rustfmt --edition 2021` plus `--check` passed over the touched Rust files, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-typed-optional-external-0617 --message-format short --color never` passed with the existing warning set. The plugin consumer check for particles, Hybrid GI, and Virtual Geometry was attempted under `zircon_plugins/Cargo.toml`, but `--locked` stopped before compilation because `zircon_plugins/Cargo.lock` would need an update.

The 2026-06-18 split postprocess exposure dependency trim removed unused `EXPOSURE_CURRENT` read declarations from `post.depth-of-field`, `post.motion-blur`, and `post.blur`. Those WGSL entry points do not sample resolved exposure, and the false reads forced the graph to schedule the split passes after `post.exposure.resolve`. The focused `compile_keeps_split_postprocess_passes_before_exposure_when_they_do_not_sample_exposure` test passed in `D:\cargo-targets\zircon-runtime-rg-required-external-0618`, while scene-composite, color-LUT bake, and uber still declare real exposure reads.

The 2026-06-17 `RgResourceResolver` slice starts the broader resolver hard cutover by renaming the resolver API, making pass-declared access the gate before physical lookup, and moving product post-process required resource validation/input selection onto resolver-backed name access. Follow-up slices carry that resolver into `RenderPassGpuExecutionContext`, move shared GPU physical lookup helpers into `gpu/resource_lookup.rs`, split deferred scene bridge methods into `gpu/deferred.rs`, split sprite/preview-sky/UI/overlay bridge methods into `gpu/surface.rs`, and migrate depth-prepass, deferred G-buffer, deferred lighting, mesh-stage, TAA reactive-mask mesh, surface bridge, particle bridge, SSR-specific bridge, and root postprocess bridge lookups through that gate. The particle bridge slice also removes the particles plugin executor's redundant direct WGPU texture-view precheck; the SSR bridge slice adds optional resolver-backed texture/mip helpers for shared fallback slots, declares `light-list` on the SSR descriptor passes, and keeps mip target alias creation behind a declared graph write check. The root postprocess bridge slice adds `gpu/post_process/{effects,computed_resources,temporal,terminal}.rs`, moves stack/color-LUT/effect-chain/compute-resource/temporal/terminal lookup through the resolver helpers, and keeps root `gpu/post_process.rs` at 490 lines. The HZB/shadow/velocity follow-up moves HZB previous-history, shadow-atlas, and object-velocity texture lookups through resolver helpers, adds a declared-optional history helper for first-frame HZB fallback, and narrows raw texture/buffer lookup methods on `RenderGraphExecutionResources` to graph-execution scope. `rustfmt --edition 2021` and follow-up `--check` passes covered the touched Rust files across the resolver slices. The latest isolated `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-rg-resolver-cutover-0617-hzb-shadow-velocity --message-format short --color never` rerun passed with the existing 142-warning set after an earlier shared-target fingerprint write failure and a cold isolated-target timeout. The package-scoped particles plugin check still stops before compilation because `zircon_plugins/Cargo.lock` would need an update under `--locked`. Remaining broad direct lookup calls are helper fallback/internal calls by design; remaining RG-M1 implementation work is non-HZB/non-shadow-atlas executor-owned External actual binding, resource lifetime validation closure, focused resolver tests, plugin lockfile resolution, and RenderDoc resource/marker comparison.

The follow-up RG-M2 budget slice keeps that WGPU-side pool bounded by explicit byte caps. Returned textures now carry the same descriptor-derived storage estimate as graph planning (`TextureDesc::checked_storage_size_bytes()`), returned buffers carry `BufferDesc.size_bytes`, and `TransientResourcePool::end_frame()` evicts stale entries before applying least-recently-used budget eviction. `RenderGraphTransientPoolReport` now distinguishes stale evictions from budget evictions and exposes retained/budget bytes for texture and buffer pools. Runtime diagnostics mirror those rows under `render.graph.execution.transient_pool.*`, keeping graph planning reservations, current physical pool retention, and memory-pressure cleanup separate.

The follow-up RG-M4 alias/profile slice adds the runtime alias evidence and CPU command-recording profile described above. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-alias-profile-0617` initially timed out after 124 seconds on a cold target dir, then passed on a 301.3 second warmed rerun with existing warnings only. Focused alias/profile runtime diagnostics and RenderDoc marker/profile tests remain deferred for the implementation-first phase.

The 2026-06-17 graph dump slice added `render_graph/dump.rs`, `CompiledRenderGraph::dump()`, structured/text dump rows, and `CapturedFrame.graph_dump`. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-graph-dump-0617` passed with existing warnings only. The focused `render_graph_dump_lists_pass_order_resources_and_culled` test was authored but did not return within a 604 second shared lib-test compile window, so it remains queued for the validation phase.

The 2026-06-17 compiled graph cache slice added the submission-path `CompiledGraphCache`, pipeline revision invalidation, and `render.graph.compiled_cache.*` diagnostics. The follow-up fingerprint audit named `extract_compile_fingerprint(...)`, added `CompiledGraphCacheLookupStatus`, and made hit-path submissions debug-assert that the live frame fingerprint still matches the cache-key frame fingerprint before reusing the cached graph. `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-graph-cache-0617` passed with existing warnings only for the initial cache slice; the fingerprint follow-up has source-contract tests authored and scoped validation recorded in Plan 01. Focused compiled-cache tests were intentionally left unrun for the implementation-first phase.

The 2026-06-17 root-driven culling slice added `RenderGraphResourceUsageFlags`, explicit readback/persistent marking APIs, non-present external import support, `MissingCullRoot`, and usage text in graph dumps. The implementation removes the old `writes_external` / no-write culling roots from `RenderGraphBuilder::cull_passes(...)`, so only present/readback/persistent resources and pass-level roots keep producer chains alive. `rustfmt --edition 2021` passed for the touched Rust files. `git diff --check -- <root culling scoped files>` passed with only Git LF-to-CRLF notices. A cold-target `cargo check` attempt timed out after 604 seconds with no compiler diagnostics, then `cargo check -q -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-graph-cache-0617` passed with existing warnings only. Focused culling tests were authored but deferred for the implementation-first phase.
