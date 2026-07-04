---
doc_type: module-detail
related_code:
  - zircon_runtime/src/core/framework/render/environment/equirect_samples.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/shaders/skybox_procedural.wgsl
implementation_files:
  - zircon_runtime/src/core/framework/render/environment/equirect_samples.rs
  - zircon_runtime/src/core/framework/render/environment/mod.rs
  - zircon_runtime/src/core/framework/render/environment/skybox.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_construct/scene_bind_group_bundle/create_scene_bind_group_bundle.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_write_scene_uniform/write_scene_uniform.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
plan_sources:
  - user: 2026-07-05 Poly Haven lakes HDRI skybox/reflection mosaic and mip blur correction
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/core/framework/render/environment/equirect_samples.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes.rs
---

# Sampled HDRI Mip Chain

This module records the runtime bridge that replaces the earlier 16x8 nearest-sampled HDRI table while the full plan 06 cubemap asset path is still being built.

The source HDRI is sampled into a 128x64 equirectangular base level and a wrapped/clamped 2x2 downsample chain. The complete chain is uploaded as a read-only storage buffer at scene group0 binding1. The scene uniform keeps only source kind and base dimensions, so the frame uniform no longer carries the environment table.

The WGSL environment helper uses bilinear sampling within a mip and linearly blends between mips. Roughness maps to mip with the Unreal reflection-capture constants `ROUGHEST_MIP = 1.0` and `ROUGHNESS_MIP_SCALE = 1.2`, keeping material roughness stable when the capture has a different mip count. The indirect specular term uses the Lazarov split-sum approximation until the real BRDF LUT from plan 06 EC-M2 is wired.

This is an interim correctness step, not the final IBL architecture. It removes the visible 16x8 block artifact and makes rough materials consume increasingly blurred environment levels, but it is still an equirectangular storage-buffer path. The planned endpoint remains a GPU `texture_cube` source skybox, GGX filtered-importance prefiltered mip chain, SH9 diffuse irradiance, and optional irradiance cube.

The project render export for this bridge writes the new validation image to `docs/tests/runtime/shader/runtime_shader_pbr_real_hdri_lakes_pmrem_reflection_20260705.png`, leaving the rejected 2026-07-04 image intact for visual comparison.
