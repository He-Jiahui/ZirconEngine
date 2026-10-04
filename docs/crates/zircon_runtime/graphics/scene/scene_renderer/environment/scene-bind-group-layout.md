---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_depth_prepass_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_gbuffer_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/create_shadow_mesh_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/scene_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core/environment_cubemap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/prewarm_pipeline_validation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/test_support.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
plan_sources:
  - docs/plans/zircon_runtime/render/index.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - user: 2026-06-12 implement wgpu-to-render-pipeline design code
tests:
  - E:\cargo-targets\zircon-plan08-default-broad-current-0704\debug\deps\zircon_runtime-90029178d239f47b.exe tests::graphics::scene::scene_renderer::shadow::shadow_map_renderer::tests::shadow_map_scene_bind_group_matches_environment_scene_layout --exact --nocapture --test-threads=1 (2026-07-05 direct binary: passed 1/1; log docs/tests/runtime/render/plan08_shadow_scene_bind_group_environment_layout_direct_binary_20260705.out.log)
  - E:\cargo-targets\zircon-plan08-default-broad-current-0704\debug\deps\zircon_runtime-90029178d239f47b.exe graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::prewarm_pipeline_validation::tests::mesh_prewarm_pipeline_validation_creates_all_builtin_pass_pipelines --exact --nocapture --test-threads=1 (2026-07-05 direct binary: passed 1/1; log docs/tests/runtime/render/plan08_mesh_prewarm_validation_scene_environment_layout_5slot_direct_binary_20260705.out.log)
  - E:\cargo-targets\zircon-plan08-default-broad-current-0704\debug\deps\zircon_runtime-90029178d239f47b.exe graphics::tests::project_render::render_quality::deferred_pipeline_uses_gbuffer_material_path_instead_of_forward_shader_path --exact --nocapture --test-threads=1 (2026-07-05 direct binary: passed 1/1; log docs/tests/runtime/render/plan08_material_deferred_pipeline_after_5slot_scene_layout_direct_binary_20260705.out.log)
  - E:\cargo-targets\zircon-plan08-default-broad-current-0704\debug\deps\zircon_runtime-90029178d239f47b.exe graphics::tests::render_product_mesh_cache::project_plugin_registry_material_passes_staged_cache::custom_second_launch::render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss --exact --nocapture --test-threads=1 (2026-07-05 direct binary: passed 1/1; log docs/tests/runtime/render/plan08_material_custom_second_launch_after_5slot_scene_layout_direct_binary_20260705.out.log)
doc_type: module-detail
---

# Scene Environment Bind Group Layout

`scene_bind_group_layout.rs` is the scene renderer owner for the group0 environment layout used by runtime scene rendering, mesh pipeline WGPU validation, and shadow-map replay. It prevents the scene bundle, prewarm validation, and pipeline tests from carrying local copies of the same layout.

## Binding ABI

The current scene group0 layout has six entries:

- Binding 0: `SceneUniform` uniform buffer, visible to vertex, fragment, and compute stages.
- Binding 1: source environment `texture_cube<f32>`, visible to fragment shaders.
- Binding 2: filtering sampler for the source/specular environment textures.
- Binding 3: BRDF LUT `texture_2d<f32>`, visible to fragment shaders.
- Binding 4: specular PMREM `texture_cube<f32>`, visible to fragment shaders.
- Binding 5: optional diffuse irradiance/IEM `texture_cube<f32>`, visible to fragment shaders.

`scene_renderer_core_construct` creates the real renderer bind group from this layout and binds the persistent fallback/runtime environment resources. `prewarm_pipeline_validation.rs`, mesh pipeline WGPU test support, and depth/GBuffer/shadow pipeline creation use the same entries so staged prewarm validation cannot accept a stale scene group shape.

`SceneEnvironmentCubemap` creates source/specular/IEM `Rgba16Float` cube textures with `COPY_SRC` in addition to binding and upload usage. This lets Plan 11 IBL runtime readback copy PMREM/IEM texture bytes into cache artifacts after a bake path has produced them; it does not by itself schedule compute or readback work.

## Shadow Fallbacks

`ShadowMapRenderer` records atlas depth passes through the full mesh pipeline layout even though the pass writes only depth. It now owns retained 1x1 fallback resources for the source environment cube, BRDF LUT, specular PMREM cube, and irradiance cube plus a filtering sampler, and binds entries 0 through 5 against the shared scene layout. This keeps `zircon-shadow-map-scene-bind-group` compatible with `zr_environment.wgsl` and the material-pass pipeline layout without adding a shadow-only compatibility layout.

## Validation State

Status `render_plan08_material_filter_scene_environment_5slot_shadow_prewarm_direct_binary_passed_ui_layout_open` records the earlier five-slot convergence. The later Plan 11 IEM carrier and RGBA16F readback helper slices extend the same environment resource set to six slots and add `COPY_SRC` usage for runtime cache readback. Focused direct-binary validation passed for the shadow-map scene bind group, WGPU prewarm pipeline validation, a deferred material pipeline probe, and a custom shading-model staged-prewarm second-launch probe; the RGBA16F readback helper has separate 2/2 unit evidence under `plan11_ibl_wgpu_rgba16float_region_readback_helper_cargo_20260706.*`. Broad direct-binary `material` still remains red because of active UI material layout text-metric failures; the scene-layout WGPU failures are gone.
