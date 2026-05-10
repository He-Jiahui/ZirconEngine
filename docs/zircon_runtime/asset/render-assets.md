---
related_code:
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
implementation_files:
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/asset/assets/texture/metadata.rs
  - zircon_runtime/src/asset/assets/texture/payload.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/model/virtual_geometry.rs
  - zircon_runtime/src/asset/assets/shader/mod.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/entry_point.rs
  - zircon_runtime/src/asset/assets/shader/language.rs
  - zircon_runtime/src/asset/assets/shader/dependency.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/alpha_mode.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
plan_sources:
  - user: 2026-05-09 implement M3A from render M4+ product pipeline plan
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
tests:
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - tests/acceptance/render-product-m3a-assets.md
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
doc_type: module-detail
---

# Render Assets

## Purpose

M3A turns render-facing assets into product contracts for `RenderProductFeature::{Image, Mesh, Shader, Material}`. The asset structs remain under `zircon_runtime::asset`, while the neutral descriptors they produce live under `zircon_runtime::core::framework::render`.

## Asset Contracts

`TextureAsset::render_image_descriptor()` exposes width, height, render format, color space, sampler, GPU usage, mip count, array-layer count, and fallback class through `RenderImageDescriptor`.

`ModelPrimitiveAsset::render_mesh_descriptor()` projects primitive vertex/index data into topology, bounds, primitive kind, 2D/3D suitability, primitive counts, and Virtual Geometry payload presence through `RenderMeshDescriptor`.

`ShaderAsset::runtime_wgsl_source()` chooses runtime WGSL by preferring non-empty `wgsl_source`, then non-empty `source` only when `source_language == ShaderSourceLanguage::Wgsl`. Non-WGSL source without emitted WGSL is not treated as render-ready WGSL.

Shader dependencies are explicit serialized `ShaderDependencyAsset` entries because M3A does not introduce a shader import language. `ShaderAsset::dependencies()` projects those entries into `RenderShaderDependency`. Pipeline layout readiness is explicit through serialized `RenderShaderPipelineLayoutDescriptor`, including bind groups, binding descriptors, resource type, stage visibility, and push-constant range labels.

`MaterialAsset` now exposes `dependency_set()`, `direct_references()`, `standard_material_descriptor()`, `color_material_descriptor()`, and `readiness_report()`. Material direct dependencies include the shader plus base color, normal, metallic-roughness, occlusion, and emissive textures when present.

## Readiness

Material readiness is structured through `RenderMaterialReadinessReport`. `AlphaMode::Mask { cutoff }` rejects non-finite values and values outside `0.0..=1.0` with `RenderMaterialValidationError::InvalidMaskCutoff`. Callers that can resolve asset references use `readiness_report_with_resolution(...)`, which records unresolved shader or texture references as validation errors plus explicit fallback usage records.

The resource streamer uses typed material readiness before preparing GPU material state and stores the resulting `RenderMaterialReadinessReport` on `MaterialRuntime`. Renderer-side consumers and tests can query it through `ResourceStreamer::material_readiness_report(...)`. Existing fallback shader and missing-texture behavior remains allowed for compatibility, but unresolved shader references, wrong-kind or load-failing dependencies, unresolved texture references, and the fallback policy used for each slot are preserved in the stored report instead of being discarded.

Shader fallback is exposed through `ensure_shader_source(...)`, which now returns the prepared shader identity and an optional structured readiness report when the requested shader reference resolves through the default fallback shader. A shader that exists but cannot provide runtime WGSL is reported as `MissingRuntimeShaderSource`, stored on the material runtime report, and then treated as a blocking material readiness error.

Texture lookup is exposed through `resolve_texture_reference(...)`, which returns the resolved texture id plus an unresolved-reference validation error and fallback usage when the declared texture locator is missing, is the wrong typed payload, cannot load as a `TextureAsset`, or is not upload-ready. M3A advertises `TexturePayload::Container` metadata through render image descriptors, but the current GPU upload path only supports `TexturePayload::Rgba8`; container textures are therefore reported as `TextureNotUploadReady` and use the fallback texture until container upload support lands. The older texture id helper remains a narrow compatibility convenience for internal paths that only need the id.

## Scope Boundary

This document covers M3A asset readiness only. It does not implement M4 core phases, sprite rendering, anti-aliasing, Solari, or deeper VG/HGI integration.
