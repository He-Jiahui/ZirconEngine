---
related_code:
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/physics.rs
  - zircon_runtime/src/core/diagnostics/animation.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
implementation_files:
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/core/diagnostics/snapshot.rs
  - zircon_runtime/src/core/diagnostics/render.rs
  - zircon_runtime/src/core/diagnostics/physics.rs
  - zircon_runtime/src/core/diagnostics/animation.rs
  - zircon_editor/src/ui/host/editor_manager_runtime_diagnostics.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views/runtime_diagnostics_view_descriptor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/runtime_diagnostics_body.ui.toml
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

`zircon_runtime::core::diagnostics` is the read-only inspection contract for editor and tooling code. It collects runtime health from the existing manager contracts instead of giving editor panes direct ownership of renderer, physics, or animation internals.

The first cut covers three runtime systems:

- render framework stats and virtual-geometry debug snapshot availability
- physics backend name, backend status, and fixed-step rate
- animation playback settings

## Runtime Boundary

`collect_runtime_diagnostics(&CoreHandle)` resolves the existing manager services through `core::manager`:

- `resolve_render_framework`
- `resolve_physics_manager`
- `resolve_animation_manager`

Every subsystem section reports `available: false` plus a string error when the service is missing or unavailable. This keeps editor panes safe in partial runtimes, tests, and startup phases where optional modules have not been registered yet.

The diagnostics snapshot intentionally stores copied data only. It does not expose manager handles, mutable state, backend objects, render resources, physics worlds, or animation assets to the editor layer.

## Editor Pane Boundary

`EditorManager::runtime_diagnostics()` exposes the runtime snapshot to editor host code. The built-in `editor.runtime_diagnostics` activity pane is registered as a bottom-right drawer view and uses the same `.ui.toml` pane-template path as console, inspector, hierarchy, and animation panes:

- descriptor id: `editor.runtime_diagnostics`
- body document id: `pane.runtime.diagnostics.body`
- payload kind: `RuntimeDiagnosticsV1`
- route namespace: `Diagnostics`
- body component: `RuntimeDiagnosticsPaneBody`

The pane payload builder converts `RuntimeDiagnosticsSnapshot` into stable presentation strings. Missing diagnostics fall back to unavailable text instead of panicking.

## Validation Notes

The runtime tests prove both missing-service and fully resolved manager paths. The editor tests prove descriptor metadata, template document registration, binding namespace projection, payload building, and TOML runtime attribute projection.

Workspace-wide validation is still required before claiming a complete green workspace because this repository currently has many unrelated in-flight editor and runtime changes.
