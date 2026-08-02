---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/parser.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/live_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/manifest.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions/feature.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions/plugin.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/native_backend.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/types.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/dependencies.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/selection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/status.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/transitions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/labels.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/tests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/parser.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/action_ids/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/live_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/manifest.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions/feature.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/project_actions/plugin.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/native_backend.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/live_host/types.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/dependencies.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/selection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/status.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/tests.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/project_policy/transitions.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/report.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/pane_data/view_rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/features.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/labels.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_projection/rows/tests.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app module-plugin host-actions ownership scan
  - app module-plugin host-actions subowner ownership scan
  - app module-plugin project-actions subowner ownership scan
  - app module-plugin live-host ownership scan
  - app module-plugin live-host subowner ownership scan
  - app module-plugin action-id/project-policy ownership scan
  - app module-plugin action-id parser ownership scan
  - app module-plugin action-id test ownership scan
  - app module-plugin project-policy subowner ownership scan
  - app module-plugin projection rows ownership scan
  - app module-plugin projection row subowner ownership scan
  - app module-plugin projection pane-data ownership scan
  - app module-plugin projection pane-data subowner ownership scan
  - app retained-host owner visibility compile-boundary scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Module Plugin Host Actions

`app/module_plugin_actions.rs` is the structural Plugin Manager action-family entry. It declares the action id, host action, live host, and project policy children, then re-exports only the live-host backend trait required by retained-host startup.

`app/module_plugin_actions/host_actions.rs` owns Plugin Manager callback action routing. It parses stable action ids, loads the active project-manifest context through its manifest child, routes parsed actions to project or live-host children, saves the context after the action, and emits user-facing status-line messages.

`app/module_plugin_projection.rs` is the structural pane projection entry for the same feature surface. `app/module_plugin_projection/pane_data.rs` owns the `ModulePluginsPaneViewData` assembly shell while its children own active project status lookup, diagnostics aggregation, and row DTO construction.

## Live Host Backend

`app/module_plugin_actions/live_host.rs` is the structural live-host action entry. `live_host/types.rs` owns `ModulePluginLiveHostBackend`, command/request/outcome DTOs, and command labels. `live_host/native_backend.rs` owns the `NativePluginLiveHost` and `Arc<NativePluginLiveHost>` adapters. `live_host/dispatch.rs` owns unload/hot-reload dispatch validation and success-message formatting. `live_host/tests.rs` owns unavailable-backend, recording-backend, and missing-native-package regressions.

The parent action module imports only the dispatch entry, success-message formatter, command enum, and exported backend trait needed by `RetainedEditorHost`. Live host request/outcome payload details and backend-specific regressions stay in the child module.

## Action Ids

`app/module_plugin_actions/action_ids.rs` is the structural action-id entry. It owns the `ModulePluginAction` declaration and re-exports the parser for host action routing. `action_ids/parser.rs` owns the stable `workbench.plugin.*` action-id grammar. It maps enable/disable, packaging cycle, target-mode cycle, feature enable/disable, dependency enablement, unload, and hot-reload ids into `ModulePluginAction`. Feature ids may contain dots, so the parser splits only the first segment after the feature action prefix as the plugin id and preserves the remainder as the feature id. `action_ids/tests.rs` owns parser regressions.

The parent action module consumes only `parse_module_plugin_action(...)` and the resulting enum. Action-id regression tests stay with the parser family so action grammar changes are reviewed next to their cases without expanding the production parser file.

## Host Actions

`app/module_plugin_actions/host_actions.rs` owns the retained-host action dispatcher. It parses action ids, resolves the manifest context through `host_actions/manifest.rs`, routes manifest-backed project mutations to `host_actions/project_actions.rs`, routes unload/hot-reload to `host_actions/live_actions.rs`, updates the status line, and marks layout dirty after successful mutations.

`host_actions/manifest.rs` owns active project root resolution, `zircon-project.toml` path construction, `ProjectManifest` loading, and manifest save.

`host_actions/project_actions.rs` owns manifest-backed project action dispatch and live-action misroute protection. `host_actions/project_actions/plugin.rs` owns native-aware project plugin enablement, packaging, and target-mode mutations plus status messages. `host_actions/project_actions/feature.rs` owns native-aware project plugin feature enablement, dependency enablement, and dependency status messages.

`host_actions/live_actions.rs` owns live plugin backend unload/hot-reload dispatch and success-message conversion.

Keeping this effect layer out of the root action module separates callback mutation from parser grammar, deterministic policy helpers, and live-host backend DTOs.

## Project Policy

`app/module_plugin_actions/project_policy.rs` is the structural deterministic project policy entry. `project_policy/selection.rs` resolves the current native-aware plugin selection from the completed manifest. `project_policy/transitions.rs` cycles packaging through `library-embed -> native-dynamic -> source-template -> library-embed` and cycles target-mode presets. `project_policy/status.rs` formats packaging/target-mode status labels. `project_policy/dependencies.rs` formats feature dependency enablement messages with enabled plugins/features plus diagnostics. `project_policy/tests.rs` owns policy regressions.

The parent action module delegates policy transitions and status formatting to this child module. Direct manifest mutation and `EditorManager` calls remain in `dispatch_module_plugin_action(...)` because they are the callback action side effect boundary.

## Pane Row Projection

`app/module_plugin_projection.rs` is the structural pane projection entry. `app/module_plugin_projection/pane_data.rs` keeps `RetainedEditorHost::module_plugins_pane_data(...)` as a small assembly method that loads the status report, maps rows, and returns the Plugin Manager pane model.

`app/module_plugin_projection/pane_data/report.rs` reads the active project path, loads `zircon-project.toml` when available, asks `EditorManager` for native-aware plugin status, falls back to the builtin plugin catalog when the manifest is unavailable, and combines lookup diagnostics with report diagnostics.

`app/module_plugin_projection/pane_data/view_rows.rs` converts `EditorPluginStatusReport` into `ModulePluginStatusViewData` rows. It owns row DTO literals, capability/diagnostic text joins, per-row action ids, and action labels.

`app/module_plugin_projection/rows.rs` is the structural row-helper entry for that pane. `rows/features.rs` owns optional feature action labels/ids and feature dependency summaries. `rows/labels.rs` owns primary action labels/ids plus target-mode and packaging labels. `rows/manifest.rs` owns the fallback project manifest. `rows/tests.rs` owns the projection helper regressions. The root projection imports these helpers only for row construction so project-status IO and deterministic row text do not share one file.

## Boundary Rules

- Keep action parse/route/status finalization and `RetainedEditorHost::dispatch_module_plugin_action(...)` in `app/module_plugin_actions/host_actions.rs`.
- Keep active project root resolution plus project manifest load/save in `app/module_plugin_actions/host_actions/manifest.rs`.
- Keep manifest-backed project action dispatch in `app/module_plugin_actions/host_actions/project_actions.rs`.
- Keep `EditorManager` native-aware project plugin enablement, packaging, and target-mode mutation calls in `app/module_plugin_actions/host_actions/project_actions/plugin.rs`.
- Keep native-aware project plugin feature and dependency mutation calls in `app/module_plugin_actions/host_actions/project_actions/feature.rs`.
- Keep live-host unload/hot-reload action dispatch in `app/module_plugin_actions/host_actions/live_actions.rs`.
- Keep the `ModulePluginAction` declaration in `app/module_plugin_actions/action_ids.rs`.
- Keep `workbench.plugin.*` action-id grammar in `app/module_plugin_actions/action_ids/parser.rs` and parser regressions in `app/module_plugin_actions/action_ids/tests.rs`.
- Keep `app/module_plugin_actions/project_policy.rs` as the structural project-policy entry.
- Keep current project-selection lookup in `app/module_plugin_actions/project_policy/selection.rs`.
- Keep deterministic packaging/target-mode cycling in `app/module_plugin_actions/project_policy/transitions.rs`.
- Keep policy status labels in `app/module_plugin_actions/project_policy/status.rs`.
- Keep dependency enablement message formatting in `app/module_plugin_actions/project_policy/dependencies.rs`.
- Keep project-policy regressions in `app/module_plugin_actions/project_policy/tests.rs`.
- Keep `app/module_plugin_actions/live_host.rs` as the structural live-host action entry.
- Keep live-host command/request/outcome DTOs and backend trait in `app/module_plugin_actions/live_host/types.rs`.
- Keep runtime native adapter implementations in `app/module_plugin_actions/live_host/native_backend.rs`.
- Keep unload/hot-reload dispatch validation and success-message formatting in `app/module_plugin_actions/live_host/dispatch.rs`.
- Keep live-host regressions in `app/module_plugin_actions/live_host/tests.rs`.
- Keep `app/module_plugin_projection.rs` as the structural projection entry.
- Keep Plugin Manager pane orchestration in `app/module_plugin_projection/pane_data.rs`; do not add pane data mapping back to action routing.
- Keep the published plugin status report lookup in `app/module_plugin_projection/pane_data/report.rs`.
- Keep `ModulePluginStatusViewData` row DTO construction in `app/module_plugin_projection/pane_data/view_rows.rs`.
- Keep `app/module_plugin_projection/rows.rs` as the structural row-helper entry.
- Keep optional feature summaries and feature action ids in `app/module_plugin_projection/rows/features.rs`.
- Keep primary action ids plus target/packaging labels in `app/module_plugin_projection/rows/labels.rs`.
- Keep row-helper regressions in `app/module_plugin_projection/rows/tests.rs`.
- Keep lifecycle recompute orchestration in `app/host_lifecycle.rs`; do not add plugin action or projection helper ownership back to lifecycle code.

## Validation Notes

The 2026-06-19 host-actions split reduced `module_plugin_actions.rs` from 160 lines to 5 lines. `module_plugin_actions/host_actions.rs` is 159 lines and owns retained-host Plugin Manager action routing, active project manifest load/save, `EditorManager` policy mutation calls, live-host unload/hot-reload dispatch, status-line updates, and layout invalidation. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin host-actions ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 host-actions subowner split reduced `module_plugin_actions/host_actions.rs` from 159 lines to a 49-line dispatcher. `host_actions/manifest.rs` is 27 lines and owns active project root/manifest context load-save. `host_actions/live_actions.rs` is 22 lines and owns live backend command dispatch. `host_actions/project_actions.rs` is 120 lines and owns manifest-backed `EditorManager` plugin mutations and status messages.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin host-actions subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test/check processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 live-host split reduced `module_plugin_actions.rs` from 768 lines to 528 lines. `module_plugin_actions/live_host.rs` is 253 lines and owns the live plugin host backend contract, native adapter, unload/hot-reload dispatch validation, success-message formatting, unavailable-backend regression, recording backend regression, and missing native package regression.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin live-host ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. The first compile pass caught a visibility mismatch on the exported backend trait; widening the trait to the app boundary fixed it, and the second compile pass succeeded with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 action-id/project-policy split reduced `module_plugin_actions.rs` to 163 lines. `module_plugin_actions/action_ids.rs` is 192 lines and owns `ModulePluginAction`, the stable action-id parser, and parser regressions. `module_plugin_actions/project_policy.rs` is 185 lines and owns project policy cycling, current selection lookup, status label formatting, dependency enablement message formatting, and policy helper regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin action-id/project-policy ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 projection row split originally placed fallback project-manifest construction under the row helpers. That fallback has since been retired: current projection reads the published plugin status report in `module_plugin_projection/pane_data/report.rs`, while `rows.rs` only mounts feature and label helpers plus their regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection rows ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 projection pane-data split reduced `module_plugin_projection.rs` from 119 lines to 2 lines. `module_plugin_projection/pane_data.rs` is 116 lines and owns `RetainedEditorHost::module_plugins_pane_data(...)`, active project manifest lookup, native plugin status report selection, builtin fallback diagnostics, `ModulePluginsPaneViewData` construction, and row helper consumption. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection pane-data ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh full `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` was not claimed because concurrent `zircon_runtime` Cargo jobs were active. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 projection pane-data report/view-row subowner split reduced `module_plugin_projection/pane_data.rs` from 113 lines to a 23-line assembly owner. `pane_data/report.rs` is 45 lines and owns status lookup/fallback diagnostics. `pane_data/view_rows.rs` is 66 lines and owns `ModulePluginStatusViewData` row construction. The focused check also caught that `action_ids/parser.rs` had remained too private for the root `action_ids.rs` re-export after the parser split, so `parse_module_plugin_action(...)` was widened only to `pub(in crate::ui::retained_host::app::module_plugin_actions)`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection pane-data subowner ownership scan, an app retained-host owner visibility compile-boundary scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 projection row subowner split reduced `module_plugin_projection/rows.rs` from 242 lines to 12 lines. `rows/features.rs` is 92 lines and owns optional feature summaries/action ids. `rows/labels.rs` is 46 lines and owns primary action ids plus target/packaging labels. `rows/manifest.rs` is 10 lines and owns fallback manifest construction. `rows/tests.rs` is 98 lines and owns row-helper regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin projection row subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 live-host subowner split reduced `module_plugin_actions/live_host.rs` to a 12-line structural entry. `live_host/types.rs` is 37 lines and owns command/request/outcome DTOs plus backend trait. `live_host/native_backend.rs` is 32 lines and owns the native live-host adapters. `live_host/dispatch.rs` is 36 lines and owns dispatch validation plus success messages. `live_host/tests.rs` is 132 lines and owns live-host regressions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin live-host subowner ownership scan, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`. The first check exposed re-export visibility that was too narrow after moving dispatch helpers; widening those helpers to the module-plugin action family fixed it, and the second check passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 project-policy subowner split reduced `module_plugin_actions/project_policy.rs` from 173 lines to 10 lines. New child owners are `project_policy/dependencies.rs` (35 lines), `selection.rs` (15 lines), `transitions.rs` (24 lines), `status.rs` (23 lines), and `tests.rs` (76 lines). The root no longer owns runtime policy imports, policy logic, or inline tests.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin project-policy subowner ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). A fresh `cargo check` was deferred for this slice because separate runtime Cargo checks are currently active in this workspace; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 owner-split compile correction widened `project_policy/dependencies.rs`, `selection.rs`, `status.rs`, and `transitions.rs` helpers to `pub(in crate::ui::retained_host::app::module_plugin_actions)`. This keeps policy helpers visible to sibling `host_actions/project_actions.rs` after the host-action split while preserving the module-plugin action family boundary. After formatting, `module_plugin_actions/project_policy.rs` is a 12-line structural entry, and the child owners remain responsible for dependency messages, current selection lookup, status labels, and deterministic packaging/target-mode transitions.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin project-policy ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 action-id parser subowner split reduced `module_plugin_actions/action_ids.rs` from 108 lines to a 32-line structural entry. `action_ids/parser.rs` is 79 lines and owns the `workbench.plugin.*` parser chain plus feature-action splitting.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin action-id parser ownership scan, and scoped `git diff --check` (only the existing CRLF conversion warning appeared). Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 project-actions subowner split reduced `module_plugin_actions/host_actions/project_actions.rs` from 120 lines to a 49-line manifest-action dispatcher. `project_actions/plugin.rs` is 80 lines and owns plugin enablement, packaging, and target-mode mutations. `project_actions/feature.rs` is 50 lines and owns feature enablement plus dependency enablement mutations.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app module-plugin project-actions subowner ownership scan, and scoped `git diff --check`. Focused `cargo check` was not rerun for this slice because independent Cargo/rustc processes were active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
