---
related_code:
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
implementation_files:
  - zircon_runtime/src/graphics/shader/template/material_surface.rs
  - zircon_runtime/src/graphics/shader/template/assemble.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/core/framework/render/shader/variant_key.rs
  - zircon_runtime/src/graphics/shader/ide_env_generation.rs
plan_sources:
  - user: 2026-07-07 mirror grazing looked unrealistic, left/right differed, right edge over-grazed
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/shader/template/tests/standard_material_surface_template.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/shader_source/tests.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/frame_assertions.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export.rs
doc_type: module-detail
---

# Standard Material Surface Template

## Purpose

`material_surface.rs` owns the generated WGSL surface function for renderer-owned StandardPBR materials. It maps `StandardMaterialDescriptor` state into shader feature bits, texture/uniform binding declarations, and the `standard_material_surface(...)` function that later pass templates rename to `zr_material_surface(...)`.

This module is part of the material/shader template layer, not the cubemap bake layer. The EC-M3o mirror-grazing fix keeps that boundary explicit: a mirror sphere with no authored normal texture must use the interpolated geometric normal. It must not sample the neutral normal-map fallback texture, because the 8-bit neutral value is only an asset binding fallback and can create visible grazing bias on mirror-like materials.

## Behavior Model

The template emits a `ZR_FEATURE_HAS_NORMAL_TEXTURE` constant through `assemble.rs`. `PipelineKey::shader_feature_bits()` sets the bit when the renderer pipeline key has `has_normal_texture`, while `standard_material_shader_features(...)` sets the same bit when a `StandardMaterialDescriptor` has `normal_texture`.

`standard_material_sampled_normal(...)` first normalizes the geometric normal from the vertex output. When `ZR_FEATURE_HAS_NORMAL_TEXTURE` is false, it returns that value immediately. When true, it builds the tangent frame from `input.tangent_ws`, `input.tangent_handedness`, and `input.normal_ws`, samples `standard_material_normal_tex`, remaps it from texture space, and returns a world-space normal.

The guard is intentionally in WGSL rather than hidden in resource binding. The renderer still binds a complete standard-material texture set for ABI stability, but shader behavior follows material authorship. This lets no-normal materials keep exact geometric normals while normal-mapped materials continue to use the same binding layout and tangent-frame code path.

## Design And Rationale

The StandardPBR template already owns alpha-test, receive-shadow, double-sided, and roughness-floor behavior. Adding the normal-map bit here keeps material authoring features in one place and avoids making the environment sampler compensate for bad normals. The pipeline key also carries the bit so shader variants with and without normal maps can produce distinct WGSL defines.

The EC-M3o mirror issue demonstrated why this matters: the cubemap lookup, PMREM mip generation, and skybox ray reconstruction were already aligned with cmft/cmftStudio, but a neutral fallback normal sample still perturbed the surface normal. On a rough material this is usually hidden. On a perfect mirror sphere it shifts grazing reflections enough to make one side look over-bright or noisy.

## Test Coverage

`standard_material_surface_template.rs` verifies that a default standard material does not set `HAS_NORMAL_TEXTURE`, that generated WGSL contains `const ZR_FEATURE_HAS_NORMAL_TEXTURE: bool = false;`, and that the WGSL contains the early return guard. The same test file also covers the authored-normal-map case and expects the define to become true.

`mesh_pipeline_cache/shader_source/tests.rs` covers the renderer pipeline source path: default mesh pipeline template source emits the false define, and a `PipelineKey` with `has_normal_texture = true` emits the true define and carries `ShaderFeatureBits::HAS_NORMAL_TEXTURE`.

`runtime_shader_pbr_hdri_export/frame_assertions.rs` validates the user-visible consequence through mirror screenshots. `MirrorSphereGrazingBalanceStats` samples left/right grazing annuli and rejects large luma or variance imbalance, so a future fallback-normal regression cannot hide behind a visually acceptable center reflection.

## Plan Sources

This document follows Shader Plan 06 EC-M3o and Render Plan 11 EL-M1/EC-M3o. It is a material-template correction supporting the HDRI cubemap validation path, not a replacement for remaining IBL work such as strict SSIM/reference compare, RenderDoc/product capture, derived/offline artifacts, or high-resolution offline bake acceptance.
