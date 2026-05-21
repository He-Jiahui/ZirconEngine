# SRP RendererData ZMaterial Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Unity SRP-style RendererData/RendererFeature asset workflow that treats `.zshader` and `.zmaterial` as shader/material contract truth, validates those contracts during runtime SRP asset compile, then exposes the result through an editor read-only projection.

**Architecture:** `zircon_runtime::asset` remains the owner of `.zshader`, `.zmaterial`, imported `ShaderAsset`, imported `MaterialAsset`, and material/shader readiness diagnostics. `zircon_runtime::graphics::pipeline` gains the SRP RendererData data model, asset-aware compile report, and contract-diagnostic layer without prewarming GPU resources or compiling shader variants. `zircon_editor` consumes the runtime projection data after the runtime layer is complete; it does not own runtime asset truth or mutate render state.

**Tech Stack:** Rust 2021, Cargo workspace packages `zircon_runtime` and `zircon_editor`, Serde/TOML, existing `AssetReference`/`AssetUri`/`ImportedAsset`, existing `RenderPipelineAsset` compile path, existing `.zshader`/`.zmaterial` contracts, Unity `dev/Graphics` SRP references.

---

## Approved Scope

- Implement approved scheme 1: runtime-only RendererData asset support first, editor structural projection second.
- Use the stronger contract boundary selected by the user: SRP asset compile resolves `.zshader`/`.zmaterial` contracts and emits diagnostics.
- Do not prewarm GPU resources, compile shader variants, specialize WGPU pipelines, or rewrite material draw paths in this plan.
- Do not reintroduce `.material.toml` compatibility.
- Avoid active M9A provider/app bootstrap surfaces: `zircon_app/src/entry/first_party_runtime_plugins.rs`, plugin manifests, VG/HGI provider arbitration, and advanced profile runtime planning.

## Reference Evidence

- `dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRendererData.cs`: Unity stores renderer feature lists in a renderer data asset and invalidates/rebuilds render passes when the asset changes.
- `dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRendererFeature.cs`: Unity renderer features have active state, resource creation, camera pre-cull hooks, and pass injection. Zircon will map only the data/diagnostic subset in this plan.
- `dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ScriptableRenderer.cs`: Unity renderer records feature-provided passes into RenderGraph after gathering feature state. Zircon already has descriptor/executor graph compile and will keep pass recording in `RenderPipelineAsset::compile_with_options(...)`.
- `zircon_runtime/src/graphics/pipeline/declarations/{render_pipeline_asset.rs,renderer_asset.rs,renderer_feature_asset.rs}`: current Zircon SRP data layer already has pipeline, renderer, and feature asset structs but lacks serialized RendererData documents and asset contract diagnostics.
- `zircon_runtime/src/asset/assets/shader/{zshader.rs,shader_asset.rs}` and `zircon_runtime/src/asset/assets/material/{zmaterial.rs,material_asset.rs}`: current shader/material truth already supports shader entry points, property schema, texture slots, `.zmaterial` overrides, readiness reports, and material/shader contract diagnostics.
- `zircon_editor/src/ui/material_editor/projection.rs`: current editor pattern for read-only material projection should guide the renderer data projection rather than adding direct renderer mutation.

## File Structure

- Add `zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs`: serialized TOML-facing RendererData document DTOs and helper conversion entry points.
- Add `zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_reference.rs`: feature-local shader/material contract references and expected entry/property/texture declarations.
- Add `zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs`: diagnostic enum/row data for asset-aware compile reports.
- Add `zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_report.rs`: wrapper containing `CompiledRenderPipeline` plus diagnostics.
- Modify `zircon_runtime/src/graphics/pipeline/declarations/{mod.rs,renderer_feature_asset.rs}`: wire new declarations and store feature contract references on `RendererFeatureAsset`.
- Add `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_with_asset_context.rs`: asset-aware compile entry point that resolves shader/material artifacts, validates contracts, then delegates pass graph compile.
- Modify `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/mod.rs`: register the new compile module.
- Modify `zircon_runtime/src/graphics/pipeline/mod.rs` and `zircon_runtime/src/graphics/mod.rs`: re-export only the public SRP data/report types needed by tests/editor.
- Add `zircon_runtime/src/graphics/tests/renderer_data_asset.rs`: focused runtime tests for parse/roundtrip, conversion, contract diagnostics, and compile report behavior.
- Modify `zircon_runtime/src/graphics/tests/mod.rs`: include the new test module.
- Add `zircon_editor/src/ui/material_editor/renderer_data_projection.rs`: read-only renderer data projection rows and diagnostic mapping.
- Modify `zircon_editor/src/ui/material_editor/mod.rs`: re-export projection types; keep the module root thin.
- Add `zircon_editor/src/tests/ui/material_editor/renderer_data_projection.rs` or extend the existing `projection.rs` only if the existing test file remains focused and under control.
- Modify `zircon_editor/src/tests/ui/material_editor/mod.rs` only if the current `material_editor` test module is directory-backed; otherwise split the test module without adding behavior to root test files.
- Add `docs/zircon_runtime/graphics/srp-renderer-data.md`: runtime SRP RendererData workflow doc with required machine-readable header.
- Update `docs/assets-and-rendering/render-framework-architecture.md`: add a concise SRP RendererData/zmaterial section and related code/test entries.
- Update `docs/zircon_editor/ui/material_editor.md`: document the renderer data projection if editor stage lands.
- Update `.codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md`: keep coordination state current during implementation and retire/archive on completion.

## Key Types And Signatures

Use these names consistently across implementation. If existing local names require small spelling adjustments, update the plan note and tests in the same change.

```rust
pub struct RendererDataDocument {
    pub version: u32,
    pub name: String,
    pub stages: Vec<String>,
    pub features: Vec<RendererFeatureDocument>,
}

pub struct RendererFeatureDocument {
    pub name: String,
    pub source: String,
    pub enabled: bool,
    pub quality_gate: Option<String>,
    pub shader: Option<AssetReference>,
    pub material: Option<AssetReference>,
    pub required_entry_points: Vec<String>,
    pub expected_properties: Vec<String>,
    pub expected_texture_slots: Vec<String>,
    pub local_config: BTreeMap<String, String>,
}

pub struct RendererFeatureAssetReferences {
    pub shader: Option<AssetReference>,
    pub material: Option<AssetReference>,
    pub required_entry_points: Vec<String>,
    pub expected_properties: Vec<String>,
    pub expected_texture_slots: Vec<String>,
}

pub enum RendererFeatureContractDiagnostic {
    ShaderMissing { feature: String, reference: AssetReference },
    MaterialMissing { feature: String, reference: AssetReference },
    MaterialShaderMismatch { feature: String, feature_shader: AssetReference, material_shader: AssetReference },
    MissingEntryPoint { feature: String, shader: AssetReference, entry_point: String },
    MissingProperty { feature: String, shader: AssetReference, property: String },
    MissingTextureSlot { feature: String, shader: AssetReference, slot: String },
    MaterialValidation { feature: String, error: RenderMaterialValidationError },
    ShaderValidation { feature: String, shader: AssetReference, diagnostic: String },
}

pub struct RenderPipelineCompileReport {
    pub pipeline: CompiledRenderPipeline,
    pub diagnostics: Vec<RendererFeatureContractDiagnostic>,
}

pub trait RenderPipelineAssetContext {
    fn load_shader_asset(&self, reference: &AssetReference) -> Option<ShaderAsset>;
    fn load_material_asset(&self, reference: &AssetReference) -> Option<MaterialAsset>;
}
```

Implementation note: keep `RenderPipelineAssetContext` in `zircon_runtime::graphics::pipeline`, not inside `asset::project::manager`, so tests can provide a simple in-memory context and graphics does not depend on `ProjectManager` internals. Add an adapter for `ProjectManager` only if implementation needs it; otherwise tests can prove the contract with a test context.

## Milestone 1: Runtime RendererData Documents And Feature References

- Goal: create a Unity SRP-style RendererData data surface that can express renderer stages, feature entries, and shader/material contract references without changing graph execution.
- In-scope behaviors: TOML parse/roundtrip; conversion from document to existing `RendererAsset`/`RendererFeatureAsset`; stage/source string validation; feature-local shader/material references; required entry/property/texture declarations stored on feature assets.
- Dependencies: existing `RendererAsset`, `RendererFeatureAsset`, `RendererFeatureSource`, `RenderPassStage`, `BuiltinRenderFeature`, `AssetReference`, and `.zshader`/`.zmaterial` contract types.

### Implementation Slices

- [x] Add `renderer_feature_reference.rs` with `RendererFeatureAssetReferences` and builder helpers `with_shader_reference`, `with_material_reference`, `with_required_entry_point`, `with_expected_property`, and `with_expected_texture_slot` on `RendererFeatureAsset`.
- [x] Extend `RendererFeatureAsset` with `asset_references: RendererFeatureAssetReferences`, defaulting to empty in `builtin`, `disabled`, and `plugin` constructors.
- [x] Add `renderer_data_document.rs` with `RendererDataDocument`, `RendererFeatureDocument`, `from_toml_str`, and `to_toml_string`.
- [x] Add conversion helpers from renderer data documents into `RendererAsset`; support built-in feature source strings matching the existing enum variants exactly first, such as `Mesh`, `Sprite`, `PostProcess`, `Ui`, `DebugOverlay`, `AntiAlias`, `Bloom`, `ColorGrading`, and `HistoryResolve`.
- [x] Add explicit string-to-`RenderPassStage` parsing for the current stage names: `DepthPrepass`, `Shadow`, `Deferred`, `AmbientOcclusion`, `Lighting`, `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `Opaque3d`, `AlphaMask3d`, `Transparent3d`, `PostProcess`, `Ui`, `Overlay`, and `Debug`.
- [x] Add unit-test code in `zircon_runtime/src/graphics/tests/renderer_data_asset.rs` for parse/roundtrip, conversion to `RendererAsset`, disabled feature preservation, shader/material reference preservation, and unknown stage/feature errors.
- [x] Add `mod renderer_data_asset;` to `zircon_runtime/src/graphics/tests/mod.rs`.
- [x] Add `docs/zircon_runtime/graphics/srp-renderer-data.md` with frontmatter listing all runtime files introduced in this milestone and the tests that cover them.

### Testing Stage: M1 Runtime Data Gate

- Commands:
  - `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never`
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`
- Debug/correction loop: fix document parsing and conversion errors before touching asset-aware compile; do not add compatibility aliases for misspelled feature or stage names.
- Lightweight checks: `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` may be run before the testing stage only if implementation becomes type-heavy.
- Exit evidence: tests prove renderer data can describe current built-in pipelines and carry shader/material references without changing graph execution.
- 2026-05-20 evidence: with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`, `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` passed 6 focused tests and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` passed. Both emitted only the pre-existing `zircon_runtime/src/scene/world/query.rs::entity_ids_matching_query_archetypes` dead-code warning.

## Milestone 2: Asset-Aware Compile Report And Contract Diagnostics

- Goal: add a compile path that resolves feature-local `.zshader` and `.zmaterial` references, emits diagnostics, and still delegates pass graph construction to the existing compile path.
- In-scope behaviors: missing shader/material diagnostics; material shader mismatch; missing shader entry points; expected property/texture-slot checks; propagation of existing material shader-contract diagnostics; propagation of shader validation diagnostics; pass graph still compiles when diagnostics are non-fatal.
- Dependencies: Milestone 1 feature references plus existing `ShaderAsset`, `MaterialAsset`, `RenderMaterialValidationError`, and `RenderPipelineAsset::compile_with_options(...)`.

### Implementation Slices

- [x] Add `renderer_feature_contract_diagnostic.rs` with `RendererFeatureContractDiagnostic` variants listed above and focused accessor helpers for tests.
- [x] Add `render_pipeline_compile_report.rs` with `RenderPipelineCompileReport { pipeline, diagnostics }`.
- [x] Add `compile_with_asset_context.rs` with `RenderPipelineAssetContext` and `RenderPipelineAsset::compile_with_asset_context(&self, extract, options, context) -> Result<RenderPipelineCompileReport, String>`.
- [x] In `compile_with_asset_context`, call the existing `compile_with_options(...)` first or last consistently. Prefer first, so descriptor/stage/resource errors remain hard compile failures while shader/material authoring mismatches remain diagnostics.
- [x] Add helper `collect_feature_contract_diagnostics(feature, context)` that resolves optional feature shader/material references.
- [x] Emit `ShaderMissing` if `feature.asset_references.shader` cannot resolve to `ShaderAsset`.
- [x] Emit `MaterialMissing` if `feature.asset_references.material` cannot resolve to `MaterialAsset`.
- [x] If both feature shader and material resolve, emit `MaterialShaderMismatch` when `material.shader` does not equal the feature shader reference.
- [x] For every `required_entry_points` entry, emit `MissingEntryPoint` when the resolved `ShaderAsset.entry_points` has no matching `name`.
- [x] For every `expected_properties` entry, emit `MissingProperty` when the resolved `ShaderAsset.property_schema` has no matching `name`.
- [x] For every `expected_texture_slots` entry, emit `MissingTextureSlot` when the resolved `ShaderAsset.texture_slots` has no matching `name`.
- [x] If material and shader resolve, append `MaterialValidation` for each `material.shader_contract_diagnostics(&shader)` error.
- [x] Append `ShaderValidation` for every string in `shader.validation_diagnostics` when shader resolves.
- [x] Add in-memory test context in `renderer_data_asset.rs` that maps `AssetReference.locator` to cloned `ShaderAsset` and `MaterialAsset` values.
- [x] Add runtime tests proving missing shader, missing material, material shader mismatch, missing entry point, missing property, missing texture slot, unknown material override, unknown material texture slot, and WGSL capture diagnostics are all reported while `report.pipeline.graph` still contains expected passes.
- [x] Update `docs/zircon_runtime/graphics/srp-renderer-data.md` with compile report behavior and diagnostic fatal/non-fatal rules.

### Testing Stage: M2 Asset-Aware Compile Gate

- Commands:
  - `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never`
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`
- Debug/correction loop: if a compile report test fails, inspect feature references and shader/material validation before changing graph compile or executor registry. If `pipeline_compile` fails, preserve existing descriptor/resource semantics and adjust only new asset-aware code.
- Lightweight checks: use scoped `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` if the new report/context types cause type errors during implementation.
- Exit evidence: asset-aware compile produces diagnostics for shader/material contract mismatches and does not make authoring mismatches hard graph compile failures.
- 2026-05-20 evidence: with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`, `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` passed 9 focused tests, `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never` passed 39 focused tests, `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never` passed 74 runtime lib tests plus 1 matching integration test, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` passed. All commands emitted only the pre-existing `zircon_runtime/src/scene/world/query.rs::entity_ids_matching_query_archetypes` dead-code warning.

## Milestone 3: Runtime Docs And Product Pipeline Integration Notes

- Goal: document how RendererData complements the existing product render pipeline and `.zshader`/`.zmaterial` workflow without claiming editor or GPU preview behavior.
- In-scope behaviors: docs explain data ownership, compile report rules, Unity SRP alignment, deliberate divergence, tests, and remaining gaps.
- Dependencies: Milestones 1 and 2.

### Implementation Slices

- [x] Update `docs/zircon_runtime/graphics/srp-renderer-data.md` frontmatter with all files and tests added in M1/M2.
- [x] Update `docs/assets-and-rendering/render-framework-architecture.md` with a concise `2026-05-18 SRP RendererData Shader/Material Contract` section.
- [x] Add this plan to `plan_sources` in affected docs.
- [x] Update `.codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md` with M1/M2 validation outcomes or current blockers.

### Testing Stage: M3 Runtime Docs Gate

- Commands:
  - `git diff --check -- docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md docs/zircon_runtime/graphics/srp-renderer-data.md docs/assets-and-rendering/render-framework-architecture.md .codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md`
- Debug/correction loop: fix whitespace/frontmatter/doc consistency before starting editor projection. Do not claim workspace green from docs-only checks.
- Exit evidence: docs include related-code headers and command evidence for runtime scoped validation.
- 2026-05-20 evidence: `docs/zircon_runtime/graphics/srp-renderer-data.md` and `docs/assets-and-rendering/render-framework-architecture.md` now list M1/M2 SRP RendererData implementation files, tests, plan source, fatal graph-compile versus non-fatal shader/material diagnostic rules, Unity SRP alignment, product-pipeline placement, and explicit GPU/editor exclusions. `.codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md` was updated for M3, and `git diff --check -- docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md docs/zircon_runtime/graphics/srp-renderer-data.md docs/assets-and-rendering/render-framework-architecture.md .codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md` passed with only the existing LF-to-CRLF warning for `render-framework-architecture.md`.

## Milestone 4: Editor RendererData Structural Projection

- Goal: expose the runtime asset-aware compile result to editor code as a read-only structural projection, matching the current material editor projection style.
- In-scope behaviors: projection rows for renderer name, stages, feature entries, enabled state, quality gate, shader/material references, required entries/properties/texture slots, and diagnostics. No mutation, no shader graph, no VFX graph, no real GPU preview.
- Dependencies: Milestones 1 through 3.

### Implementation Slices

- [x] Add `zircon_editor/src/ui/material_editor/renderer_data_projection.rs` with `RendererDataEditorProjection`, `RendererDataFeatureRow`, and `RendererDataDiagnosticRow`.
- [x] Implement `RendererDataEditorProjection::from_renderer_asset(renderer: &RendererAsset, diagnostics: &[RendererFeatureContractDiagnostic]) -> Self`.
- [x] Keep projection input runtime-owned: consume `RendererAsset` and diagnostic rows, not `ProjectManager`, `WgpuRenderFramework`, or renderer internals.
- [x] Re-export the new projection types from `zircon_editor/src/ui/material_editor/mod.rs`.
- [x] If `zircon_editor/src/tests/ui/material_editor.rs` is a file, extend it only if it stays readable; otherwise convert `material_editor` tests into a directory with `mod.rs`, `projection.rs`, and `renderer_data_projection.rs` using a direct hard cutover.
- [x] Add editor tests proving feature rows show shader/material references and diagnostics map to feature rows by feature name.
- [x] Update `docs/zircon_editor/ui/material_editor.md` with the renderer data projection section and frontmatter updates.
- [x] Update `.codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md` with editor projection scope and validation outcomes.

### Testing Stage: M4 Editor Projection Gate

- Commands:
  - `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --color never`
  - `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never`
- Debug/correction loop: if editor tests fail, fix projection mapping first; do not pull runtime project manager, graphics renderer, or WGPU state into editor projection.
- Lightweight checks: `cargo check -p zircon_editor --lib --locked --jobs 1 --color never` may be used before the testing stage if module restructuring becomes type-heavy.
- Exit evidence: editor projection can render read-only SRP RendererData state and diagnostics without owning runtime asset truth.
- 2026-05-20 evidence: with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`, `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never` passed 8 focused tests including the 2 new RendererData projection tests; `cargo check -p zircon_editor --lib --locked --jobs 1 --color never` passed; and `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` passed 10 focused tests after review added material-local validation diagnostics to the SRP report. Warnings were outside this SRP projection lane: the pre-existing `entity_ids_matching_query_archetypes` dead-code warning and editor sprite-atlas unused-item warnings.

## Milestone 5: Final Scoped Acceptance And Handoff

- Goal: close the approved phase with scoped validation evidence, docs, and coordination state while clearly leaving GPU prewarm, shader variant compilation, and mutable editor authoring for later work.
- In-scope behaviors: scoped runtime/editor checks, docs diff hygiene, active session note retired or updated, remaining gaps recorded.
- Dependencies: Milestones 1 through 4.

### Implementation Slices

- [x] Update `docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md` checkboxes and evidence after validation.
- [x] Update `docs/zircon_runtime/graphics/srp-renderer-data.md`, `docs/assets-and-rendering/render-framework-architecture.md`, and `docs/zircon_editor/ui/material_editor.md` with final command evidence.
- [x] Move `.codex/sessions/20260518-2331-srp-rendererdata-zmaterial.md` to `.codex/sessions/archive/` with completed status if no handoff is needed, or update it with exact blockers if work remains.

### Testing Stage: Final Scoped Acceptance

- Commands:
  - `cargo fmt --all --check`
  - `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never`
  - `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never`
  - `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never`
  - `cargo check -p zircon_editor --lib --locked --jobs 1 --color never`
- Optional expansion when active dirty sessions settle:
  - `cargo build --workspace --locked --verbose`
  - `cargo test --workspace --locked --verbose`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked --all-targets --verbose`
- Debug/correction loop: if broad workspace/plugin validation fails outside touched SRP RendererData files, check fresh coordination notes before editing unrelated owners.
- Exit evidence: scoped runtime/editor checks pass. Workspace/plugin green is claimed only if optional expansion commands pass in the same closeout.
- 2026-05-20 final scoped evidence with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo fmt --all --check` passed after formatting the two new RendererData projection files; `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` passed 10 focused tests; `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never` passed 39 focused tests; `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never` passed 75 runtime lib tests plus 1 matching integration test; `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never` passed 8 focused tests; `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` passed; and `cargo check -p zircon_editor --lib --locked --jobs 1 --color never` passed. Warnings remained outside the SRP RendererData lane: `entity_ids_matching_query_archetypes` dead code and editor sprite-atlas unused-item warnings. External review found one accepted issue, material-local `.zmaterial` diagnostics missing from compile reports, which was fixed via `MaterialDiagnostic` plus material `validation_errors()` forwarding and covered by the new tenth `renderer_data_asset` test. The review concern about `RendererFeatureDocument.name` not becoming runtime identity was left as a documented boundary because current graph compile already rejects duplicate feature entries and diagnostics intentionally use runtime descriptor names such as `mesh`.

## Self-Review

- Spec coverage: the plan covers runtime-only RendererData, shader/material asset-aware compile diagnostics, editor projection second, docs, and validation. It explicitly excludes GPU prewarm, shader variants, mutable authoring, and M9A provider/app bootstrap.
- Placeholder scan: no `TBD`, vague error-handling placeholders, or undefined later-task names remain. Every milestone names concrete files, types, tests, commands, and exit evidence.
- Type consistency: names are consistent across milestones: `RendererDataDocument`, `RendererFeatureAssetReferences`, `RendererFeatureContractDiagnostic`, `RenderPipelineCompileReport`, `RenderPipelineAssetContext`, `compile_with_asset_context`, and `RendererDataEditorProjection`.
- Boundary review: runtime asset truth remains in `zircon_runtime::asset`; SRP compile/report ownership stays in `zircon_runtime::graphics::pipeline`; editor projection consumes DTOs only and does not own runtime world, renderer, or WGPU state.
