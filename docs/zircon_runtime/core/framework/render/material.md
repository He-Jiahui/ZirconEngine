---
related_code:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs
  - zircon_runtime/src/core/framework/render/material/property_value.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/property_values.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_new.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/mod.rs
  - zircon_runtime/src/core/framework/render/material/mod.rs
  - zircon_runtime/src/core/framework/render/material/alpha_mode.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/color_material.rs
  - zircon_runtime/src/core/framework/render/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/fallback_policy.rs
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs
  - zircon_runtime/src/core/framework/render/material/property_value.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/property_values.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_validate_material_shader_layout.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_new.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_material_uniform/gpu_material_uniform_resource.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_material.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_new/layouts/create_material_bind_group_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_draw/render_pass_bindings.rs
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/base_stats.rs
plan_sources:
  - user: 2026-05-09 implement M3A from render M4+ product pipeline plan
  - user: 2026-05-17 continue M5A PBR material and light runtime baseline
  - docs/superpowers/plans/2026-05-08-render-m4-plus-product-pipeline.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - user: 2026-05-27 continue shader/material management
tests:
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/render_product.rs
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs
  - zircon_runtime/src/asset/tests/assets/material.rs::material_asset_readiness_includes_shader_payload_readiness_diagnostics
  - zircon_runtime/src/asset/tests/assets/material.rs::material_asset_readiness_reports_material_local_diagnostics_without_blocking
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_material_report_includes_shader_readiness_diagnostics
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_reports_shader_material_layout_abi_diagnostics
  - zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs::render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics
  - zircon_runtime/src/graphics/tests/render_product_submit.rs
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_submit_material_stats_count_non_blocking_diagnostics
  - zircon_runtime/src/graphics/tests/render_product_submit.rs::render_product_submit_material_stats_count_material_uniform_diagnostics
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_prepares_shader_texture_slot_runtime_mapping
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests.rs::render_product_streamer_prepares_shader_property_runtime_values
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_material_properties_prepare_uniform_payload
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_exposes_material_uniform_debug_counts
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report
  - zircon_runtime/src/graphics/scene/render_product_material_property_tests.rs::render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs::tests::material_readiness_report_deduplicates_material_uniform_diagnostics
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_aligns_and_encodes_numeric_values
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_records_unsupported_strings
  - zircon_runtime/src/core/framework/render/material/property_uniform.rs::tests::material_property_uniform_payload_reports_unsupported_diagnostics
  - cargo test -p zircon_runtime --lib material_property_uniform_payload_reports_unsupported_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material uniform diagnostics: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never -- --format terse (2026-05-27 M5 runtime gate: passed, 2102 passed, 0 failed, after restoring the top-level render facade export for `RenderMaterialPropertyUniformSummary`)
  - cargo test -p zircon_runtime --lib render_product_submit_material_stats_count_non_blocking_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material diagnostic stats validation retry: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_submit_material_stats_count_material_uniform_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 MaterialUniform submit stats: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - D:/cargo-targets/zircon-material-diagnostic-stats-0527/debug/deps/zircon_runtime-b34ee8d8fc52f1fd.exe render_product_streamer_reports_material_uniform_diagnostics_in_readiness_report --test-threads=1 --nocapture (2026-05-27 MaterialUniform readiness detail: passed, 1 passed after the Cargo wrapper timed out during concurrent build activity)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_material_uniform_diagnostics_for_shader_string_defaults --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-diagnostic-stats-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 MaterialUniform shader default detail: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - F:/cargo-targets/zircon-platform-m5-workspace/debug/deps/zircon_runtime-030785730509538c.exe material_readiness_report_deduplicates_material_uniform_diagnostics --test-threads=1 --nocapture (2026-05-27 MaterialUniform diagnostic dedup: passed, 1 passed; standard Cargo wrappers timed out under concurrent workspace/editor build load before producing a local material target binary)
  - tests/acceptance/render-product-m5a-pbr-light.md
  - tests/acceptance/render-product-m3a-assets.md
  - cargo test -p zircon_runtime --locked render_product_assets
  - cargo test -p zircon_runtime --locked render_product_pbr
  - cargo test -p zircon_runtime --locked material
  - cargo check -p zircon_runtime --lib --locked
  - rustfmt --edition 2021 --check zircon_runtime/src/core/framework/render/material/diagnostic_source.rs zircon_runtime/src/core/framework/render/material/readiness_report.rs zircon_runtime/src/core/framework/render/material/mod.rs zircon_runtime/src/core/framework/render/mod.rs zircon_runtime/src/asset/assets/material/material_asset.rs zircon_runtime/src/asset/tests/assets/material.rs zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs (2026-05-27 material-local readiness diagnostics: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527 --message-format short --color never (2026-05-27 material-local readiness diagnostics: blocked before material tests by unrelated UI a11y private re-export errors in zircon_runtime/src/ui/accessibility/action/text.rs)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527-core-min --message-format short --color never (2026-05-27 material-local readiness diagnostics: same unrelated UI a11y private re-export blocker)
  - cargo test -p zircon_runtime --lib material_asset_readiness_reports_material_local_diagnostics_without_blocking --locked --jobs 1 --target-dir D:/cargo-targets/zircon-material-local-diagnostics-0527 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-27 material-local readiness diagnostics: attempted; wrapper timed out before producing a test binary or Rust diagnostics)
  - git diff --check -- touched material-local readiness diagnostics files (2026-05-27 material-local readiness diagnostics: passed with LF-to-CRLF warnings only)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs (2026-05-27 shader pipeline layout readiness summary: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group --message-format short --color never (2026-05-27 shader pipeline layout readiness summary: passed with existing warnings)
  - D:/cargo-targets/zircon-shader-readiness-layout-0527/debug/deps/zircon_runtime-b34ee8d8fc52f1fd.exe shader_readiness --test-threads=1 --nocapture (2026-05-27 shader pipeline layout readiness summary: passed, 6 passed; standard cargo test wrapper timed out after producing the test binary during concurrent Cargo activity)
  - rustfmt --edition 2021 --check zircon_runtime/src/graphics/scene/mod.rs zircon_runtime/src/graphics/scene/render_product_zshader_import_tests.rs (2026-05-27 imported zshader renderer ABI diagnostics: passed)
  - rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/zshader.rs zircon_runtime/src/asset/importer/ingest/import_shader_package.rs zircon_runtime/src/asset/tests/project/zmeta.rs (2026-05-27 zshader pipeline layout persistence: passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group --message-format short --color never (2026-05-27 imported zshader renderer ABI diagnostics: passed with existing warnings)
  - cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 zshader pipeline layout persistence: passed, 1 passed)
  - cargo test -p zircon_runtime --lib documented_zmeta_shader_material_fixture_parses --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 zshader pipeline layout persistence: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_shader_material_layout_abi_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 renderer material ABI diagnostics: passed, 1 passed)
  - cargo test -p zircon_runtime --lib render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-material-bind-group -- --test-threads=1 --nocapture (2026-05-27 imported zshader renderer ABI diagnostics: passed, 1 passed)
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked shader --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked project_manager_imports_zshader_with_wgsl_capture_diagnostics --jobs 1
doc_type: module-detail
---

# Render Material Contracts

## Purpose

`zircon_runtime::core::framework::render::material` owns neutral material DTOs. Asset-side `MaterialAsset` converts into these descriptors, while concrete WGPU bind groups, pipeline keys, runtime fallback material preparation, and renderer stats remain under `zircon_runtime::graphics`.

The top-level `zircon_runtime::core::framework::render` facade re-exports the material DTOs that resource streaming and renderer diagnostics consume. `RenderMaterialPropertyUniformSummary` must remain available through that facade because `resource_streamer_accessors.rs` depends on the neutral render-framework surface instead of reaching into the child material module directly.

## Product Surface

`StandardMaterialDescriptor` carries the M3A PBR-ready material surface: shader and texture dependencies, base color, normal, metallic/roughness, metallic-roughness texture, occlusion, emissive data, alpha behavior, unlit flag, double-sided flag, and fallback policy.

`ColorMaterialDescriptor` provides the simple unlit color/texture material contract for 2D and fallback paths. It shares the same dependency, alpha, double-sided, and fallback fields so later Core2d/Core3d classification can consume a common material shape.

`RenderMaterialDependencySet` stores the required shader and deduplicated texture references. `MaterialAsset::direct_references()` projects those dependencies into the normal imported-asset dependency path.

`RenderMaterialReadinessReport` is the structured acceptance result for asset readiness. It carries the material name, dependency set, fallback policy, validation errors, fallback usage records, an optional prepared uniform summary, and non-blocking diagnostic rows instead of relying on silent fallback-only behavior.

`RenderMaterialDiagnosticSource` tags validation and diagnostic records with the layer that produced them: material asset metadata, shader schema, shader payload readiness, WGSL capture, material override, texture slot, renderer material ABI, material uniform preparation, or dependency resolution. The enum is intentionally neutral render-framework data so asset import, runtime streaming, and editor projection can show the same typed diagnostics without passing runtime/editor objects across the boundary.

`RenderMaterialPropertyValue` is the neutral typed value payload for shader schema properties. It covers bool, float, signed and unsigned integers, string, vec2, vec3, and vec4/color values. `RenderMaterialPropertyValueSummary` is the companion inspection DTO for the projected map; it records total values, per-kind counts, uniform-eligible values, and non-uniform values such as strings before any byte encoding occurs. The value type and summary are not a GPU uniform layout and do not imply a bind-group ABI; they are the renderer-facing value projection that lets later uniform-buffer or reflection work consume material properties without reparsing TOML.

`RenderMaterialPropertyUniformPayload` is the CPU-side preparation step for shader property bytes. It converts the deterministic shader property value map into a byte buffer plus field layout rows using scalar/vector alignments that the graphics resource streamer can upload. String properties are retained as unsupported entries instead of being silently dropped, and `unsupported_diagnostics()` projects those rows into non-blocking `MaterialUniform` readiness diagnostics. They remain authoring/runtime metadata until a separate binding path exists for non-uniform property kinds. This core DTO still does not own WGPU objects or infer a complete shader reflection ABI.

M5A wires the `StandardMaterialDescriptor` surface into the runtime resource streamer. `MaterialRuntime` now stores base color, emissive color, metallic, roughness, double-sided, alpha, unlit, standard PBR texture ids, shader property values projected from shader schema, non-standard shader texture slot ids, a `PipelineKey`, and the readiness report that was produced while resolving material, shader, and texture dependencies.

`PipelineKey` is intentionally renderer-owned. It includes shader identity/revision, double-sided state, alpha-blend/alpha-mask state, alpha cutoff bits, unlit state, and authored StandardMaterial texture-slot presence bits. The texture presence bits come from authored descriptor references rather than only successfully uploaded GPU texture ids, so a KTX/container texture that falls back still compiles the same material variant requested by the asset.

## Validation

`RenderMaterialValidationError::InvalidMaskCutoff` records invalid `AlphaMode::Mask` values. The accepted range is finite `0.0..=1.0`.

`RenderMaterialReadinessDiagnostic` carries material-local diagnostic strings that should remain visible without marking the material unavailable. `MaterialAsset.validation_diagnostics` entries are projected into this non-blocking list with `RenderMaterialDiagnosticSource::MaterialAsset` and stable paths such as `material.validation_diagnostics[0]`. `RenderMaterialReadinessReport::is_ready()` still depends only on validation errors and fallback usage.

`RenderMaterialValidationError::UnresolvedShaderReference` and `UnresolvedTextureReference` record dependency resolution failures when the caller supplies resolver functions through `MaterialAsset::readiness_report_with_resolution(...)` or when the resource streamer cannot load the declared locator as the expected typed asset. The matching `RenderMaterialFallbackUsage` entries record whether the fallback was for a shader or a named texture slot. `RenderMaterialReadinessReport::is_ready()` now requires both no validation errors and no fallback usage.

`RenderMaterialValidationError::UnknownPropertyOverride`, `PropertyOverrideTypeMismatch`, and `UnknownTextureSlot` are emitted by `MaterialAsset::shader_contract_diagnostics(...)` when a `.zmaterial` instance does not match the loaded `.zshader` contract. Unknown overrides are sourced from `MaterialOverride`, wrong override types from `ShaderSchema`, and unknown texture slots from `TextureSlot`; each error preserves a stable document path such as `overrides.base_color` or `textures.albedo` for editor highlighting.

`RenderMaterialValidationError::MissingWgslCapture` records declared shader properties or texture slots that were not found by the lightweight WGSL source scan. Compound `.zshader` import stores those misses on `ShaderAsset.validation_diagnostics`, and `MaterialAsset::readiness_report_with_shader_contract(...)` folds them into the material readiness report with `WgslCapture` as the diagnostic source.

`RenderMaterialValidationError::ShaderReadinessDiagnostic` records shader payload readiness rows that do not have a narrower material error shape. Material readiness currently uses it for invalid entry-point stage tokens and duplicate or empty normalized shader definitions from `ShaderAsset::readiness_report()`. The row keeps the original diagnostic string plus a stable path such as `entry_points.fs_main` or `shader_defs.USE_UNLIT`.

`RenderMaterialDiagnosticSource::MaterialUniform` records renderer uniform preparation notes that should not block material readiness. The current producer is `RenderMaterialPropertyUniformPayload::unsupported_diagnostics()`, which reports unsupported non-uniform property kinds such as strings with stable paths like `uniform.debug_label` after the typed property map has been projected.

The resource streamer also folds authored shader pipeline-layout compatibility into `ShaderReadinessDiagnostic` when a `ShaderAsset` declares bind groups, including bind groups persisted from compound `.zshader` `[pipeline_layout]` TOML. The current mesh renderer material ABI is fixed at group 3 binding 0 as a uniform buffer; a missing group 3, missing binding 0, wrong binding resource type, compute-only visibility, duplicate descriptors, or extra group-3 material bindings is reported with stable paths such as `pipeline_layout.group3` and `pipeline_layout.group3.binding0`. Empty shader pipeline layouts remain accepted because the current renderer can still use its fixed fallback layouts without serialized reflection. The asset-owned `ShaderReadinessReport` separately exposes a layout summary for inspection; the renderer-owned ABI check remains the only path that decides whether authored material bindings conflict with the current mesh renderer contract.

`RenderMaterialValidationError::MissingRuntimeShaderSource` is emitted by the streamer when a declared shader asset exists but cannot provide runtime WGSL. The streamer stores the report on `MaterialRuntime` before returning the blocking material error so renderer/debug consumers can inspect the cause.

`RenderMaterialValidationError::TextureNotUploadReady` records texture payloads that are valid assets but cannot be uploaded by the current M3A texture path. `TexturePayload::Container` remains visible in `RenderImageDescriptor` metadata, but GPU streaming currently accepts only `TexturePayload::Rgba8`; container textures use the default texture fallback until a container upload path is implemented.

`RenderMaterialValidationError::UnresolvedMaterialReference` records a missing material handle at runtime. The resource streamer resolves that case to `builtin://missing-material`, inserts `RenderMaterialFallbackReason::Material`, and keeps the fallback runtime under the originally requested material id so mesh submission can continue while stats and diagnostics remain tied to the missing handle.

`RenderMaterialReadinessReport::push_validation_error_once(...)`, `push_fallback_usage_once(...)`, and `push_diagnostic_once(...)` keep merged streamer diagnostics stable when one material resolution path reports the same shader, texture, material fallback, or local material diagnostic more than once. `MaterialUniform` rows use the same DTO-level de-duplication, so repeated unsupported uniform entries such as `uniform.debug_label` do not inflate readiness diagnostics or submit stats.

`MaterialAsset::readiness_report_with_shader_contract(...)` composes the normal dependency-resolution report with shader-contract diagnostics and shader payload readiness. Missing runtime WGSL is folded into the existing blocking `MissingRuntimeShaderSource` row; imported WGSL capture diagnostics stay `MissingWgslCapture`; invalid entry-point stages and shader definition problems are folded into `ShaderReadinessDiagnostic`. This makes schema mismatches and shader payload gaps readiness failures without turning `.zmaterial` or `.zshader` import into a hard failure, which keeps authoring tools able to open broken material assets and show actionable rows.

## Runtime PBR Baseline

M5A makes StandardMaterial fields visible to the concrete mesh path without moving asset or editor ownership into graphics. `resource_streamer_ensure_material.rs` reads the descriptor, resolves every PBR texture slot, resolves non-standard material texture slots declared outside the fixed PBR aliases, projects shader schema properties into typed runtime values, resolves shader runtime source, records fallback diagnostics, and builds a `MaterialRuntime` plus `PipelineKey`. Missing textures and missing shaders can degrade to default fallback resources; invalid alpha-mask cutoff and missing runtime WGSL remain blocking validation errors.

Shader property values are runtime prepared state, not uniform-buffer encoding yet. `MaterialAsset::shader_property_values_for_shader(...)` walks the loaded shader property schema, takes a material override first and then the shader default, converts supported TOML bool/number/string/vector rows into `RenderMaterialPropertyValue`, and stores the result on `MaterialRuntime.shader_property_values`. `RenderMaterialPropertyValueSummary::from_values(...)` gives runtime/editor code a compact count of the projected map before uniform payload encoding, so a panel can distinguish "the renderer received four shader values, three can become uniform bytes, and one remains non-uniform metadata" from later upload state. Unknown or mistyped overrides remain readiness diagnostics from the shader-contract validation path; the projection only forwards values that match supported schema kinds.

Material property uniform payloads are prepared from the same map during `ResourceStreamer::ensure_material(...)` and stored on `MaterialRuntime.shader_property_uniform_payload`. Numeric and vector rows produce stable byte ranges and field metadata; unsupported rows such as strings are reported inside the payload and merged into the material readiness report as non-blocking `MaterialUniform` diagnostics so later reflection work can make an explicit decision. Both material overrides and shader schema defaults enter this path after `MaterialAsset::shader_property_values_for_shader(...)` projects them into typed values. `ResourceStreamer::material_readiness_report(...)` exposes those rows with stable `uniform.<name>` paths and now stores `RenderMaterialPropertyUniformSummary` directly on the report after resource preparation, so editor/runtime panels can read payload byte length, encoded field count, and unsupported row count from the same readiness DTO. `ResourceStreamer::material_uniform_summary(...)` remains a direct accessor for callers that only need the compact uniform record, and the existing individual accessors still expose backing buffer length and legacy count queries. The streamer now also creates a renderer-owned `GpuMaterialUniformResource` per prepared material, backed by a WGPU uniform buffer and bind group with a 64-byte minimum allocation for the current neutral material binding contract. Submit-level stats preserve those unsupported rows through `RenderStats.last_material_diagnostic_count`, so a material can remain ready while still reporting that a string property was retained but not uploaded.

When the referenced shader carries an authored `pipeline_layout`, `resource_streamer_validate_material_shader_layout.rs` checks that the material section matches the renderer's fixed group-3 uniform contract before the runtime material report is stored. Because compound `.zshader` import now stores `[pipeline_layout]` on `ShaderAsset.pipeline_layout`, these diagnostics operate on real authored shader packages as well as programmatic shader fixtures; `render_product_streamer_reports_imported_zshader_material_layout_abi_diagnostics` covers the imported package path. `ShaderAsset::readiness_report()` now exposes the same authored layout shape as context, but does not run this renderer ABI policy. These diagnostics are non-blocking readiness rows for now; they make an incompatible custom `.zshader` visible before later pipeline creation or shader reflection work tries to consume unsupported material bindings.

Non-standard material texture slots are runtime prepared state, not full shader reflection yet. `MaterialRuntime.non_standard_texture_slots` records the resolved `ResourceId` for ready slot textures and keeps `None` for slots that fell back, so later renderer bind-group and shader-reflection work can distinguish "slot existed but fell back" from "slot was never authored" without reparsing `.zmaterial`.

Mesh pipeline creation consumes `PipelineKey`: `double_sided` disables back-face culling, `alpha_blend` controls transparent blend/depth-write behavior, `alpha_mask` and cutoff bits keep mask variants distinct, and PBR texture slots select authored material variants. Alpha-mask materials are not treated as transparent; only blend mode reports `PipelineKey::is_transparent()`. The mesh and deferred geometry pipeline layouts now include material group 3, and mesh draws bind the prepared material uniform after model and texture bindings. The fallback and deferred geometry shaders declare the neutral group-3 material uniform but do not yet use it for shading until shader reflection/property binding semantics are expanded.

`ResourceStreamer::ensure_scene_resources(...)` counts prepared materials and folds each material readiness report into renderer-facing counters. `RenderStats` exposes `last_material_count`, `last_material_ready_count`, `last_material_fallback_count`, `last_material_validation_error_count`, and `last_material_diagnostic_count` so submit tests and tools can distinguish ready materials, fallback/degraded materials, blocking validation errors, and non-blocking import/authoring notes.

The `.zmaterial` assetization lane now owns material source parsing: built-in material import accepts `.zmaterial`, shader-owned overrides, and texture-slot references. This render-framework module still owns neutral runtime readiness and PBR descriptor projection only; material-editor authoring, automatic shader reflection, and a full physically based shader rewrite remain outside this document's runtime material contract scope.
