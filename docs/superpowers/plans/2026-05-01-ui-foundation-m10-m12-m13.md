# UI Foundation M10/M12/M13 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the shared Runtime UI foundation batch for M13 descriptor registry/capabilities, M10 component public contract/private boundary, and M12 invalidation/compile cache.

**Architecture:** Keep canonical UI asset/template/component authority in `zircon_runtime::ui`; `zircon_editor` consumes runtime-owned registry views and authoring DTOs for palette authoring. Implement bottom-up in milestone order: M13 descriptor authority, M10 component contract/privacy validation, then M12 invalidation graph and compiler cache keyed by document/import/descriptor/contract fingerprints.

**Tech Stack:** Rust, Serde, TOML, existing `zircon_runtime::ui` template/component/compiler modules, existing `zircon_editor::ui::asset_editor` session tests, Markdown docs.

---

## Execution Policy

- Work directly on `main` in the existing checkout. Do not create worktrees or feature branches.
- Do not commit unless the user explicitly asks for a commit.
- During implementation slices, add production code, focused test code, comments, and docs without forcing per-slice build/test loops.
- Each milestone has a named testing stage. Run compile/build/unit-test commands there, debug from the lowest shared support layer upward, then record evidence before promoting the milestone.
- Avoid graphics/GI/VG, Runtime UI showcase data-source paths, physics/animation, workspace watcher, and unrelated editor host/chrome files.

## Design Sources

- `docs/superpowers/specs/2026-05-01-ui-foundation-m10-m12-m13-design.md`
- `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`
- `.codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md`
- `dev/slint/tools/lsp/preview/ui/palette.rs`
- `dev/bevy/examples/ecs/change_detection.rs`
- `dev/bevy/examples/ui/widgets/standard_widgets.rs`
- `dev/Fyrox/project-manager/src/settings.rs`

## Affected Files

### Runtime Descriptor And Catalog

- Replace `zircon_runtime/src/ui/component/descriptor.rs` with folder-backed `zircon_runtime/src/ui/component/descriptor/mod.rs`.
- Create `zircon_runtime/src/ui/component/descriptor/component_descriptor.rs` for `UiComponentDescriptor` fields and narrow builders.
- Create `zircon_runtime/src/ui/component/descriptor/prop_schema.rs` for `UiPropSchema`.
- Create `zircon_runtime/src/ui/component/descriptor/slot_schema.rs` for `UiSlotSchema`.
- Create `zircon_runtime/src/ui/component/descriptor/option_descriptor.rs` for `UiOptionDescriptor`.
- Create `zircon_runtime/src/ui/component/descriptor/host_capability.rs` for `UiHostCapability` and `UiHostCapabilitySet`.
- Create `zircon_runtime/src/ui/component/descriptor/render_capability.rs` for `UiRenderCapability`.
- Create `zircon_runtime/src/ui/component/descriptor/palette_metadata.rs` for `UiPaletteMetadata`.
- Create `zircon_runtime/src/ui/component/descriptor/default_node_template.rs` for `UiDefaultNodeTemplate`.
- Create `zircon_runtime/src/ui/component/descriptor/fallback_policy.rs` for `UiWidgetFallbackPolicy`, editor fallback, and runtime fallback enums.
- Create `zircon_runtime/src/ui/component/descriptor/validation.rs` for `UiComponentDescriptorError` and descriptor validation.
- Replace `zircon_runtime/src/ui/component/catalog.rs` with folder-backed `zircon_runtime/src/ui/component/catalog/mod.rs`.
- Create `zircon_runtime/src/ui/component/catalog/registry.rs` for `UiComponentDescriptorRegistry`, revisioning, registration, lookup, and host filtering.
- Create `zircon_runtime/src/ui/component/catalog/editor_showcase.rs` for built-in V1 descriptors.
- Create `zircon_runtime/src/ui/component/catalog/palette_view.rs` for descriptor-to-palette view generation.
- Modify `zircon_runtime/src/ui/component/mod.rs` to keep public re-exports stable from folder-backed modules.
- Modify `zircon_runtime/src/ui/tests/component_catalog.rs` to cover descriptor validation, capabilities, palette metadata, fallback policy, and registry revisioning.

### Editor Palette Consumption

- Create `zircon_editor/src/ui/asset_editor/palette/mod.rs` as structural palette module root.
- Create `zircon_editor/src/ui/asset_editor/palette/entry.rs` for `UiAssetPaletteEntry` and `UiAssetPaletteEntryKind`.
- Create `zircon_editor/src/ui/asset_editor/palette/build.rs` for descriptor-driven palette construction plus local component/imported reference entries.
- Create `zircon_editor/src/ui/asset_editor/palette/instantiate.rs` for descriptor default-node instantiation and reference/local component node creation.
- Create `zircon_editor/src/ui/asset_editor/palette/placement.rs` for `UiAssetPaletteInsertionPlacement` and placement merge helpers.
- Modify `zircon_editor/src/ui/asset_editor/mod.rs` only as a structural entry file by adding the `palette` child module and any narrow re-exports needed by existing callers.
- Modify `zircon_editor/src/ui/asset_editor/tree/tree_editing.rs` to remove palette declarations/building/instantiation behavior and retain only tree-editing functions that are not palette-specific.
- Modify `zircon_editor/src/ui/asset_editor/session/palette_state.rs` to import palette helpers from `asset_editor::palette`.
- Modify `zircon_editor/src/ui/asset_editor/session/lifecycle.rs` to import `build_palette_entries` from `asset_editor::palette`.
- Modify `zircon_editor/src/ui/asset_editor/session/presentation_state.rs` to import `build_palette_entries` from `asset_editor::palette`.
- Modify `zircon_editor/src/ui/asset_editor/session/promotion_state.rs` to import palette entry kinds from `asset_editor::palette`.
- Modify `zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs` to import palette entry and placement types from `asset_editor::palette`.
- Create `zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs` for focused editor palette tests.
- Modify `zircon_editor/src/tests/ui/ui_asset_editor/mod.rs` to register `palette_descriptor_registry`.

### Component Public Contract

- Create `zircon_runtime/src/ui/template/asset/component_contract/mod.rs` as structural contract module root.
- Create `zircon_runtime/src/ui/template/asset/component_contract/api_version.rs` for `UiComponentApiVersion` parsing, serde, and compatibility checks.
- Create `zircon_runtime/src/ui/template/asset/component_contract/public_contract.rs` for `UiComponentPublicContract`.
- Create `zircon_runtime/src/ui/template/asset/component_contract/public_part.rs` for `UiPublicPart` and part lookup.
- Create `zircon_runtime/src/ui/template/asset/component_contract/root_class_policy.rs` for `UiRootClassPolicy`.
- Create `zircon_runtime/src/ui/template/asset/component_contract/focus_contract.rs` for `UiComponentFocusContract`.
- Create `zircon_runtime/src/ui/template/asset/component_contract/binding_contract.rs` for `UiComponentBindingContract` and public binding route schemas.
- Create `zircon_runtime/src/ui/template/asset/component_contract/validation.rs` for component contract validation and private-boundary checks.
- Modify `zircon_runtime/src/ui/template/asset/document.rs` to add `contract: UiComponentPublicContract` to `UiComponentDefinition` and `component_api_version: Option<UiComponentApiVersion>` to reference/component nodes.
- Modify `zircon_runtime/src/ui/template/asset/mod.rs` to expose the `component_contract` module and public contract types.
- Modify `zircon_runtime/src/ui/template/asset/style.rs` to parse public-part selector tokens such as `:part(label)` into a structured selector token.
- Modify `zircon_runtime/src/ui/template/asset/compiler/shape_validator.rs` to invoke component contract validation before expansion.
- Modify `zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs` to enforce required reference API version compatibility when expanding imported references.
- Modify `zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs` only for narrow facade access to imports/registry needed by contract validation.
- Create `zircon_runtime/src/ui/tests/asset_component_contract.rs` for focused M10 contract tests.
- Modify `zircon_runtime/src/ui/tests/mod.rs` to register `asset_component_contract`.

### Invalidation And Compiler Cache

- Create `zircon_runtime/src/ui/template/asset/invalidation/mod.rs` as structural invalidation module root.
- Create `zircon_runtime/src/ui/template/asset/invalidation/stage.rs` for `UiInvalidationStage`.
- Create `zircon_runtime/src/ui/template/asset/invalidation/impact.rs` for `UiInvalidationImpact` and runtime `UiDirtyFlags` mapping.
- Create `zircon_runtime/src/ui/template/asset/invalidation/change.rs` for document/import/descriptor/contract change descriptors.
- Create `zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs` for `UiAssetFingerprint` and deterministic document/import fingerprints.
- Create `zircon_runtime/src/ui/template/asset/invalidation/graph.rs` for `UiInvalidationGraph` stage propagation.
- Create `zircon_runtime/src/ui/template/asset/invalidation/diagnostic.rs` for large-document diagnostics and named threshold constants.
- Create `zircon_runtime/src/ui/template/asset/invalidation/report.rs` for invalidation reports.
- Create `zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs` as structural compiler cache root.
- Create `zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs` for `UiCompileCacheKey`.
- Create `zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs` for `UiAssetCompileCache` storage.
- Create `zircon_runtime/src/ui/template/asset/compiler/cache/outcome.rs` for `UiCompileCacheOutcome`.
- Modify `zircon_runtime/src/ui/template/asset/compiler/mod.rs` to expose cache types without adding cache behavior to the root file.
- Modify `zircon_runtime/src/ui/template/asset/compiler/compile.rs` to add `compile_with_cache(...)` as a narrow wrapper around the existing full compile path.
- Modify `zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs` to expose descriptor registry revision and import fingerprints needed by cache keys.
- Create `zircon_runtime/src/ui/tests/asset_invalidation.rs` for invalidation graph and diagnostics tests.
- Create `zircon_runtime/src/ui/tests/asset_compile_cache.rs` for compile cache hit/miss tests.
- Modify `zircon_runtime/src/ui/tests/mod.rs` to register `asset_invalidation` and `asset_compile_cache`.

### Documentation And Archive

- Create `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` with required YAML frontmatter listing related runtime/editor files, implementation files, plan sources, tests, and validation commands.
- Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` frontmatter and body with descriptor, contract, invalidation, and cache behavior.
- Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` after milestone validation with actual evidence and remaining gaps.
- Update `.codex/sessions/20260501-1518-ui-foundation-m10-m12-m13-design.md` as live coordination state changes.

## Core Type Shapes

Use these concrete public shapes unless implementation evidence requires a narrower equivalent.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiHostCapability {
    Editor,
    Runtime,
    TextInput,
    PointerInput,
    KeyboardNavigation,
    GamepadNavigation,
    ImageRender,
    CanvasRender,
    VirtualizedLayout,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHostCapabilitySet {
    #[serde(default)]
    pub capabilities: BTreeSet<UiHostCapability>,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPaletteMetadata {
    pub display_name: String,
    pub category: UiComponentCategory,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub sort_key: String,
    #[serde(default)]
    pub default_node: UiDefaultNodeTemplate,
}
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct UiComponentApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl UiComponentApiVersion {
    pub const DEFAULT: Self = Self { major: 1, minor: 0, patch: 0 };

    pub const fn is_compatible_with(self, required: Self) -> bool {
        self.major == required.major && self.minor >= required.minor
    }
}
```

`component_contract/api_version.rs` must implement `Serialize` and `Deserialize` manually so TOML stores `api_version = "1.0.0"` instead of a nested table.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentPublicContract {
    #[serde(default)]
    pub api_version: UiComponentApiVersion,
    #[serde(default)]
    pub public_parts: BTreeMap<String, UiPublicPart>,
    #[serde(default)]
    pub root_class_policy: UiRootClassPolicy,
    #[serde(default)]
    pub focus: UiComponentFocusContract,
    #[serde(default)]
    pub bindings: UiComponentBindingContract,
}
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiInvalidationStage {
    SourceParse,
    DocumentShape,
    ImportGraph,
    DescriptorRegistry,
    ComponentContract,
    SelectorMatch,
    StyleValue,
    Layout,
    Render,
    Interaction,
    Projection,
}
```

## Milestone 1: M13 Descriptor Registry And Descriptor-Driven Palette

- Goal: Existing runtime component descriptors become validated widget descriptors with capabilities, fallback policy, palette metadata, default node templates, registry revisioning, and editor palette consumption.
- In-scope behaviors: descriptor validation, host capability filtering, render capability declaration, palette metadata generation, default native node templates, registry revision bumping, deterministic descriptor/category ordering, editor palette native entries generated from descriptors.
- Dependencies: existing `UiComponentDescriptor`, `UiComponentDescriptorRegistry`, `UiComponentCategory`, `UiValue`, `UiNodeDefinition`, editor asset editor palette/session tests.

### Implementation Slices

- [ ] Convert `zircon_runtime/src/ui/component/descriptor.rs` into the folder-backed descriptor module listed in Affected Files. Keep public type names stable through `descriptor/mod.rs` re-exports.
- [ ] Add `UiHostCapability`, `UiHostCapabilitySet`, and set helpers: `editor_authoring()`, `runtime_basic()`, `contains_all(required)`, and `missing(required)`.
- [ ] Add `UiRenderCapability` and attach render requirements to `UiComponentDescriptor`.
- [ ] Add `UiPaletteMetadata`, `UiDefaultNodeTemplate`, and `UiWidgetFallbackPolicy` to `UiComponentDescriptor`.
- [ ] Add descriptor builder methods: `requires_host_capability(...)`, `requires_render_capability(...)`, `palette(...)`, `default_node_template(...)`, and `fallback_policy(...)`.
- [ ] Add `UiComponentDescriptor::validate()` returning `Result<(), UiComponentDescriptorError>`.
- [ ] Convert `zircon_runtime/src/ui/component/catalog.rs` into the folder-backed catalog module listed in Affected Files.
- [ ] Change registry registration to validate descriptors and bump a deterministic `revision: u64` only when the registered descriptor set changes.
- [ ] Add registry views: `descriptors_for_host(&UiHostCapabilitySet)`, `palette_entries_for_host(&UiHostCapabilitySet)`, and `missing_capabilities(component_id, host_caps)`.
- [ ] Update `editor_showcase()` descriptors to declare host/render capabilities, fallback policy, palette metadata, and default node templates for all existing V1 component ids.
- [ ] Move editor palette entry types and palette construction from `tree_editing.rs` into `zircon_editor/src/ui/asset_editor/palette/`.
- [ ] Update editor palette construction so native entries are built from `UiComponentDescriptorRegistry::editor_showcase().palette_entries_for_host(&UiHostCapabilitySet::editor_authoring())`.
- [ ] Preserve local component entries from `document.components.keys()` and imported reference entries from `widget_imports.keys()` as separate non-descriptor palette sources.
- [ ] Add runtime tests in `component_catalog.rs` for descriptor validation, required host capabilities, palette metadata, default node templates, fallback policy, and registry revisioning.
- [ ] Add editor tests in `palette_descriptor_registry.rs` proving native palette entries come from descriptors and still include local component/imported reference entries.
- [ ] Update docs and session note with M13 files and planned validation evidence.

### Testing Stage: M13 Descriptor Authority Gate

- Run targeted formatting for changed Rust files:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/component/mod.rs zircon_runtime/src/ui/component/descriptor/*.rs zircon_runtime/src/ui/component/catalog/*.rs zircon_runtime/src/ui/tests/component_catalog.rs zircon_editor/src/ui/asset_editor/mod.rs zircon_editor/src/ui/asset_editor/tree/tree_editing.rs zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs zircon_editor/src/ui/asset_editor/session/palette_state.rs zircon_editor/src/ui/asset_editor/palette/*.rs zircon_editor/src/tests/ui/ui_asset_editor/mod.rs zircon_editor/src/tests/ui/ui_asset_editor/palette_descriptor_registry.rs
```

- Run runtime descriptor tests:

```powershell
cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run focused editor palette tests:

```powershell
cargo test -p zircon_editor --lib palette_descriptor_registry --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run scoped type checks for touched crates:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

### Debug / Correction Loop

- If runtime descriptor tests fail, fix descriptor validation or registry revisioning before touching editor palette code.
- If editor palette tests fail, check whether descriptor palette metadata is missing before reintroducing hardcoded native lists.
- If `cargo check -p zircon_editor` fails in Runtime UI showcase or Slint host areas, check active session notes before editing those files.

### Exit Evidence

- Runtime component catalog tests pass with descriptor validation and capability coverage.
- Editor UI Asset Editor palette tests prove descriptor-driven native entries and preserved local/reference entries.
- Runtime and editor scoped checks pass or blockers are documented as unrelated active-session drift.

## Milestone 2: M10 Component Public Contract And Private Boundary

- Goal: UI asset local/imported components have explicit public contracts, semantic API version compatibility, public parts, and private-boundary validation before compiler expansion.
- In-scope behaviors: contract parsing/defaults, semantic version parsing/serde, part export validation, root class policy, focus contract validation, binding contract structure validation, selector private-target rejection, public part acceptance, reference API compatibility checks.
- Dependencies: M13 descriptor authority, existing `UiAssetDocument`, existing `UiComponentDefinition.params`, existing `UiComponentDefinition.slots`, existing asset compiler import registration.

### Implementation Slices

- [ ] Add the folder-backed `component_contract` module and public re-exports from `asset/mod.rs`.
- [ ] Implement `UiComponentApiVersion` parsing from string form `major.minor.patch`, serde serialization as the same string, and default `1.0.0`.
- [ ] Implement `UiComponentPublicContract` with defaults: `api_version = 1.0.0`, no public parts, `UiRootClassPolicy::AppendOnly`, empty focus contract, and empty binding contract.
- [ ] Add `contract: UiComponentPublicContract` to `UiComponentDefinition` with serde default.
- [ ] Add `component_api_version: Option<UiComponentApiVersion>` to `UiNodeDefinition` with serde default for component/reference nodes.
- [ ] Add public part validation: every exported part must name a node inside the component root tree, part names must be non-empty and unique, and private node ids remain private unless exported.
- [ ] Add root class policy validation: component instances may append root classes only when the contract allows it; internal selector/class targeting stays private.
- [ ] Extend `UiSelector` parsing with a structured `UiSelectorToken::Part(String)` for `:part(name)` syntax.
- [ ] Add contract validation to `shape_validator.rs` before compile expansion so invalid privacy/API contracts fail before `UiTemplateNode` internals are produced.
- [ ] Add imported reference boundary validation using registered widget imports: selectors, focus targets, and binding targets in the caller document cannot target imported component private `node_id` or `control_id` values unless the imported component exports the matching public part/root target.
- [ ] Add API compatibility validation in reference expansion: if a reference node declares `component_api_version`, the imported component contract version must have the same major and a minor version greater than or equal to the required minor.
- [ ] Add tests in `asset_component_contract.rs` for default contract values, invalid semver, private selector rejection, public part acceptance, binding/focus private target rejection, compatible minor/patch reference, and incompatible major reference.
- [ ] Update UI asset docs with component contract TOML examples and M10 validation evidence.

### Testing Stage: M10 Component Contract Gate

- Run targeted formatting for changed Rust files:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/asset/document.rs zircon_runtime/src/ui/template/asset/style.rs zircon_runtime/src/ui/template/asset/component_contract/*.rs zircon_runtime/src/ui/template/asset/compiler/mod.rs zircon_runtime/src/ui/template/asset/compiler/shape_validator.rs zircon_runtime/src/ui/template/asset/compiler/component_instance_expander.rs zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/asset_component_contract.rs
```

- Run focused component contract tests:

```powershell
cargo test -p zircon_runtime --lib asset_component_contract --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run existing asset compiler tests affected by reference expansion and selector parsing:

```powershell
cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never -- --nocapture
```

- Run scoped runtime type check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

### Debug / Correction Loop

- If private-boundary tests fail, inspect `component_contract::validation` and selector token parsing before modifying compiler expansion.
- If existing asset tests fail after adding `:part(...)`, preserve old selector semantics for type/class/id/state/host and only add the part token path.
- If API compatibility tests fail, fix `UiComponentApiVersion::is_compatible_with(...)` and serde parsing before changing document fixtures.

### Exit Evidence

- Focused component contract tests pass.
- Existing runtime UI asset tests still pass with contract defaults and selector parsing unchanged for existing syntax.
- Docs contain component contract examples and validation evidence.

## Milestone 3: M12 Invalidation Graph And Compiler Cache

- Goal: Runtime UI asset/template compilation exposes invalidation stages, cache keys, cache outcomes, dirty-flag impacts, and large-document diagnostics.
- In-scope behaviors: stage graph propagation, document/import fingerprints, descriptor registry revision in cache keys, component contract revision in cache keys, cache hit/miss reporting, dirty impact mapping, large-document diagnostics, non-virtualized large list warning, broad selector warning.
- Dependencies: M13 registry revisioning, M10 component contract validation, existing `UiDocumentCompiler::compile(...)`, existing `UiDirtyFlags`, existing virtual list/scrollable support.

### Implementation Slices

- [x] Add the folder-backed `invalidation` module and public re-exports from `asset/mod.rs`.
- [x] Implement `UiInvalidationStage` with the exact stage set from the spec.
- [x] Implement `UiInvalidationImpact` mapping stages to runtime dirty consequences using `UiDirtyFlags`.
- [x] Implement deterministic `UiAssetFingerprint` creation for a document by hashing stable TOML serialization plus asset id/kind/version.
- [x] Implement import fingerprints for registered widget/style imports using the same document fingerprint helper.
- [x] Implement component contract revision collection from local/imported component public contracts.
- [x] Implement `UiInvalidationGraph::classify(previous, next)` for source/document/import/descriptor/contract/style/layout/interaction/projection changes.
- [x] Implement `UiInvalidationDiagnostic` with named constants such as `LARGE_DOCUMENT_NODE_WARNING_THRESHOLD`, `NON_VIRTUALIZED_SCROLL_CHILD_WARNING_THRESHOLD`, and `BROAD_SELECTOR_WARNING_THRESHOLD` in `diagnostic.rs`.
- [x] Add `UiCompileCacheKey` containing root document fingerprint, widget import fingerprints, style import fingerprints, descriptor registry revision, and component contract revision hash.
- [x] Add `UiAssetCompileCache` with deterministic entry replacement and `clear()` / `len()` helpers.
- [x] Add `UiCompileCacheOutcome { compiled, cache_hit, invalidation_report }`.
- [x] Add `UiDocumentCompiler::compile_with_cache(&self, document: &UiAssetDocument, cache: &mut UiAssetCompileCache) -> Result<UiCompileCacheOutcome, UiAssetError>` that reuses exact cache hits and falls back to existing full `compile(...)` for misses.
- [x] Add tests in `asset_invalidation.rs` for stage propagation and dirty impact mapping.
- [x] Add tests in `asset_compile_cache.rs` for exact reuse, document change miss, import change miss, descriptor revision miss, component contract revision miss, and diagnostic emission.
- [x] Update docs with invalidation graph, cache key fields, large-document thresholds, and M12 validation evidence.

### Testing Stage: M12 Invalidation And Cache Gate

- Run targeted formatting for changed Rust files:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/asset/invalidation/*.rs zircon_runtime/src/ui/template/asset/compiler/mod.rs zircon_runtime/src/ui/template/asset/compiler/compile.rs zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs zircon_runtime/src/ui/template/asset/compiler/cache/*.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/asset_invalidation.rs zircon_runtime/src/ui/tests/asset_compile_cache.rs
```

- Run focused invalidation tests:

```powershell
cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run focused compile cache tests:

```powershell
cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run existing asset and component catalog tests because cache keys depend on descriptors and contracts:

```powershell
cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never -- --nocapture
cargo test -p zircon_runtime --lib component_catalog --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- Run scoped runtime type check:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

### Debug / Correction Loop

- If cache reuse tests fail, inspect `UiCompileCacheKey` fingerprint components before changing compiler behavior.
- If invalidation stage tests fail, inspect `UiInvalidationGraph` propagation before changing dirty flag mapping.
- If diagnostics tests fail, keep threshold constants local to `diagnostic.rs` and adjust fixture shape before weakening diagnostics.

### Exit Evidence

- Focused invalidation and compile cache tests pass.
- Existing asset/compiler and descriptor catalog tests still pass.
- Runtime scoped check passes or unrelated active-session blockers are documented.
- 2026-05-01 acceptance used `E:\cargo-targets\zircon-ui-foundation-m10-contract-fresh` after disk-policy cleanup because the shared M10/M12/M13 target was busy. Post-review cache validation refreshed after the per-asset snapshot fix; exact commands, counts, warnings, and the transient target-cleanup race are recorded in the module docs and session evidence.

## Milestone 4: Documentation, Archive, And Batch Acceptance

- Goal: Record implemented M13/M10/M12 behavior, tests, validation commands, remaining gaps, and owner boundaries in docs and live/archive coordination.
- In-scope behaviors: machine-readable module docs, UI asset protocol update, plan archive update, session note update, whitespace/diff checks for touched docs and source paths.
- Dependencies: Milestones 1-3 testing stage evidence.

### Implementation Slices

- [x] Create `docs/ui-and-layout/ui-asset-foundation-descriptors-contracts-invalidation.md` with YAML frontmatter listing related code, implementation files, plan sources, tests, and `doc_type: module-detail`.
- [x] Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` frontmatter to include descriptor, catalog, component contract, invalidation, compiler cache, and editor palette files.
- [x] Update `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` body with concise links to descriptor registry, component contract, invalidation, compile cache, and acceptance evidence.
- [x] Update `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md` M10/M12/M13 entries only after validation evidence exists; preserve any still-open gaps instead of marking completion prematurely.
- [x] Update archived session evidence for `.codex/sessions/archive/20260501-1518-ui-foundation-m10-m12-m13-design.md` and `.codex/sessions/archive/20260501-2030-ui-foundation-m12-postreview-validation.md` with final M12 validation, blockers, and handoff notes.

### Testing Stage: Documentation And Batch Acceptance Gate

- Run whitespace check on touched files:

```powershell
git diff --check -- zircon_runtime/src/ui/component zircon_runtime/src/ui/template/asset zircon_runtime/src/ui/tests zircon_editor/src/ui/asset_editor zircon_editor/src/tests/ui/ui_asset_editor docs/ui-and-layout docs/superpowers/specs/2026-05-01-ui-foundation-m10-m12-m13-design.md docs/superpowers/plans/2026-05-01-ui-foundation-m10-m12-m13.md ".codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md" ".codex/sessions/20260501-1518-ui-foundation-m10-m12-m13-design.md"
```

- Run scoped runtime and editor checks after all milestones because both crates are touched:

```powershell
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13 --message-format short --color never
```

- If shared APIs changed broadly, run the validator package path for runtime and editor:

```powershell
.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13
.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -TargetDir E:\cargo-targets\zircon-ui-foundation-m10-m12-m13
```

### Debug / Correction Loop

- If docs headers miss implementation files, update headers before claiming docs are complete.
- If `git diff --check` reports only CRLF normalization warnings, record them separately from whitespace errors.
- If validator package runs fail in unrelated graphics/GI/VG active-session drift, document the blocker and keep focused UI evidence separate.

### Exit Evidence

- New/updated UI foundation docs include related code, implementation files, plan sources, tests, and validation evidence.
- Archive plan records M13/M10/M12 progress with exact validation commands and remaining gaps.
- Session note is either updated for handoff or retired according to cross-session coordination rules.

## Batch Acceptance Boundaries

- M13 is not accepted until descriptor validation, host capability filtering, registry revisioning, descriptor-driven palette entries, and docs exist.
- M10 is not accepted until closed private-boundary validation and semantic API version compatibility are covered by focused negative tests.
- M12 is not accepted until invalidation graph propagation, cache hit/miss causes, descriptor/contract invalidation, dirty impact mapping, and large-document diagnostics are covered.
- The batch is not accepted if implementation expands hardcoded editor palette lists, creates a parallel widget registry, or moves shared authority into `zircon_editor`.
- The batch is not accepted if it touches graphics/GI/VG, Runtime UI showcase data-source ownership, physics/animation, or watcher/chrome files to make unrelated tests pass.
