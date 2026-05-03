---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/scene/world/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/tests/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/graphics_surface/mod.rs
  - zircon_runtime/src/tests/scene_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/extensions/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/slint_host/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/mod.rs
implementation_files:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/tests/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/graphics_surface/mod.rs
  - zircon_runtime/src/tests/scene_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/extensions/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/slint_host/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
plan_sources:
  - user: 2026-04-20 implement the workspace hard cutover and standardize the result
  - .codex/plans/ZirconEngine 全仓结构硬切换与规范固化计划.md
  - .codex/plans/Zircon Editor UI 回迁 + 树形 TOML Cutover 实施计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_app/src/entry/tests/mod.rs
  - zircon_runtime/src/tests/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/builtin_modules.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/graphics_surface/mod.rs
  - zircon_runtime/src/tests/scene_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_runtime/src/tests/ui_boundary/module_absorption.rs
  - zircon_runtime/src/tests/ui_boundary/template_namespaces.rs
  - zircon_runtime/src/tests/ui_boundary/surface_dispatch_namespaces.rs
  - zircon_runtime/src/tests/ui_boundary/binding_event_namespaces.rs
  - zircon_runtime/src/tests/ui_boundary/runtime_host.rs
  - zircon_runtime/src/tests/ui_boundary/assets.rs
  - zircon_runtime/src/tests/extensions/mod.rs
  - zircon_runtime/src/tests/extensions/absorption_surface.rs
  - zircon_runtime/src/tests/extensions/manager_handles.rs
  - zircon_runtime/src/tests/extensions/root_entries.rs
  - zircon_runtime/src/scene/tests/mod.rs
  - zircon_runtime/src/ui/tests/mod.rs
  - zircon_editor/src/tests/mod.rs
doc_type: module-detail
---

# Workspace Ownership Cutover Map

## Purpose

This document is the cutover authority for the current workspace refactor. Every move, delete, or root-surface contraction in this task uses the owner map below. If a file or type still points at an old owner after the cutover, that is migration debt, not an acceptable compatibility surface.

## Fixed Root Packages

- `zircon_app`
  - Owns process entry, profile selection, builtin module assembly, and loop handoff only.
  - Must not own runtime builtin ordering details other than choosing whether `zircon_editor::EditorModule` is appended for editor mode.
- `zircon_runtime`
  - Owns runtime absorption, runtime-facing public module surfaces, and the internal `core/{runtime,framework,manager,math,resource}` spine.
  - Must not leak editor authoring state or migration-only graphics/runtime helper seams from crate root.
- `zircon_editor`
  - Owns editor authoring behavior only.
  - Must stay organized under `core/`, `scene/`, and `ui/`, with crate root acting as a narrow entry surface instead of a flattened implementation umbrella.

## Owner Map

### `zircon_app`

- Old owner: app-side knowledge of runtime builtin ordering
- New owner: `zircon_runtime::builtin_runtime_modules()`
- App responsibility that remains: append `EditorModule` for editor profile and start the selected host path

### `zircon_runtime`

- Old owner: crate root flattened exports for internal graphics/runtime seams and plugin/native/export DTOs
- New owner: direct submodule paths plus crate-private visibility where appropriate; plugin package manifests, project selections, export plans, native loader/ABI types, runtime extension registries, and runtime plugin catalogs are imported through `zircon_runtime::plugin`
- Old owner: builtin module list implementation living directly in `src/builtin/mod.rs`
- New owner: `src/builtin/runtime_modules.rs`, with `src/builtin/mod.rs` reduced to a structural entry that only re-exports `builtin_runtime_modules()`
- Old owner: production runtime fixture resources under `src/ui/runtime_ui/fixtures`
- New owner: crate assets under `zircon_runtime/assets/ui/runtime/fixtures`
- Old owner: one oversized root test umbrella in `src/tests/mod.rs`
- New owner: grouped root test tree with domain buckets such as `runtime_absorption`, `graphics_surface`, `scene_boundary`, `ui_boundary`, and `extensions`
- Old owner: migration-smell directories such as `graphics/compat` or service/helper roots that exist only to preserve old paths
- New owner: either explicit runtime-owned subtree names with real responsibilities, or deletion if they are only transitional shells

### `zircon_editor`

- Old owner: `zircon_editor/src/lib.rs` as a flattened public surface for host, workbench, asset editor, viewport, and binding specialists
- New owner: direct `core`, `scene`, and `ui` subtrees with minimal curated crate-root exports only
- Old owner: crate-root `EditorState` re-export
- New owner: `zircon_editor::ui::workbench::state::EditorState`
- Old owner: `core` holding UI host, workbench, layout, window, or asset-editor session implementation
- New owner: `ui/host` and `ui/workbench`
- Old owner: `ui/slint_host` mixing shell glue with business ownership
- New owner: `ui/slint_host` only for Slint/native-window/platform glue; business state stays in `ui/host`, `ui/workbench`, and `ui/asset_editor`
- Old owner: `ui/asset_editor` flat files coexisting with same-domain folder-backed modules
- New owner: folder-backed `binding/`, `preview/`, `session/`, `source/`, `style/`, and `tree/` subtrees only

## Root Surface Rules

- `lib.rs`, `mod.rs`, `main.rs`, and `binding.rs` stay structural.
- Root files may declare modules, re-export the curated public surface, and perform minimal entry wiring.
- Root files may not keep parsing, routing, mutation, orchestration, render assembly, or mixed-domain helpers after the cutover.
- Broken imports during the cutover are acceptable evidence that stale consumers still exist. Compatibility re-exports are not.

## Delete-On-Sight Migration Debt

- `pub use` chains whose only job is to preserve an old owner path
- `compat`, `shim`, `bridge`, or `legacy` folders/modules that do not own real behavior
- flat `ui/asset_editor/*.rs` duplicates once the folder-backed replacement is the authority
- production `.ui.toml` assets under any crate `src/` tree
- tests and docs that still assert old locations after the owner has moved

## Validation Expectations

- Structural guard tests must prove the new owners exist and old ones are gone.
- `zircon_app` validation must confirm runtime builtin ordering is runtime-owned.
- `zircon_runtime` validation must confirm graphics/runtime public-surface narrowing, runtime UI asset loading from crate assets, and grouped root test ownership.
- `zircon_runtime` validation must also confirm `src/builtin/mod.rs` stays structural while `builtin_runtime_modules()` continues to own runtime builtin ordering.
- `zircon_editor` validation must confirm crate-root narrowing, `ui/host` ownership, and the absence of flat+folder dual ownership inside `ui/asset_editor`.
