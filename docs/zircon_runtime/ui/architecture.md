---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/runtime_ui
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/style_mapping.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/pointer
  - zircon_runtime/src/ui/surface/navigation
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/component/mod.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/v2/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime_interface/src/ui/v2/mod.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
implementation_files:
  - docs/zircon_runtime/ui/architecture.md
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/ui_architecture_boundary.py
plan_sources:
  - user: 2026-06-13 runtime architecture implementation request
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/ui-and-layout/shared-ui-template-runtime.md
  - CLAUDE.md
tests:
  - runtime_09_m0_ui_architecture_static_passed
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_doc_records_current_boundaries
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_ui_architecture_baselines_match_current_source_scan
  - zircon_runtime::tests::runtime_absorption::ui_architecture::runtime_09_v2_verdict_matches_runtime_and_interface_modules
  - ui_architecture_boundary targeted audit
  - docs/zircon_runtime/ui/v2.md
  - docs/zircon_runtime/ui/dispatch/input_manager.md
  - docs/zircon_runtime/ui/layout/pass.md
  - docs/zircon_runtime/ui/template/asset/dependency_index.md
doc_type: module-detail
---

# Runtime UI Architecture M0 Boundary

runtime_09_m0_ui_architecture_static_passed

本文件完成 Runtime 09 的 M0.1 模块边界图与 M0.2 v2 双代裁决。它只记录当前实仓边界和后续切片工作集，不修改 `zircon_runtime::ui` 生产代码。当前 UI 生产代码与 UI 文档仍有活动 editor UI 会话，因此 M1-M3 的源码切片必须等待 owner 空窗或重新协调。

## Owner Verdict

`zircon_runtime::ui` owns runtime-only UI behavior: layout passes, dispatch, render extraction inputs, text/layout engines, template compilation, surface/tree mutation, runtime v2 prototype loading, v2 style resolution, and v2 surface construction.

`zircon_runtime_interface::ui` owns neutral UI contract DTOs. Its `ui::v2` surface is the stable data schema layer for arenas, asset records, compiled graphs, repeat expansion records, and style DTOs. It must not own runtime mutation, route ordering, layout pass execution, cache invalidation, or render extraction.

`zircon_editor::ui` owns editor workbench authoring and retained-host consumption. It can consume v2 assets and runtime projection APIs, but it cannot define runtime UI route authority, layout backend ownership, or template compilation behavior.

## Module Boundary Map

Current scan baseline:

- `ui/` top-level entries: 17 = 15 directories plus `module.rs` and `style.rs`.
- `surface/` entries: 20 in the current worktree scan.
- Full UI-tree `legacy` hits: `ui_legacy_hits=167`.
- Production UI `legacy` hits/files after excluding tests and fixtures: `ui_legacy_production_hits=102` / `ui_legacy_production_files=12`.
- Production UI `taffy` hits/files after excluding tests and fixtures: `ui_taffy_production_hits=161` / `ui_taffy_production_files=7`.

| Module | Runtime owner | Boundary note |
|---|---|---|
| `module.rs` | Runtime UI module declaration | Module descriptor/config wiring only. |
| `runtime_ui/` | Runtime preview and fixture manager | Crate-private runtime UI manager path; consumes v2 cache/building and surface projection. |
| `layout/` | Constraints, pass sequence, scroll, style mapping, Taffy bridge, virtualization | Owns layout execution and backend adaptation. Taffy ownership is not yet single-file-clean; M2.1 must decide whether `style_mapping.rs` and `pass/taffy_arrange.rs` remain explicit backend owners or move behind a narrower bridge. |
| `surface/` | Retained surface state and runtime interaction state | Owns arranged output, hit testing, focus, popup stack, input state, component state, property mutation, reflection snapshots, render collection data, timeline, and diagnostics. |
| `dispatch/` | UI route manager | Owns the route authority entry point documented by `UiInputManager`: capture, popup, preview, target, bubble, focus, default action. |
| `template/` | Asset/build/instance/loader/validate boundary | Owns template compilation, validation, dependency indexing, hot reload coordination, and old template migration surfaces until M3 closes them. |
| `v2/` | Runtime v2 loader, compiler, prototype cache, style resolver, surface builder, surface tree | Runtime implementation of the v2 schema. It consumes `zircon_runtime_interface::ui::v2` DTOs and creates runtime surfaces. |
| `component/` | Component descriptor catalog and state reducers | Owns runtime catalog metadata and component state reduction, including editor shell component entries that are still runtime projection data. |
| `tree/` | Runtime tree extensions and hit-test helpers | Utility owner for UI node tree traversal and hit-test integration. |
| `binding/` | Binding event router and update report | Runtime binding/event bridge into surface mutation reports. |
| `event_ui/` | UI event codec and manager | Runtime UI event serialization/manager layer. |
| `style.rs` and `theme/` | Typed style fields and active theme registry | Owns runtime style resolution inputs and theme token registry over v2 resolved style DTOs. |
| `text/` | Crate-private text layout support | Runtime internal helper; broader text stack ownership remains Runtime 01 M2. |
| `accessibility/` | Runtime accessibility extraction | Reads surface state into accessibility output; current legacy property helpers remain M1/M3 debt. |
| `icon_atlas/` | Runtime icon atlas support | Leaf runtime UI support module. |
| `tests/` | Runtime UI test tree | Not a production owner; excluded from production debt file counts above. |

Dependency direction:

```text
template asset/build -> v2 cache/compiler -> v2 surface_tree -> surface
component catalog/state -> v2 surface_tree + surface default interactions
surface -> layout pass -> arranged output
surface input/focus/popup state -> dispatch route authority
binding/event_ui -> surface mutation reports and route results
runtime_ui -> v2 prototype cache + surface + theme
interface::ui::v2 DTOs -> runtime::ui::v2 implementation
```

No M0 blocker-level owner inversion was found in the scanned module graph. The remaining work is not "unknown ownership"; it is explicit cleanup of debt-bearing areas:

1. UI legacy naming and migration terms: 12 production files currently contain `legacy`, concentrated in accessibility extraction, layout pass backend naming, surface input routing, pointer capture, property mutation, render table splitting, template schema migration, and template interaction fallback.
2. Taffy backend exposure: 7 production files currently mention `taffy`. The planned M2.1 DoD says only `taffy_bridge.rs` should remain. Current code also has legitimate-looking owner files in `layout/style_mapping.rs` and `layout/pass/taffy_arrange.rs`, so M2.1 must first harden the intended owner shape before moving code.
3. Template generation and migration: old template asset/build/instance paths still coexist with v2 runtime paths. M3 owns the generated marker and failure-path closure.

## V2 Verdict

v2-replacement-mainline

The v2 UI path is the replacement mainline for authored runtime/editor UI assets, not dead code and not a second unconstrained runtime contract.

The source-profile split is:

- `.zui` is the production component asset suffix for imported project/editor component assets.
- `.v2.ui.toml` remains valid for v2 view/style/runtime fixture/editor chrome assets that are loaded through the v2 cache path, but it is not the general production component importer path.
- recursive `UiTemplateNode` and old template document paths are legacy/migration/test-only surfaces until M3 proves their remaining owners and deletion conditions.

Current evidence:

- `zircon_runtime_interface::ui::v2` contains the neutral v2 DTO schema.
- `zircon_runtime::ui::v2` contains the runtime loader/compiler/cache/style/surface-builder/surface-tree implementation.
- Runtime UI fixture/manager docs describe `UiV2PrototypeStoreFileCache -> UiV2SurfaceBuilder -> surface_tree -> UiSurface`.
- Editor view/chrome assets and runtime fixtures already consume v2 assets.
- Asset importer/plugin registry docs enforce `.zui` for production component assets while keeping explicit v2 fixture/view paths.

Migration route:

1. Keep `zircon_runtime_interface::ui::v2` as the neutral contract layer.
2. Keep `zircon_runtime::ui::v2` as the sole runtime implementation layer for v2 cache, compilation, style resolution, and surface construction.
3. Keep editor usage on asset/cache/projection APIs; editor code may not grow a parallel runtime builder.
4. Delete or isolate old recursive template paths only after M3 records owner files, generated-file rules, fixture exceptions, and failure-path tests.

Deletion conditions for non-v2 runtime paths:

- production project/editor component importers do not accept old recursive `.ui.toml` component documents;
- component catalog and editor shell projection use v2 assets or `.zui` component assets;
- runtime fixtures and preview manager have no fallback through `UiTemplateTreeBuilder` or old `UiTemplateSurfaceBuilder`;
- migration fixtures are named and isolated from production asset registration;
- template compile/instance/validate failure paths have explicit tests and generated output markers where they write files.

## Static Acceptance

This M0 pass is documentation and status only. It intentionally did not start Cargo because active `cargo`/`rustc` lanes were already present in the workspace. The accepted static evidence is:

- current owner map covers all 17 UI top-level entries;
- current `surface/` scan is recorded as 20 entries rather than the stale 2026-06-12 value;
- `legacy` full-tree and production-file baselines are recorded separately;
- `taffy` production-file baseline is recorded before M2.1;
- v2 is explicitly classified as replacement mainline with a source-profile split and deletion conditions.
- `runtime_absorption::ui_architecture` now guards the module count, baseline scan values, v2 runtime/interface module shape, and the plan/index anchors. Cargo has not been run for the guard yet because active lanes were present.
- `ui_architecture_boundary` mirrors the same static facts: source/doc files 11/11, `ui/` entries 17/17, `surface/` entries 20/20, UI legacy full-tree hits 167/167, UI legacy production hits 102/102, UI legacy production files 12/12, UI taffy production hits 161/161, UI taffy production files 7/7, runtime `ui::v2` anchors 10/10, interface `ui::v2` anchors 9/9, guard anchors 4/4, pending UI owner/Cargo gate anchors 7/7, doc anchors 10/10, and `risks = []`. This is static structure evidence only.
- `runtime_09_ui_architecture_cargo_gate_stays_visible_until_ui_owner_validation` keeps Runtime 09 on the `ui/input/naming_boundary/layout/template` owner/Cargo gate until editor UI owner coordination and the declared Cargo filters provide real evidence.
