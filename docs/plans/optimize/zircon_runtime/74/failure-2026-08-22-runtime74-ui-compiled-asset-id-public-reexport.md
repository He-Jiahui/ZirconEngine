---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-22
summary_slug: runtime74-ui-compiled-asset-id-public-reexport
origin_plan: docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md
fixing_plan: docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/12
fixing_child_dir: docs/plans/optimize/zircon_runtime/74
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime_interface/src/ui/template/asset/mod.rs
  - zircon_runtime_interface/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_editor/src/ui/asset_editor/binding/binding_inspector.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/scene_tree_rows.rs
  - zircon_editor/src/ui/workbench/reference/builder/nodes.rs
---

# Runtime74 UiCompiledAssetId public re-export: validation failure handoff

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/12-unreal-magicavoxel-zui-design-convergence.md`
- 来源执行切片：M6 current-source Editor bundle and real WGPU visual acceptance
- 修复责任计划：`docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md`
- 交接原因：Runtime74 owns the compiled binding IR type and its public template namespace.

## 失败现象与复现证据

- Managed command: `tools/build-editor.ps1 -OutputDirectory D:\ZirconBuilds\ui12-editor-aa-current-bee4c707-20260822`
- Managed Job: `47ed12d350d54373b1611878d573c5cc`
- Result: released with exit code 1 at 2026-08-22 18:11:43 +08:00; no final bundle was published.
- Production diagnostic: `E0432` at `zircon_runtime/src/ui/template/asset/compiler/binding_program.rs:6:35`, unresolved import `zircon_runtime_interface::ui::template::UiCompiledAssetId`.
- Independent current-source test fingerprint at 2026-08-22 18:23:44 +08:00 reproduced the same `E0432` before test-only diagnostics.
- Partial repair observed at 2026-08-22 18:53:30 +08:00: `ui/template/mod.rs` now requests
  `asset::UiCompiledAssetId`, but `ui/template/asset/mod.rs` still does not forward the symbol from
  `compiler`. The outer forwarding list therefore remains unresolved until the inner list is also
  updated.
- Current-source follow-up build at 2026-08-22 23:49 +08:00 used the coordinator-validated manual
  target `D:\cargo-targets\zircon-engine\ui12\bundle-current-bee4c707-20260822`. Cargo advanced
  through all shared dependencies and reached `zircon_runtime`, then failed with exactly three
  Runtime74-owned diagnostics:
  - `E0603` at `zircon_runtime/src/ui/v2/component_instancer.rs:18:26`: direct import from private
    module `crate::ui::template::asset`;
  - `E0282` at `component_instancer.rs:385:19` in the
    `validate_typed_component_params(...).map_err(...)` closure;
  - `E0282` at `component_instancer.rs:418:15` in the
    `resolve_component_binding_params(...).map_err(...)` closure.
- Lease evidence at 2026-08-22 23:49 +08:00 assigns `component_instancer.rs`,
  `ui/template/mod.rs`, and `ui/template/asset/mod.rs` to
  `optimize-runtime74-param-ref-compile-r3-bee4c707-20260822`; UI12 therefore did not edit them.
- The persistent managed target remains available for an incremental Editor rebuild after the
  Runtime74 repair; no final Editor bundle was published from this failed attempt.
- Runtime74 repaired the helper boundary at 2026-08-23 00:07 +08:00 by publishing the resolver
  helpers from the canonical crate-visible `crate::ui::template` namespace. The incremental
  managed build advanced through `zircon_runtime` and reached `zircon_editor`, confirming that the
  prior `E0603` and two `E0282` diagnostics are closed.
- The same incremental build then failed with five `E0560` diagnostics because Runtime74 removed
  `UiBindingRef.mode` while its leased Editor constructors still initialize that field:
  - `binding_inspector.rs:256:9`;
  - `component_property_rows.rs:112:13` and `:121:13`;
  - `scene_tree_rows.rs:108:9`;
  - `workbench/reference/builder/nodes.rs:344:9`.
- One concurrent `E0308` in `native_panes/viewport.rs:28:13` was stale by build completion:
  `HostViewportImageData::rgba()` and its viewport caller were both updated at 00:25:29 +08:00 to
  preserve `Arc<[u8]>`. Current source now matches the shared-image drawing signature, so this item
  is not routed to Runtime74 and requires no additional UI12 edit.

## 最低共享层根因

`UiCompiledAssetId` is declared in
`zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs` and exported by the
compiler submodule, but it is absent from the established forwarding lists in
`ui/template/asset/mod.rs` and `ui/template/mod.rs`. The runtime compiler imports all compiled IR
identifiers through the public `ui::template` namespace, so the current public contract is
internally incomplete.

The later component-instancer diagnostics expose the same boundary class at crate scope:
Runtime74 exported the param-resolution helpers from `template::asset` as `pub(crate)`, while
`component_instancer.rs` reaches through the private `asset` module instead of using a canonical
crate-visible `template` re-export. The two inference errors are downstream at calls to those
helpers and must be re-evaluated after the import boundary is corrected.

## 架构修复验收

- Forward `UiCompiledAssetId` through the same canonical public template namespace as the sibling
  `UiCompiledNodeId`, `UiCompiledControlId`, and `UiCompiledBindingTargetId` types.
- The current-source managed Editor build advances past this production `E0432`.
- Expose the component-param resolver helpers through the canonical crate-visible
  `crate::ui::template` boundary and import them there; do not make the whole `asset` module
  visible.
- Re-run the persistent-target managed Editor build and advance past the three
  `component_instancer.rs` diagnostics.
- Remove the obsolete `mode` initializers from all five leased Editor construction sites and
  re-run the persistent-target build past the resulting `E0560` set.
- Re-run the existing Runtime74 canonical-loader failure record separately; its test-only
  `UiAssetLoader::load_str` and inference errors are not accepted as part of this repair.

## 禁止临时方案

- Do not add a duplicate ID type, compatibility module, deprecated alias, or runtime-side private
  import that bypasses the intended public contract.
- Do not modify UI12 AA, theme, capture, or profiling code to hide the shared compile failure.

## 修复结果与回传

Open state: the compiled-ID and component-param helper boundaries have advanced; five
Runtime74-owned stale `UiBindingRef.mode` initializers remain pending owner repair and managed
incremental validation.
