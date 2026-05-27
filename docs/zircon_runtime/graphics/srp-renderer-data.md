---
related_code:
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_reference.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_report.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pass_stage.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_source.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/mod.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_with_asset_context.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/pipeline/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/lib.rs
implementation_files:
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_data_document.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_reference.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_contract_diagnostic.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_report.rs
  - zircon_runtime/src/graphics/pipeline/declarations/renderer_feature_asset.rs
  - zircon_runtime/src/graphics/pipeline/declarations/mod.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_with_asset_context.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/mod.rs
  - zircon_runtime/src/graphics/pipeline/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-05-18 continue SRP RendererData workflow with zshader/zmaterial contract validation
  - user: 2026-05-25 continue material shader renderer functionality
  - docs/superpowers/plans/2026-05-18-srp-rendererdata-zmaterial-workflow.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
tests:
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs
  - zircon_runtime/src/graphics/tests/mod.rs
  - cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --color never
  - zircon_runtime/src/graphics/tests/renderer_data_asset.rs::asset_aware_compile_reports_shader_payload_readiness_gaps
doc_type: module-detail
---

# SRP RendererData Documents

`RendererDataDocument` is the TOML-facing data surface for Unity SRP-style renderer data assets. It describes the renderer name, ordered `RenderPassStage` list, and a list of renderer features without changing the existing render graph execution path.

The runtime ownership boundary stays narrow: `zircon_runtime::graphics::pipeline` owns RendererData structure, feature references, asset-aware compile reports, and non-fatal SRP diagnostics, while `.zshader` and `.zmaterial` truth remains in `zircon_runtime::asset`.

## Runtime Data Model

`RendererDataDocument` parses and serializes a document with `version`, `name`, `stages`, and `features`. `RendererFeatureDocument` stores authoring-oriented fields for a feature: `name`, built-in `source`, `enabled`, optional `quality_gate`, optional shader/material `AssetReference`s, required shader entry points, expected material properties, expected texture slots, and string `local_config`.

The document converts into the existing `RendererAsset` and `RendererFeatureAsset` runtime structures. Conversion validates strings against known built-in feature names and the current renderer stage names instead of silently accepting aliases. This preserves hard-cutover behavior and keeps misspelled authoring data visible before graph compile.

## Feature Contract References

`RendererFeatureAssetReferences` is stored on every `RendererFeatureAsset`. It carries optional shader/material references plus the required entry/property/texture-slot names that M2 will resolve against imported `ShaderAsset` and `MaterialAsset` contracts.

The `RendererFeatureAsset` constructors default these references to empty values, so existing default pipelines and plugin feature descriptors keep their current graph behavior. Builder helpers add contract references fluently for tests and future asset importers: `with_shader_reference`, `with_material_reference`, `with_required_entry_point`, `with_expected_property`, and `with_expected_texture_slot`.

## Stage And Feature Names

M1 accepts exact built-in feature names such as `Mesh`, `Sprite`, `PostProcess`, `Ui`, `DebugOverlay`, `AntiAlias`, `Bloom`, `ColorGrading`, and `HistoryResolve`. It also supports the other currently declared built-ins so existing pipeline feature vocabulary remains complete.

Stage names are accepted only in the current explicit RendererData vocabulary: `DepthPrepass`, `Shadow`, `Deferred`, `AmbientOcclusion`, `Lighting`, `Opaque2d`, `AlphaMask2d`, `Transparent2d`, `Opaque3d`, `AlphaMask3d`, `Transparent3d`, `PostProcess`, `Ui`, `Overlay`, and `Debug`. Legacy aggregate stages such as `Opaque` and `Transparent` are intentionally not accepted by the document parser for this milestone.

## Validation Scope

The focused M1 tests cover TOML roundtrip, conversion to `RendererAsset`, disabled feature preservation, shader/material reference preservation, and unknown stage/feature errors. The milestone testing stage is responsible for running the focused `renderer_data_asset` tests and a scoped `zircon_runtime` library check before M1 is considered complete.

M1 runtime data validation passed on 2026-05-20 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` ran 6 focused tests, 6 passed, and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` completed successfully. Both commands emitted only the pre-existing `zircon_runtime/src/scene/world/query.rs::entity_ids_matching_query_archetypes` dead-code warning outside the SRP RendererData files.

## Asset-Aware Compile Reports

`RenderPipelineAsset::compile_with_asset_context(...)` validates the normal graph path first by delegating to `compile_with_options(...)`. Descriptor, stage, phase, resource, and core-pipeline errors therefore remain hard compile failures. Shader/material authoring mismatches are gathered afterward into `RenderPipelineCompileReport::diagnostics` and do not prevent a `CompiledRenderPipeline` from being returned.

The compile context is abstracted as `RenderPipelineAssetContext`, which can load `ShaderAsset` and `MaterialAsset` by `AssetReference`. This keeps `graphics::pipeline` independent of `ProjectAssetManager` internals and lets tests use a small in-memory context.

M2 diagnostics cover missing shader/material assets, feature shader versus material shader mismatch, required shader entry points, expected shader properties, expected shader texture slots, existing material-local validation errors, stored material validation diagnostics, material shader-contract diagnostics, and shader payload readiness diagnostics. Material validation errors are wrapped as `RendererFeatureContractDiagnostic::MaterialValidation`, stored material diagnostic strings are wrapped as `MaterialDiagnostic`, and shader diagnostics are wrapped as `ShaderValidation`.

The shader side now consumes `ShaderAsset::readiness_report()` instead of forwarding only `shader.validation_diagnostics`. RendererData therefore reports asset-owned shader readiness gaps before GPU preparation: missing runtime WGSL for non-WGSL sources without emitted WGSL, invalid entry-point stage tokens, empty or duplicate shader definition names, and copied shader validation diagnostics. This is still a compile-report diagnostic surface only. It does not compose WGSL imports, create shader modules, specialize typed shader definitions, allocate bind group layouts, or prewarm renderer pipelines.

M2 asset-aware compile validation passed on 2026-05-20 with `CARGO_TARGET_DIR=F:\cargo-targets\zircon-srp-rendererdata-m1`: `cargo test -p zircon_runtime --locked renderer_data_asset --jobs 1 --message-format short --color never` ran 10 focused tests, 10 passed after review added material-local validation diagnostics to the SRP report; `cargo test -p zircon_runtime --locked pipeline_compile --jobs 1 --message-format short --color never` ran 39 focused tests, 39 passed; `cargo test -p zircon_runtime --locked material --jobs 1 --message-format short --color never` ran 75 runtime lib tests plus 1 matching integration test, all passed; and `cargo check -p zircon_runtime --lib --locked --jobs 1 --color never` completed successfully. All commands emitted only the pre-existing `entity_ids_matching_query_archetypes` dead-code warning outside this SRP lane.

## Product Pipeline Placement

RendererData complements the product render pipeline instead of replacing it. Runtime product profiles still choose which renderer product is active, and `RenderPipelineAsset::compile_with_options(...)` remains the hard graph compiler for descriptors, pass stages, phase ordering, resource IO, and core-pipeline requirements. RendererData supplies an authoring-facing document and feature contract reference layer that can be converted into the same `RendererAsset` and `RendererFeatureAsset` declarations consumed by those existing compilers.

The asset-aware compile path is therefore a reporting layer around the graph compiler. It resolves `.zshader` and `.zmaterial` references after hard graph validation, records authoring diagnostics in `RenderPipelineCompileReport`, and leaves the compiled pipeline usable when only shader/material contract mismatches are present. This matches the SRP intent from Unity renderer data assets while deliberately diverging from Unity's runtime resource creation and render-pass invalidation hooks for this milestone.

The remaining gaps are explicit: no GPU prewarm, no shader variant compilation, no WGPU pipeline specialization, no mutable editor authoring surface, no ShaderGraph or VFX graph, and no real GPU preview. Editor work after this runtime milestone should consume the `RendererAsset` and diagnostic rows as read-only projection data.

Final scoped acceptance on 2026-05-20 also ran `cargo fmt --all --check`, `cargo test -p zircon_editor --lib material_editor --locked --jobs 1 --message-format short --color never`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --color never`. The editor tests passed 8 focused material-editor tests including RendererData projection coverage. Workspace and plugin-wide green were not claimed because optional broad expansion commands were not run in this closeout.
