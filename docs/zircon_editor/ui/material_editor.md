---
related_code:
  - zircon_editor/src/ui/material_editor/mod.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/tests/ui/material_editor/mod.rs
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/mod.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
implementation_files:
  - zircon_editor/src/ui/material_editor/mod.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/tests/ui/material_editor/mod.rs
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - zircon_editor/src/tests/ui/mod.rs
plan_sources:
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
tests:
  - zircon_editor/src/tests/ui/material_editor/projection.rs
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m3-wsl cargo test -p zircon_editor --lib material_editor_projection --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_editor --lib material_editor --locked --offline --jobs 1 --message-format short --color never
  - cargo test -p zircon_editor --lib material_editor --locked --jobs 1
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

## Scope Boundary

The current module is a structural preview foundation. It does not implement a live sphere/plane preview, material graph editing, shader reflection beyond the runtime lightweight capture diagnostics, or Asset Inspector mutations. The dedicated Material Editor window descriptor already exists separately; this module supplies the model that window can consume in a later UI integration slice.
