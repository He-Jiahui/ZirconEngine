---
related_code:
  - zircon_editor/assets/ui/editor/component_showcase.v2.ui.toml
  - zircon_editor/assets/ui/editor/material_component_lab.v2.ui.toml
  - zircon_editor/assets/ui/editor/host/activity_drawer_window.zui
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/graph.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/support.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/src/ui/v2/loader.rs
implementation_files:
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/graph.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance/support.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_v2_asset.rs
plan_sources:
  - .codex/plans/Zircon UI .zui 组件资产与 Unreal 风格入口重构计划.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
tests:
  - cargo test -p zircon_editor --lib zui_asset_governance --locked --jobs 1
  - cargo test -p zircon_runtime --lib fixture_v2_toml_importer_rejects_component_kind_in_favor_of_zui --locked --jobs 1
  - cargo test -p zircon_runtime --lib importer_decodes_zui_component_assets_from_zui --locked --jobs 1
doc_type: workflow-detail
---

# ZUI Asset Governance

`.zui` is the production suffix for reusable UI component prototypes. A `.zui` document uses the UI v2 TOML schema in memory, but it is not a view root. `UiZuiAssetLoader` enforces the component profile: `asset.kind = "component"`, no `[root]` view entry, exactly one component, and a component root node that exists in the node table.

Production `.zui` files must live under explicit component directories such as `ui/editor/components` or `ui/editor/material_components`. Files outside those directories are allowed only as registered builtin alias exceptions; the current host-path exception is `ui/editor/host/activity_drawer_window.zui`. This keeps reusable component prototypes from being scattered through view/root folders while still allowing a short migration window for stable builtin ids.

Production `.v2.ui.toml` files remain the builtin view/style roots. A production `.v2.ui.toml` asset may declare only `asset.kind = "view"` or `asset.kind = "style"`; component prototypes belong in `.zui` files. Their `imports.widgets` entries may reference only `.zui` component assets or explicitly registered builtin `.zui` aliases. A direct widget import must name the component fragment explicitly, for example `res://ui/editor/components/showcase_visual_section.zui#ShowcaseVisualSection`; alias imports follow the same `asset.id#ComponentName` shape. Importing a `.zui` file without `#ComponentName`, or importing an old `.v2.ui.toml#ComponentName` widget source, is invalid for production assets because widget composition now flows through `.zui` component prototypes.

Production `.zui` `[asset].id` values default to their file-derived `res://` locator. This keeps component assets relocatable through the same path alias that `UiV2PrototypeStoreFileCache` registers for loaded source files. Builtin aliases are allowed only when they are explicitly documented in the governance test and are still referenced by a production `.v2.ui.toml` root. The current exception is `zircon_editor/assets/ui/editor/host/activity_drawer_window.zui`, whose stable id is `editor.host.activity_drawer_window`; the window roots import that alias while the file cache also exposes the path alias `res://ui/editor/host/activity_drawer_window.zui`.

Production UI asset ids are globally unique across `.v2.ui.toml` view/style roots and `.zui` component prototypes. A `.zui` component may use a registered builtin alias instead of its locator during migration, but that alias still participates in the same production asset id namespace and must not collide with a root view, style asset, or another component.

Production UI asset headers must stay authorable and current. Every production `.v2.ui.toml` and `.zui` asset must declare the current UI v2 schema version and a non-empty `display_name`, so the Widget Editor, asset browser, diagnostics, and future migration tools can present stable human-facing labels instead of deriving names from paths.

Production `.zui` component names must stay derived from their file stems. The default component name is the file stem converted from snake case to PascalCase, such as `showcase_visual_section.zui` declaring `ShowcaseVisualSection`. Authoring/demo prototype assets may append `Prototype`, such as `material_alert.zui` declaring `MaterialAlertPrototype`. This gives widget imports, palettes, previews, and migration tools a predictable relationship between file paths and component fragments.

Every production `.zui` file must be reachable from at least one production `.v2.ui.toml` widget import. Reachability can be direct, through `res://...zui#ComponentName`, or through an explicitly registered builtin alias such as `editor.host.activity_drawer_window#ActivityDrawerWindow`. This prevents component prototypes from drifting as orphan files after showcase or workbench roots move.

Inside each production `.zui` component asset, the node table must also be fully reachable from the single component root through `children` links. `.zui` is a component prototype, not a scratch document, so unreachable node definitions are treated as stale asset data that would confuse previews, diffs, migration tools, and future Widget Editor hierarchy views.

Production `.zui` node ids must stay lower_snake_case. Node ids are persistent authoring identifiers used by hierarchy diffs, child links, preview selection, and future Widget Editor edit state; keeping them lowercase and path-like avoids accidental divergence from display labels or control ids.

Production `.zui` child mount node references must be non-empty and trimmed. Child mounts are the serialized hierarchy edges between persistent node ids; allowing empty strings or whitespace-padded references would make source diffs, slot editing, preview selection, and hierarchy diagnostics disagree about the actual parent-child link.

Production `.zui` node attribute keys must be non-empty and trimmed wherever author-facing node metadata is declared. This applies recursively to `props`, `state`, `layout`, `style.self`, and `style.slot` tables. Attribute keys flow into runtime metadata, style resolution, layout inference, retained-host inspection, and Widget Editor forms; whitespace-only key differences would make those surfaces disagree about which field is being edited.

Production `.zui` node component type strings must be non-empty and trimmed. These strings are the bridge from authored hierarchy nodes to native controls or imported component prototypes; invisible empty values or whitespace-only differences would make prototype expansion, diagnostics, and future Widget Editor palettes harder to reason about.

Each production `.zui` node graph must remain a single-root tree. The component root cannot be mounted as a child, and any other node may be mounted by only one parent. This keeps component instancing deterministic and gives the Widget Editor a stable hierarchy instead of a graph with aliases, cycles, or multiple ownership paths.

Within a single `.zui` component asset, `control_id` values must be non-empty, trimmed, and unique. The same `control_id` may appear in another asset, but duplicates or whitespace-only differences inside one component would make preview selection, binding diagnostics, retained-host testing, and Widget Editor hierarchy tooling ambiguous.

Production `.zui` slot metadata must use non-empty, trimmed names and keys. This applies to component-declared slot names, node slot maps, child mount slot keys, and child mount `slot.name` values. The rule deliberately does not enforce a casing convention, because imported Material/MUI semantics include camelCase names such as `inputRoot` and `popupIndicator`; it only removes invisible empty or whitespace-padded values that would destabilize hierarchy editing and Inspector state.

Production `.zui` event bindings must stay authorable and dispatchable. Each binding id must be non-empty, trimmed, and unique inside its `.zui` asset. A binding may dispatch through a legacy route or a structured action target, but it must declare at least one of them; when a route is present, the route string must also be non-empty and trimmed. This keeps Binding Inspector diagnostics, preview interaction, and future action-target migration from carrying duplicate or inert bindings.

Production `.zui` component prototypes must keep `style_scope = "closed"`. The shared UI plan allows open style scopes only when a component explicitly exposes public style parts; that contract is not present in the current production `.zui` profile. Keeping component styles closed prevents parent view roots from accidentally depending on private internal node ids, classes, or selector structure before the Widget Editor can expose and validate public style parts.

Production `.zui` class lists must be clean where they are declared. Component `default_classes` and node `classes` may be absent, but present lists must not contain empty, whitespace-padded, or duplicate class tokens. This keeps selector matching, style diagnostics, preview overlays, and future class editing from carrying redundant or invisible tokens.

If a `.zui` component asset declares its own imports, those imports follow the same boundary as production roots. Component/widget imports must target `.zui#ComponentName` or a registered builtin `.zui` alias with a fragment, and the fragment must name the component declared by the target `.zui` asset. Style imports must be fragment-free `res://` locators that resolve to style assets. This keeps future component nesting and component-local styling on the same asset graph instead of creating a second import dialect.

Production `.zui` widget imports must not point back at the same component asset. The rule checks both the file-derived `res://...zui` locator and any registered builtin alias for that file, so future component expansion cannot create a trivial self-recursive prototype through either import spelling.

Production UI import lists must not repeat dependencies within the same asset. This applies to both `.v2.ui.toml` and `.zui` `imports.widgets` / `imports.styles` lists, keeping dependency graphs deterministic for hot reload, diagnostics, cache invalidation, and future authoring tools.

Production `.v2.ui.toml` `imports.styles` entries must use `res://` locators and resolve to UI v2 documents whose `[asset].kind` is `style`. Style imports do not name component fragments; fragment-bearing imports are reserved for widget/component prototypes. This keeps theme chaining explicit and prevents a view root from silently importing another view or component document through the style pipeline.

The builtin template registry must not directly register `.zui` files. Registry entries point at `.v2.ui.toml` roots such as `component_showcase.v2.ui.toml`; those roots then import `.zui` prototypes through the v2 prototype store. This keeps component prototypes reusable without making them standalone windows.

`zui_asset_governance.rs` guards this boundary across production editor/runtime UI roots, with `zui_asset_governance/support.rs` owning shared asset-root scanning, `res://` resolution, widget import parsing, file-stem component naming, and registered builtin `.zui` alias lookup. The rule file parses each `.v2.ui.toml`, requires the document kind to stay `view` or `style`, requires every widget import to target a direct `.zui` locator or registered builtin `.zui` alias, requires an explicit `#ComponentName`, resolves the import to a production `.zui` source file, parses the `.zui` document, and verifies the fragment names the declared component. It also checks that production `.zui` files live under component directories unless their path is an explicit alias exception, requires each `.zui` component name to match the file stem PascalCase with an optional `Prototype` suffix, checks that every production `.zui` source is reachable from widget imports, applies the same widget/style import rules to imports declared inside `.zui` component assets, verifies `.zui` internal widget import fragments against the target component document, rejects `.zui` widget imports that self-reference the current component asset, rejects duplicate widget/style import entries inside each production UI asset, verifies every production style import resolves to a style document, scans production `.zui` files to require `[asset].id` to match the derived locator unless the file is listed as a builtin alias and that alias is actively referenced, requires production UI asset ids to stay unique across `.v2.ui.toml` roots and `.zui` components, requires production UI asset headers to use the current schema version and non-empty display names, and confirms `builtin_template_documents()` contains no `.zui` paths.

`zui_asset_governance/graph.rs` owns the intra-component graph invariants: walking each `.zui` component root to ensure the full node table is reachable, requiring persistent node ids to stay lower_snake_case, requiring child mount node references to stay non-empty and trimmed, requiring node attribute keys to stay non-empty and trimmed across props/state/layout/style metadata, requiring node component type strings to stay non-empty and trimmed, rejecting root-as-child and duplicate-parent node mounts so each component graph remains a single-root tree, requiring `control_id` values to stay non-empty, trimmed, and unique inside each `.zui` asset, keeping slot metadata names/keys non-empty and trimmed, requiring event bindings to have clean unique ids and dispatch targets, requiring component style scopes to stay closed until public style parts are governed, and keeping component/node class lists free of empty, whitespace-padded, or duplicate tokens. Keeping those rules separate leaves the top-level governance file focused on cross-asset identity and import boundaries.

The `.v2.ui.toml` importer also rejects documents whose `[asset].kind` is `component`. Component documents must use `.zui`, while `.v2.ui.toml` remains available for view/style roots and selected test fixtures.
