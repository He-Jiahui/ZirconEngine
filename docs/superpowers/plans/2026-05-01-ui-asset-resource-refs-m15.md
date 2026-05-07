# UI Asset Resource References M15 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the M15 shared UI resource reference foundation so UI assets expose typed font, image, media, and generic asset dependencies with fallback policy, compiler diagnostics, invalidation/cache fingerprints, and a minimal editor dependency view.

**Architecture:** `zircon_runtime::ui::template::asset` owns the resource reference contract and collector. The compiler emits dependencies and diagnostics on `UiCompiledDocument`; invalidation/cache consume a resource fingerprint; `zircon_editor` reads compiler output without owning parsing or resolver policy.

**Tech Stack:** Rust 2021, Serde/TOML, existing `zircon_runtime` UI asset compiler, existing `zircon_editor` UI asset editor session model, scoped Cargo validation with `--locked`.

---

## Repository Context

- Work directly on `main`; do not create branches or worktrees.
- Do not commit unless the user explicitly asks for a commit.
- Respect active session boundaries: avoid graphics/GI/VG/plugin renderer files, Runtime UI showcase/data-source projection files, world-space UI handoff, SDF font-rendering paths, watcher UX, and editor chrome.
- Current M15 baseline from `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`: media/font/resource refs are missing; only widget/style imports are structured UI asset dependencies.
- Current lower-layer dependencies that M15 must preserve: M7 schema migrator, M10 component contracts, M12 invalidation/cache, M13 descriptor registry/palette authority.

## File Structure

Create a folder-backed runtime module for resource references:

- Create: `zircon_runtime/src/ui/template/asset/resource_ref/mod.rs` for module wiring and public re-exports.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/resource_kind.rs` for `UiResourceKind` and inference helpers.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/fallback_policy.rs` for `UiResourceFallbackMode` and `UiResourceFallbackPolicy`.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/resource_ref.rs` for `UiResourceRef` parsing/validation.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/dependency.rs` for `UiResourceDependency` and `UiResourceDependencySource`.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/diagnostic.rs` for `UiResourceDiagnostic` and severity.
- Create: `zircon_runtime/src/ui/template/asset/resource_ref/collect.rs` for walking documents/imports and collecting dependencies.
- Modify: `zircon_runtime/src/ui/template/asset/mod.rs` to expose `resource_ref` types.
- Modify: `zircon_runtime/src/ui/template/mod.rs` to re-export the new public resource types.
- Modify: `zircon_runtime/src/ui/template/asset/document.rs` to add `UiAssetImports.resources`.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs` to add resource outputs to `UiCompiledDocument`.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/compile.rs` to collect dependencies during compile and cache-key generation.
- Modify: `zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs` to add `resource_dependencies_revision`.
- Modify: `zircon_runtime/src/ui/template/asset/invalidation/change.rs`, `fingerprint.rs`, `graph.rs`, `impact.rs`, `stage.rs` to track resource dependency invalidation.
- Create: `zircon_runtime/src/ui/tests/asset_resource_refs.rs` for M15 runtime tests.
- Modify: `zircon_runtime/src/ui/tests/mod.rs` to register the new test module.
- Modify: `zircon_runtime/src/ui/tests/asset_compile_cache.rs` and `zircon_runtime/src/ui/tests/asset_invalidation.rs` for cache/invalidation resource cases.
- Modify: `zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs` to store resource dependency and diagnostic vectors.
- Modify: `zircon_editor/src/ui/asset_editor/session/lifecycle.rs` to refresh those vectors after preview compile.
- Create: `zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs` for editor session view tests.
- Modify: `zircon_editor/src/tests/ui/ui_asset_editor/mod.rs` to register the editor test module.
- Modify docs after implementation: `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` and `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md`.
- Update archive after implementation: `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`.

## Milestone 1: Runtime Resource Reference Model

### Goal

Create the serialized and in-memory M15 resource reference contract inside `zircon_runtime::ui::template::asset` without changing compiler behavior yet.

### In-Scope Behaviors

- `UiAssetImports.resources` serializes/deserializes typed explicit dependencies.
- `UiResourceKind` supports `font`, `image`, `media`, and `generic_asset`.
- `UiResourceFallbackPolicy` supports `none`, `placeholder`, and `optional`.
- `UiResourceRef` validates URI, scheme, fallback shape, self-reference, and fallback kind.
- Legacy string inference classifies `res://`, `asset://`, and `project://` strings by typed table, property path, and extension.

### Dependencies

- `UiAssetDocument` tree authority and Serde model are stable from M7.
- No editor integration is required for this milestone.

### Implementation Slices

- [ ] Create `resource_kind.rs` with this public declaration and deterministic inference entry point:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceKind {
    Font,
    Image,
    Media,
    GenericAsset,
}

impl UiResourceKind {
    pub fn infer_from_path_and_uri(path: &str, uri: &str) -> Self;
}
```

- [ ] Create `fallback_policy.rs` with `Default` mapping to required resources:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceFallbackMode {
    #[default]
    None,
    Placeholder,
    Optional,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceFallbackPolicy {
    #[serde(default)]
    pub mode: UiResourceFallbackMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}
```

- [ ] Create `resource_ref.rs` with `UiResourceRef` and `validate(&self, path: &str) -> Result<(), UiResourceDiagnostic>`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceRef {
    pub kind: UiResourceKind,
    pub uri: String,
    #[serde(default)]
    pub fallback: UiResourceFallbackPolicy,
}
```

- [ ] Create `diagnostic.rs` with structured diagnostics:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiResourceDiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResourceDiagnostic {
    pub code: String,
    pub severity: UiResourceDiagnosticSeverity,
    pub message: String,
    pub path: String,
}
```

- [ ] Create `dependency.rs` with source identity and dependency rows:

```rust
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiResourceDependencySource {
    DocumentImport,
    NodeProp,
    NodeLayout,
    NodeStyleOverride,
    ChildMountSlot,
    StyleRuleDeclaration,
    TokenValue,
    ImportedWidget,
    ImportedStyle,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceDependency {
    pub reference: UiResourceRef,
    pub source: UiResourceDependencySource,
    pub path: String,
}
```

- [ ] Modify `UiAssetImports` in `document.rs` to add:

```rust
#[serde(default)]
pub resources: Vec<UiResourceRef>,
```

- [ ] Export all new resource types from `asset/mod.rs` and `template/mod.rs` without placing behavior in root wiring files.

### Unit-Test Code To Write

- [ ] Add `zircon_runtime/src/ui/tests/asset_resource_refs.rs` with tests named:
  - `ui_asset_imports_resources_roundtrip_typed_refs`
  - `ui_resource_ref_rejects_empty_uri`
  - `ui_resource_ref_rejects_unsupported_scheme`
  - `ui_resource_ref_rejects_placeholder_without_uri`
  - `ui_resource_ref_rejects_self_referential_placeholder`
  - `ui_resource_kind_infers_from_property_path_before_extension`

### Testing Stage

- [ ] Run targeted format check:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs zircon_runtime/src/ui/tests/mod.rs
```

- [ ] Run focused runtime tests:

```powershell
cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

- [ ] Fix compile, test, or formatting failures in the resource model layer before moving to Milestone 2.

### Exit Evidence

- `asset_resource_refs` tests pass.
- The new resource module is folder-backed.
- Root `mod.rs` files only wire modules and re-exports.

## Milestone 2: Compiler Dependency Collection And Diagnostics

### Goal

Collect resource dependencies from root documents, imported widgets, imported styles, node props/layout/style overrides, style rules, child slot metadata, and tokens; expose them on compiled documents.

### In-Scope Behaviors

- Explicit `imports.resources` contributes dependencies.
- Typed resource tables in TOML values contribute dependencies.
- Legacy strings contribute dependencies when resource-like.
- Diagnostics are attached to compile output for recoverable warnings and returned as `UiAssetError::InvalidDocument` for invalid typed resource contracts.
- Dependencies are deduplicated for packaging identity while source paths remain visible.

### Dependencies

- Milestone 1 resource model compiles and tests pass.
- Existing compiler expansion and style application behavior must remain unchanged.

### Implementation Slices

- [ ] Implement `collect.rs` with:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiResourceCollectionReport {
    pub dependencies: Vec<UiResourceDependency>,
    pub diagnostics: Vec<UiResourceDiagnostic>,
}

pub fn collect_document_resource_dependencies(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiResourceCollectionReport, UiAssetError>;
```

- [ ] Walk `UiAssetDocument.imports.resources`, `tokens`, `root`, `components.*.root`, and `stylesheets` in deterministic order.
- [ ] Walk imported widget/style documents with source path prefixes `imported_widget:<reference>` and `imported_style:<reference>`.
- [ ] Parse typed resource tables by requiring `kind` and `uri` string fields, and optional `fallback` table.
- [ ] Infer legacy string refs only when the string has `res://`, `asset://`, or `project://`, or when the path name is resource-specific.
- [ ] Deduplicate dependencies by `UiResourceRef` in a `BTreeSet`, then return sorted rows.
- [ ] Add these fields to `UiCompiledDocument` in `ui_document_compiler.rs`:

```rust
pub resource_dependencies: Vec<UiResourceDependency>,
pub resource_diagnostics: Vec<UiResourceDiagnostic>,
```

- [ ] Add accessors to `UiCompiledDocument`:

```rust
pub fn resource_dependencies(&self) -> &[UiResourceDependency];
pub fn resource_diagnostics(&self) -> &[UiResourceDiagnostic];
```

- [ ] In `compile.rs`, call `collect_document_resource_dependencies(...)` after component contract validation and before returning the compiled document.

### Unit-Test Code To Write

- [ ] Extend `asset_resource_refs.rs` with tests named:
  - `compiler_collects_explicit_resource_imports`
  - `compiler_collects_node_prop_resource_tables`
  - `compiler_collects_style_rule_resource_refs`
  - `compiler_collects_imported_widget_and_style_resource_refs`
  - `compiler_deduplicates_resource_dependencies_by_reference`
  - `compiler_reports_fallback_kind_mismatch_with_path`

### Testing Stage

- [ ] Run targeted format check:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/compiler/*.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs
```

- [ ] Run focused resource compiler tests:

```powershell
cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

- [ ] Run existing asset compiler coverage to detect regressions:

```powershell
cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never -- --nocapture
```

### Exit Evidence

- Resource collection tests pass.
- Existing asset compiler tests still pass under the scoped command.
- `UiCompiledDocument` exposes dependency and diagnostic views without changing `UiTemplateInstance` shape.

## Milestone 3: Cache And Invalidation Integration

### Goal

Make resource dependency changes visible to compile cache misses and invalidation reports.

### In-Scope Behaviors

- `UiCompileCacheKey` includes `resource_dependencies_revision`.
- `UiInvalidationSnapshot` includes `resource_dependencies_revision`.
- `UiAssetChange::ResourceDependency` and `UiInvalidationStage::ResourceDependency` are emitted when only resource dependency fingerprints differ.
- Resource dependency changes mark rebuild, render dirty, and projection dirty.

### Dependencies

- Milestone 2 collector can compute dependencies without compiling a full preview host.

### Implementation Slices

- [ ] In `fingerprint.rs`, add:

```rust
pub fn resource_dependencies_fingerprint(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiAssetFingerprint, UiAssetError>;
```

- [ ] Fingerprint sorted unique `UiResourceRef` values and exclude source paths.
- [ ] Add `resource_dependencies_revision: UiAssetFingerprint` to `UiCompileCacheKey` and `UiInvalidationSnapshot`.
- [ ] Populate `resource_dependencies_revision` from runtime `compile_cache_key_from_compiler(...)` when building the interface-owned `UiCompileCacheKey`.
- [ ] Add `UiAssetChange::ResourceDependency` to `change.rs`.
- [ ] Add `UiInvalidationStage::ResourceDependency` to `stage.rs`.
- [ ] Update `UiInvalidationGraph::classify(...)` so resource-only changes produce `ResourceDependency`, `Render`, and `Projection` stages.
- [ ] Update `UiInvalidationImpact::include_stage(...)` so `ResourceDependency` sets `rebuild_required`, `dirty.render`, and `projection_dirty`.

### Unit-Test Code To Write

- [ ] Extend `asset_compile_cache.rs` with `compile_cache_misses_when_resource_dependencies_change`.
- [ ] Extend `asset_invalidation.rs` with `invalidation_graph_reports_resource_dependency_changes`.
- [ ] Extend `asset_resource_refs.rs` with `resource_dependency_fingerprint_ignores_source_path_order`.

### Testing Stage

- [ ] Run targeted format check:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/compiler/cache/*.rs zircon_runtime/src/ui/template/asset/invalidation/*.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs zircon_runtime/src/ui/tests/asset_compile_cache.rs zircon_runtime/src/ui/tests/asset_invalidation.rs
```

- [ ] Run resource, cache, and invalidation tests:

```powershell
cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

- [ ] Run runtime type check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

### Exit Evidence

- Resource-only cache miss is covered.
- Resource-only invalidation report is covered.
- Runtime crate type-checks with existing unrelated warnings only.

## Milestone 4: Minimal Editor Dependency And Diagnostic View

### Goal

Expose compiler resource dependencies and resource diagnostics to the UI asset editor session without creating resource authoring UX.

### In-Scope Behaviors

- `UiAssetEditorSession` stores resource dependency rows from the latest successful preview compile.
- `UiAssetEditorSession` stores resource diagnostics from the latest successful preview compile.
- Existing string diagnostics remain unchanged for parse/compile failures.
- Resource view data refreshes after source edits and preview recompilation.

### Dependencies

- Milestone 2 compiled document outputs are available.

### Implementation Slices

- [x] Add fields to `UiAssetEditorSession`:

```rust
pub(super) resource_dependencies: Vec<UiResourceDependency>,
pub(super) resource_diagnostics: Vec<UiResourceDiagnostic>,
```

- [x] Import `UiResourceDependency` and `UiResourceDiagnostic` from `zircon_runtime_interface::ui::template`.
- [x] Add public session accessors in `lifecycle.rs` or a focused session child file:

```rust
pub fn resource_dependencies(&self) -> &[UiResourceDependency];
pub fn resource_diagnostics(&self) -> &[UiResourceDiagnostic];
```

- [x] In `from_source(...)`, populate the fields from `last_valid_compiled` when preview compile succeeds and use empty vectors when it fails.
- [x] In `apply_valid_document(...)`, refresh the fields when `compile_preview(...)` succeeds and clear them when compile fails before resource dependency collection completes.
- [x] Keep presentation changes minimal: expose the session accessors to tests; do not add UI widgets, slint callbacks, browser state, or watcher state.

### Unit-Test Code To Write

- [x] Create `zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs` with tests named:
  - `ui_asset_editor_session_exposes_resource_dependencies_after_compile`
  - `ui_asset_editor_session_exposes_resource_diagnostics_after_compile`
  - `ui_asset_editor_resource_dependencies_refresh_after_source_edit`

### Testing Stage

- [x] Run targeted format check:

```powershell
rustfmt --edition 2021 --check zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
```

- [x] Run focused editor tests:

```powershell
cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

- [x] Run editor type check:

```powershell
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

### Exit Evidence

- Editor session tests pass.
- Editor crate type-checks with existing unrelated warnings only.
- No resource browser, watcher, or chrome behavior was added.

### 2026-05-07 Execution Evidence

- `rustfmt --edition 2021 --check zircon_editor\src\ui\asset_editor\session\lifecycle.rs zircon_editor\src\ui\asset_editor\session\ui_asset_editor_session.rs zircon_editor\src\ui\asset_editor\session\presentation_state.rs zircon_editor\src\ui\asset_editor\session\runtime_report_state.rs zircon_editor\src\ui\asset_editor\presentation.rs zircon_editor\src\ui\slint_host\host_contract\data\ui_asset.rs zircon_editor\src\ui\slint_host\ui\pane_data_conversion\pane_ui_asset_conversion.rs zircon_editor\src\tests\ui\ui_asset_editor\mod.rs zircon_editor\src\tests\ui\ui_asset_editor\resource_dependency_view.rs` passed.
- `cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never -- --nocapture --test-threads=1` passed, `4 passed; 0 failed; 1136 filtered out`.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m15-resource-ux --message-format short --color never` passed with existing runtime/editor warnings.

## Milestone 5: Documentation, Archive, And Batch Acceptance

### Goal

Update the UI asset documentation and archive with M15 implementation evidence, then run the full scoped M15 validation set.

### In-Scope Behaviors

- Docs explain the resource reference model, fallback policy, dependency collection, diagnostics, cache/invalidation integration, and editor boundary.
- Archive records M15 as foundation-gate implemented only after tests pass.
- Active session note is retired cleanly.

### Dependencies

- Milestones 1-4 are implemented and focused tests pass.

### Implementation Slices

- [x] Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` frontmatter to include resource-ref files, new tests, and this spec/plan.
- [x] Add an M15 section to `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` describing serialized resource syntax and editor/runtime consumption.
- [x] Update `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` frontmatter to include resource-ref, cache, invalidation, and editor session files.
- [x] Add an M15 section to `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` describing compiler diagnostics and cache/invalidation fingerprints.
- [x] Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` only after validation: change M15 from missing to partial-to-mostly-complete foundation gate, record exact commands and remaining productization gaps.
- [x] Move the active 2026-05-07 M15 continuation session note to `.codex/sessions/archive/` with `status: completed` and evidence bullets.

### Testing Stage

- [ ] Run complete M15 scoped validation:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/template/asset/compiler/*.rs zircon_runtime/src/ui/template/asset/compiler/cache/*.rs zircon_runtime/src/ui/template/asset/invalidation/*.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs zircon_runtime/src/ui/tests/asset_compile_cache.rs zircon_runtime/src/ui/tests/asset_invalidation.rs zircon_runtime/src/ui/tests/mod.rs zircon_editor/src/ui/asset_editor/session/lifecycle.rs zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

- [ ] If `E:` free space is still under 50 GB, follow `cargo-target-disk-policy.md`: avoid conflicting Cargo writers and clean the selected target directory only when no process owns it.
- [ ] If a failure occurs in broad graphics/plugin/test drift outside M15, record it as external only after confirming the M15 focused tests and scoped checks passed.

2026-05-07 editor-view closeout did not rerun the already accepted runtime/package M15/M16 suite because `E:` was below the 50 GB Cargo target threshold and unrelated Cargo writers were active. Fresh validation for this continuation used the D-drive editor target and was scoped to the newly closed minimal editor dependency view.

### Exit Evidence

- Spec, plan, module docs, archive, and session note are consistent.
- All scoped M15 commands have fresh recorded output.
- Remaining gaps are explicitly kept for M16 or editor resource UX.

## Plan Self-Review

- Spec coverage: the milestones cover typed model, four resource kinds, fallback policy, compiler collection, structured diagnostics, invalidation/cache fingerprinting, minimal editor dependency/diagnostic view, docs, archive, and validation.
- Placeholder scan: the plan contains concrete file paths, type names, test names, commands, and explicit non-goals.
- Type consistency: `UiResourceRef`, `UiResourceKind`, `UiResourceFallbackPolicy`, `UiResourceDependency`, `UiResourceDiagnostic`, `UiResourceCollectionReport`, and `resource_dependencies_revision` are named consistently across milestones.

## Execution Recommendation

Use subagent-driven development for implementation: one subagent for runtime model/collector, one for cache/invalidation, one for editor session view, with review after each milestone. If executing inline, complete milestones in order and do not run full validation until each milestone testing stage.
