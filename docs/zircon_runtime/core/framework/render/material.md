---
related_code:
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/alpha_mode.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/fallback_policy.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
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

# Render Material Contracts

## Purpose

`zircon_runtime::core::framework::render::material` owns neutral material DTOs. Asset-side `MaterialAsset` converts into these descriptors, while concrete WGPU bind groups and pipeline keys remain under `zircon_runtime::graphics`.

## Product Surface

`StandardMaterialDescriptor` carries the M3A PBR-ready material surface: shader and texture dependencies, base color, normal, metallic/roughness, metallic-roughness texture, occlusion, emissive data, alpha behavior, unlit flag, double-sided flag, and fallback policy.

`ColorMaterialDescriptor` provides the simple unlit color/texture material contract for 2D and fallback paths. It shares the same dependency, alpha, double-sided, and fallback fields so later Core2d/Core3d classification can consume a common material shape.

`RenderMaterialDependencySet` stores the required shader and deduplicated texture references. `MaterialAsset::direct_references()` projects those dependencies into the normal imported-asset dependency path.

`RenderMaterialReadinessReport` is the structured acceptance result for asset readiness. It carries the material name, dependency set, fallback policy, validation errors, and fallback usage records instead of relying on silent fallback-only behavior.

## Validation

`RenderMaterialValidationError::InvalidMaskCutoff` records invalid `AlphaMode::Mask` values. The accepted range is finite `0.0..=1.0`.

`RenderMaterialValidationError::UnresolvedShaderReference` and `UnresolvedTextureReference` record dependency resolution failures when the caller supplies resolver functions through `MaterialAsset::readiness_report_with_resolution(...)` or when the resource streamer cannot load the declared locator as the expected typed asset. The matching `RenderMaterialFallbackUsage` entries record whether the fallback was for a shader or a named texture slot. `RenderMaterialReadinessReport::is_ready()` now requires both no validation errors and no fallback usage.

`RenderMaterialValidationError::MissingRuntimeShaderSource` is emitted by the streamer when a declared shader asset exists but cannot provide runtime WGSL. The streamer stores the report on `MaterialRuntime` before returning the blocking material error so renderer/debug consumers can inspect the cause.

`RenderMaterialValidationError::TextureNotUploadReady` records texture payloads that are valid assets but cannot be uploaded by the current M3A texture path. `TexturePayload::Container` remains visible in `RenderImageDescriptor` metadata, but GPU streaming currently accepts only `TexturePayload::Rgba8`; container textures use the default texture fallback until a container upload path is implemented.

M3A intentionally does not claim full PBR renderer integration. It defines and validates the product contract that M5A can wire into phase keys, mesh pipeline cache keys, and renderer fallback statistics.
