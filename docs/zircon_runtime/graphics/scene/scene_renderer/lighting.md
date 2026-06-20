---
related_code:
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/shared_product_reports.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs
  - zircon_runtime/src/core/framework/render/light/shadow_settings.rs
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl
  - zircon_runtime/src/core/framework/render/post_process/stack.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_gpu_scene.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/asset/pipeline/manager/builtins/builtin_pbr_wgsl.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/execute_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/forward_shadow_receiver.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_bind_group_layout/create.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/shared_product_reports.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
plan_sources:
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - dev/bevy/crates/bevy_pbr/src/render/light.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
tests:
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs::tests::gpu_light_data_layout_matches_plan_05_std430_contract
  - zircon_runtime/src/core/framework/render/light/gpu_light.rs::tests::gpu_light_type_is_encoded_as_bits_for_wgsl_bitcast
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs::tests::pack_light_slices_preserves_all_point_lights_without_scene_uniform_limit
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs::tests::pack_light_slices_encodes_directional_shadow_and_layer_contract
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs::tests::pack_light_slices_encodes_spot_angles_and_rect_size
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene.rs::tests::render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_reads_gpu_light_buffer_for_all_builtin_light_types
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_receives_gpu_light_buffer
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_is_valid_wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_builder_marks_directional_light_across_all_tiles_and_bins
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_builder_culls_point_light_to_screen_and_depth_ranges
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_builder_increases_tile_size_to_fit_mask_budget
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_builder_zbin_header_tracks_min_and_max_light_indices
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs::tests::light_grid_shader_include_is_valid_wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs::tests::clustered_lighting_declares_light_grid_build_outputs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs::tests::fallback_mesh_shader_receives_light_grid_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs::deferred_lighting_shader_receives_light_grid_resources
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs::tests::update_light_grid_stats_records_latest_grid_report
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/light_grid_stats.rs::tests::update_light_grid_stats_resets_when_no_report
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/product.rs::tests::render_product_diagnostics_record_light_grid_stats
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_many_point_lights
  - zircon_runtime/src/graphics/tests/render_product_shadows.rs::render_product_many_point_lights_forward_deferred_capture_parity
---

# Scene Renderer Lighting

Plan 05 LS-M1 moves authored lights out of the fixed scene-uniform contract
and into a GPUScene-owned light storage buffer. Framework snapshots carry
stable `light_id`, `layer_mask`, and optional `LightShadowSettings`;
`lighting/light_buffer.rs` packs directional, point, spot, and rect snapshots
into the 96-byte `GpuLightData` ABI; and `build_mesh_draws` writes the packed
array into `GpuScene` before the existing GPUScene flush.

`GpuLightData` is a framework POD contract, not a WGPU object. Its layout is
kept in `core/framework/render/light/gpu_light.rs` so forward, deferred,
clustered, and shadow code can share the same ABI. The graphics-side packer is
WGPU-free and intentionally preserves every point light in the extract; it does
not apply the old `BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT` truncation.
`LightShadowSettings` now also carries `pcf_quality`; the light-buffer packer
continues to place strength/bias in `GpuLightData.shadow_params`, while
`shadow/plan.rs` copies the per-light PCF quality into shadow slot flags for
shader-side 1/5/9 tap selection.

GPUScene now owns a real `light_buffer` capacity, CPU mirror, and upload dirty
flag. Capacity grows by powers of two from one light, rebinding the GPUScene
bind group when the storage buffer is replaced. Light uploads are whole-buffer
for LS-M1 because light counts are small relative to instance data; per-light
dirty ranges can be added later if profiling proves it matters. The GPUScene
remap params uniform now also carries the active light count, so shaders can
loop only authored lights even when the storage buffer has spare capacity.

The built-in forward fallback shader, deferred lighting shader, and built-in PBR
shader now read the same GPUScene `GpuLightData` buffer through a shared
light-grid path. Directional lights use the existing single-shadow receiver
when the packed light marks `casts_shadow`, while point, spot, and rect lights
use range attenuation and type-specific cone or facing filters. Deferred
lighting binds the same group3 GPUScene layout used by mesh passes.

`SceneUniform` no longer contains directional, point, or spot light payloads;
it keeps camera matrices, ambient color, and motion state. Non-lighting WGSL
consumers that only need a camera matrix now declare the smaller prefix they
read. `RenderLightReadinessReport` no longer mirrors scene-uniform limits:
directional, point, and spot lights are ready according to the GPU light-buffer
path, while ambient and rect readiness still preserve their existing degraded
flags.

Plan 05 LS-M2 now has the clustered-grid foundation in this module.
`lighting/light_grid_builder.rs` builds a URP-style zbin + tile mask grid on
the CPU from packed `GpuLightData`, camera projection data, and viewport size.
It emits `LightGridParams`, zbin headers, tile bitmasks, and coarse occupancy
stats while respecting the fixed 4096-word zbin and 8192-word tile-mask
budgets. Directional lights cover every tile and bin; point, spot, and rect
lights use the same packed range field and a view-space sphere approximation
for their first grid pass.

`lighting/light_grid_pass.rs` is the graph-facing upload shim. The
`light-grid-build` feature descriptor now declares `LIGHT_GRID_PARAMS`,
`LIGHT_ZBINS`, and `LIGHT_TILE_MASKS` transient buffer writes, and the runtime
executor writes those buffers before keeping the old `LIGHT_LIST` write alive
for the legacy tile-tint/post-process path. `zr_light_grid.wgsl` contains the
shared grid query helpers, and forward fallback, deferred lighting, and built-in
PBR include it before their entry-point shader. Fragment lighting computes
view-space z, resolves a zbin and tile base, intersects zbin and tile words, and
iterates only set light bits with `firstTrailingBit(mask)`.

Forward group1 and deferred lighting group1 both reserve bindings 20/21/22 for
light-grid params, zbin storage, and tile-mask storage. Runtime mesh passes bind
the graph-produced buffers; motion-vector and overlay paths bind disabled/empty
fallback buffers so they can keep the shared layout without requiring a
light-grid build pass.

`LightGridStats` now flows through `RenderGraphLightGridReport`, the last
render graph execution record, `SceneRenderer::last_light_grid_report()`, and a
submit-side `SharedViewportProductReports` snapshot before `RenderStats` is
updated. That keeps light-grid diagnostics on the same named shared viewport
product owner as `RenderStats`, graphics-debugger capture, and the last
virtual-geometry debug snapshot inside Plan 09 camera-stack submits. Product
diagnostics emit `render.light_grid.*` samples for reported state,
light/tile/zbin counts, non-empty tile/zbin/cluster counts, peak lights per
cluster, and average lights per cluster. LS-M3 still owns shadow atlas/CSM
replacement.

`render_product_many_point_lights` locks the current many-light product
contract at the source and graph level. It builds a 64 point-light extract,
asserts the packer preserves every point light, checks the light grid crosses
into a second 32-bit mask word for zbin and tile data, and verifies default
Forward+ mesh passes plus Deferred lighting all consume the graph-owned
`LIGHT_GRID_PARAMS`, `LIGHT_ZBINS`, and `LIGHT_TILE_MASKS` buffers.

`render_product_many_point_lights_forward_deferred_capture_parity` closes the
same contract at the real WGPU product-capture layer. The test registers a lit
PBR material, renders the same cube scene through Forward+ and Deferred
baseline/64-point-light paths, asserts `lighting.light-grid` execution and 64
light-grid stats, then compares center-region luma so both pipelines prove a
visible many-light contribution in the final captured image.

Latest validation:
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-compact-replay-coremin --message-format short --color never` passed with the repository warning set.
- A temporary Naga validator parsed and validated the concatenated fallback
  mesh, deferred lighting, and built-in PBR WGSL sources.
- Filtered `cargo test` attempts for the same library test target timed out
  during Windows lib-test code generation, so no new focused test binary result
  is claimed for this slice.
- `cargo fmt --all`, `cargo fmt --all -- --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-many-point-contracts-coremin --message-format short --color never` passed after adding the 64 point-light product source contract; the check reported the repository's existing warning set. The matching lib-test target compiled with `cargo test -p zircon_runtime --lib render_product_many_point_lights --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-vc3-many-point-contracts-coremin --message-format short --color never --no-run`, and direct execution of `zircon_runtime-5d2828c2001649f6.exe render_product_many_point_lights --nocapture` passed 1 filtered test.
- The 2026-06-21 many-point real capture parity slice passed `rustfmt --edition 2021 zircon_runtime\src\graphics\tests\render_product_shadows.rs` and `cargo test -p zircon_runtime --lib render_product_many_point_lights_forward_deferred_capture_parity --no-default-features --features core-min --locked --jobs 1 --target-dir target\codex-runtime-hzb-storage-limit-0620 --message-format short --color never -- --test-threads=1 --nocapture` with 1 filtered test. A cold target-dir Cargo wrapper timed out during shared lib-test compilation before producing a binary; the warmed target-dir run is the counted result. Direct exact runs from the same binary also passed `render_product_many_point_lights`, `render_product_csm_directional`, and `render_product_multi_spot_shadows`.
- The LS-M4 PCF quality authoring-field update passed `cargo fmt --all -- --check`, scoped `git diff --check`, and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-ls-m4-pcf-coremin --message-format short --color never` with existing warnings. The matching `pcf_quality` lib-test no-run command timed out after 904 seconds during shared test-target compilation, so no filtered test result is claimed for this lighting-adjacent field update.
- The 2026-06-20 Plan 09 shared light-grid product-report boundary passed scoped `rustfmt --edition 2021 --check` and `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-light-grid-shared-products-0620 --message-format short --color never` with the repository warning set. The follow-up direct lib-test binary filter `light_grid_stats --test-threads=1 --nocapture` passed 3 tests, covering stats update, reset, and product diagnostics rows.
