---
related_code:
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/style.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime/src/ui/template/asset/invalidation/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime/src/ui/template/asset/invalidation/graph.rs
  - zircon_runtime/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime/src/ui/template/asset/invalidation/report.rs
  - zircon_runtime/src/ui/template/asset/invalidation/stage.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/resource_ref/mod.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resource_ref.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/resource_kind.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/fallback_policy.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/dependency.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/diagnostic.rs
  - zircon_runtime/src/ui/template/asset/resource_ref/collect.rs
  - zircon_runtime/src/ui/template/asset/compiler/compile.rs
  - zircon_runtime/src/ui/template/asset/compiler/ui_document_compiler.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/cache_key.rs
  - zircon_runtime/src/ui/template/asset/invalidation/change.rs
  - zircon_runtime/src/ui/template/asset/invalidation/fingerprint.rs
  - zircon_runtime/src/ui/template/asset/invalidation/graph.rs
  - zircon_runtime/src/ui/template/asset/invalidation/impact.rs
  - zircon_runtime/src/ui/template/asset/invalidation/stage.rs
  - zircon_editor/src/ui/asset_editor/session/lifecycle.rs
  - zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs
plan_sources:
  - user: 2026-05-01 implement M15 UI media/font/resource refs foundation
  - .codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
tests:
  - zircon_runtime/src/ui/tests/asset_resource_refs.rs
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/tests/asset_invalidation.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
doc_type: milestone-detail
---

# UI Asset Resource References M15 Design

## Goal

M15 adds a shared runtime-owned resource reference model for UI assets. The milestone makes font, image, media, and generic asset dependencies explicit enough for the compiler, invalidation graph, cache key, diagnostics, and future package validation to agree on the same dependency list.

This is a foundation gate. It does not build a full editor resource browser, file watcher, asset database resolver, package manifest writer, or runtime loading backend.

## Current Baseline

The UI asset document model already has tree-shaped authority, widget/style imports, component contracts, descriptor registry authority, and invalidation/cache foundation. Resource-like values still appear as raw strings in node props, style declarations, render metadata, and preview mocks. The compiler has no structured resource dependency output, and the cache key only fingerprints document, widget imports, style imports, descriptor registry revision, and component contract revision.

Existing `UiValueKind::AssetRef` describes generic Runtime UI component values, but it is not a UI asset dependency model. M15 must not reuse that enum as the document-level resource contract because it lacks kind-specific resource categories, fallback policy, source location, diagnostic identity, and dependency fingerprints.

## Ownership And Boundaries

`zircon_runtime::ui::template::asset` owns the resource reference contract. It is part of the serialized UI asset protocol and compiler boundary, not an editor-only convention. The editor consumes compiler-produced dependency and diagnostic views but does not own resource parsing, classification, fallback validation, or cache fingerprints.

The implementation stays within:

- `zircon_runtime/src/ui/template/asset/resource_ref/` for declarations, collection, validation, diagnostics, and fingerprints.
- `zircon_runtime/src/ui/template/asset/compiler/` for exposing dependencies on compiled documents and cache outcomes.
- `zircon_runtime/src/ui/template/asset/invalidation/` for resource dependency changes.
- `zircon_editor/src/ui/asset_editor/session/` for a minimal dependency/diagnostic view.

The implementation must not touch graphics, plugin renderer migration, Runtime UI showcase projection, world-space UI handoff, workspace watcher UX, or editor chrome.

## Data Model

M15 introduces these runtime asset types:

```rust
pub enum UiResourceKind {
    Font,
    Image,
    Media,
    GenericAsset,
}

pub enum UiResourceFallbackMode {
    None,
    Placeholder,
    Optional,
}

pub struct UiResourceFallbackPolicy {
    pub mode: UiResourceFallbackMode,
    pub uri: Option<String>,
}

pub struct UiResourceRef {
    pub kind: UiResourceKind,
    pub uri: String,
    pub fallback: UiResourceFallbackPolicy,
}

pub struct UiResourceDependency {
    pub reference: UiResourceRef,
    pub source: UiResourceDependencySource,
    pub path: String,
}
```

`UiResourceDependencySource` identifies where the dependency was found:

- `DocumentImport`
- `NodeProp`
- `NodeLayout`
- `NodeStyleOverride`
- `ChildMountSlot`
- `StyleRuleDeclaration`
- `TokenValue`
- `ImportedWidget`
- `ImportedStyle`

`UiAssetImports` gains `resources: Vec<UiResourceRef>`. This is the explicit dependency list for resources that are not discoverable from props or style declarations, and it gives future M16 package validation a stable manifest input.

## Serialized Forms

The explicit form is a TOML table:

```toml
[[imports.resources]]
kind = "image"
uri = "res://textures/editor/folder.png"

[imports.resources.fallback]
mode = "placeholder"
uri = "res://textures/editor/missing-image.png"
```

The same table form may appear inside `props`, `layout`, style declarations, token values, and child mount slot metadata when a value is a resource:

```toml
[root.props]
image = { kind = "image", uri = "res://textures/editor/logo.png", fallback = { mode = "placeholder", uri = "res://textures/editor/missing-image.png" } }

[[stylesheets.rules]]
selector = "Label.title"
set.self.font = { kind = "font", uri = "res://fonts/display.font.toml" }
```

Legacy string forms remain accepted as dependency inputs when the string looks like a resource URI or when the property name strongly implies a resource:

- `font = "res://fonts/default.font.toml"` maps to `Font`.
- `image = "res://textures/icon.png"` maps to `Image`.
- `media = "res://video/intro.webm"` maps to `Media`.
- `asset = "res://data/palette.asset.toml"` maps to `GenericAsset`.

String inference is compatibility input only. Canonical authoring should prefer typed table values when fallback behavior matters.

## Resource Classification

Classification follows a deterministic order:

1. Typed table `kind` wins over property-name or extension inference.
2. Known source path names (`font`, `font.asset`, `image`, `icon`, `background_image`, `media`, `video`, `audio`, `asset`, `resource`) infer kind for legacy strings.
3. File extensions fill gaps when a property name is generic:
   - `ttf`, `otf`, `woff`, `woff2`, `font.toml` map to `Font`.
   - `png`, `jpg`, `jpeg`, `webp`, `bmp`, `tga`, `svg`, `ico` map to `Image`.
   - `mp3`, `ogg`, `wav`, `flac`, `mp4`, `webm`, `mov` map to `Media`.
   - Everything else with `res://`, `asset://`, or `project://` maps to `GenericAsset`.
4. Non-resource strings stay normal UI values and do not produce dependencies.

## Fallback Policy

Fallback policy is intentionally simple for the foundation gate:

- `None`: resource is required. Missing resolver integration is out of scope, so this only means package validation must eventually treat it as required.
- `Placeholder`: use another resource URI as fallback. The fallback kind must match the primary resource kind.
- `Optional`: absence is allowed and no fallback URI is required.

Validation rejects empty resource URIs, unsupported schemes, placeholder fallback without a URI, fallback URI equal to the primary URI, and fallback kind mismatches.

## Compiler Behavior

`UiDocumentCompiler::compile(...)` still returns compiled UI structure, but `UiCompiledDocument` gains a `resource_dependencies: Vec<UiResourceDependency>` field and a `resource_diagnostics: Vec<UiResourceDiagnostic>` field.

The compiler collects dependencies after document shape and component contract validation and before returning `UiCompiledDocument`. Collection walks:

- `document.imports.resources`
- root and component node trees
- node `props`, `layout`, `params`, `style_overrides.self_values`, `style_overrides.slot`
- child mount `slot`
- document stylesheets
- imported widget documents and imported style documents registered on the compiler

The collector deduplicates identical `(kind, uri, fallback)` resources for packaging while preserving individual source paths for diagnostics. Source paths use stable strings such as `root.props.image`, `node:logo.props.image`, `stylesheet:main.rule:hero.set.self.font`, and `import:res://widgets/card.ui.toml#Card.node:title.props.image`.

## Diagnostics

`UiResourceDiagnostic` is separate from `UiInvalidationDiagnostic` because it describes resource contract issues, not performance warnings. It contains:

```rust
pub enum UiResourceDiagnosticSeverity {
    Warning,
    Error,
}

pub struct UiResourceDiagnostic {
    pub code: String,
    pub severity: UiResourceDiagnosticSeverity,
    pub message: String,
    pub path: String,
}
```

Initial diagnostic codes:

- `empty_resource_uri`
- `unsupported_resource_scheme`
- `resource_kind_mismatch`
- `placeholder_fallback_missing_uri`
- `placeholder_fallback_self_reference`
- `placeholder_fallback_kind_mismatch`

Compiler behavior for M15 is conservative: diagnostics are returned on compiled documents when the input can still compile, but malformed typed resource tables that make the resource model ambiguous are errors returned as `UiAssetError::InvalidDocument` during resource collection. This keeps authoring feedback visible without allowing invalid contract data into cache fingerprints.

## Invalidation And Cache

`UiCompileCacheKey` gains `resource_dependencies_revision: UiAssetFingerprint`. The fingerprint is computed from the sorted unique resource dependency set from the root document and registered imports. It deliberately excludes dependency source paths so moving the same dependency within the same document is already covered by the root document fingerprint, while package-impacting dependency changes have their own stable signal.

`UiInvalidationSnapshot` gains `resource_dependencies_revision`. `UiAssetChange` gains `ResourceDependency`. `UiInvalidationStage` gains `ResourceDependency`. Resource changes affect:

- `Render` for image, media, and generic asset dependencies.
- `StyleValue`, `Layout`, `Render`, and text dirty flags for font dependencies.
- `Projection` because editor dependency views must refresh.

`UiInvalidationImpact::include_stage(ResourceDependency)` sets `dirty.render = true`, `projection_dirty = true`, and `rebuild_required = true`. If the invalidation graph later receives kind-aware delta information it may refine font-only impacts, but the foundation gate keeps stage impact conservative.

## Editor Consumption

`UiAssetEditorSession` gains minimal read-only views:

- `resource_dependencies(&self) -> &[UiResourceDependency]`
- `resource_diagnostics(&self) -> &[UiResourceDiagnostic]`

Session validation stores the compiler-produced resource list from the last successful preview compile. If compile fails before dependency collection, these lists are cleared and normal diagnostics remain string-based. Editor presentation may show dependency rows and diagnostic rows, but M15 does not add a picker, resource browser, asset existence checker, drag-drop resource authoring, watcher integration, or package export UI.

## Tests And Acceptance

Runtime tests must cover:

- explicit `imports.resources` serialization and compile collection
- legacy string inference from node props and style declarations
- typed table fallback collection
- fallback validation failures
- duplicate dependency deduplication with source path preservation
- imported widget/style resource dependency collection
- cache miss and invalidation report when only resource dependency fingerprint changes

Editor tests must cover:

- `UiAssetEditorSession` exposes resource dependency rows after successful compile
- resource diagnostics appear without enabling editing of invalid resource data
- dependency view refreshes after source edits that change resource refs

Acceptance commands are scoped to the foundation gate:

```powershell
rustfmt --edition 2021 --check zircon_runtime/src/ui/template/asset/resource_ref/*.rs zircon_runtime/src/ui/template/asset/*.rs zircon_runtime/src/ui/template/asset/compiler/*.rs zircon_runtime/src/ui/template/asset/compiler/cache/*.rs zircon_runtime/src/ui/template/asset/invalidation/*.rs zircon_runtime/src/ui/tests/asset_resource_refs.rs zircon_runtime/src/ui/tests/asset_compile_cache.rs zircon_runtime/src/ui/tests/asset_invalidation.rs zircon_editor/src/ui/asset_editor/session/*.rs zircon_editor/src/tests/ui/ui_asset_editor/resource_dependency_view.rs
cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo test -p zircon_editor --lib resource_dependency_view --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-asset-resource-refs-m15 --message-format short --color never
```

Workspace-wide validation is not part of this foundation gate unless the implementation changes public workspace wiring beyond `zircon_runtime` and `zircon_editor` UI surfaces.

## Non-Goals

- No resource file existence checks.
- No image/font/media loading backend.
- No editor resource browser or picker.
- No hot reload or watcher wiring.
- No package manifest writer or compiled artifact header. Those remain M16.
- No graphics or renderer changes.
- No Runtime UI showcase projection or data-source adapter changes.

## Approval State

The user approved starting M15 from this foundation scope: typed `UiResourceRef`, four resource kinds, fallback policy, compiler dependency collection, structured diagnostics, invalidation/cache fingerprints, and minimal editor dependency/diagnostic consumption.
