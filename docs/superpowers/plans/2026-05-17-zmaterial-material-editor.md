# ZMaterial Material Editor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Hard-cut material source assets to `.zmaterial`, make `.zshader` own shader property and texture-slot schema, import material instances through shader-driven overrides, and scaffold a dedicated Material Editor structural preview path.

**Architecture:** `zircon_runtime::asset` owns `.zshader` and `.zmaterial` document contracts and importers, `zircon_runtime::core::framework::render::material` owns neutral readiness diagnostics, and `zircon_editor` owns authoring presentation. `.zmeta` remains the only asset identity/dependency authority; `.material.toml` support is removed rather than kept as a compatibility bridge.

**Tech Stack:** Rust 2021, Cargo workspace packages `zircon_runtime` and `zircon_editor`, TOML via `toml`, Serde DTOs, existing asset importer/project manager, existing editor activity-window/view descriptor infrastructure.

---

## Current Baseline

- `zircon_runtime/src/asset/assets/shader/zshader.rs` already defines `ZShaderDocument` and `ShaderMaterialPropertyAsset`.
- `zircon_runtime/src/asset/assets/material/material_asset.rs` already parses TOML and stores PBR fields plus `property_values`.
- `zircon_runtime/src/asset/importer/ingest/asset_importer.rs` still registers `.material.toml` via `zircon.builtin.toml.material`.
- `zircon_runtime/src/asset/importer/ingest/import_shader_package.rs` imports compound `.zshader` packages and propagates `property_schema`.
- `zircon_editor` already has a `editor.material_editor_window` descriptor and menu event, but no dedicated material asset projection module.
- Active coordination notes report heavy dirty workspace and possible Cargo process pressure; milestone testing stages should use scoped commands first.

## File Structure

- Modify `zircon_runtime/src/asset/assets/shader/zshader.rs`: add texture-slot document/asset types and lightweight capture helpers.
- Modify `zircon_runtime/src/asset/assets/shader/shader_asset.rs`: add `texture_slots` and capture diagnostics fields.
- Modify `zircon_runtime/src/asset/assets/shader/mod.rs`: re-export new shader texture-slot types.
- Add `zircon_runtime/src/asset/assets/material/zmaterial.rs`: source-document DTOs for `.zmaterial`.
- Add `zircon_runtime/src/asset/assets/material/texture_slot.rs`: material texture slot value and helpers.
- Modify `zircon_runtime/src/asset/assets/material/material_asset.rs`: add shader-driven fields while keeping existing fields only where needed for compile compatibility.
- Modify `zircon_runtime/src/asset/assets/material/dependency_set.rs`: derive dependencies from shader plus new texture slot map.
- Modify `zircon_runtime/src/asset/assets/material/mod.rs`: register new child modules/re-exports.
- Modify `zircon_runtime/src/asset/importer/ingest/import_material.rs`: parse `.zmaterial` and emit dependency locators.
- Modify `zircon_runtime/src/asset/importer/ingest/asset_importer.rs`: replace `.material.toml` registration with `.zmaterial`.
- Modify `zircon_runtime/src/core/framework/render/material/{readiness_report.rs,validation_error.rs,mod.rs}`: add diagnostic source/path fields without breaking existing report callers.
- Modify/add tests under `zircon_runtime/src/asset/tests/assets/material.rs`, `zircon_runtime/src/asset/tests/project/zmeta.rs`, and render-product/material readiness tests.
- Add `zircon_editor/src/ui/material_editor/` with a small projection model for M1 structural preview if existing UI paths do not already own this behavior.
- Modify `zircon_editor/src/lib.rs` or `zircon_editor/src/ui/mod.rs` only if needed for structural module wiring; keep root files thin.
- Update docs `docs/zircon_runtime/asset/zmeta-shader-material.md` and `docs/zircon_runtime/asset/render-assets.md`.

## Milestone 1: Runtime Document Contracts And Hard Cutover

- Goal: `.zshader` can declare texture slots, `.zmaterial` can parse shader overrides and texture slots, and the built-in material importer accepts only `.zmaterial`.
- In-scope behaviors: `.zshader` property schema remains valid; new texture slots parse/roundtrip; `.zmaterial` parses/roundtrips; `.material.toml` is no longer registered; material direct references include shader and textures.
- Dependencies: current `AssetReference`, `AssetUri`, `AssetKind::Material`, `ImportedAsset::Material`, and `AssetImporterRegistry` suffix matching.

### Implementation Slices

- [ ] Add `ZShaderTextureSlotDocument` and `ShaderTextureSlotAsset` in `zircon_runtime/src/asset/assets/shader/zshader.rs` with fields `name`, `kind`, `default`, `sampler`, `group`, `label`, and `editor`.
- [ ] Add `texture_slots: Vec<ShaderTextureSlotAsset>` to `ShaderAsset` and populate it in both `import_shader.rs` and `import_shader_package.rs`.
- [ ] Add `ZMaterialDocument` in `zircon_runtime/src/asset/assets/material/zmaterial.rs` with `version`, `name`, `shader: AssetReference`, `overrides: BTreeMap<String, toml::Value>`, and `textures: BTreeMap<String, MaterialTextureSlotValue>`.
- [ ] Add `MaterialTextureSlotValue` in `zircon_runtime/src/asset/assets/material/texture_slot.rs` with `reference: Option<AssetReference>` and `fallback: Option<String>`.
- [ ] Extend `MaterialAsset` with `property_overrides` and `texture_slots`; keep `property_values` mapped to `property_overrides` where necessary to avoid breaking existing callers during this milestone.
- [ ] Replace built-in material importer registration id/suffix with `zircon.builtin.zmaterial` and `.zmaterial` only.
- [ ] Update `import_material.rs` to parse `.zmaterial`, build `MaterialAsset`, and call `AssetImportOutcome::with_dependency` for shader and texture references.
- [ ] Add/adjust runtime asset tests for `.zmaterial` roundtrip, texture slot dependency extraction, `.zshader` texture slot parse, and `.material.toml` not matching a material importer.
- [ ] Update docs `docs/zircon_runtime/asset/zmeta-shader-material.md` with the hard cutover and source examples.

### Testing Stage: M1 Runtime Contract Gate

- Commands:
  - `cargo test -p zircon_runtime --locked material --jobs 1`
  - `cargo test -p zircon_runtime --locked shader --jobs 1`
  - `cargo test -p zircon_runtime --locked asset::tests::project::zmeta --jobs 1`
- Debug/correction loop: if a high-level project test fails, inspect suffix selection and direct dependency extraction before changing editor or graphics paths.
- Exit evidence: tests pass or failures are documented as unrelated active-session/environment blockers with exact diagnostics.

## Milestone 2: Diagnostics, Capture, And Readiness Reports

- Goal: shader/material/WGSL mismatches import successfully with structured diagnostics and readiness fallback reporting.
- In-scope behaviors: unknown override, unknown texture slot, missing shader, missing texture, missing WGSL capture, and wrong property type become diagnostics/readiness errors without parse failure.
- Dependencies: Milestone 1 document and import model.

### Implementation Slices

- [ ] Add a neutral diagnostic source enum under `zircon_runtime/src/core/framework/render/material/` for `ShaderSchema`, `WgslCapture`, `MaterialOverride`, `TextureSlot`, and `DependencyResolution`.
- [ ] Extend `RenderMaterialValidationError` variants or add diagnostic wrapper structs so field path and source can be preserved.
- [ ] Add material validation helpers in `zircon_runtime/src/asset/assets/material/validation.rs` that compare `MaterialAsset` overrides/texture slots with a `ShaderAsset` contract.
- [ ] Add lightweight WGSL capture checks in shader import that search expected property and texture-slot names in the combined WGSL source and emit diagnostics when absent.
- [ ] Update `MaterialAsset::readiness_report_with_resolution` to include new texture slot map and diagnostic source records while preserving existing fallback behavior.
- [ ] Add tests for unknown override, unknown texture slot, missing WGSL capture, unresolved shader, unresolved texture, and fallback usage.
- [ ] Update `docs/zircon_runtime/asset/render-assets.md` with readiness/capture details.

### Testing Stage: M2 Readiness Gate

- Commands:
  - `cargo test -p zircon_runtime --locked material --jobs 1`
  - `cargo test -p zircon_runtime --locked render_product_assets --jobs 1`
  - `cargo check -p zircon_runtime --lib --locked`
- Debug/correction loop: fix the lowest shared material validation or render-framework diagnostic issue before touching resource streamer tests.
- Exit evidence: readiness tests prove diagnostics are preserved and import does not hard-fail for schema/capture mismatch.

## Milestone 3: Material Editor M1 Structural Projection

- Goal: the dedicated Material Editor window has a structural projection model for `.zmaterial` shader/override/texture/diagnostic state.
- In-scope behaviors: descriptor/open route remains, a projection model can be built from a `MaterialAsset` plus optional `ShaderAsset`, and tests prove grouped fields and texture slots are visible. No runtime sphere/plane preview yet.
- Dependencies: Milestones 1 and 2 runtime DTOs.

### Implementation Slices

- [ ] Add `zircon_editor/src/ui/material_editor/mod.rs` as thin wiring.
- [ ] Add `zircon_editor/src/ui/material_editor/projection.rs` with `MaterialEditorProjection`, `MaterialEditorPropertyRow`, `MaterialEditorTextureSlotRow`, and `MaterialEditorDiagnosticRow`.
- [ ] Implement projection from `MaterialAsset` plus optional `ShaderAsset`, grouping by shader property/texture slot hints and showing default vs override state.
- [ ] Add a minimal test module under `zircon_editor/src/tests/ui/material_editor.rs` or the existing UI boundary tree to assert projection rows for shader properties, texture slots, and diagnostics.
- [ ] If needed, expose the module through `zircon_editor/src/ui/mod.rs` without adding behavior to root files.
- [ ] Update editor docs only if a code-facing document already owns material editor UI; otherwise create `docs/zircon_editor/ui/material_editor.md` with the required header.

### Testing Stage: M3 Editor Projection Gate

- Commands:
  - `cargo test -p zircon_editor --lib material_editor --locked --jobs 1`
  - `cargo test -p zircon_editor --lib builtin_window_descriptors --locked --jobs 1`
  - `cargo check -p zircon_editor --lib --locked`
- Debug/correction loop: keep editor projection isolated from runtime world and avoid direct graphics/wgpu dependencies.
- Exit evidence: editor tests prove the Material Editor M1 model is structural and does not require runtime preview.

## Milestone 4: Final Docs And Scoped Acceptance

- Goal: documentation and scoped validation reflect the hard cutover and M1 editor/runtime behavior.
- In-scope behaviors: docs headers updated, plan/spec references included, scoped validation run, active coordination note retired or updated.
- Dependencies: Milestones 1-3.

### Implementation Slices

- [ ] Update `docs/zircon_runtime/asset/zmeta-shader-material.md` related code, implementation files, plan sources, and tests.
- [ ] Update `docs/zircon_runtime/asset/render-assets.md` related code and tests for material readiness changes.
- [ ] Add or update `docs/zircon_editor/ui/material_editor.md` if editor projection code lands.
- [ ] Update `.codex/sessions/20260516-1455-zmaterial-material-editor-design.md` with validation outcomes or retire it after completion.

### Testing Stage: Final Scoped Acceptance

- Commands:
  - `cargo fmt --all -- --check`
  - `cargo test -p zircon_runtime --locked material --jobs 1`
  - `cargo test -p zircon_runtime --locked shader --jobs 1`
  - `cargo test -p zircon_editor --lib material_editor --locked --jobs 1`
  - `cargo check -p zircon_runtime --lib --locked`
  - `cargo check -p zircon_editor --lib --locked`
- If shared contract changes reach broader workspace surfaces and local Cargo pressure allows it, also run `.
.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime`.
- Exit evidence: all scoped commands pass or exact blockers are reported with failing output and owner scope.

## Self-Review

- Spec coverage: the plan covers hard `.zmaterial` cutover, shader-owned schema/defaults, texture slot table, diagnostics/readiness, Material Editor M1, M2 preview deferral, and docs.
- Placeholder scan: no `TBD` or undefined implementation steps are used; every milestone names files, behaviors, and commands.
- Type consistency: names are consistent across plan/spec: `ZMaterialDocument`, `MaterialTextureSlotValue`, `ShaderTextureSlotAsset`, `.zmaterial`, `.zshader`, `texture_slots`, `property_overrides`.
