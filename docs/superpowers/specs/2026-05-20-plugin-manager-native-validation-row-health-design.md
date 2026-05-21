---
title: Plugin Manager Native Validation Row Health Design
date: 2026-05-20
status: approved-for-spec-review
scope: design
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/status/native_load_state.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/reports/editor_plugin_status.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/module_plugins.rs
plan_sources:
  - docs/superpowers/specs/2026-05-19-native-dynamic-v3-hardening-design.md
  - docs/superpowers/plans/2026-05-20-native-dynamic-v3-hardening.md
  - docs/zircon_runtime/plugin/native_plugin_loader/index.md
  - docs/editor-and-tooling/editor-host-minimal-plugin-loading.md
  - docs/engine-architecture/runtime-editor-pluginized-export.md
tests:
  - cargo test -p zircon_editor --lib native_plugin_status_report --locked --jobs 1
  - cargo test -p zircon_editor --lib module_plugins --locked --jobs 1
doc_type: design-spec
---

# Plugin Manager Native Validation Row Health Design

## Goal

Expose NativeDynamic ABI v3 behavior validation health in the existing Plugin Manager row summary so users can immediately tell whether a native plugin is clean, degraded, or invalid after discovery/load.

The slice is intentionally narrow. Runtime remains the source of validation truth, and editor UI only projects a row-level summary plus the existing diagnostics text.

## User-Approved Scope

The selected approach is row health only:

- Add a per-plugin native validation summary to existing Plugin Manager row data.
- Show `Native: Clean`, `Native: Degraded`, or `Native: Invalid` in the row metadata when a native validation report exists.
- Preserve the existing diagnostics stream for detailed validation messages.
- Avoid expanding the ABI, adding callbacks, changing Runtime Diagnostics, changing export UX, adding expand/collapse UI, or doing a broad Plugin Manager visual redesign.

## Architecture

`zircon_runtime::plugin::native_plugin_loader` owns validation. It already produces `NativePluginBehaviorValidationReport` and `NativePluginBehaviorHealth` from copied ABI metadata without invoking plugin behavior callbacks.

`zircon_editor` must not reimplement validation rules. The editor consumes runtime-owned load reports through `NativePluginLoadReport` and converts the report into display data owned by `EditorPluginStatus` and the existing Module Plugins pane view data.

The owner boundaries are:

- Runtime loader: computes validation health and diagnostic text.
- Editor status projection: aggregates runtime/editor native health into one row health label.
- Plugin Manager pane: renders the label in the existing row metadata line.

No Rust trait objects, callback pointers, runtime world objects, editor state, or new ABI structs cross this boundary.

## Health Aggregation

A plugin row can have runtime behavior validation, editor behavior validation, both, or neither. The Plugin Manager row should show one aggregate health label by using worst-severity ordering:

1. `Invalid`
2. `Degraded`
3. `Clean`

If no native behavior validation report exists for the plugin, the native validation label is empty and the row metadata stays unchanged.

If a plugin has both runtime and editor reports and one is degraded while the other is clean, the row shows `Native: Degraded`. If either report is invalid, the row shows `Native: Invalid`.

The label is display-only. It does not enable or disable actions, does not change `load_state`, and does not change packaging or target-mode behavior.

## Data Flow

`EditorManager::native_plugin_status_report(...)` already loads/discovers native packages and merges native package diagnostics into `EditorPluginStatusReport`. This slice adds a native validation display field to the editor status data path.

The expected flow is:

1. `NativePluginLoader.load_discovered_all(...)` returns `NativePluginLoadReport`.
2. Editor status projection finds the loaded plugin entries for each native package id.
3. Projection reads each loaded plugin's runtime/editor behavior validation report health.
4. Projection stores a display string such as `Native: Clean` on `EditorPluginStatus`.
5. `RetainedEditorHost::module_plugins_pane_data(...)` copies the string into `ModulePluginStatusViewData`.
6. `pane_payload_builders/module_plugins.rs` copies the string into `ModulePluginStatusPayload`.
7. `pane_data_conversion/module_plugins.rs` includes the string in the row metadata text.

This keeps the Plugin Manager presentation deterministic and avoids parsing diagnostics to infer health.

## UI Behavior

The existing Plugin Manager row metadata currently combines package source, load state, packaging, and target modes. After this slice, native rows with validation reports include the validation segment in the same metadata line.

Example row metadata:

```text
native | loaded with diagnostics | Native: Degraded | NativeDynamic | EditorHost
```

Clean native plugin example:

```text
native | loaded | Native: Clean | NativeDynamic | EditorHost
```

Builtin-only plugin rows or native manifest-only rows without behavior validation continue to omit the native validation segment.

Detailed messages stay in the existing diagnostics label. For example, an invalid plugin can show `Native: Invalid` in metadata and keep the runtime-provided validation diagnostic in the warning-toned diagnostics text.

## Error Handling

Missing project manifest behavior remains unchanged. The pane still falls back to builtin catalog status and reports the pane-level fallback diagnostic.

Native load failures remain represented by `load_state` and existing diagnostics. Validation health is only shown when a behavior validation report exists; the row must not fabricate `Invalid` from generic load failures.

Diagnostics are still sorted and deduplicated in the existing native status report path. This slice does not change runtime diagnostic wording.

## Tests

Focused editor tests should cover:

- Clean NativeDynamic v3 runtime/editor reports project `Native: Clean` into Plugin Manager status/payload/row metadata.
- A degraded validation report projects `Native: Degraded` without changing `load_state`.
- An invalid validation report projects `Native: Invalid` and preserves the validation diagnostic text.
- Non-native or native rows without behavior validation omit the native validation metadata segment.

Runtime native validation tests remain the source of truth for health classification rules. This slice should not duplicate every ABI validation case in editor tests.

Suggested scoped validation commands:

```powershell
cargo test -p zircon_editor --lib native_plugin_status_report --locked --jobs 1
cargo test -p zircon_editor --lib module_plugins --locked --jobs 1
```

## Reference Evidence

Repository-local evidence leads this design:

- `docs/zircon_runtime/plugin/native_plugin_loader/index.md` defines the runtime-owned validation report, health states, and report flow.
- `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md` defines Plugin Manager as the built-in minimal-host view for plugin status, actions, and diagnostics.
- Existing `NativePluginLoadReport` APIs already scope diagnostics by runtime/editor module kind.

External precedent supports keeping the UI as a compact status summary with diagnostics preserved elsewhere:

- Godot editor plugin/export paths aggregate plugin warnings into editor-facing warning text rather than making plugin warnings control load-state semantics.
- Slint LSP preview tracks a diagnostic summary separately from detailed diagnostics, matching the split between a row health label and existing diagnostics text.
- Bevy plugin groups keep plugin enable/disable state separate from diagnostic/logging signals, supporting the decision not to overload Plugin Manager actions or `load_state`.

## Non-Goals

This design does not include:

- ABI v4 or ABI v3 struct changes.
- New native callbacks or callback invocation during validation.
- Typed command/event manifest parsing.
- Runtime Diagnostics pane integration.
- Export pane validation summaries.
- Expandable Plugin Manager details.
- New Plugin Manager actions.
- Broad retained-host styling or layout redesign.
- Changes to Hub, material UI, profiling, ECS, ZrVM, asset, render, or input work.

## Acceptance Criteria

The slice is accepted when:

- Native plugin rows show the correct aggregate validation label when validation reports exist.
- Rows without validation reports do not show a fabricated validation label.
- Detailed validation diagnostics still appear in the existing row diagnostics text.
- Existing Plugin Manager actions and load-state labels are unchanged.
- Focused editor tests pass for native status projection and module plugin row projection.
