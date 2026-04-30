---
related_code:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_plugins/mod.rs
  - zircon_plugins/navigation/mod.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/scene/mod.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/module.rs
implementation_files:
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/platform/module.rs
  - zircon_plugins/mod.rs
  - zircon_plugins/registration.rs
  - zircon_plugins/navigation/module.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/module.rs
plan_sources:
  - user: 2026-04-20 implement the workspace hard cutover and standardize the result
  - .codex/plans/ZirconEngine 全仓结构硬切换与规范固化计划.md
  - docs/engine-architecture/workspace-ownership-cutover-map.md
tests:
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/root_surfaces.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
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
doc_type: module-detail
---

# Workspace Root Rules And Hard Cutover

## Purpose

This document defines the structural rules that stay true after the workspace hard cutover. It complements the owner map by stating what each root package may own, what root files may contain, and what must be deleted instead of bridged.

## Fixed Package Shape

- `zircon_app`
  - Owns entry profile choice, host bootstrap, and final handoff only.
  - Must get runtime builtins from `zircon_runtime::builtin_runtime_modules()`.
- `zircon_runtime`
  - Owns runtime absorption, runtime-facing contracts, and runtime-private subsystem internals.
  - Must not widen crate root or graphics root just to preserve stale callers.
  - Builtin runtime ordering remains runtime-owned through `zircon_runtime::builtin_runtime_modules()`, with `src/builtin/mod.rs` kept structural and the module list assembly living under `src/builtin/runtime_modules.rs`.
  - Subsystem roots such as `platform/` and `extensions/*/` must keep module/config/descriptor ownership below the root entry file.
- `zircon_editor`
  - Owns authoring state and editor host behavior only.
  - Must stay organized under `core/`, `scene/`, and `ui/`.
  - Crate-root `EditorModule` ownership belongs under `ui/host/module.rs`, not in `src/lib.rs`.

## Root File Rules

- `lib.rs`, `mod.rs`, `main.rs`, and equivalent root wiring files stay structural.
- Allowed content:
  - child module declarations
  - narrow curated re-exports
  - minimal entry wiring
- Forbidden content:
  - mixed-domain behavior
  - migration-only forwarding shells
  - deep renderer, layout, asset-session, or host orchestration logic
  - inline `EngineModule` implementations or descriptor-builder helpers in subsystem root `mod.rs` files when a child owner file exists

## Hard Cutover Rules

- Move every live caller to the new owner path in the same change that introduces it.
- Delete the superseded path immediately after the last intended caller moves.
- Do not keep migration-only `pub use`, alias modules, facade folders, or bridge layers.
- Empty or dead `compat`, `shim`, `bridge`, `legacy`, and similarly named directories are deletion candidates, not documentation of history.
- Structural breakage during the move is acceptable; compatibility glue is not.

## Public Surface Rules

- `zircon_editor` crate root keeps only high-level entry points; specialist asset-editor, workbench, viewport, and host types must come from their owner modules.
- `zircon_editor/src/lib.rs` may re-export `EditorModule`, but the `EngineModule` implementation and module descriptor wiring live under `zircon_editor/src/ui/host/module.rs`.
- `EditorState` is not a crate-root entry point; callers use `zircon_editor::ui::workbench::state::EditorState` so the workbench state owner remains visible.
- `zircon_runtime` crate root and `graphics` root expose only stable runtime-facing contracts. Deep frame assembly, renderer construction helpers, and overlay seams stay internal.
- `zircon_runtime::builtin` keeps `builtin_runtime_modules()` as the public entry, but `src/builtin/mod.rs` must stay structural and delegate the actual module list assembly to a child owner file such as `runtime_modules.rs`.
- `zircon_runtime::platform` and `zircon_plugins::{navigation,net,particles,sound,texture}` roots may re-export their public module/config/service types, but the actual `EngineModule` implementation and descriptor wiring must live in child owner files such as `module.rs`, `service_types.rs`, or `registration.rs`.
- Runtime production `.ui.toml` resources live under crate `assets/`, never under `src/`.

## Required Follow-Through

- Update docs and skills to the converged path immediately after the move.
- Structural tests must search for removed owners, path-based forwarding, and root flattening regressions.
- Remaining validation failures may be reported only when they are unrelated active-workstream issues, not migration leftovers.
