---
related_code:
  - zircon_editor/src/ui/material_editor/mod.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/src/ui/material_editor/renderer_data_projection.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/tests/ui/material_editor/mod.rs
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs
  - zircon_editor/src/tests/ui/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs
implementation_files:
  - zircon_editor/src/ui/material_editor/mod.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/src/ui/material_editor/renderer_data_projection.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/tests/ui/material_editor/mod.rs
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs
  - zircon_editor/src/tests/ui/mod.rs
plan_sources:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md
tests:
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs
  - zircon_editor/src/ui/material_editor/projection.rs::tests::texture_dimension_mismatch_projects_dependency_diagnostic
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs::renderer_data_projection_groups_diagnostics_by_feature_name
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs::renderer_data_projection_groups_diagnostics_by_source
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs::renderer_data_projection_groups_diagnostics_by_severity
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs::renderer_data_projection_uses_runtime_diagnostic_ownership_without_shader_duplicates
  - cargo test -p zircon_editor --lib material_editor_projection_maps_runtime_validation_errors_to_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0603-clean --message-format short --color never -- --test-threads=1
  - cargo test -p zircon_editor --lib renderer_data_projection_maps_diagnostics_to_feature_rows --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-m6-root-0603-clean --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m3-wsl cargo test -p zircon_editor --lib material_editor_projection --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_editor --lib material_editor --locked --offline --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib material_editor --locked --jobs 1
  - cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never (2026-05-20 SRP RendererData final: 8 focused tests passed with CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1)
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m3-wsl cargo test -p zircon_editor --lib builtin_window_descriptors --locked --jobs 1 --message-format short --color never
  - cargo check -p zircon_editor --lib --locked
doc_type: module-detail
---

# Material Editor

## Purpose

`zircon_editor::ui::material_editor` owns the editor-only structural projection for `.zmaterial` authoring. It does not prepare GPU resources, mutate runtime world state, or parse source files. The runtime asset and render-framework crates remain the owners of `.zmaterial`, `.zshader`, dependency readiness, and typed material diagnostics.

## Projection Model

`MaterialEditorProjection::from_material(...)` accepts a `MaterialAsset` plus an optional loaded `ShaderAsset`. The optional shader keeps the Material Editor openable when the shader reference is unresolved or still loading; in that state the projection shows authored material overrides and texture slots without shader kind/default metadata.

When a shader contract is available, property rows are emitted in shader schema order and include `kind`, `group`, `label`, default value, authored override value, and an `is_overridden` flag. Material overrides that are not declared by the shader are appended as schema-less rows so the editor can still render and highlight them instead of dropping the authored data.

Texture slot rows follow the same pattern. Shader-owned slots provide `kind`, `group`, `label`, and default fallback metadata. Material-authored slots add concrete texture references or fallback-only values. Unknown material texture slots are appended as schema-less rows and remain visible for diagnostics and repair.

## Diagnostics

The projection maps runtime material validation into `MaterialEditorDiagnosticRow` values with a diagnostic source, stable source path, and human-facing message. Material-owned validation rows such as invalid `overrides.alpha_mode.cutoff` and invalid `overrides.lighting_model` keep their authored override path and stay unclassified, because they are local material controls rather than shader-schema, texture-slot, or dependency-resolution failures. Stored material diagnostics from `MaterialAsset.validation_diagnostics` stay visible as generic material diagnostics at `material.validation_diagnostics`.

Shader contract mismatches come from `MaterialAsset::shader_contract_diagnostics(...)`. Required shader properties and required texture slots both surface as repairable schema diagnostics: property rows point at `overrides.<name>`, while texture-slot rows point at `textures.<slot>` and explain that a concrete material texture reference is required. Shader-side `validation_diagnostics` are preserved at `shader.validation_diagnostics`; only entries with the importer's `wgsl_capture` prefix are tagged as `RenderMaterialDiagnosticSource::WgslCapture`, while generic shader validation text remains unclassified instead of being misrouted as a capture miss. Shader payload readiness rows emitted as `RenderMaterialValidationError::ShaderReadinessDiagnostic` preserve the runtime `ShaderReadiness` source, stable path, and diagnostic message so entry-point and shader-definition issues remain visible in the editor.

Texture dimension mismatches are dependency-resolution diagnostics. Both the material projection and RendererData projection preserve the stable `textures.<slot>` path, the texture locator, and the typed expected/actual dimensions so a cubemap-vs-2D binding failure is actionable rather than collapsed into an unknown validation row.

This M1 editor projection intentionally keeps diagnostics read-only. It gives later UI panels enough structure to group rows by property, texture slot, and diagnostic source without coupling the authoring view to renderer internals.

## RendererData Projection

`RendererDataEditorProjection::from_renderer_asset(...)` adds a read-only editor view over runtime-owned SRP RendererData state. It consumes a `RendererAsset` plus `RendererFeatureContractDiagnostic` rows from `zircon_runtime::graphics::pipeline`; it does not load assets, query `ProjectManager`, own runtime asset truth, mutate render state, or touch WGPU objects.

Renderer feature rows expose the compiled runtime feature name, source, enabled state, optional quality gate, shader/material references, required entry points, expected material properties, expected texture slots, and a per-feature diagnostic count. The projection keeps feature names aligned with runtime descriptor names such as `mesh`, so editor diagnostics group against the same identifiers emitted by asset-aware SRP compile reports.

RendererData diagnostic rows preserve the runtime feature name, runtime severity, and stable editor paths. Missing shader/material references are dependency-resolution diagnostics; missing entry points/properties/texture slots are shader-schema or texture-slot diagnostics; material-owned shader misses, material-shader mismatches, material validation errors, and stored material validation strings expose `material_reference` so editor panels can group or jump to the `.zmaterial` asset that produced the row. Rows that point at `.zshader` assets expose `shader_references`. `RendererDataEditorProjection::diagnostics_by_feature()` groups diagnostics by the same runtime feature names used by compile reports, `RendererDataEditorProjection::diagnostics_by_shader()` groups shader-owned diagnostics, shader mismatches, and shader-backed material-contract rows by `.zshader`, `RendererDataEditorProjection::diagnostics_by_material()` provides the canonical read-only grouping for material-owned rows while intentionally excluding feature-shader-only diagnostics, `RendererDataEditorProjection::diagnostics_by_source()` groups classified rows by the runtime diagnostic source accessor while leaving unclassified material-local rows out, and `RendererDataEditorProjection::diagnostics_by_severity()` groups repair-blocking errors separately from warning-level stored material/shader validation strings. The neutral diagnostic source and severity enums are orderable/hashable so editor projections can expose stable typed grouping maps without stringifying runtime DTO values.

The RendererData projection derives those material, shader, source, and severity grouping fields from the runtime `RendererFeatureContractDiagnostic::material_reference()`, `RendererFeatureContractDiagnostic::shader_references()`, `RendererFeatureContractDiagnostic::source()`, and `RendererFeatureContractDiagnostic::severity()` accessors instead of re-decoding the diagnostic enum inside editor match arms. This keeps editor grouping aligned with `RenderPipelineCompileReport` grouping and collapses duplicate shader references when a material validation error and its shader-contract source name the same `.zshader`.

Material validation rows reuse the same messages and `RenderMaterialDiagnosticSource` mapping as the material projection, including material-owned lighting-model failures, required texture-slot failures, and shader payload readiness rows emitted by runtime shader/material contract validation. Shader validation strings remain generic unless they carry the importer `wgsl_capture` prefix.

The 2026-07-10 editor architecture M1 WSL gate exposed `TextureDimensionMismatch` as a newly added runtime validation variant that the two editor projections did not yet consume. The projections now map it exhaustively and include focused behavioral coverage; exact Windows and WSL test results are recorded in the active Plan 01 M1 acceptance evidence after the shared test target finishes rebuilding.

Final SRP RendererData editor validation on 2026-05-20 used `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never` passed 8 focused tests, and `cargo check -p zircon_editor --lib --locked --jobs 1 --color never` passed. The only editor warnings were unrelated sprite-atlas unused-item warnings outside this projection lane.

## Scope Boundary

The current module is a structural preview foundation. It does not implement a live sphere/plane preview, material graph editing, shader reflection beyond the runtime lightweight capture diagnostics, or Asset Inspector mutations. The dedicated Material Editor window descriptor already exists separately; this module supplies the model that window can consume in a later UI integration slice.

The RendererData projection has the same boundary. It is not a mutable renderer-data authoring UI, GPU preview, ShaderGraph/VFX Graph surface, shader variant compiler, or WGPU pipeline-specialization hook.
