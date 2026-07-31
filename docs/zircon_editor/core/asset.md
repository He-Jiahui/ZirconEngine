---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/src/core/asset/
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/ui/host/editor_asset_manager/
  - zircon_editor/src/ui/layouts/views/asset_browser/
implementation_files:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/source_authority.rs
  - zircon_editor/src/core/asset/toolkit_route.rs
  - zircon_editor/src/core/asset/type_registry/mod.rs
  - zircon_editor/src/core/asset/type_registry/asset_type_id.rs
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/asset/type_registry/contribution.rs
  - zircon_editor/src/core/asset/type_registry/context_command.rs
  - zircon_editor/src/core/asset/type_registry/creation_template.rs
  - zircon_editor/src/core/asset/type_registry/definition.rs
  - zircon_editor/src/core/asset/type_registry/error.rs
  - zircon_editor/src/core/asset/type_registry/presentation.rs
  - zircon_editor/src/core/asset/type_registry/registry.rs
  - zircon_editor/src/core/asset/type_registry/thumbnail_provider.rs
  - zircon_editor/src/core/asset/type_registry/toolkit.rs
  - zircon_editor/src/ui/host/editor_event_runtime_access.rs
  - zircon_editor/src/ui/host/editor_event_execution/asset_event.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/
  - zircon_editor/src/ui/host/asset_editor_sessions/lifecycle.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/workbench/snapshot/asset/
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_editor/src/ui/layouts/views/assets_activity.rs
  - zircon_editor/src/ui/workbench/asset_content_layout/paint_metadata.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/template_node_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/projector.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/scrollbar/asset.rs
  - zircon_editor/src/ui/layouts/views/assets_activity/
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-asset-type-registry-clone-on-augment.md
  - docs/plans/zircon_editor/editor/09/failure-2026-07-17-asset-pane-projector-repeated-model-scans.md
  - docs/plans/zircon_editor/editor/09/2026-07-19-asset-content-generation-projection.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-current-state-and-hard-cutover-audit.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-approved-asset-type-registry-design.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-asset-type-registry-core.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-extension-registry-hard-cut.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-browser-preview-registry-projection.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-source-authority-write-guards.md
  - docs/plans/zircon_editor/editor/09/2026-07-14-m1-asset-toolkit-route-hard-cut.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - zircon_editor/src/core/asset/type_registry/builtin.rs::tests::builtin_lookup_does_not_construct_an_owned_asset_type_id
  - zircon_editor/src/tests/editor_asset_type_registry/asset_type_id.rs
  - zircon_editor/src/tests/editor_asset_type_registry/builtins.rs
  - zircon_editor/src/tests/editor_asset_type_registry/extension_registry.rs
  - zircon_editor/src/tests/editor_asset_type_registry/materialization.rs
  - zircon_editor/src/tests/editor_asset_type_registry/consumer_projection.rs
  - zircon_editor/src/tests/editor_asset_type_registry/source_authority.rs
  - zircon_editor/src/tests/editor_asset_type_registry/toolkit_route.rs
  - zircon_editor/src/tests/commands/descriptor_when.rs
  - zircon_editor/src/tests/editor_asset_type_registry/typed_authoring_descriptors.rs
  - zircon_editor/src/tests/editor_event/runtime/integration.rs
  - zircon_plugins/ui_asset_authoring/editor/src/plugin.rs
  - tools/tests/test_editor09_asset_content_generation_projection.py
  - zircon_editor/src/ui/workbench/asset_content_layout/tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/template_nodes/asset_content/tests.rs
doc_type: module-detail
---

# Editor asset type registry

`core::asset` is the headless editor owner for canonical asset-type identity and the materialized
asset-type definition graph. It is folder-backed and does not import `crate::ui`; browser rows,
preview scheduling and editor dispatch consume its projections at the UI boundary.

## Identity contract

`AssetTypeId` is an open validated string-newtype. Each dotted segment starts with a lowercase ASCII
letter and continues with lowercase letters, digits or underscore. This accommodates both the 26
runtime `ResourceKind` values and plugin-defined types such as `support.asset` without reverting to an
unchecked string table. Serialization calls the same parser used by direct construction, so invalid
wire values cannot bypass validation.

`AssetTypeId::from_resource_kind` is the sole canonical mapping for runtime kinds. The mapping aligns
first-party authoring keys such as `material.graph`, `animation.state_machine`,
`terrain.heightfield` and `tilemap_2d.tilemap`; the previous uppercase SDK key is not an alias.

## Materialization and ownership

`AssetTypeContribution` is the serializable plugin input. `AssetTypeRegistry` begins with one complete
built-in definition for every runtime kind and atomically applies contributions. A contribution can
define an open custom type by supplying its required presentation and thumbnail base, or augment an
existing type with a toolkit operation and creation templates. Scalar fields retain their owner and
reject a second owner with `AssetTypeRegistryError`; creation-template and context-command ids also
retain both owners for deterministic conflict diagnostics. Failed merges do not mutate the live
registry.

`AssetTypeDefinition` is the query result used by consumers. Presentation, thumbnail and toolkit are
descriptors rather than Rust trait objects or UI instances, so the contract can cross plugin and
cdylib boundaries. `EditorOperationPath` remains the typed operation identity inside toolkit and
operation-backed thumbnail descriptors.

## Consumer projection and dispatch

The host materializes built-ins plus enabled plugin contributions once per snapshot projection.
`AssetTypeProjectionSnapshot` carries type id, display name, badge, icon and color token into Asset
Browser, Assets Activity, selection, reference and subasset rows. Those layouts no longer resolve
presentation from `ResourceKind` or maintain UI-local type helpers. Creation templates and context
commands are projected as typed operation records; toolkit view id and open operation are projected
only from the enabled materialized definition.

Open dispatch requires an indexed asset type, a registered toolkit view, an existing command entry
for the toolkit operation and a passing command `when` evaluation. File suffixes do not infer a
toolkit. Create/context invocation resolves the descriptor again from the materialized registry and
then enters the normal operation host with typed JSON arguments.

Preview generation reads `ThumbnailProviderDescriptor` and its placeholder palette from the
canonical asset-type definition. The parallel `EditorAssetMetaDocument` and
`*.editor.meta.toml` channel are deleted; runtime `.zmeta` remains the only asset sidecar owner.

## Toolkit route and source resolution

`AssetToolkitOpenRoute` is the project-stable handoff between registry dispatch and a domain
toolkit. It serializes only a canonical `AssetUri` and the typed `EditorOperationPath` that opened
the view. It deliberately has no OS path field and rejects unknown fields, so a saved workspace
cannot retain the retired `{ path, operation_id }` payload or silently reinterpret a machine-local
file as project identity.

`OpenAsset` parses the locator before catalog lookup. A catalog record selects the asset type, the
materialized definition selects the toolkit, and the typed route is the only serializable view
payload. A domain host resolves the locator through the current project's
`ProjectManager::source_path_for_uri` when it restores a file-backed session; project and registered
package roots therefore use one authority. Animation sessions keep the typed route alongside their
in-memory document, preserve it after edits and saves, and request reimport using the locator rather
than reverse-mapping a source path. UI asset restore accepts this generic typed route or its current
domain route, but no longer accepts the old generic `path` fallback.

## Source authority and mutation guard

`AssetSourceAuthority` is the typed editor projection of a validated `ResourceLocator`. It maps
`res://` to `Project`, `package://` to `Package`, `builtin://` to `Builtin`, `lib://` to
`Library`, and `mem://` to `Transient`; derived artifacts use the explicit `Derived` constructor
because `derived://` is not a supported runtime resource scheme. `AssetSourceWritePolicy` belongs
to each materialized asset-type definition. Its default is fail-closed `ReadOnly`; built-in and
newly defined authoring types explicitly use `ProjectOnly`, which makes only `Project` locators
writable. Package, builtin, library, derived and transient sources always remain read-only.

Mutation routing is guarded at both discovery and execution. Creation templates automatically
associate their command with the `asset_type` and `target_folder` arguments. Context commands are
read-only by default and opt into `Mutation`; mutation commands associate `asset_type` and
`asset_locator`. Host registration writes this association into the canonical
`EditorCommandDescriptor`, whose effective `when` adds `AssetWritable` and survives serialization.
Browser/menu evaluation projects the selected locator's access into `CommandEvalCtx`.

Actual `invoke_operation` dispatch independently resolves the invocation arguments against the
current materialized `AssetTypeRegistry`, validates the target through `ResourceLocator`, and
rejects a read-only source before an operation event can execute. The same path handles UI, menu,
CLI and remote invocation, so direct operation requests cannot bypass a disabled UI command.
Missing or malformed arguments, an unknown type, or an unsupported scheme all fail closed. There
is no suffix inference, URI string prefix exception, compatibility alias or second writable-state
truth.

## Project generation consumption

Editor asset consumers resolve paths and reference views against the Runtime asset manager's active project generation. Project open transfers the `ProjectAuthority`-validated manager through `open_prepared_project`; document load/save, workspace watcher, UI asset external promotion and undo/redo side effects use an explicit generation snapshot. Hot locator resolution instead calls manager-owned `current_project_source_path`, which resolves under the authoritative project read lock from the Runtime manager's `(scheme, path)` source index without cloning the complete `ProjectManager` or probing project roots for every asset. The retired root-path helpers that reopened or cloned a manager only to recover its root are deleted.

Layout preset names are derived from `current_project_asset_uris`, a lightweight locator projection of the same authoritative registry, and never enumerate or parse preset files during normal projection. Explicit preset save writes once and requests a Runtime import refresh. That refresh is currently full-project; the transactional targeted form remains open in Runtime04 and must preserve atomic sidecar/artifact/registry publication, dependency edges, duplicate GUID ownership and compound topology. Preset save/load errors retain `SceneProjectError` as the typed `EditorError` source. Editor does not add a second path cache to hide the Runtime gap.

## Hard-cut boundary

`EditorExtensionRegistry` and first-party editor plugins now publish only
`AssetTypeContribution`. The old `AssetEditorDescriptor`, parallel `asset_creation_templates` table,
old register/read methods, and importer/graph/timeline bare authoring-type fields were hard-deleted.
Host registration materializes built-ins plus enabled plugin contributions and preserves each plugin
package id as the merge owner. The old `adapter_key`/`AssetBrowserAdapter*` presentation vocabulary,
suffix-based open dispatch, UI-local kind label helpers, asset-details toolkit fallback fields and
the editor-only sidecar were deleted rather than aliased. The UI Asset Authoring plugin contributes
its toolkit, creation templates and context commands through the same typed contribution.

## Validation status

The TDD contracts cover canonical-id parsing and serde, unique completeness for all 26 runtime kinds,
built-in augmentation, complete custom-type materialization, duplicate toolkit/template/context
ownership, extension-registry serialization, retired API guards, typed importer/graph/timeline
descriptors, indexed toolkit open dispatch and rejection of incomplete custom definitions. The
latest accepted Windows M1.1-M1.3 binary passed registry 22/22, indexed/suffix open 2/2, Asset
Browser 41/41, Assets Activity 9/9, asset workspace 4/4 and reference drag projection 9/9; the UI
Asset Authoring editor plugin passed 2/2. M1.4 source policy/authority, descriptor when and direct
dispatch contracts pass in the newer Windows binary: the complete asset-type registry suite is
24/24 and descriptor target serde/when is 1/1. The initial invalid folder-tree fixture was corrected
to exercise the canonical `res://ui` folder. This proves M1.4 complete, but does not claim the
complete Editor09 M1 test stage while shared Runtime hard-cut handoffs still block the current full
suite.

The 2026-07-14 toolkit-route correction has an accepted RED: its initial two contracts failed to compile
only because `AssetToolkitOpenRoute` did not yet exist, recorded in
`.codex/tmp/editor09-toolkit-route-red-20260714.log`. The typed route/source-resolution implementation,
formatting and scoped diff checks are complete. A second RED proved that the canonical
`EditorOperationPath` derive decoder accepts an invalid wire value. The route now owns a strict wire
decoder that rejects unknown fields and calls `EditorOperationPath::parse`; a standalone `rustc --test`
harness directly compiled the current route and operation sources, and serde roundtrip, retired-payload
rejection and invalid-operation rejection passed 3/3 in 0.02 seconds
(`.codex/tmp/editor09-toolkit-route-standalone-green-20260714.log`). The canonical operation decoder gap
remains an Editor08 handoff. A coordinator-managed Windows test stage then compiled the current
`zircon_editor` lib-test binary; the typed route contracts passed 3/3 and indexed toolkit open plus
suffix-rejection passed 2/2. The full package run compiled successfully but later returned Cargo 101
without a test summary, so its resource-lifecycle gap remains with the existing Runtime11/Editor14
handoffs; this focused evidence completes the route correction without claiming the complete Editor09
M1 or the full package suite green.

## Performance review status

The 2026-07-17 performance pass changed built-in lookup to use the canonical static
`ResourceKind` id and `BTreeMap`'s borrowed `str` lookup, so a query no longer constructs an owned
`AssetTypeId(String)`. The source guard is green and formatting/diff checks pass; current-source
Cargo remains pending.

`apply_contribution` still clones the complete materialized entry before every augmentation and
sorts growing descriptor vectors after each delta. The validate-then-commit/generation-finalize
repair is owned by Editor09's linked failure record; this module is not performance-accepted yet.

## Asset content generation projection

Assets Activity and Asset Browser publish one `AssetContentPaintMetadata` with their final
`ModelRc<ViewTemplateNodeData>`. The metadata owns parsed control identity, content viewport and
extent, Activity folder count, fixed node rows and vertically ordered scroll groups. It is built
after layout completes, so the retained painter never reconstructs stable asset structure.

`ModelRc` carries clone-shared typed metadata and `project_nodes` preserves that same allocation
while converting view nodes into host nodes. Activity/Browser projectors use the metadata for
scroll, hover, clip and exact row visitation; the scrollbar reads the same viewport/extent. The
retired painter-local identity module is deleted. Stable paint therefore performs no model-wide
`row_data` loop and no identity parsing, while damage/scroll work is bounded by fixed nodes plus
visible scroll groups rather than total asset count.

Static contracts, shared-metadata tests and a 10,000-thumbnail visible-group boundary are present.
Managed Cargo, product pixel equivalence, allocation/CPU p95 evidence and independent review remain
pending, so the linked performance handoff is still open and this section does not claim acceptance.
