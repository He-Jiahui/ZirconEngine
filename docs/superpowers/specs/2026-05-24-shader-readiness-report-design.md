# Shader Readiness Report Design

## Goal

Add an asset-owned shader readiness report for `zircon_runtime::asset` that explains whether a `ShaderAsset` is ready for downstream material and render preparation. The report must stay read-only and value-based: it inspects the current `ShaderAsset` fields, projects existing dependency/import/entry/definition/diagnostic facts, and does not load artifacts, resolve handles, mutate residency, or allocate graphics objects.

This is the next M6 shader/material-lane slice in `.codex/plans/Bevy-Style Asset Stack Completion Plan.md`. It follows the completed asset readiness diagnostics slice, but it is narrower: it reports shader-internal readiness, while generic asset graph readiness remains `ProjectAssetManager::readiness_report<TAsset>()` and renderer readiness remains in `zircon_runtime::graphics`.

## Current Context

- `ShaderAsset` already stores `uri`, `source_language`, `source`, `wgsl_source`, optional `import_path`, `entry_points`, explicit `dependencies`, `source_files`, `imports`, string `shader_defs`, `property_schema`, `texture_slots`, `pipeline_layout`, and `validation_diagnostics`.
- `ShaderAsset::runtime_wgsl_source()` already chooses emitted WGSL first, then raw WGSL source only when `source_language == ShaderSourceLanguage::Wgsl`.
- `ShaderAsset::entry_point_descriptors()`, `dependencies()`, `variant_keys()`, and `pipeline_layout_descriptor()` already project shader data into render-framework DTOs.
- Compound `.zshader` import preserves source-only imports in `ShaderAsset.imports`, converts redirected imports into `ShaderAsset.dependencies` and import-outcome dependency locators, copies `shader_defs`, and stores WGSL capture misses in `validation_diagnostics`.
- `MaterialAsset::readiness_report_with_shader_contract(...)` currently consumes shader validation diagnostics through the material readiness path, but standalone shader assets do not have an asset-owned readiness report.
- `docs/zircon_runtime/core/framework/render/shader.md`, `docs/zircon_runtime/asset/render-assets.md`, and `docs/zircon_runtime/asset/zmeta-shader-material.md` already document the current shader/material asset contract and must be updated when code lands.

## Reference Evidence

### Bevy

Bevy is the primary reference because this slice is Rust-native shader asset infrastructure.

- `dev/bevy/crates/bevy_shader/src/shader.rs` keeps raw source, import path, imports, shader defs, and file dependencies on the shader asset. `Shader::from_wgsl_with_defs(...)` and `ShaderSettings` show that shader definitions are asset/cache inputs, not renderer-only state.
- `dev/bevy/crates/bevy_shader/src/shader_cache.rs` treats import availability and shader definition values as shader cache/readiness inputs. `ShaderCacheError::ShaderImportNotYetAvailable` is an explicit not-ready state, while module compilation remains renderer-owned through the `load_module` callback.
- `dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs` consumes shader cache errors and shader defs when queuing pipelines, reinforcing that asset/source readiness and concrete pipeline creation are separate layers.

### Godot

Godot is the secondary reference for explicit shader diagnostics and preprocessing boundaries.

- `dev/godot/scene/resources/shader.h` stores shader code, include path, include dependencies, default textures, and shader RID state on a resource, while renderer allocation is separate.
- `dev/godot/servers/rendering/shader_preprocessor.h` keeps preprocessing state, include tracking, define maps, error text, line positions, and regions distinct from the shader resource object.
- `dev/godot/tests/servers/rendering/test_shader_preprocessor.cpp` has focused diagnostics/regression coverage for defines, invalid concatenations, undefined behavior inputs, and expected error handling. This supports giving Zircon shader definitions/imports deterministic report rows instead of relying on renderer failure strings.

### Fyrox

Fyrox is the boundary cross-check for renderer ownership.

- `dev/Fyrox/fyrox-impl/src/renderer/cache/shader.rs` compiles shader resources into GPU programs and caches render pass containers in renderer code. Errors are logged or returned by the renderer cache, not the asset document. This supports keeping this Zircon slice asset-only and leaving GPU module/pipeline readiness for later render prepare/cache work.

## Chosen Design

Add a dedicated report surface under `zircon_runtime/src/asset/assets/shader/`. The report is computed from a borrowed `ShaderAsset` and contains no handles, leases, resource manager references, WGPU objects, renderer cache objects, or project-manager references.

The public entry point is:

```rust
impl ShaderAsset {
    pub fn readiness_report(&self) -> ShaderReadinessReport;
}
```

The report is owned by the shader asset module rather than the generic asset facade. Generic asset graph readiness remains responsible for resource records, dependency rows, load state, and diagnostics attached to `ResourceRecord`. Shader readiness explains the shader payload itself: runtime WGSL availability, imports, entry points, shader defs, validation diagnostics, and pipeline layout declaration presence.

## DTO Shape

The implementation should keep DTOs small and serializable-friendly, with derives matching existing asset DTO style where practical.

```rust
pub struct ShaderReadinessReport {
    pub uri: AssetUri,
    pub runtime_source: ShaderRuntimeSourceReadiness,
    pub imports: Vec<ShaderImportReadiness>,
    pub entry_points: Vec<ShaderEntryPointReadiness>,
    pub shader_defs: Vec<ShaderDefinitionReadiness>,
    pub validation_diagnostics: Vec<String>,
    pub dependency_count: usize,
    pub has_pipeline_layout: bool,
}
```

The report should expose convenience predicates:

- `is_ready()`: true only when runtime WGSL is available, all declared entry-point stages are valid, all shader definition rows are valid, and no validation diagnostics exist.
- `uses_runtime_wgsl()`: true when runtime source is available from emitted WGSL or raw WGSL fallback.
- `has_redirected_import_dependencies()`: true when at least one import row has a redirect and contributes a dependency.

The supporting rows should capture:

- `ShaderRuntimeSourceReadiness`: source language, source kind (`EmittedWgsl`, `RawWgslSource`, or `Unavailable`), and optional diagnostic string.
- `ShaderImportReadiness`: source import string, optional redirect, and whether it contributes to the dependency graph.
- `ShaderEntryPointReadiness`: name, raw stage string, optional canonical `RenderShaderStage`, and optional diagnostic string for invalid stages.
- `ShaderDefinitionReadiness`: raw string, normalized name, and optional diagnostic string for empty names or duplicates.

Use a narrow enum for runtime source kind instead of stringly status values. Diagnostics can remain strings for this slice because `ShaderAsset.validation_diagnostics` is already string-backed and material readiness already consumes those strings.

## Readiness Semantics

Runtime source readiness:

- `EmittedWgsl` when `wgsl_source.trim()` is non-empty.
- `RawWgslSource` when emitted WGSL is empty, `source_language == ShaderSourceLanguage::Wgsl`, and `source.trim()` is non-empty.
- `Unavailable` otherwise, with a diagnostic explaining that non-WGSL source without emitted WGSL is not runtime-ready.

Entry-point readiness:

- Declared entry points are valid when `ShaderEntryPointAsset::descriptor()` can project their stage into a canonical `RenderShaderStage`.
- An empty entry-point list is not fatal in this slice. The single-file WGSL importer can infer entries, while compound `.zshader` packages may record validation diagnostics when inference fails. Later renderer or material features can require specific entry points through their own contract checks.
- Invalid stage strings produce row diagnostics and make `is_ready()` false.

Import readiness:

- Every `ShaderImportRedirectAsset` row appears in the report.
- Rows with `redirect = Some(...)` contribute to the dependency graph and should match explicit `ShaderAsset.dependencies` semantics.
- Rows without redirects are authoring/composition imports. They are reported as not contributing dependencies but are not fatal yet, because Zircon has not implemented Bevy-style WGSL import composition in this milestone.
- The report should include `dependency_count = self.dependencies.len()` so callers can compare authored redirects with dependency projection without walking render descriptors.

Shader definition readiness:

- Each `shader_defs` string is trimmed to a normalized name for diagnostics.
- Empty names are invalid.
- Duplicate normalized names are invalid because Bevy treats shader defs as cache-significant inputs, and duplicate or redundant values can cause avoidable cache churn or ambiguous feature state.
- The design does not introduce typed bool/int/uint shader defs yet. All existing public surfaces continue to use `Vec<String>` and `RenderShaderVariantKey.defines: Vec<String>`.

Validation diagnostics:

- Existing `ShaderAsset.validation_diagnostics` are copied into the report and make `is_ready()` false.
- WGSL capture diagnostics remain non-import-blocking, but readiness consumers can now see them on a standalone shader report without requiring a material instance.

Pipeline layout readiness:

- `has_pipeline_layout` is true when `pipeline_layout.bind_groups` or `pipeline_layout.push_constant_ranges` are non-empty.
- A missing pipeline layout is not fatal for this slice because existing renderer paths still support shaders without serialized layout reflection. The report surfaces this as readiness context only.

## Ownership And Boundaries

In scope:

- `zircon_runtime/src/asset/assets/shader/*`
- shader asset tests under `zircon_runtime/src/asset/tests/assets/*` or nearby existing shader/project tests
- documentation updates under `docs/zircon_runtime/asset/*` and `docs/zircon_runtime/core/framework/render/shader.md`

Out of scope:

- `ProjectAssetManager::readiness_report<TAsset>()` behavior
- artifact loading, importer reload, watcher behavior, or resource residency
- material readiness semantics beyond consuming the new report in docs or future work
- graphics/resource streamer shader preparation, WGPU module creation, pipeline cache invalidation, or render fallback policy
- typed `ShaderDefVal` migration
- mesh, glTF, editor UI, plugin workspace, and lockfile changes

## Tests

Focused tests should cover:

- Runtime source kind: emitted WGSL wins, raw WGSL fallback works, non-WGSL without emitted WGSL is unavailable and not ready.
- Imports: redirected imports produce dependency-contributing rows; source-only imports are visible but non-fatal.
- Entry points: valid aliases project to canonical stages; invalid stage strings produce diagnostics and make the report not ready.
- Shader defs: normal names are ready; empty and duplicate normalized names produce diagnostics and make the report not ready.
- Existing `validation_diagnostics`: copied into the report and make the report not ready.
- Compound `.zshader` project fixture: clean shader package yields a ready report; WGSL capture diagnostics make the report not ready while import still succeeds.

Milestone testing commands should be scoped until a broader integration stage is explicitly selected:

```powershell
cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness
```

Add `rustfmt --edition 2021 --check` for touched Rust files and `git diff --check -- <touched files>` in the final hygiene stage. Do not claim workspace-wide build/test success unless the workspace validator is run after this slice and passes.

## Documentation

Update existing code-facing docs rather than creating a duplicate topic page:

- `docs/zircon_runtime/core/framework/render/shader.md`: describe where shader readiness stops relative to render module/pipeline readiness.
- `docs/zircon_runtime/asset/render-assets.md`: add the asset-level shader readiness report semantics and validation evidence.
- `docs/zircon_runtime/asset/zmeta-shader-material.md`: mention that compound `.zshader` payloads expose standalone shader readiness, including WGSL capture diagnostics.

Each touched doc must keep machine-readable headers current with `related_code`, `implementation_files`, `plan_sources`, and focused tests.

## Deliberate Divergence

- Bevy supports typed `ShaderDefVal::{Bool, Int, UInt}` and import composition through `naga_oil`. Zircon stays with existing `Vec<String>` shader defs for this slice and only reports duplicate/empty names. Typed defs and composition are follow-up milestones.
- Bevy `ShaderCache` can wait on imports and requeue pipelines. Zircon does not add cache invalidation here; redirected imports are reported as asset dependencies and graph readiness handles resource-level status.
- Godot's shader preprocessor has detailed line/region diagnostics. Zircon does not add line/column shader preprocessor diagnostics in this slice because `.zshader` import composition is not implemented yet.
- Fyrox compiles render-pass GPU programs in renderer cache code. Zircon keeps GPU readiness out of asset-owned shader readiness.

## Open Gaps After This Slice

- Typed shader definition values and stable cache-key normalization.
- WGSL import composition and import-path resolver behavior.
- Shader include graph diagnostics with line/column positions.
- Pipeline layout reflection from WGSL and bind-group compatibility validation.
- Render-side shader prepare/cache readiness, invalidation, and pipeline requeue reporting.
- Editor UI surfacing of standalone shader readiness rows.
