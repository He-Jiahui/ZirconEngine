---
related_code:
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/alpha_mode.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/fallback_policy.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - user: 2026-05-09 implement M3A from render M4+ product pipeline plan
  - user: 2026-05-17 continue M5A PBR material and light runtime baseline
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
tests:
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - tests/acceptance/render-product-m5a-pbr-light.md
  - tests/acceptance/render-product-m3a-assets.md
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked render_product_pbr
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked shader --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked project_manager_imports_zshader_with_wgsl_capture_diagnostics --jobs 1
doc_type: module-detail
---

# Render Material Contracts

## Purpose

`zircon_runtime::core::framework::render::material` owns neutral material DTOs. Asset-side `MaterialAsset` converts into these descriptors, while concrete WGPU bind groups, pipeline keys, runtime fallback material preparation, and renderer stats remain under `zircon_runtime::graphics`.

## Product Surface

`StandardMaterialDescriptor` carries the M3A PBR-ready material surface: shader and texture dependencies, base color, normal, metallic/roughness, metallic-roughness texture, occlusion, emissive data, alpha behavior, unlit flag, double-sided flag, and fallback policy.

`ColorMaterialDescriptor` provides the simple unlit color/texture material contract for 2D and fallback paths. It shares the same dependency, alpha, double-sided, and fallback fields so later Core2d/Core3d classification can consume a common material shape.

`RenderMaterialDependencySet` stores the required shader and deduplicated texture references. `MaterialAsset::direct_references()` projects those dependencies into the normal imported-asset dependency path.

`RenderMaterialReadinessReport` is the structured acceptance result for asset readiness. It carries the material name, dependency set, fallback policy, validation errors, and fallback usage records instead of relying on silent fallback-only behavior.

`RenderMaterialDiagnosticSource` tags validation records with the layer that produced them: shader schema, WGSL capture, material override, texture slot, or dependency resolution. The enum is intentionally neutral render-framework data so asset import, runtime streaming, and editor projection can show the same typed diagnostics without passing runtime/editor objects across the boundary.

M5A wires the `StandardMaterialDescriptor` surface into the runtime resource streamer. `MaterialRuntime` now stores base color, emissive color, metallic, roughness, double-sided, alpha, unlit, texture ids, a `PipelineKey`, and the readiness report that was produced while resolving material, shader, and texture dependencies.

`PipelineKey` is intentionally renderer-owned. It includes shader identity/revision, double-sided state, alpha-blend/alpha-mask state, alpha cutoff bits, unlit state, and authored StandardMaterial texture-slot presence bits. The texture presence bits come from authored descriptor references rather than only successfully uploaded GPU texture ids, so a KTX/container texture that falls back still compiles the same material variant requested by the asset.

## Validation

`RenderMaterialValidationError::InvalidMaskCutoff` records invalid `AlphaMode::Mask` values. The accepted range is finite `0.0..=1.0`.

`RenderMaterialValidationError::UnresolvedShaderReference` and `UnresolvedTextureReference` record dependency resolution failures when the caller supplies resolver functions through `MaterialAsset::readiness_report_with_resolution(...)` or when the resource streamer cannot load the declared locator as the expected typed asset. The matching `RenderMaterialFallbackUsage` entries record whether the fallback was for a shader or a named texture slot. `RenderMaterialReadinessReport::is_ready()` now requires both no validation errors and no fallback usage.

`RenderMaterialValidationError::UnknownPropertyOverride`, `PropertyOverrideTypeMismatch`, and `UnknownTextureSlot` are emitted by `MaterialAsset::shader_contract_diagnostics(...)` when a `.zmaterial` instance does not match the loaded `.zshader` contract. Unknown overrides are sourced from `MaterialOverride`, wrong override types from `ShaderSchema`, and unknown texture slots from `TextureSlot`; each error preserves a stable document path such as `overrides.base_color` or `textures.albedo` for editor highlighting.

`RenderMaterialValidationError::MissingWgslCapture` records declared shader properties or texture slots that were not found by the lightweight WGSL source scan. Compound `.zshader` import stores those misses on `ShaderAsset.validation_diagnostics`, and `MaterialAsset::readiness_report_with_shader_contract(...)` folds them into the material readiness report with `WgslCapture` as the diagnostic source.

`RenderMaterialValidationError::MissingRuntimeShaderSource` is emitted by the streamer when a declared shader asset exists but cannot provide runtime WGSL. The streamer stores the report on `MaterialRuntime` before returning the blocking material error so renderer/debug consumers can inspect the cause.

`RenderMaterialValidationError::TextureNotUploadReady` records texture payloads that are valid assets but cannot be uploaded by the current M3A texture path. `TexturePayload::Container` remains visible in `RenderImageDescriptor` metadata, but GPU streaming currently accepts only `TexturePayload::Rgba8`; container textures use the default texture fallback until a container upload path is implemented.

`RenderMaterialValidationError::UnresolvedMaterialReference` records a missing material handle at runtime. The resource streamer resolves that case to `builtin://missing-material`, inserts `RenderMaterialFallbackReason::Material`, and keeps the fallback runtime under the originally requested material id so mesh submission can continue while stats and diagnostics remain tied to the missing handle.

`RenderMaterialReadinessReport::push_validation_error_once(...)` and `push_fallback_usage_once(...)` keep merged streamer diagnostics stable when one material resolution path reports the same shader, texture, or material fallback more than once.

`MaterialAsset::readiness_report_with_shader_contract(...)` composes the normal dependency-resolution report with shader-contract diagnostics and imported shader validation diagnostics. This makes schema mismatches and capture gaps readiness failures without turning `.zmaterial` or `.zshader` import into a hard failure, which keeps authoring tools able to open broken material assets and show actionable rows.

## Runtime PBR Baseline

M5A makes StandardMaterial fields visible to the concrete mesh path without moving asset or editor ownership into graphics. `resource_streamer_ensure_material.rs` reads the descriptor, resolves every PBR texture slot, resolves shader runtime source, records fallback diagnostics, and builds a `MaterialRuntime` plus `PipelineKey`. Missing textures and missing shaders can degrade to default fallback resources; invalid alpha-mask cutoff and missing runtime WGSL remain blocking validation errors.

Mesh pipeline creation consumes `PipelineKey`: `double_sided` disables back-face culling, `alpha_blend` controls transparent blend/depth-write behavior, `alpha_mask` and cutoff bits keep mask variants distinct, and PBR texture slots select authored material variants. Alpha-mask materials are not treated as transparent; only blend mode reports `PipelineKey::is_transparent()`.

`ResourceStreamer::ensure_scene_resources(...)` counts prepared materials and folds each material readiness report into renderer-facing counters. `RenderStats` exposes `last_material_count`, `last_material_ready_count`, `last_material_fallback_count`, and `last_material_validation_error_count` so submit tests and tools can distinguish ready materials from fallback/degraded materials.

The `.zmaterial` assetization lane now owns material source parsing: built-in material import accepts `.zmaterial`, shader-owned overrides, and texture-slot references. This render-framework module still owns neutral runtime readiness and PBR descriptor projection only; material-editor authoring, automatic shader reflection, and a full physically based shader rewrite remain outside this document's runtime material contract scope.
