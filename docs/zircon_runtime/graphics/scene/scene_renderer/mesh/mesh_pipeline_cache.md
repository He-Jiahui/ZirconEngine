---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - tools/zircon_build.py
  - zircon_runtime/src/core/framework/render/shader/variant_miss_report.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/shader_variant.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/staged_prewarm.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_pass_processor.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_velocity_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_taa_reactive_mask_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/shader/template/deferred_gbuffer.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/deferred_scene_resources/record_gbuffer_geometry.rs
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm/main.rs
  - tools/zircon_build.py
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_reuses_pass_pipeline_shape_id
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_separates_pass_and_pipeline_shape
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_derives_material_shader_variant_key
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_maps_depth_prepass_to_normal_gbuffer_template
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_maps_deferred_gbuffer_to_gbuffer_pass_type
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/mesh_draw_command_list.rs::tests::mesh_pass_command_buffers_assign_cache_variants_by_pipeline_kind
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_deferred_gbuffer_template_source_writes_albedo_and_material_targets
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_depth_prepass_template_source_writes_normal_target
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_gbuffer_pipeline.rs::tests::gbuffer_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs::tests::gbuffer_mesh_pipeline_declares_albedo_material_targets_and_static_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs::tests::gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_depth_prepass_pipeline.rs::tests::depth_prepass_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs::tests::depth_prepass_mesh_pipeline_declares_normal_target_template_entries_and_static_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs::tests::depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs::tests::velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_velocity_mesh_pipeline.rs::tests::velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs::tests::taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_taa_reactive_mask_mesh_pipeline.rs::tests::taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source.rs::tests::mesh_pipeline_shadow_template_source_uses_shadow_pass_surface_only_when_alpha_masked
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/ensure_shadow_pipeline.rs::tests::shadow_mesh_shader_key_includes_shader_variant_identity_and_source_hash
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs::tests::shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_shader_template_assembly/gbuffer_cache.rs::runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-render-main-chain --message-format short --color never
  - rustfmt --edition 2021 zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616 (2026-06-17 PipelineKey to ShaderVariantKey bridge slice: passed with existing warnings)
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs::tests::mesh_pipeline_variant_registry_counts_variant_misses_and_memory_hits
  - zircon_runtime/src/graphics/shader/variant_cache/disk.rs::tests::render_shader_variant_cache_hits_disk_after_restart
  - zircon_runtime/src/graphics/shader/variant_cache/prewarm.rs::tests::render_shader_variant_prewarm_writes_disk_entries
  - rustfmt --edition 2021 on Plan 08 MS-M4-S1b touched files (2026-06-17 shader variant disk cache slice: passed)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617 (2026-06-17 shader variant disk cache slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
  - cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617 (2026-06-17 shader prewarm slice: passed with existing warnings)
doc_type: module-detail
---

# Mesh Pipeline Cache

`MeshPipelineCache` owns built-in mesh WGPU pipeline state for the scene renderer. It keeps shader modules, forward base mesh pipelines, deferred GBuffer pipelines, depth-prepass normal-target pipelines, velocity pipelines, TAA auxiliary pipelines, shadow pipelines, and the pass/pipeline variant registry used by the MD-M1 mesh command layer.

## Variant Registry

`MeshPipelineVariantRegistry` assigns stable `MeshPipelineVariantId` values for cache-backed pipeline variants. Its key is `MeshPassPipelineKind` plus the complete `PipelineKey`, so the same material pipeline used by the base pass and motion-vector pass resolves to different ids. Repeated requests for the same pass and pipeline shape reuse the same id.

Each `MeshPipelineVariantKey` also stores the Plan 08 neutral `ShaderVariantKey` derived from that `PipelineKey`. The registry maps base and TAA reactive passes to `ShaderPassType::Forward`, deferred GBuffer and current normal-target depth prepass variants to `ShaderPassType::GBuffer`, shadow variants to `Shadow`, and motion vectors to `Velocity`, using `wgpu-runtime` as the current platform token. DepthPrepass uses the GBuffer template today because the compiled graph pass still writes `GBUFFER_NORMAL` alongside `SCENE_DEPTH`; Deferred GBuffer uses a dedicated albedo/material template source because it writes `GBUFFER_ALBEDO` and `GBUFFER_MATERIAL`. The pure depth-only template is now covered by built-in standard-material staged prewarm source/hash tests, but it remains a future runtime/product graph contract until normal production is separated.

The registry also records a per-frame `ShaderVariantMissReport`. Variant id resolution counts requests and memory hits. Base mesh shader-module creation adds disk hits, compile misses, disk writes, and disk errors when it consults `ShaderVariantCacheDisk`.

Variant id `0` is intentionally left outside the registry for any remaining fixed pass-owned pipelines. Base, Deferred GBuffer, DepthPrepass, Velocity, TAA reactive, and Shadow mesh command paths now use registry ids so command sorting and replay can distinguish cache-backed variants by stable `MeshPipelineVariantId` values instead of fixed sentinels.

## Integration

`MeshPipelineCache` implements `MeshPipelineVariantResolver`. `SceneRendererCore::render_compiled_scene` and overlay mesh recording pass the live cache into `build_mesh_pass_command_buffers`, while unit tests pass a pure `MeshPipelineVariantRegistry`. `MeshPassBuildContext` forwards processor requests to that resolver when processors create `MeshDrawCommand` values.

Concrete WGPU pipeline lookup now has variant-id bridge methods for cache-backed mesh pipelines. Forward/base mesh recording uses `ensure_pipeline_for_variant(...)`; Deferred GBuffer uses `ensure_gbuffer_pipeline_for_variant(...)`; the current normal-target DepthPrepass uses `ensure_depth_prepass_pipeline_for_variant(...)`; Velocity uses `ensure_velocity_pipeline_for_variant(...)`; TAA reactive masks use `ensure_taa_reactive_mask_pipeline_for_variant(...)`; and Shadow uses `ensure_shadow_pipeline_for_variant(...)`. Each method looks up the registry key for the variant id, verifies the expected `MeshPassPipelineKind`, then delegates to its focused WGPU pipeline construction path.

`ensure_pipeline(...)` now checks the shader variant disk cache before creating the base mesh shader module. It hashes the selected WGSL source with blake3, combines that hash with the derived `ShaderVariantKey`, and stores compressed WGSL plus metadata under `.zircon-cache/shader_variants` or `ZR_SHADER_CACHE_DIR`. It also has a read-only staged fallback root at `cache/shader_variants`, which is where the build prewarm tool writes packaged cache entries. Disk corruption falls back to the current source and increments the report error count.

Builtin fallback prewarm template source alignment is the source-owner boundary that keeps this cache path and staged prewarm path identical. `builtin_fallback_shader_prewarm_manifest_uses_mesh_template_source` and `runtime_15_builtin_fallback_prewarm_uses_template_source` lock that `dynamic_api/shader_prewarm.rs` emits the same `mesh_pipeline_standard_material_template_source(...)` payload, content hashes, and template revision that `ensure_pipeline(...)` consumes. Status: `render_plan08_builtin_fallback_prewarm_template_source_static_passed_cargo_deferred_implementation_cadence`.

`zircon_shader_prewarm` can prepopulate built-in standard material staged variants into that staged root for Forward, GBuffer, DepthPrepass, Shadow, and Velocity. Runtime Base, Velocity, TAA reactive, Shadow, DepthPrepass normal-target, and Deferred GBuffer template source consumers now feed the same source/hash cache contract. The pass-aware staged prewarm source owner `mesh_pipeline_standard_material_template_source_for_shader_pass(...)` reuses this owner and locks pure depth-only DepthPrepass prewarm source/hash identity under `render_plan08_builtin_material_multi_pass_depth_only_prewarm_tests_passed_renderdoc_deferred`. Builtin standard material staged prewarm now also has write/restart-hit/WGPU shader-module validation under `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`: the focused test writes static and skinned five-pass manifests, reopens `ShaderVariantCacheDisk`, requires `ShaderVariantCacheDiskLookup::Hit` for every `ShaderVariantCacheDiskKey::from_variant_key(...)`, and creates shader modules from read-back WGSL under WGPU validation scope. DepthPrepass normal-target, Deferred GBuffer, Shadow, Velocity, and TAA reactive masks now have focused WGPU device pipeline validation through the default PRIMARY offscreen path. The Cargo wrapper for the five-filter rerun still timed out during lib-test compile/link, so the accepted evidence is direct execution of the generated `zircon_runtime` lib-test binary. Product Base mesh second-launch staged prewarm now closes the Base/Opaque product-level zero-compile-miss slice under `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`; RenderDoc capture and runtime pure-depth product migration remain follow-up work.

Asset-root builtin standard material template prewarm is the staged-cache consumer of this same owner: `dynamic_api::builtin_standard_material_shader_prewarm_manifest(...)` and `dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry(...)` feed `.zmaterial` `builtin://shader/pbr.wgsl` requests through the pass-aware template source path, and `shader_prewarm_asset_root_manifest_templates_builtin_standard_material_source`, `builtin_standard_material_prewarm_writes_restart_hits_and_wgpu_modules`, and `runtime_15_shader_prewarm_builtin_standard_material_template_source_is_wired` keep the status anchors visible here. Statuses: `render_plan08_asset_root_builtin_standard_material_template_prewarm_static_passed_cargo_deferred_implementation_cadence`, `render_plan08_asset_root_builtin_standard_material_multi_geometry_prewarm_static_passed_cargo_deferred_implementation_cadence`, `render_plan08_builtin_material_multi_pass_depth_only_prewarm_tests_passed_renderdoc_deferred`, and `render_plan08_builtin_material_staged_prewarm_cache_hit_wgpu_module_passed_renderdoc_deferred`.

Runtime Base mesh staged prewarm cache hit is now covered at the actual `ensure_pipeline_for_variant(...)` consumer. `runtime_base_mesh_pipeline_uses_staged_prewarm_without_compile_miss` writes the builtin fallback/standard-material manifest with `prewarm_shader_variants_to_disk(...)`, injects `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` into a fresh `MeshPipelineCache`, resolves the Base mesh variant, and creates the WGPU render pipeline under validation scope. The accepted miss report has `request_count == 1`, `disk_hit_count == 1`, `compile_miss_count == 0`, `disk_write_count == 0`, and `disk_error_count == 0`. Status: `render_plan08_runtime_base_mesh_staged_prewarm_cache_hit_wgpu_pipeline_passed_renderdoc_deferred`.

Product Base mesh second-launch staged prewarm proves the same cache path through render-product submission. `graphics/tests/render_product_mesh_cache/staged_prewarm.rs::render_product_base_mesh_second_launch_uses_staged_prewarm_without_compile_miss` writes the staged manifest once, then runs first and second fresh `WgpuRenderFramework` product launches with `ShaderVariantCacheDisk::with_fallback_roots(runtime_root, [staged_root])` injected through the test-only `replace_shader_variant_disk_cache_for_tests(...)` seam. The product pipeline uses a `mesh.opaque` side-effect pass and a shaded skinned direct replay fixture, then asserts both launches report variant requests, staged disk hits, `compile_miss_count == 0`, no runtime write/error, mesh replay state changes, skinned draws, and executed `mesh.opaque` evidence. `runtime_15_product_base_mesh_staged_prewarm_is_wired` locks this child owner and status anchors. Status: `render_plan08_product_base_mesh_second_launch_staged_prewarm_passed_renderdoc_deferred`.

DepthPrepass normal-target template source cache cutover status is `render_plan08_depth_prepass_template_source_cache_static_passed_cargo_check_wgpu_deferred`; device validation status is `render_plan08_depth_prepass_wgpu_device_pipeline_validation_passed_renderdoc_deferred`. `ensure_depth_prepass_pipeline_for_variant(...)`, `create_depth_prepass_mesh_pipeline.rs`, and `runtime_15_depth_prepass_pipeline_template_cache_is_mesh_cache_owned` lock that the old `NormalPrepassPipeline` path stays removed and the current graph pass binds GPUScene, standard material, geometry, normal target, and depth write through the mesh cache owner. `depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader` passed focused `zircon_runtime --lib` execution on the warmed Plan 08 target, creating a real offscreen WGPU pipeline from the current normal-target DepthPrepass template shader, scene/material/GPUScene bind group layouts, and WGPU validation error scope. The DepthPrepass structure guard passed when run directly from the generated lib-test binary after the cargo wrapper returned warning-only exit `-1` with no test result. Static command caching and RenderDoc/product acceptance remain separate acceptance work.

Deferred GBuffer template source cache cutover status is `render_plan08_deferred_gbuffer_template_source_cache_static_passed_cargo_check_wgpu_deferred`; device validation status is `render_plan08_deferred_gbuffer_wgpu_device_pipeline_validation_passed_renderdoc_deferred`. `mesh_pipeline_deferred_gbuffer_template_source_for_geometry(...)`, `ensure_gbuffer_pipeline_for_variant(...)`, `create_gbuffer_mesh_pipeline.rs`, and `runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned` lock that the old `deferred/geometry_pipeline` / `deferred_geometry.wgsl` path stays removed and the Deferred graph stage binds GPUScene, standard material, geometry, `GBUFFER_ALBEDO`, `GBUFFER_MATERIAL`, and depth read through the mesh cache owner. The scoped lib cargo check passed with existing warnings; Windows/WSL focused lib-test no-run attempts hit tooling timeouts or hung cargo parents after producing a test binary, and the generated WSL lib-test binary passed the `runtime_15_deferred_gbuffer_pipeline_template_cache_is_mesh_cache_owned` guard directly. `gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader` now creates a real offscreen WGPU pipeline from the Deferred GBuffer template shader, current scene/material/GPUScene bind group layouts, and WGPU validation error scope. Static command caching and RenderDoc/product acceptance remain separate acceptance work.

Shadow template source cache cutover status is `render_plan08_shadow_pipeline_template_source_cache_static_passed_cargo_check_test_compile_wgpu_deferred`; device-validation-code status is `render_plan08_shadow_wgpu_device_pipeline_validation_implemented_validation_not_closed`, now closed at default-path direct execution by `render_plan08_mesh_pipeline_default_wgpu_device_validation_passed_renderdoc_deferred`. `mesh_pipeline_shadow_template_source_for_geometry(...)`, `ensure_shadow_pipeline_for_variant(...)`, `create_shadow_mesh_pipeline.rs`, and `runtime_15_render_shader_template_assembly_is_folder_backed` lock that the old renderer-local shadow shader body stays removed and ShadowDepth/ShadowDepthAlphaMask consume mesh-cache variants. `shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader` mirrors the GBuffer/DepthPrepass WGPU fixture shape for both opaque and alpha-mask shadow templates. The old default/GL WSL path segfaulted on direct execution, but the current Windows default PRIMARY direct lib-test run passes the Shadow filter along with Velocity, TAA, GBuffer, and DepthPrepass. RenderDoc/product acceptance remains open.

Velocity/TAA WGPU device pipeline validation code status is `render_plan08_velocity_taa_wgpu_device_pipeline_validation_implemented_validation_not_closed`; the offscreen backend follow-up status is `render_plan08_offscreen_backend_primary_default_implemented_recompile_not_closed`; both are closed at default-path direct execution by `render_plan08_mesh_pipeline_default_wgpu_device_validation_passed_renderdoc_deferred`. `create_velocity_mesh_pipeline.rs`, `create_taa_reactive_mask_mesh_pipeline.rs`, and the test-only `mesh_pipeline/test_support.rs` mirror the same scene/material/GPUScene layout fixture for Velocity and TAA reactive mask pipeline creation. `velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader` creates the Velocity `Rg16Float` target pipeline from the template source and previous-position vertex slot; `taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader` creates both TAA reactive and material mask `R8Unorm` pipelines from the TAA template source. Direct execution of the current lib-test binary on the no-env default PRIMARY path passes Velocity, TAA, Shadow, GBuffer, and DepthPrepass device filters 5/5, and `render_backend_config_honors_renderdoc_wgpu_env_selection` passes 1/1 to keep default PRIMARY and explicit GL parsing guarded. RenderDoc/product acceptance remains open.
