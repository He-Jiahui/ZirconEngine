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
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md
tests:
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs
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

The projection maps runtime material validation into `MaterialEditorDiagnosticRow` values with a diagnostic source, stable source path, and human-facing message. Stored material diagnostics from `MaterialAsset.validation_diagnostics` stay visible as generic material diagnostics at `material.validation_diagnostics`.

Shader contract mismatches come from `MaterialAsset::shader_contract_diagnostics(...)`. Shader-side `validation_diagnostics` are preserved at `shader.validation_diagnostics`; only entries with the importer's `wgsl_capture` prefix are tagged as `RenderMaterialDiagnosticSource::WgslCapture`, while generic shader validation text remains unclassified instead of being misrouted as a capture miss.

This M1 editor projection intentionally keeps diagnostics read-only. It gives later UI panels enough structure to group rows by property, texture slot, and diagnostic source without coupling the authoring view to renderer internals.

## RendererData Projection

`RendererDataEditorProjection::from_renderer_asset(...)` adds a read-only editor view over runtime-owned SRP RendererData state. It consumes a `RendererAsset` plus `RendererFeatureContractDiagnostic` rows from `zircon_runtime::graphics::pipeline`; it does not load assets, query `ProjectManager`, own runtime asset truth, mutate render state, or touch WGPU objects.

Renderer feature rows expose the compiled runtime feature name, source, enabled state, optional quality gate, shader/material references, required entry points, expected material properties, expected texture slots, and a per-feature diagnostic count. The projection keeps feature names aligned with runtime descriptor names such as `mesh`, so editor diagnostics group against the same identifiers emitted by asset-aware SRP compile reports.

RendererData diagnostic rows preserve the runtime feature name and map SRP contract diagnostics into stable editor paths. Missing shader/material references are dependency-resolution diagnostics; missing entry points/properties/texture slots are shader-schema or texture-slot diagnostics; material validation errors reuse the same messages and `RenderMaterialDiagnosticSource` mapping as the material projection; stored material validation strings remain generic material diagnostics; shader validation strings remain generic unless they carry the importer `wgsl_capture` prefix.

Final SRP RendererData editor validation on 2026-05-20 used `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never` passed 8 focused tests, and `cargo check -p zircon_editor --lib --locked --jobs 1 --color never` passed. The only editor warnings were unrelated sprite-atlas unused-item warnings outside this projection lane.

## Scope Boundary

The current module is a structural preview foundation. It does not implement a live sphere/plane preview, material graph editing, shader reflection beyond the runtime lightweight capture diagnostics, or Asset Inspector mutations. The dedicated Material Editor window descriptor already exists separately; this module supplies the model that window can consume in a later UI integration slice.

The RendererData projection has the same boundary. It is not a mutable renderer-data authoring UI, GPU preview, ShaderGraph/VFX Graph surface, shader variant compiler, or WGPU pipeline-specialization hook.
