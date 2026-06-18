---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app module-plugin host-actions ownership scan
  - app module-plugin live-host ownership scan
  - app module-plugin action-id/project-policy ownership scan
  - app module-plugin projection rows ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Module Plugin Host Actions

`app/module_plugin_actions.rs` is the structural Plugin Manager action-family entry. It declares the action id, host action, live host, and project policy children, then re-exports only the live-host backend trait required by retained-host startup.

`app/module_plugin_actions/host_actions.rs` owns Plugin Manager callback action routing. It loads and saves the active `zircon-project.toml`, dispatches parsed actions to `EditorManager` or the live-host backend, and emits user-facing status-line messages.

`app/module_plugin_projection.rs` owns the pane data projection for the same feature surface: active project plugin status lookup, diagnostics aggregation, and `ModulePluginsPaneViewData` construction.

## Live Host Backend

`app/module_plugin_actions/live_host.rs` owns the editor-side live plugin host backend contract and runtime adapter. It defines `ModulePluginLiveHostBackend`, unload/hot-reload command payloads, dispatch validation, success-message formatting, and the `NativePluginLiveHost` adapter used by the retained editor host.

The parent action module imports only the dispatch entry, success-message formatter, command enum, and exported backend trait needed by `RetainedEditorHost`. Live host request/outcome payload details and backend-specific regressions stay in the child module.

## Action Ids

`app/module_plugin_actions/action_ids.rs` owns the stable `workbench.plugin.*` action-id grammar. It maps enable/disable, packaging cycle, target-mode cycle, feature enable/disable, dependency enablement, unload, and hot-reload ids into `ModulePluginAction`. Feature ids may contain dots, so the parser splits only the first segment after the feature action prefix as the plugin id and preserves the remainder as the feature id.

The parent action module consumes only `parse_module_plugin_action(...)` and the resulting enum. Action-id regression tests stay with the parser so action grammar changes are reviewed next to their cases.

## Host Actions

`app/module_plugin_actions/host_actions.rs` owns the retained-host action side effects. It parses action ids, resolves the active project root, loads and saves the project manifest, calls `EditorManager` for native-aware enablement, packaging, target-mode, feature, and dependency actions, dispatches unload/hot-reload to the live plugin backend, updates the status line, and marks layout dirty after successful mutations.

Keeping this effect layer out of the root action module separates callback mutation from parser grammar, deterministic policy helpers, and live-host backend DTOs.

## Project Policy

`app/module_plugin_actions/project_policy.rs` owns deterministic project plugin policy helpers. It resolves the current native-aware plugin selection from the completed manifest, cycles packaging through `library-embed -> native-dynamic -> source-template -> library-embed`, cycles target-mode presets, formats packaging/target-mode status labels, and formats feature dependency enablement messages with enabled plugins/features plus diagnostics.

The parent action module delegates policy transitions and status formatting to this child module. Direct manifest mutation and `EditorManager` calls remain in `dispatch_module_plugin_action(...)` because they are the callback action side effect boundary.

## Pane Row Projection

`app/module_plugin_projection.rs` reads the active project path, loads `zircon-project.toml` when available, asks `EditorManager` for native-aware plugin status, falls back to the builtin plugin catalog when the manifest is unavailable, and builds the Plugin Manager pane model.

`app/module_plugin_projection/rows.rs` owns pure row presentation helpers for that pane. It formats primary action labels/ids, packaging and target-mode cycle labels, optional feature action labels/ids, feature dependency summaries, target-mode labels, packaging labels, the fallback project manifest, and the projection helper regressions. The root projection imports these helpers only for row construction so project-status IO and deterministic row text do not share one file.

## Boundary Rules

- Keep project manifest load/save, `EditorManager` mutation calls, live-host action dispatch, and `RetainedEditorHost::dispatch_module_plugin_action(...)` in `app/module_plugin_actions/host_actions.rs`.
- Keep `workbench.plugin.*` action-id grammar and parser regressions in `app/module_plugin_actions/action_ids.rs`.
- Keep deterministic packaging/target-mode cycling, current project-selection lookup, policy status labels, and dependency enablement message formatting in `app/module_plugin_actions/project_policy.rs`.
- Keep live host request/outcome DTOs, backend trait, runtime native adapter, unload/hot-reload dispatch validation, and live-host status message formatting in `app/module_plugin_actions/live_host.rs`.
- Keep Plugin Manager pane status lookup, diagnostics aggregation, and view-model construction in `app/module_plugin_projection.rs`; do not add pane data mapping back to action routing.
- Keep Plugin Manager row labels, action-id construction, feature summaries, target/packaging labels, fallback manifest construction, and helper regressions in `app/module_plugin_projection/rows.rs`.
- Keep lifecycle recompute orchestration in `app/host_lifecycle.rs`; do not add plugin action or projection helper ownership back to lifecycle code.

## Validation Notes

The 2026-06-19 host-actions split reduced `module_plugin_actions.rs` from 160 lines to 5 lines. `module_plugin_actions/host_actions.rs` is 159 lines and owns retained-host Plugin Manager action routing, active project manifest load/save, `EditorManager` policy mutation calls, live-host unload/hot-reload dispatch, status-line updates, and layout invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin host-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 live-host split reduced `module_plugin_actions.rs` from 768 lines to 528 lines. `module_plugin_actions/live_host.rs` is 253 lines and owns the live plugin host backend contract, native adapter, unload/hot-reload dispatch validation, success-message formatting, unavailable-backend regression, recording backend regression, and missing native package regression.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin live-host ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. The first compile pass caught a visibility mismatch on the exported backend trait; widening the trait to the app boundary fixed it, and the second compile pass succeeded with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action-id/project-policy split reduced `module_plugin_actions.rs` to 163 lines. `module_plugin_actions/action_ids.rs` is 192 lines and owns `ModulePluginAction`, the stable action-id parser, and parser regressions. `module_plugin_actions/project_policy.rs` is 185 lines and owns project policy cycling, current selection lookup, status label formatting, dependency enablement message formatting, and policy helper regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin action-id/project-policy ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 projection row split reduced `module_plugin_projection.rs` from 349 lines to 114 lines. `module_plugin_projection/rows.rs` is 242 lines and owns row/action label helpers, optional feature summaries, target/packaging labels, fallback project manifest construction, and the moved projection helper regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection rows ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
