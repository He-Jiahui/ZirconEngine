# Shader Readiness Report Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a read-only, asset-owned `ShaderReadinessReport` for `ShaderAsset` that reports runtime WGSL availability, import rows, entry-point validity, shader definition hygiene, validation diagnostics, dependency count, and pipeline-layout presence.

**Architecture:** Keep the report inside `zircon_runtime::asset::assets::shader` as a value projection from an existing borrowed `ShaderAsset`. The implementation must not touch importer reloads, artifact loading, resource residency, project-manager mutation, graphics preparation, WGPU objects, editor UI, plugins, or lockfiles. Root `mod.rs` files only declare and re-export the new shader-owned types.

**Tech Stack:** Rust 2021, `serde`, `zircon_runtime::asset`, `zircon_runtime::core::framework::render`, Cargo scoped validation from PowerShell.

---

## Current Baseline

- The repository root `Cargo.toml` has root workspace members `zircon_app`, `zircon_runtime`, `zircon_runtime/reflection_macros`, `zircon_editor`, `zircon_runtime_interface`, and `zircon_hub`; `default-members = ["zircon_runtime"]` means unscoped Cargo commands land on runtime by default.
- CI runs `cargo build --workspace --locked --verbose`, `cargo test --workspace --locked --verbose`, then plugin workspace check/build/test commands from `.github/workflows/ci.yml`.
- This slice is crate-local to `zircon_runtime` asset shader code, with docs under `docs/zircon_runtime/asset/*` and `docs/zircon_runtime/core/framework/render/shader.md`.
- `ShaderAsset` already owns `uri`, `source_language`, `source`, `wgsl_source`, optional `import_path`, `entry_points`, `dependencies`, `source_files`, `imports`, `shader_defs`, `property_schema`, `texture_slots`, `pipeline_layout`, and `validation_diagnostics`.
- `ShaderAsset::runtime_wgsl_source()` already prefers emitted WGSL over raw WGSL fallback and rejects non-WGSL source without emitted WGSL.
- `ShaderEntryPointAsset::descriptor()` already maps authoring aliases such as `vertex`, `vert`, `vs`, `fragment`, `frag`, `fs`, `compute`, `comp`, and `cs` into canonical `RenderShaderStage` values.
- Generic graph readiness already lives in `ProjectAssetManager::readiness_report<TAsset>()` and must not be folded into this shader payload report.
- Material/render readiness already lives in `RenderMaterialReadinessReport` and `zircon_runtime::graphics`; this slice must only expose shader asset payload facts.
- `.codex/sessions/20260524-1728-shader-import-path-slice.md` was active when this work was handed off and touches shader import paths, importer ingest, shader docs, and `project::zmeta` tests. Recheck coordination before editing `zircon_runtime/src/asset/tests/project/zmeta.rs` or importer-adjacent files.

## File Structure

- Create `zircon_runtime/src/asset/assets/shader/readiness.rs` as the only production implementation file for report DTOs and report construction.
- Modify `zircon_runtime/src/asset/assets/shader/mod.rs` only to declare `mod readiness;` and re-export `ShaderDefinitionReadiness`, `ShaderEntryPointReadiness`, `ShaderImportReadiness`, `ShaderReadinessReport`, `ShaderRuntimeSourceKind`, and `ShaderRuntimeSourceReadiness`.
- Modify `zircon_runtime/src/asset/assets/mod.rs` only to re-export the same public shader readiness DTOs through `zircon_runtime::asset::assets`.
- Modify `zircon_runtime/src/asset/mod.rs` only to re-export the same public shader readiness DTOs through `zircon_runtime::asset`.
- Create `zircon_runtime/src/asset/tests/assets/shader_readiness.rs` for focused pure-asset tests that construct `ShaderAsset` values directly.
- Modify `zircon_runtime/src/asset/tests/assets/mod.rs` only to register `mod shader_readiness;`.
- Modify `zircon_runtime/src/asset/tests/project/zmeta.rs` only after the coordination gate confirms no fresh overlapping shader-import session owns this file.
- Modify `docs/zircon_runtime/asset/render-assets.md`, `docs/zircon_runtime/asset/zmeta-shader-material.md`, and `docs/zircon_runtime/core/framework/render/shader.md` to document the new asset-level report and keep their YAML headers current.
- Update `.codex/sessions/20260524-1721-shader-readiness-report.md` while work is active, then retire it when implementation and acceptance finish.
- Do not modify `Cargo.toml`, `Cargo.lock`, plugin workspace manifests, importer implementation files, mesh/glTF files, editor UI files, graphics resource preparation, or WGPU-facing code in this slice.

## Milestone 1: Shader-Owned Report DTOs And Pure Asset Tests

Goal: Add the report API and cover all shader-payload readiness semantics without touching project import, artifact loading, resource residency, graphics, editor, plugins, or lockfiles.

In-scope behaviors:

- `ShaderAsset::readiness_report()` returns a value report derived only from the current `ShaderAsset` fields.
- Runtime source readiness reports `EmittedWgsl`, `RawWgslSource`, or `Unavailable`.
- Source-only import rows are visible and non-fatal.
- Redirected import rows report `contributes_dependency = true` and preserve the `AssetReference` redirect.
- Entry point rows keep the raw stage string, expose the canonical `RenderShaderStage` when valid, and emit a diagnostic for invalid stage strings.
- Shader definition rows trim each string, emit a diagnostic for empty normalized names, emit a diagnostic for duplicate normalized names, and make `is_ready()` false when invalid.
- Existing `ShaderAsset.validation_diagnostics` are copied and make `is_ready()` false.
- `dependency_count` equals `self.dependencies.len()`.
- `has_pipeline_layout` is true when `bind_groups` or `push_constant_ranges` are non-empty.
- `is_ready()` is true only when runtime WGSL is available, entry-point rows are valid, shader definition rows are valid, and no validation diagnostics exist.
- `uses_runtime_wgsl()` is true for emitted WGSL and raw WGSL fallback.
- `has_redirected_import_dependencies()` is true when at least one import row contributes a dependency.

Dependencies:

- Existing `ShaderAsset` fields in `zircon_runtime/src/asset/assets/shader/shader_asset.rs`.
- Existing `ShaderEntryPointAsset::descriptor()` in `zircon_runtime/src/asset/assets/shader/entry_point.rs`.
- Existing render DTOs under `zircon_runtime/src/core/framework/render/shader/*`.

Implementation slices:

- [ ] Create `zircon_runtime/src/asset/assets/shader/readiness.rs` with this public shape and equivalent helper logic:

```rust
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::asset::{AssetReference, AssetUri};
use crate::core::framework::render::RenderShaderStage;

use super::{ShaderAsset, ShaderSourceLanguage};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

impl ShaderReadinessReport {
    pub fn is_ready(&self) -> bool {
        self.uses_runtime_wgsl()
            && self
                .entry_points
                .iter()
                .all(|entry| entry.diagnostic.is_none())
            && self
                .shader_defs
                .iter()
                .all(|definition| definition.diagnostic.is_none())
            && self.validation_diagnostics.is_empty()
    }

    pub fn uses_runtime_wgsl(&self) -> bool {
        matches!(
            self.runtime_source.source_kind,
            ShaderRuntimeSourceKind::EmittedWgsl | ShaderRuntimeSourceKind::RawWgslSource
        )
    }

    pub fn has_redirected_import_dependencies(&self) -> bool {
        self.imports.iter().any(|import| import.contributes_dependency)
    }

    fn from_shader(shader: &ShaderAsset) -> Self {
        Self {
            uri: shader.uri.clone(),
            runtime_source: runtime_source_readiness(shader),
            imports: import_readiness(shader),
            entry_points: entry_point_readiness(shader),
            shader_defs: shader_definition_readiness(shader),
            validation_diagnostics: shader.validation_diagnostics.clone(),
            dependency_count: shader.dependencies.len(),
            has_pipeline_layout: !shader.pipeline_layout.bind_groups.is_empty()
                || !shader.pipeline_layout.push_constant_ranges.is_empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderRuntimeSourceReadiness {
    pub source_language: ShaderSourceLanguage,
    pub source_kind: ShaderRuntimeSourceKind,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderRuntimeSourceKind {
    EmittedWgsl,
    RawWgslSource,
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderImportReadiness {
    pub source: String,
    pub redirect: Option<AssetReference>,
    pub contributes_dependency: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderEntryPointReadiness {
    pub name: String,
    pub stage: String,
    pub canonical_stage: Option<RenderShaderStage>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShaderDefinitionReadiness {
    pub raw: String,
    pub normalized_name: String,
    pub diagnostic: Option<String>,
}

impl ShaderAsset {
    pub fn readiness_report(&self) -> ShaderReadinessReport {
        ShaderReadinessReport::from_shader(self)
    }
}

fn runtime_source_readiness(shader: &ShaderAsset) -> ShaderRuntimeSourceReadiness {
    if !shader.wgsl_source.trim().is_empty() {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::EmittedWgsl,
            diagnostic: None,
        }
    } else if shader.source_language == ShaderSourceLanguage::Wgsl
        && !shader.source.trim().is_empty()
    {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::RawWgslSource,
            diagnostic: None,
        }
    } else {
        ShaderRuntimeSourceReadiness {
            source_language: shader.source_language,
            source_kind: ShaderRuntimeSourceKind::Unavailable,
            diagnostic: Some(format!(
                "shader `{}` does not provide emitted WGSL and cannot use `{}` source directly at runtime",
                shader.uri,
                shader.source_language.as_str()
            )),
        }
    }
}

fn import_readiness(shader: &ShaderAsset) -> Vec<ShaderImportReadiness> {
    shader
        .imports
        .iter()
        .map(|import| ShaderImportReadiness {
            source: import.source.clone(),
            redirect: import.redirect.clone(),
            contributes_dependency: import.redirect.is_some(),
        })
        .collect()
}

fn entry_point_readiness(shader: &ShaderAsset) -> Vec<ShaderEntryPointReadiness> {
    shader
        .entry_points
        .iter()
        .map(|entry| match entry.descriptor() {
            Some(descriptor) => ShaderEntryPointReadiness {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
                canonical_stage: Some(descriptor.stage),
                diagnostic: None,
            },
            None => ShaderEntryPointReadiness {
                name: entry.name.clone(),
                stage: entry.stage.clone(),
                canonical_stage: None,
                diagnostic: Some(format!(
                    "shader entry point `{}` uses unsupported stage `{}`",
                    entry.name, entry.stage
                )),
            },
        })
        .collect()
}

fn shader_definition_readiness(shader: &ShaderAsset) -> Vec<ShaderDefinitionReadiness> {
    let mut seen = BTreeSet::new();
    shader
        .shader_defs
        .iter()
        .map(|definition| {
            let normalized_name = definition.trim().to_string();
            let diagnostic = if normalized_name.is_empty() {
                Some("shader definition is empty after trimming".to_string())
            } else if !seen.insert(normalized_name.clone()) {
                Some(format!(
                    "shader definition `{}` is duplicated",
                    normalized_name
                ))
            } else {
                None
            };

            ShaderDefinitionReadiness {
                raw: definition.clone(),
                normalized_name,
                diagnostic,
            }
        })
        .collect()
}
```

- [ ] Modify `zircon_runtime/src/asset/assets/shader/mod.rs` structurally:

```rust
mod readiness;

pub use readiness::{
    ShaderDefinitionReadiness, ShaderEntryPointReadiness, ShaderImportReadiness,
    ShaderReadinessReport, ShaderRuntimeSourceKind, ShaderRuntimeSourceReadiness,
};
```

- [ ] Modify `zircon_runtime/src/asset/assets/mod.rs` structurally by extending the existing `pub use shader::{ ... };` block with the six readiness DTO names.
- [ ] Modify `zircon_runtime/src/asset/mod.rs` structurally by extending the existing `pub use assets::{ ... };` block with the six readiness DTO names.
- [ ] Create `zircon_runtime/src/asset/tests/assets/shader_readiness.rs` with direct asset-construction tests. Use this helper shape so tests stay pure and do not need the project manager:

```rust
use crate::asset::{
    AssetReference, AssetUri, ShaderAsset, ShaderDependencyAsset, ShaderEntryPointAsset,
    ShaderImportRedirectAsset, ShaderRuntimeSourceKind, ShaderSourceLanguage,
};
use crate::core::framework::render::{
    RenderShaderBindGroupLayoutDescriptor, RenderShaderPipelineLayoutDescriptor, RenderShaderStage,
};
use crate::core::resource::ResourceKind;

fn base_shader(uri: &str) -> ShaderAsset {
    ShaderAsset {
        uri: locator(uri),
        source_language: ShaderSourceLanguage::Wgsl,
        source: "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(); }".to_string(),
        wgsl_source: String::new(),
        import_path: None,
        entry_points: Vec::new(),
        dependencies: Vec::new(),
        source_files: Vec::new(),
        imports: Vec::new(),
        shader_defs: Vec::new(),
        property_schema: Vec::new(),
        texture_slots: Vec::new(),
        editor: Default::default(),
        pipeline_layout: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn locator(uri: &str) -> AssetUri {
    AssetUri::parse(uri).unwrap()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(locator(uri))
}
```

- [ ] Add `shader_readiness_reports_runtime_source_kinds` to prove emitted WGSL wins, raw WGSL fallback works, non-WGSL source without emitted WGSL is unavailable, and `is_ready()` follows the runtime-source result:

```rust
#[test]
fn shader_readiness_reports_runtime_source_kinds() {
    let mut emitted = base_shader("res://shaders/emitted.shader");
    emitted.source_language = ShaderSourceLanguage::Glsl;
    emitted.source = "void main() {}".to_string();
    emitted.wgsl_source = "@fragment fn fs_main() -> @location(0) vec4f { return vec4f(); }".to_string();

    let fallback = base_shader("res://shaders/raw.wgsl");

    let mut unavailable = base_shader("res://shaders/raw.glsl");
    unavailable.source_language = ShaderSourceLanguage::Glsl;
    unavailable.source = "void main() {}".to_string();
    unavailable.wgsl_source.clear();

    let emitted_report = emitted.readiness_report();
    let fallback_report = fallback.readiness_report();
    let unavailable_report = unavailable.readiness_report();

    assert_eq!(
        emitted_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::EmittedWgsl
    );
    assert_eq!(
        fallback_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::RawWgslSource
    );
    assert_eq!(
        unavailable_report.runtime_source.source_kind,
        ShaderRuntimeSourceKind::Unavailable
    );
    assert!(emitted_report.uses_runtime_wgsl());
    assert!(fallback_report.uses_runtime_wgsl());
    assert!(!unavailable_report.uses_runtime_wgsl());
    assert!(emitted_report.is_ready());
    assert!(fallback_report.is_ready());
    assert!(!unavailable_report.is_ready());
    assert!(unavailable_report
        .runtime_source
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("does not provide emitted WGSL"));
}
```

- [ ] Add `shader_readiness_reports_import_rows_without_blocking_source_only_imports` to prove source-only imports are visible and non-fatal while redirected imports contribute dependencies:

```rust
#[test]
fn shader_readiness_reports_import_rows_without_blocking_source_only_imports() {
    let mut shader = base_shader("res://shaders/imports.shader");
    let redirect = asset_reference("res://shaders/shared_lighting");
    shader.imports = vec![
        ShaderImportRedirectAsset {
            source: "zircon::lighting".to_string(),
            redirect: Some(redirect.clone()),
        },
        ShaderImportRedirectAsset {
            source: "naga_oil::math".to_string(),
            redirect: None,
        },
    ];
    shader.dependencies = vec![ShaderDependencyAsset {
        kind: ResourceKind::Shader,
        reference: redirect.clone(),
    }];

    let report = shader.readiness_report();

    assert!(report.is_ready());
    assert!(report.has_redirected_import_dependencies());
    assert_eq!(report.dependency_count, 1);
    assert_eq!(report.imports.len(), 2);
    assert_eq!(report.imports[0].source, "zircon::lighting");
    assert_eq!(report.imports[0].redirect, Some(redirect));
    assert!(report.imports[0].contributes_dependency);
    assert_eq!(report.imports[1].source, "naga_oil::math");
    assert!(report.imports[1].redirect.is_none());
    assert!(!report.imports[1].contributes_dependency);
}
```

- [ ] Add `shader_readiness_reports_entry_stage_diagnostics` to prove valid aliases become canonical stages and invalid stages make the report not ready:

```rust
#[test]
fn shader_readiness_reports_entry_stage_diagnostics() {
    let mut shader = base_shader("res://shaders/entries.shader");
    shader.entry_points = vec![
        ShaderEntryPointAsset {
            name: "vs_main".to_string(),
            stage: "vs".to_string(),
        },
        ShaderEntryPointAsset {
            name: "fs_main".to_string(),
            stage: "pixel".to_string(),
        },
    ];

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(report.entry_points.len(), 2);
    assert_eq!(
        report.entry_points[0].canonical_stage,
        Some(RenderShaderStage::Vertex)
    );
    assert!(report.entry_points[0].diagnostic.is_none());
    assert!(report.entry_points[1].canonical_stage.is_none());
    assert!(report.entry_points[1]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("unsupported stage `pixel`"));
}
```

- [ ] Add `shader_readiness_reports_shader_def_diagnostics` to prove normal defines are ready, trimmed duplicates are invalid, and empty names are invalid:

```rust
#[test]
fn shader_readiness_reports_shader_def_diagnostics() {
    let mut shader = base_shader("res://shaders/defs.shader");
    shader.shader_defs = vec![
        "USE_UNLIT".to_string(),
        "  ".to_string(),
        "ALPHA_CLIP".to_string(),
        " USE_UNLIT ".to_string(),
    ];

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(report.shader_defs[0].normalized_name, "USE_UNLIT");
    assert!(report.shader_defs[0].diagnostic.is_none());
    assert_eq!(report.shader_defs[1].normalized_name, "");
    assert!(report.shader_defs[1]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("empty after trimming"));
    assert_eq!(report.shader_defs[2].normalized_name, "ALPHA_CLIP");
    assert!(report.shader_defs[2].diagnostic.is_none());
    assert_eq!(report.shader_defs[3].normalized_name, "USE_UNLIT");
    assert!(report.shader_defs[3]
        .diagnostic
        .as_deref()
        .unwrap()
        .contains("duplicated"));
}
```

- [ ] Add `shader_readiness_copies_validation_diagnostics_and_pipeline_context` to prove existing validation diagnostics block readiness while layout presence is context-only:

```rust
#[test]
fn shader_readiness_copies_validation_diagnostics_and_pipeline_context() {
    let mut shader = base_shader("res://shaders/diagnostics.shader");
    shader.validation_diagnostics = vec![
        "wgsl_capture property `base_color` was not found".to_string(),
    ];
    shader.pipeline_layout = RenderShaderPipelineLayoutDescriptor {
        bind_groups: vec![RenderShaderBindGroupLayoutDescriptor {
            group: 0,
            label: Some("material".to_string()),
            bindings: Vec::new(),
        }],
        push_constant_ranges: Vec::new(),
    };

    let report = shader.readiness_report();

    assert!(!report.is_ready());
    assert_eq!(
        report.validation_diagnostics,
        vec!["wgsl_capture property `base_color` was not found".to_string()]
    );
    assert!(report.has_pipeline_layout);
}
```

- [ ] Modify `zircon_runtime/src/asset/tests/assets/mod.rs` structurally:

```rust
mod shader_readiness;
```

Testing stage:

- [ ] Run `rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs`.
- [ ] Run `cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] Run `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness`.
- [ ] If a test fails above the report layer, inspect the report DTO/helper behavior first before changing project manager, importer, graphics, or material code.
- [ ] Record command results in `.codex/sessions/20260524-1721-shader-readiness-report.md` before moving to Milestone 2.

Lightweight checks allowed before the testing stage:

- `rustfmt --edition 2021 --check` on the new `readiness.rs` file if syntax looks uncertain.
- Avoid Cargo build/test commands until the testing stage unless a concrete compile blocker prevents progress.

Exit evidence:

- The focused `shader_readiness` lib tests pass.
- Runtime lib/test type checking passes for `zircon_runtime` with `--locked`.
- No importer, artifact, project manager, graphics, editor, plugin, or lockfile files are touched.

## Milestone 2: Compound `.zshader` Fixture Coverage

Goal: Prove imported compound shader assets expose the same readiness semantics without changing importer behavior or fixture schema.

In-scope behaviors:

- A clean compound `.zshader` package yields a ready standalone shader report.
- Redirected imports contribute dependency context in the report.
- Source-only imports appear in the report and do not block readiness.
- WGSL capture diagnostics copied into `ShaderAsset.validation_diagnostics` make the standalone shader report not ready while import still succeeds.

Dependencies:

- Milestone 1 report API.
- Existing `ProjectManager::scan_and_import()` compound shader package behavior.
- Existing tests in `zircon_runtime/src/asset/tests/project/zmeta.rs`.

Coordination gate:

- [ ] Run `.\.codex\skills\zircon-project-skills\cross-session-coordination\scripts\Get-RecentCoordinationContext.ps1 -RepoRoot E:\Git\ZirconEngine -LookbackHours 4` immediately before editing `zircon_runtime/src/asset/tests/project/zmeta.rs`.
- [ ] Read `.codex/sessions/20260524-1728-shader-import-path-slice.md` if it still exists and was modified within the last 4 hours or appears in the scan.
- [ ] If a fresh active session owns `zircon_runtime/src/asset/tests/project/zmeta.rs`, `zircon_runtime/src/asset/importer/ingest/import_shader_package.rs`, or shader import-path semantics, pause Milestone 2 edits and record the blocker in `.codex/sessions/20260524-1721-shader-readiness-report.md`.
- [ ] If no fresh overlap exists, proceed with the test-only edits below. Do not edit importer code for this milestone.

Implementation slices:

- [ ] Extend the `ImportedAsset::Shader(shader)` branch in `project_manager_imports_compound_zshader_package_with_subassets` with these assertions after existing shader field assertions:

```rust
let readiness = shader.readiness_report();
assert!(readiness.is_ready());
assert!(readiness.uses_runtime_wgsl());
assert!(readiness.has_redirected_import_dependencies());
assert_eq!(readiness.dependency_count, 1);
assert_eq!(readiness.imports.len(), 2);
assert_eq!(readiness.imports[0].source, "zircon::lighting");
assert!(readiness.imports[0].contributes_dependency);
assert_eq!(readiness.imports[1].source, "naga_oil::math");
assert!(!readiness.imports[1].contributes_dependency);
assert_eq!(readiness.entry_points.len(), 2);
assert!(readiness
    .entry_points
    .iter()
    .all(|entry| entry.diagnostic.is_none()));
assert_eq!(readiness.shader_defs.len(), 2);
assert!(readiness
    .shader_defs
    .iter()
    .all(|definition| definition.diagnostic.is_none()));
assert!(readiness.validation_diagnostics.is_empty());
```

- [ ] Extend the `ImportedAsset::Shader(shader)` branch in `project_manager_imports_zshader_with_wgsl_capture_diagnostics` with these assertions after existing diagnostic presence checks:

```rust
let readiness = shader.readiness_report();
assert!(!readiness.is_ready());
assert!(readiness.uses_runtime_wgsl());
assert_eq!(readiness.validation_diagnostics, shader.validation_diagnostics);
assert!(readiness
    .validation_diagnostics
    .iter()
    .any(|diagnostic| diagnostic.contains("wgsl_capture property `base_color`")));
assert!(readiness
    .validation_diagnostics
    .iter()
    .any(|diagnostic| diagnostic.contains("wgsl_capture texture slot `albedo`")));
```

Testing stage:

- [ ] Run `rustfmt --edition 2021 --check zircon_runtime/src/asset/tests/project/zmeta.rs`.
- [ ] Run `cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] Run `cargo test -p zircon_runtime --lib project_manager_imports_zshader_with_wgsl_capture_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] Run `cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] If project fixture tests fail before readiness assertions, inspect current shader import-path session notes and importer behavior before changing the readiness report.
- [ ] Record command results and any coordination decision in `.codex/sessions/20260524-1721-shader-readiness-report.md`.

Lightweight checks allowed before the testing stage:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/tests/project/zmeta.rs` after the test assertions are inserted.
- Avoid Cargo build/test commands until the Milestone 2 testing stage.

Exit evidence:

- The clean compound shader package test passes and asserts a ready standalone shader report.
- The WGSL capture diagnostics package test passes and asserts a not-ready standalone shader report.
- The broader runtime `shader` filter passes for this slice.
- No importer implementation files are modified.

## Milestone 3: Docs, Hygiene, And Acceptance

Goal: Document the asset-vs-render shader readiness boundary, update machine-readable doc headers, and complete scoped acceptance without claiming workspace-wide validation.

In-scope behaviors:

- Existing docs explain `ShaderAsset::readiness_report()` and where it stops.
- Docs clarify that source-only imports are visible but non-fatal and redirected imports contribute dependency context.
- Docs clarify that existing WGSL capture diagnostics are copied into standalone shader readiness and block `is_ready()`.
- Docs retain machine-readable `related_code`, `implementation_files`, `plan_sources`, and `tests` headers.
- Final hygiene checks cover Rust formatting, workspace formatting, and whitespace diff checks for touched files.

Dependencies:

- Milestone 1 report API and pure asset tests.
- Milestone 2 fixture tests, unless Milestone 2 is blocked by a fresh overlapping shader-import session. If Milestone 2 is blocked, document the blocker and only claim Milestone 1 acceptance.

Implementation slices:

- [ ] Update `docs/zircon_runtime/asset/render-assets.md` header:

```yaml
related_code:
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
implementation_files:
  - zircon_runtime/src/asset/assets/shader/readiness.rs
plan_sources:
  - docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md
  - docs/superpowers/plans/2026-05-24-shader-readiness-report.md
tests:
  - zircon_runtime/src/asset/tests/assets/shader_readiness.rs
```

- [ ] In `docs/zircon_runtime/asset/render-assets.md`, extend the shader asset paragraphs after the existing `ShaderAsset::runtime_wgsl_source()` and shader import paragraphs with this content adapted to the surrounding prose:

```markdown
`ShaderAsset::readiness_report()` is the asset-owned payload readiness query for a standalone shader. It reports the selected runtime source kind, preserved import rows, entry-point stage projection, shader definition diagnostics, copied validation diagnostics, dependency count, and whether a serialized pipeline layout is present. The report is read-only: it does not load artifacts, resolve handles, mutate residency, run importers, allocate graphics resources, or prepare shader modules.

Source-only `.zshader` imports stay visible in `ShaderImportReadiness` and are not fatal until a later WGSL composition milestone exists. Redirected imports report `contributes_dependency = true`, while `dependency_count` lets callers compare redirected authoring rows with the explicit dependency graph. Empty shader definition names, duplicate normalized shader definitions, invalid entry-point stages, missing runtime WGSL, and existing `validation_diagnostics` make `ShaderReadinessReport::is_ready()` false. Missing pipeline layout remains context-only because the current renderer can still consume shaders without serialized reflection.
```

- [ ] Update `docs/zircon_runtime/asset/zmeta-shader-material.md` header with `zircon_runtime/src/asset/assets/shader/readiness.rs`, `zircon_runtime/src/asset/tests/assets/shader_readiness.rs`, `docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md`, and this plan path.
- [ ] In `docs/zircon_runtime/asset/zmeta-shader-material.md`, extend the shader/material section with this content adapted to the surrounding prose:

```markdown
Standalone shader readiness is now visible directly on `ShaderAsset` through `readiness_report()`. Compound `.zshader` packages therefore expose WGSL capture diagnostics without requiring a material instance: the import can succeed, the shader asset can keep its authoring rows, and readiness consumers can still see that `wgsl_capture` diagnostics make the shader not ready for downstream material/render preparation. Source-only imports are reported as non-dependency authoring rows, while redirected imports appear as dependency-contributing readiness rows.
```

- [ ] Update `docs/zircon_runtime/core/framework/render/shader.md` header with `zircon_runtime/src/asset/assets/shader/readiness.rs`, `zircon_runtime/src/asset/tests/assets/shader_readiness.rs`, `docs/superpowers/specs/2026-05-24-shader-readiness-report-design.md`, and this plan path.
- [ ] In `docs/zircon_runtime/core/framework/render/shader.md`, extend the Asset Projection and Current Limits sections with this content adapted to the surrounding prose:

```markdown
`ShaderAsset::readiness_report()` sits above the neutral render DTOs and below renderer preparation. It validates whether the asset payload has runtime WGSL, canonical entry-point stages, non-empty and non-duplicated shader definition names, and no shader-side validation diagnostics. It deliberately does not compose WGSL imports, create Naga modules, allocate WGPU shader modules, build bind group layouts, or queue pipelines; those remain shader-cache and graphics responsibilities.
```

- [ ] Update `.codex/sessions/20260524-1721-shader-readiness-report.md` with the implementation plan path, touched files, validation commands run, current blocker state, and next step.
- [ ] After all accepted work finishes, retire the active session note by deleting it if no handoff is needed, or moving it to `.codex/sessions/archive/` with `status: completed` if another shader session still needs the record.

Testing stage:

- [ ] Run `rustfmt --edition 2021 --check zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs zircon_runtime/src/asset/tests/project/zmeta.rs` for the Rust files touched in this slice. Omit `zircon_runtime/src/asset/tests/project/zmeta.rs` from the command only if Milestone 2 was blocked and that file was not edited.
- [ ] Run `cargo fmt --all --check`.
- [ ] Run `cargo test -p zircon_runtime --lib shader_readiness --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] Run `cargo test -p zircon_runtime --lib shader --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1`.
- [ ] Run `cargo test -p zircon_runtime --lib project_manager_imports_compound_zshader_package_with_subassets --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1` if Milestone 2 edited `zmeta.rs`.
- [ ] Run `cargo test -p zircon_runtime --lib project_manager_imports_zshader_with_wgsl_capture_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness -- --test-threads=1` if Milestone 2 edited `zmeta.rs`.
- [ ] Run `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-shader-readiness`.
- [ ] Run `git diff --check -- zircon_runtime/src/asset/assets/shader/readiness.rs zircon_runtime/src/asset/assets/shader/mod.rs zircon_runtime/src/asset/assets/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/asset/tests/assets/mod.rs zircon_runtime/src/asset/tests/assets/shader_readiness.rs zircon_runtime/src/asset/tests/project/zmeta.rs docs/zircon_runtime/asset/render-assets.md docs/zircon_runtime/asset/zmeta-shader-material.md docs/zircon_runtime/core/framework/render/shader.md docs/superpowers/plans/2026-05-24-shader-readiness-report.md .codex/sessions/20260524-1721-shader-readiness-report.md`.
- [ ] If `git diff --check` reports CRLF line-ending warnings only, record them as warnings and continue; fix any trailing whitespace or conflict-marker failures before completion.
- [ ] Do not claim workspace-wide success unless `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose` or `\.codex\skills\zircon-dev\scripts\validate-matrix.ps1` are run after this slice and pass.

Debug/correction loop:

- If a pure `shader_readiness` test fails, fix `readiness.rs` first.
- If a compound project test fails before the readiness assertions, inspect shader-import coordination state and current importer behavior before modifying readiness code.
- If `cargo check` fails in unrelated dirty workspace files, record the external failure and do not repair unrelated modules without a fresh coordination pass.
- If any failure points to lower shared support, follow support-first regression discipline and fix the lowest shared layer in scope before rerunning the relevant stage.

Exit evidence:

- All scoped runtime tests listed above either pass or have a recorded blocker outside this slice.
- `cargo check -p zircon_runtime --lib --tests --locked` passes for this slice, unless blocked by an explicitly recorded unrelated workspace failure.
- Formatting and diff hygiene pass for touched files.
- Docs list the new implementation file, tests, spec, and plan in machine-readable headers.
- Active session note is retired or left with a concrete handoff only when another active session needs it.

## Acceptance Boundary

- Accepted scope ends at asset-owned value reporting from `ShaderAsset`.
- This plan does not add typed `ShaderDefVal`, WGSL import composition, include graph resolution, shader cache invalidation, render shader preparation, WGPU module creation, pipeline layout reflection, editor UI display, plugin changes, or lockfile updates.
- This plan does not modify generic `ProjectAssetManager::readiness_report<TAsset>()`.
- This plan does not modify mesh/glTF paths.

## Self-Review

Spec coverage:

- Runtime source kind coverage is in Milestone 1 pure asset tests.
- Import row visibility, source-only non-fatal semantics, redirected dependency contribution, and dependency count are in Milestone 1 and Milestone 2.
- Entry-point canonical stage and invalid-stage diagnostics are in Milestone 1.
- Shader definition empty and duplicate diagnostics are in Milestone 1.
- Existing validation diagnostics are in Milestone 1 and Milestone 2.
- Compound `.zshader` clean and WGSL-capture diagnostic fixture coverage is in Milestone 2.
- Asset-vs-render ownership and docs updates are in Milestone 3.

Placeholder scan:

- The plan contains concrete file paths, type names, method names, snippets, validation commands, expected boundaries, and coordination gates.
- The plan avoids placeholder markers and does not ask a future worker to invent unspecified error handling or tests.

Type consistency:

- `ShaderReadinessReport`, `ShaderRuntimeSourceReadiness`, `ShaderRuntimeSourceKind`, `ShaderImportReadiness`, `ShaderEntryPointReadiness`, and `ShaderDefinitionReadiness` are declared in Milestone 1 and re-exported through shader, assets, and asset surfaces.
- Test snippets use the same public names declared by the implementation snippet.
- Documentation snippets reference the same `ShaderAsset::readiness_report()` and `ShaderReadinessReport::is_ready()` API.
