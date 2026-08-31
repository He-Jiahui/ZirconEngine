---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_camera.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/shaders/velocity_camera.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/skinning/joint_palette_storage.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/temporal_frame_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/builtin_postprocess_executors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/previous_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_vertices/build_particle_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle_velocity.wgsl
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/particle_previous_sprites.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_morph_weights.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_camera.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/execute_velocity_object.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/shaders/velocity_camera.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/temporal_frame_index.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/build.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/viewport_record_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit_runtime_frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry.rs
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/previous_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/prepared_queue.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/particle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/hzb/shaders/hzb_occlusion_cull.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/camera_matrices/view_projection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_reflection_probes/encode.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/post_process_screen_space_reflection.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_runtime/src/tests/runtime_diagnostics/motion_vector.rs
  - zircon_runtime/src/tests/runtime_diagnostics/support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/particle_renderer/record_velocity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/particle/shaders/particle_velocity.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline/render_frame_with_pipeline.rs
  - zircon_runtime/src/graphics/backend/render_backend/read_texture_rgba.rs
  - zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph/skinned_velocity.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - user: 2026-06-14 implement WGPU render pipeline architecture code and update plan progress
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-velocity-0614 --message-format short --color never
  - cargo test -p zircon_runtime pipeline_compile --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-velocity-0614 --message-format short --color never
  - cargo test -p zircon_runtime compile_options_fallback_async_compute_passes_to_graphics_queue --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-velocity-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S3 touched Rust files
  - source scan for retired CPU object-history symbols
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-cpu-history-cut-0614 --message-format short --color never (initial pass succeeded; final rerun blocked by unrelated UI BeginEdit match drift)
  - rustfmt --edition 2021 --check on TP-M1-S4 skinned previous-palette files
  - source scan for GPUScene previous skinned-palette staging/rolling symbols
  - rustfmt --edition 2021 --check on TP-M1-S5 temporal velocity naming files
  - source scan for retired object motion-vector internal symbols
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S6 CPU-morphed previous-shape diagnostic files
  - source scan for skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count and RenderStats surface
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S7 CPU-morphed stable morph-shape files
  - source scan for morph_shape_signature and PendingSkinnedGpuSource::CpuMorphed
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never
  - cargo test -p zircon_runtime --lib morph_shape_signature --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never -- --test-threads=1 --nocapture (timed out after 244s without test output)
  - rustfmt --edition 2021 --check on TP-M1-S8 particle velocity diagnostic files
  - source scan for last_particle_velocity_missing_sprite_count, particle_velocity_missing_sprite_count, and particle_sprite_count
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never
  - cargo test -p zircon_runtime --lib particle_velocity_gap_counts_sprites_only_when_reconstructed_velocity_is_requested --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never -- --test-threads=1 --nocapture (timed out after 304s without test output; leftover cargo/rustc processes terminated)
  - zircon_runtime/src/core/framework/render/frame_extract.rs::tests::particle_extract_counts_previous_state_by_entity
  - zircon_runtime/src/core/framework/render/frame_extract.rs::tests::particle_extract_consumes_duplicate_entity_previous_state_once_per_row
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs::tests::particle_velocity_gap_excludes_sprites_with_previous_state
  - zircon_runtime/src/graphics/tests/render_product_anti_alias.rs::render_product_particle_previous_state_suppresses_velocity_gap_stats
  - rustfmt --edition 2021 --check on TP-M2-S1a jitter contract and scene uniform files
  - source scan for TemporalJitterSample, TemporalJitterSequence, ViewProjectionMatrixPair, temporal_jitter, and SceneUniform::from_frame
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never (blocked by unrelated UI tree-view helper compile errors)
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/frame_submission_context.rs::tests::render_taa_jitter_zero_when_taa_inactive
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_temporal_camera_history.rs::tests::successful_submit_records_camera_history_for_next_frame
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::tests::scene_uniform_exposes_jittered_and_unjittered_current_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs::tests::scene_uniform_inverse_view_projection_is_unjittered
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs::tests::render_velocity_camera_params_use_unjittered_camera_matrices
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/build_post_process_params/build.rs::tests::post_process_projection_params_ignore_temporal_jitter
  - cargo fmt --package zircon_runtime
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never
  - cargo fmt --package zircon_runtime -- --check
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s2-0614 --message-format short --color never
  - zircon_runtime/src/core/framework/render/post_process/stack.rs::tests::taa_resolve_declares_history_velocity_and_final_composite_input
  - zircon_runtime/src/graphics/tests/pipeline_compile/temporal_and_ops.rs::taa_resolve_compiles_temporal_history_pass_when_taa_stack_is_effective
  - cargo test -p zircon_runtime particle_velocity_vertices --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S11 particle stable identity files
  - constructor scan for RenderParticleSpriteSnapshot/RenderParticlePreviousSpriteSnapshot stable_sprite_key coverage
  - cargo test -p zircon_runtime stable_sprite_key --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked before filtered tests by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - rustfmt --edition 2021 --check on TP-M1-S12 renderer-owned particle previous-state files
  - cargo test -p zircon_runtime successful_submit_records_particle_previous_state_for_next_frame --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (blocked before filtered tests by unrelated ui/surface/render/command_palette.rs:311 E0282)
  - rustfmt --edition 2021 --check on TP-M1-S13 particle previous billboard-basis files
  - constructor scan for RenderParticlePreviousSpriteSnapshot billboard_basis coverage
  - cargo test -p zircon_runtime previous_billboard_basis --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::runtime::render_framework::submit_frame_extract::submit::update_particle_previous_state::tests::successful_submit_records_particle_previous_state_for_next_frame --exact --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - rustfmt --edition 2021 --check on TP-M1-S14 scene-velocity readback files
  - cargo test -p zircon_runtime execution_record_preserves_scene_velocity_readback_report --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::tests::render_product_particle_velocity::render_product_particle_velocity_writer_writes_nonzero_scene_velocity_pixels --exact --nocapture
  - D:\cargo-targets\zircon-runtime-temporal-s4d-0614\debug\deps\zircon_runtime-5d2828c2001649f6.exe graphics::tests::render_product_particle_velocity --nocapture
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S15 renderer-owned second-frame readback baseline)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S16 same-entity keyed multi-sprite readback baseline)
  - cargo test -p zircon_runtime anonymous_stream --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S17 key=0 anonymous stream diagnostics)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S17 anonymous key product diagnostic)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S18 keyed multi-sprite three-frame dynamic baseline)
  - cargo test -p zircon_runtime previous_skinned_joint_palette --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S19 previous-palette gate module split)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S19 module split)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs (TP-M1-S20 keyed stress field clean rerun)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S20 production check)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1-S20 stress product route; passed 8 filtered product tests after ui/surface/input/editable_text/ime_context.rs covered UiInputEvent::ToastTimer)
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/frame_extract.rs zircon_runtime/src/core/framework/render/frame_extract/particle_extract_policy.rs zircon_runtime/src/graphics/scene/scene_renderer/particle/build_particle_velocity_vertices/build_particle_velocity_vertices.rs zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/update_particle_previous_state.rs zircon_runtime/src/scene/world/render_particles.rs zircon_runtime/src/graphics/tests/render_product_particle_velocity.rs (TP-M1/S22 key=0 hard enforcement)
  - cargo test -p zircon_runtime particle_extract --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 ParticleExtract policy; passed 4 filtered tests)
  - cargo test -p zircon_runtime particle_velocity_vertices --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 velocity builder policy; passed 6 filtered tests)
  - cargo test -p zircon_runtime successful_submit --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 renderer-owned previous cache; passed 3 filtered tests)
  - cargo test -p zircon_runtime scene::world::render_particles::tests::world_hud_bar --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 world-HUD producer key migration; passed 2 filtered tests)
  - cargo test -p zircon_runtime render_product_particle_velocity --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 key=0 hard-reject product route; passed 9 filtered product tests)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S22 production check; passed with 70 existing warnings)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 CPU-morphed changing-shape previous source velocity; passed with existing warnings)
  - cargo test -p zircon_runtime previous_skinned --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 previous source/palette policy; passed 6 filtered tests)
  - cargo test -p zircon_runtime velocity_mesh_pipeline_declares_previous_position_vertex_slot --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 velocity pipeline previous-position slot; passed 1 filtered test)
  - cargo test -p zircon_runtime gpu_mesh_previous_position_layout --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 mesh vertex previous-position layout; passed 1 filtered test)
  - cargo test -p zircon_runtime mesh_batch_ref_attaches_previous_geometry_only_to_velocity_commands --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 velocity-only previous geometry handoff; passed 1 filtered test)
  - cargo test -p zircon_runtime velocity_geometry_bind_key_includes_previous_geometry_slot --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never (TP-M1/S23 replay bind key includes previous geometry; passed 1 filtered test)
  - cargo check -p zircon_runtime --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never
  - cargo test -p zircon_runtime render_product_direct_mesh_morph_weight_change_writes_scene_velocity_pixels --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime render_product_skinned_mesh_morph_weight_change_writes_scene_velocity_pixels --lib --no-default-features --features target-server --locked --jobs 1 --target-dir E:\cargo-targets\zircon-plan08-morph-velocity-product-0701 --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Temporal Velocity

The temporal velocity module owns the camera velocity parameter contract and the camera velocity draw path for Plan 06 TP-M1-S2. The render graph feature descriptor lives in `feature_descriptors/temporal.rs` and declares two DepthPrepass passes:

- `velocity-object` with executor id `temporal.velocity-object`; it reads `SCENE_DEPTH` and writes `SCENE_VELOCITY` with `clear_store`.
- `velocity-camera` with executor id `temporal.velocity-camera`; it reads `SCENE_DEPTH` and writes `SCENE_VELOCITY` with `load_store`.

`SCENE_VELOCITY` is the graph-owned raw velocity buffer named `scene-velocity`. The pipeline compiler assigns it `TextureFormat::Rg16Float`. Motion blur and SSR still use the existing post-process tile/max chain, but the first tile-max pass now reads `SCENE_VELOCITY` instead of the removed `SCENE_MOTION_VECTOR` name.

## Execution

`VelocityCameraParams` is the renamed camera reprojection uniform. It preserves the previous cut-detection thresholds for projection mode, viewport, dynamic resolution, FOV, clip planes, translation, rotation, and finite matrix validation. Invalid or cut frames clear the velocity target and return the existing `MotionVectorCameraStatus` value used by runtime diagnostics.

`ScenePostProcessResources::execute_velocity_camera(...)` stays on the existing post-process resource bundle for this slice, because the WGPU pipeline, bind group layout, and uniform buffer are still allocated by that bundle. The execution method, params type, and WGSL shader source now live under `scene_renderer/temporal/velocity`; the post-process resource bundle only owns the WGPU pipeline/layout allocation as `velocity_camera_*`.

The object path records through `record_velocity_object_to_resource(...)` in `execute_velocity_object.rs` and is registered under `temporal.velocity-object`. It begins its render pass before checking the velocity stream, so the `clear_store` declared by the graph clears `SCENE_VELOCITY` even when no object velocity draws are emitted in the frame. The mesh pipeline and cache are named `create_velocity_mesh_pipeline.rs` and `ensure_velocity_pipeline.rs`; the pass kind is `MeshPassPipelineKind::Velocity`.

Object previous transform state is no longer copied through a CPU viewport history sideband. `build_mesh_draws` reads only the GPUScene rolled previous transform (`previous_world_from_local(...)`) when deciding whether a mesh draw has previous object motion data. Successful submit updates the temporal camera snapshot through `update_temporal_camera_history_after_success(...)`; object history is owned by GPUScene's previous-transform roll.

Skinned previous-palette and CPU-morphed previous-source state are now owned by GPUScene. `build_mesh_draws` stages the current `SkinnedMeshJointPaletteStorage` with a skeleton-derived compatibility signature and reads the previous staged state from GPUScene on the next frame. The matching WGPU resource owner keeps two storage buffers per stable instance: current writes target the non-committed slot, velocity binds the committed slot, and only successful submission swaps them. CPU-morphed sources also stage the current morphed-but-unskinned `GpuMeshResource` behind the same stable instance key, so a later frame can recover the exact previous source mesh when morph weights change. The previous-palette selection gate lives in `build_mesh_draws/build/previous_skinned_palette.rs`, keeping skeleton signature, joint-count, and CPU-morphed morph-shape compatibility checks out of the draw orchestration file. A skinned GPU-skinning draw receives `previous_skinned_joint_palette` when the previous/current skeleton signatures and joint counts match; CPU-morphed sources additionally require a current morph-shape signature plus either a matching previous morph-shape signature or a matching previous source mesh rolled by GPUScene.

CPU-morphed GPU-skinning sources now have an explicit previous-shape policy. `PendingSkinnedGpuSource::CpuMorphed` carries a `morph_shape_signature` built from mesh id and morph weights. GPUScene rolls that optional signature with the previous skinned-palette state and rolls the previous source mesh in `prev_skinned_source.rs` after successful submit. When current and previous signatures match, the current morphed-but-unskinned source mesh is a valid previous shape. When the signature changes, the previous source mesh is used as the velocity pass's second vertex buffer, and `fallback_mesh.wgsl` reads `@location(8) previous_position` for `skin_previous_vertex_position(...)`. `MeshDrawCommand::geometry_bind_key()` includes that previous geometry id for velocity commands, so replay rebinding is correct even when the current geometry id is unchanged. Missing signatures or missing previous source state remain diagnosed through `skinned_gpu_cpu_morphed_previous_shape_velocity_missing_count`.

GPU-morphed Velocity sources now have a matching previous-weight policy. `gpu_scene/prev_morph_weights.rs` stages current morph source weights during draw sync and rolls them after successful submit. `PendingMeshDraw.source_morph_weights` keeps direct mesh history staging independent from active payload creation, so a first frame with explicit `[0.0]` source weights is retained for a later nonzero velocity frame. The next frame's `morph_payload_upload.rs` writes current weights at `GpuMorphPayload.weight_base` and previous weights at `weight_base + target_count`, preserving previous-only targets when current weights return to zero. `zr_geometry_morphed.wgsl` and `zr_geometry_skinned_morphed.wgsl` use that previous block in `fetch_prev_position(...)`; skinned-morphed velocity reconstructs the previous morphed position before applying the previous skin matrix. The focused product guards `render_product_direct_mesh_morph_weight_change_writes_scene_velocity_pixels` and `render_product_skinned_mesh_morph_weight_change_writes_scene_velocity_pixels` cover direct and skinned 0.0 -> 1.0 morph-weight changes through forward-plus/Core3d WGPU frames and verify nonzero `scene-velocity` readback. RenderDoc capture and broader product miss=0 acceptance remain separate Plan 08 gates.

Particle billboards now have a V1 `scene-velocity` writer. `RenderParticleSpriteSnapshot` carries current-frame billboard inputs plus `stable_sprite_key`, and `RenderParticlePreviousSpriteSnapshot` can carry the previous position, size, aspect ratio, billboard offset, rotation, matching key, and optional `RenderParticleBillboardBasisSnapshot` for the same emitter identity. `RenderParticleSpriteIdentity { entity, stable_sprite_key }` is the shared match key for `ParticleExtract::previous_state_sprite_count()` and `build_particle_velocity_vertices(...)`; key `0` remains the anonymous compatibility stream for single-sprite producers, while nonzero keys let same-entity emitters avoid cross-matching previous rows. `ParticleExtract` owns the key `0` policy in `frame_extract/particle_extract_policy.rs`: when a frame contains more than one key `0` sprite for the same entity, those sprites are counted as anonymous-stream ambiguity, excluded from previous-state matching, and therefore remain missing for velocity. `ViewportRecord` now owns renderer-generated previous particle rows, populated from current sprites after successful submit/present/direct runtime-frame paths, records the submitted camera right/up basis into those rows, and skips ambiguous anonymous rows so an invalid key `0` stream cannot seed the next frame. `FrameSubmissionContext` injects renderer-owned rows into runtime frames unless the incoming extract already supplies explicit previous rows. `particle.velocity` is a built-in graph executor that writes graph `SCENE_VELOCITY` through `ParticleRenderer::record_velocity(...)`; it expands matched current/previous billboards into paired corner vertices, uses the previous row's stored basis when available, projects current corners with the unjittered current matrix, projects previous corners with the unjittered previous matrix, and skips unmatched or ambiguous anonymous sprites rather than writing fabricated zero motion. Explicit previous rows without stored basis still fall back to the current camera basis. `FrameSubmissionContext` carries current particle sprite count, matched previous-state count, and key `0` anonymous ambiguity count into submit stats; `update_base_stats(...)` publishes `last_particle_velocity_missing_sprite_count` and `last_particle_velocity_anonymous_stream_ambiguity_count` only when reconstructed velocity is requested by motion blur or SSR and `particle.transparent` actually executes. Runtime diagnostics mirror these as `render.particle.velocity.missing_sprite_count` and `render.particle.velocity.anonymous_stream_ambiguity_count`. Test builds also attach `RenderSceneVelocityReadbackReport` to `RenderStats.last_scene_velocity_readback_report`, reading the graph-owned `Rg16Float` velocity surface after submission and proving explicit previous rows, renderer-owned second-frame rows, same-entity nonzero-key multi-sprite rows, and a four-sprite three-frame keyed motion sequence produce nonzero raw velocity pixels; the anonymous key product tests prove same-entity key `0` multi-sprite streams are diagnosed and hard-rejected instead of silently treated as stable. The 32-sprite same-entity nonzero-key stress-field product test locks the first-frame missing count and second-frame nonzero readback path, and world-HUD bar extraction now emits nonzero stable keys for its generated background/fill sprites. RenderDoc validation remains follow-up work.

The TP-M2 jitter data slice now spans submit, scene uniform upload, and velocity. `ViewportCameraSnapshot` carries `temporal_jitter`, `TemporalJitterSequence` provides Halton(2,3) sample generation, and `FrameSubmissionContext` chooses a nonzero sample only when the effective anti-aliasing mode is TAA. The sample is keyed by `ViewportRecord.temporal_frame_index`, which advances after successful submit/present paths while failures keep the same sample for the next attempt.

`ViewProjectionMatrixPair::from_camera(...)` builds current jittered and current unjittered projection matrices from the camera plus viewport size. `SceneUniform::from_frame(...)` writes the current jittered `view_proj`, explicit `view_proj_unjittered`, unjittered `inverse_view_proj`, `previous_view_proj_unjittered`, and `jitter_params`; fallback/deferred/HZB/builtin PBR shader layouts match that ABI. Velocity object WGSL, `VelocityCameraParams`, deferred lighting reconstruction, HZB reprojection, and post-process SSR/reflection-probe projection now stay on unjittered matrices or equivalent unjittered camera scalars, so TAA jitter does not create false motion or unstable screen-space reconstruction.

## TAA Resolve Contract

TP-M3-S1a extends the same `temporal` feature descriptor with `taa-resolve` and executor id `temporal.taa-resolve`. The pass is retained only for an effective TAA post-process stack. It consumes raw `SCENE_VELOCITY` directly and does not request the tile-max/coarse/neighbor reconstructed motion-vector chain. The detailed TAA resource contract is documented in `taa.md`.

## Remaining Work

The old post-process producer pass ids, old `SCENE_MOTION_VECTOR` resource name, CPU `ViewportMotionVectorObjectHistory` submit path, object-pass `motion_vector` internal file/function names, CPU-morphed previous-shape diagnostic ambiguity, hidden particle velocity gap, renderer-owned particle previous-basis gap, raw particle velocity readback gap, and changing morph-weight previous-shape velocity gap are removed. Particle velocity now has a graph-executed WGPU writer, stable identity matching, renderer-owned previous rows, renderer-owned previous billboard basis, test-build nonzero `scene-velocity` pixel evidence for explicit matched sprites, renderer-owned second-frame motion, same-entity nonzero-key multi-sprite motion, three-frame keyed dynamic motion, a validated 32-sprite keyed stress-field product baseline, key `0` anonymous-stream ambiguity diagnostics, hard rejection for ambiguous key `0` streams, and world-HUD producer nonzero-key migration. CPU-morphed skinned GPU velocity now uses GPUScene's renderer-owned previous source mesh when morph weights change, and GPU-morphed velocity now uses GPUScene's renderer-owned previous morph-weight block for direct and skinned previous-position reconstruction, with focused WGPU readbacks for both 0.0 -> 1.0 morph cases. TP-M2-S1b completed TAA-state jitter injection, scene uniform ABI expansion, velocity unjittered reads, and temporal frame index/history roll; TP-M2-S2 completed upstream projection-matrix auditing, and TP-M2/S21 found no recoverable in-repo historical pre-jitter hash artifact. The remaining temporal work is RenderDoc/product velocity acceptance.

Validation note for TP-M1-S3 through TP-M2-S2: the first scoped `cargo check` over `zircon_runtime` core-min passed after the CPU history cut. A final TP-M1-S3 rerun after docs/formatting was blocked before renderer diagnostics by unrelated UI work in `zircon_runtime/src/ui/surface/surface/default_interactions.rs`, where `UiComponentKeyboardAction::BeginEdit` is not covered by an active match. The skinned previous-palette slice has scoped rustfmt/check and static source scans. Its first cargo rerun exposed two local visibility errors (`E0364` private max-joint constant re-export and `E0603` private mesh module access), both fixed. The S4 final rerun, S5 naming/file-ownership slice, S6 CPU-morphed previous-shape diagnostic slice, S7 stable morph-shape slice, and S8 particle velocity diagnostic slice all pass `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-skinned-prev-palette-0614 --message-format short --color never` with 65 existing warnings. The focused S7 filtered `cargo test` command timed out during Windows lib-test build without output; the focused S8 filtered `cargo test` command timed out after 304s without output and its leftover cargo/rustc processes were terminated. The S9 particle previous-state contract passes `particle_extract_counts_previous_state_by_entity`, `particle_velocity_gap_excludes_sprites_with_previous_state`, and `render_product_particle_previous_state_suppresses_velocity_gap_stats` under `D:\cargo-targets\zircon-runtime-temporal-s4d-0614`, plus a fresh core-min `cargo check` with 70 existing warnings. TP-M2-S1a rustfmt/source scans pass. TP-M2-S1b passes `cargo fmt --package zircon_runtime`, `cargo fmt --package zircon_runtime -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never` with 65 existing warnings. `cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s1b-0614 --message-format short --color never` is blocked by unrelated UI table test/interaction drift: `table_pointer_routes.rs` expects removed `UiInputDispatchDiagnostics.capture_started/capture_released` fields, and `default_interactions/table.rs` borrows `field` after move. TP-M2-S2 passes `cargo fmt --package zircon_runtime -- --check` and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s2-0614 --message-format short --color never` with 65 existing warnings. TP-M1/S23 passes the core-min `cargo check` in `D:\cargo-targets\zircon-runtime-temporal-s4d-0614` plus focused filters for `previous_skinned`, velocity pipeline previous-position layout, GPU mesh previous-position layout, velocity-only previous geometry handoff, and geometry bind-key replay.
