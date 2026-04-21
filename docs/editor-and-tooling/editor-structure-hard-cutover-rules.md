---
related_code:
  - zircon_editor/build.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/core/editing/state/editor_state_render.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/slint_host/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/ui/workbench/state/mod.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/startup/mod.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/snapshot/data/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/host/template_runtime/mod.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/mod.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/root_surfaces.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/workbench_state_cutover.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
implementation_files:
  - zircon_editor/build.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/editing/state/editor_state_render.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/editor_state_asset_workspace.rs
  - zircon_editor/src/ui/workbench/state/mod.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/workbench/startup/mod.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/snapshot/data/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/host/template_runtime/mod.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/mod.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/root_surfaces.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/workbench_state_cutover.rs
  - zircon_editor/assets/ui/editor/host/workbench_shell.ui.toml
plan_sources:
  - user: 2026-04-20 implement the workspace hard cutover and standardize the result
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 是指加载入口资源文件
  - .codex/plans/ZirconEngine 全仓结构硬切换与规范固化计划.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_editor/src/tests/ui/boundary/root_surfaces.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/boundary/workbench_state_cutover.rs
  - zircon_editor/src/tests/host/template_runtime/structure_split.rs
  - zircon_editor/src/tests/ui/template/catalog_registry.rs
  - zircon_editor/src/tests/ui/template/binding_resolution.rs
  - zircon_editor/src/tests/ui/template/repository_assets.rs
  - zircon_editor/src/tests/editing/state.rs
  - cargo check -p zircon_editor
  - cargo test -p zircon_editor boundary
  - cargo test -p zircon_editor template
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_editor -SkipTest -VerboseOutput
  - zircon_editor/tests/native_window_hosts.rs
doc_type: module-detail
---

# Editor Structure Hard Cutover Rules

## Purpose

This document is the editor-side authority for the converged `core/scene/ui` split and the owner rules inside `zircon_editor`.

## Top-Level Split

- `core/`
  - Owns editor-state mutation behavior, intent/history, command flow, and editor-event runtime.
  - Must not own the `EditorState` declaration, workbench layout/view logic, native window orchestration, asset-editor UI sessions, or editor host module wiring.
- `scene/`
  - Owns authoring-time scene viewport, gizmo, handle, pointer routing, and viewport state.
  - Must not absorb runtime world ownership or generic UI host behavior.
- `ui/`
  - Owns editor host surfaces, workbench shell, asset editor UI, Slint integration, and editor-only UI bindings.

## UI Owner Rules

- `ui/host/`
  - Sole owner of `EditorManager`, `EditorModule`, module descriptor/service names, window host state, startup/workspace persistence, builtin layout/view registration, and asset-editor session orchestration.
- `ui/slint_host/`
  - Slint adapter and native-window glue only.
  - No business ownership for workbench state or asset-editor session logic.
  - `callback_dispatch/mod.rs` plus namespace roots such as `asset/mod.rs`, `inspector/mod.rs`, `layout/mod.rs`, `shared_pointer/mod.rs`, `viewport/mod.rs`, and `workbench/mod.rs` stay structural; pointer/layout/menu/template dispatch behavior lives below those roots.
- `ui/asset_editor/`
  - Folder-backed owner only.
  - `binding/`, `preview/`, `session/`, `source/`, `style/`, and `tree/` are authoritative; flat duplicates are not allowed.
- `ui/workbench/`
  - Owns `state/`, layout, model, view, reflection, snapshot, startup, and project workspace structures.
  - Owns the `EditorState` declaration under `ui/workbench/state/`; `core/editing/state/` keeps behavior impl files only.
  - Owns asset-browser/workspace presentation state and project overview snapshot shaping.
  - Owns the thin `EditorState` asset-workspace accessors that forward into `AssetWorkspaceState`; `core/editing/state` must not keep a duplicate asset-workspace owner file.
  - Owns `EditorState` constructors that shape welcome/project startup state, default asset-workspace state, and initial selection policy.
  - Owns `EditorState` welcome/project session transition helpers such as `replace_world`, `clear_project`, and `set_welcome_snapshot`; `core/editing/state` must not keep a startup-session owner file.
  - Owns `EditorDataSnapshot` shaping for `EditorState`; `core/editing/state` keeps render/state accessors, but must not own UI DTO snapshot builders.

## Root Surface Rules

- `zircon_editor/src/lib.rs` exposes only high-level entry points.
- `zircon_editor/src/lib.rs` must not own the `EditorModule` type or its `EngineModule` implementation; that ownership lives under `ui/host/module.rs`.
- `zircon_editor/src/ui/mod.rs` may declare structure and curated exports, but must not become a mixed-domain umbrella.
- `zircon_editor/src/ui/mod.rs` must not re-export `asset_editor`, `binding`, `control`, or `template` specialist types for convenience; callers import those owner paths directly.
- Specialist workbench, asset-editor, host, and viewport types are referenced through their owner modules.

## File Hygiene Rules

- Large subsystem roots split into folders before they become new umbrella files.
- Same-domain folder ownership beats flat file accumulation.
- `zircon_editor/build.rs` must recursively track the full `ui/` tree so nested `.slint` and template files invalidate generated bindings immediately.
- Production builtin host template entry files must stay outside `src/`; the current owner path is `zircon_editor/assets/ui/editor/host/*.ui.toml`.
- Production template-runtime loading inside `ui/template_runtime/runtime/runtime_host.rs` must accept tree `UiAssetDocument` input only. `UiTemplateLoader` / `UiTemplateDocument` may remain in legacy adapters or tests, but not in editor production runtime.
- `ui/template/registry.rs` must not reintroduce a dual authority store for legacy template documents. Compiled asset documents are the only registry authority on the production path.
- Structural tests should reject:
  - path re-exports to deleted flat files
  - crate-root flattening of workbench and asset-editor specialists
  - crate-root ownership of `EditorModule`
  - UI host logic drifting back into `core/`
