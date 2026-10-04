---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/animation.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
implementation_files:
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/mod.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/core/runtime/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/animation.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.zui
plan_sources:
  - user: 2026-04-25 开始执行 runtime diagnostics contract and editor diagnostics pane cut
  - assistant design: runtime inspection surface + editor debug pane recommendation
tests:
  - zircon_runtime/src/tests/runtime_diagnostics/mod.rs
  - zircon_editor/src/tests/host/pane_template_descriptor.rs
  - zircon_editor/src/tests/host/pane_presentation.rs
  - zircon_editor/src/tests/host/template_runtime/pane_body_documents.rs
  - zircon_editor/src/tests/host/template_runtime/pane_payload_projection.rs
  - cargo test -p zircon_runtime --locked --target-dir target/codex-runtime-diagnostics runtime_diagnostics --lib
  - cargo test -p zircon_editor --locked --target-dir target/codex-editor-diagnostics pane_template_descriptor --lib
  - cargo test -p zircon_editor --locked --target-dir target/codex-editor-diagnostics pane_presentation --lib
  - cargo test -p zircon_editor --locked --target-dir target/codex-editor-diagnostics pane_body_documents --lib
  - cargo test -p zircon_editor --locked --target-dir target/codex-editor-diagnostics pane_payload_projection --lib
doc_type: module-detail
---

# Runtime Diagnostics Contract

## Purpose

`zircon_runtime::core::runtime::diagnostics` owns the read-only DTO/store/projector contract for editor and tooling code. The top-level `zircon_runtime::runtime_diagnostics` facade collects runtime health from existing manager contracts instead of making the core diagnostics contract or editor panes own renderer, physics, or animation internals.

The first cut covers three runtime systems:

- render framework stats and virtual-geometry debug snapshot availability
- physics backend name, backend status, and fixed-step rate
- animation playback settings

## Runtime Boundary

`runtime_diagnostics::collect_runtime_diagnostics(&CoreHandle)` resolves the existing manager services through the facade-owned `core::manager` access layer:

- `render_framework_handle` plus `resolve_manager_service`
- `physics_manager_handle` plus `resolve_manager_service` when `physics-contracts` is enabled
- `animation_manager_handle` plus `resolve_manager_service`

Every subsystem section reports `available: false` plus a string error when the service is missing or unavailable. This keeps editor panes safe in partial runtimes, tests, and startup phases where optional modules have not been registered yet.

The diagnostics snapshot intentionally stores copied data only. It does not expose manager handles, mutable state, backend objects, render resources, physics worlds, or animation assets to the editor layer.

The core diagnostics module does not import `core::manager` or re-export either collector. Devtools
registry projection accepts an already-collected diagnostics snapshot; the top-level facade owns the
only orchestration entry points. The former core collector files and public paths are deleted rather
than retained as aliases.

## Editor Pane Boundary

`EditorManager::runtime_diagnostics()` exposes the runtime snapshot to editor host code. The built-in `editor.runtime_diagnostics` activity pane is registered as a bottom-right drawer view and uses the same `.zui` pane-template path as console, inspector, hierarchy, and animation panes:

- descriptor id: `editor.runtime_diagnostics`
- body document id: `pane.runtime.diagnostics.body`
- payload kind: `RuntimeDiagnosticsV1`
- route namespace: `Diagnostics`
- body component: `RuntimeDiagnosticsPaneBody`

The pane payload builder converts `RuntimeDiagnosticsSnapshot` into stable presentation strings. Missing diagnostics fall back to unavailable text instead of panicking.

## Validation Notes

The runtime tests prove both missing-service and fully resolved manager paths. The editor tests prove descriptor metadata, template document registration, binding namespace projection, payload building, and TOML runtime attribute projection.

Workspace-wide validation is still required before claiming a complete green workspace because this repository currently has many unrelated in-flight editor and runtime changes.
