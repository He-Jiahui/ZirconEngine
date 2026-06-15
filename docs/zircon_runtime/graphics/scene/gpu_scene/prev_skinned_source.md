---
related_code:
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/previous_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
implementation_files:
  - zircon_runtime/src/graphics/scene/gpu_scene/prev_skinned_source.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/previous_skinned_palette.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/mesh_pass_batch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/replay.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
plan_sources:
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/zircon_runtime/graphics/scene/scene_renderer/temporal/velocity.md
tests:
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime previous_skinned --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime velocity_mesh_pipeline_declares_previous_position_vertex_slot --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime gpu_mesh_previous_position_layout --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime mesh_batch_ref_attaches_previous_geometry_only_to_velocity_commands --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
  - cargo test -p zircon_runtime velocity_geometry_bind_key_includes_previous_geometry_slot --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-temporal-s4d-0614 --message-format short --color never
doc_type: module-detail
---

# GPUScene Previous Skinned Source

This module is the renderer-owned previous-source surface for Plan 06 CPU-morphed skinned GPU velocity. It stores only CPU-morphed, morphed-but-unskinned `GpuMeshResource` sources because ordinary prepared skinned meshes can reuse their current geometry for previous-position input.

`stage_current_skinned_gpu_source(...)` is called while mesh draws are synchronized into GPUScene. A live CPU-morphed draw stages the current source mesh plus its `morph_shape_signature`; a draw without that source removes the current entry for its stable instance key. `roll_prev_skinned_gpu_sources_after_success(...)` runs only after a successful render submission and copies the current source map into the previous source map for the next frame. That mirrors the existing previous-transform and previous skinned-palette roll policy: the current frame never mutates the previous source values that its velocity pass can still read.

`previous_skinned_gpu_source_state(...)` is consumed by `build_mesh_draws/build/previous_skinned_palette.rs`. When the current and previous morph signatures match, the current morphed source remains a valid previous shape. When they differ, the previous source state must exist and its signature must match the previous palette state's signature before the draw receives a previous palette and previous source. Missing or mismatched state keeps the draw out of GPU-skinned object velocity and preserves the existing previous-shape missing diagnostic.

The velocity draw path binds this previous source as a second vertex buffer only for `MeshPassPipelineKind::Velocity`. `GpuMeshVertex::previous_position_layout()` exposes the source position at shader location 8, `fallback_mesh.wgsl` reads it as `previous_position`, and `MeshDrawCommand::geometry_bind_key()` includes the previous geometry id so pass replay cannot skip rebinding when two consecutive velocity commands share current geometry but use different previous geometry.

Current tests cover successful source rolling, stale source removal, previous palette/source selection for changed CPU-morphed shapes, the velocity pipeline previous-position slot, the GPU mesh layout contract, velocity-only previous geometry attachment, and replay bind-key identity.
