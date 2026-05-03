# Editor Core Usable Loop Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Converge the `zircon_editor` workbench body so Plugin Manager and Build Export host panes open, refresh, diagnose, and project through one stable editor shell path.

**Architecture:** `zircon_editor` owns the editor host loop: builtin view descriptors, workbench model routing, authoring/host snapshots, shell presentation, Slint host contract projection, and host-level pane actions. `zircon_runtime` remains the owner of runtime world/service state and concrete plugin/export behavior; this plan only consumes runtime/editor-manager facades and does not implement plugin packages or export packaging internals.

**Tech Stack:** Rust, Cargo, Slint host contract data, editor UI TOML pane templates, `zircon_editor`, `zircon_runtime` facades, `target\codex-shared-a` validation target.

---

## Architecture Notes

- Owner boundary: `zircon_editor::ui::host` owns descriptor registration and `EditorManager`; `zircon_editor::ui::workbench` owns pane routing and view models; `zircon_editor::ui::layouts::windows::workbench_host_window` owns shell presentation DTOs; `zircon_editor::ui::slint_host` owns Slint adapter projection and callback dispatch only.
- Required abstractions: `ViewDescriptor`, `ViewContentKind`, `PanePayloadKind`, `PanePayload`, `ModulePluginsPaneViewData`, `BuildExportPaneViewData`, `PaneData.native_body`, `ShellPresentation`, host-contract `PaneData`, and stable action ids.
- Reference grounding: Fyrox and Godot keep plugin/export settings as editor host surfaces while concrete runtime/plugin/export implementations stay behind runtime or plugin-owned services. Slint remains a projection backend and does not own editor authoring state.
- Coordination boundary: the checkout is dirty and active sessions touch plugin/runtime/export internals. Preserve unrelated changes. Do not edit `zircon_plugins/**`, `zircon_runtime/src/plugin/**`, or export packaging internals unless a scoped editor compile failure proves a minimal import update is required.
- Branch policy: work in the existing `main` checkout. Do not create a worktree, feature branch, commit, or PR unless the user explicitly asks.

## Current Baseline

- `ViewContentKind::{ModulePlugins, BuildExport}` currently exists in `zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs`.
- `editor.module_plugins` and `editor.build_export_desktop` descriptor mappings currently exist in `descriptor_content_kind.rs`, `view_menu.rs`, `name_mapping.rs`, and builtin layout files.
- `ModulePluginsPaneViewData`, `BuildExportPaneViewData`, `PanePayload::{ModulePluginsV1, BuildExportV1}`, and pane payload builders currently exist.
- `ShellPresentation::from_state(...)`, `pane_projection`, `floating_windows`, and `apply_presentation` currently pass both pane data objects.
- `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs` is above the large-file threshold. Any new projection behavior in this milestone must first move the Plugin Manager and Build Export projection code into focused child modules instead of appending more code to the root file.
- `zircon_editor/src/ui/slint_host/app/host_lifecycle.rs` is above the large-file threshold. Any new pane data assembly behavior in this milestone must be extracted into focused child modules instead of appending more lifecycle logic to the root file.

## File Map

- Modify as needed: `zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs` for Plugin Manager descriptor metadata.
- Modify as needed: `zircon_editor/src/ui/host/builtin_views/activity_views/build_export_view_descriptor.rs` for Build Export descriptor metadata.
- Modify as needed: `zircon_editor/src/ui/host/builtin_views/activity_views/activity_view_descriptors.rs` for descriptor registration order.
- Modify as needed: `zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs` for default Plugin Manager and Build Export view instances.
- Modify as needed: `zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs` for default drawer placement.
- Modify as needed: `zircon_editor/src/ui/workbench/snapshot/workbench/view_content_kind.rs` for content-kind variants.
- Modify as needed: `zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs` for descriptor-to-content mapping.
- Modify as needed: `zircon_editor/src/ui/workbench/reflection/name_mapping.rs` for host reflection names.
- Modify as needed: `zircon_editor/src/ui/workbench/model/menu/view_menu.rs` for View menu entries.
- Modify as needed: `zircon_editor/src/ui/workbench/model/menu_item_model.rs` for operation paths `View.PluginManager.Open` and `View.BuildExport.Open`.
- Modify as needed: `zircon_editor/src/ui/workbench/view/pane_payload_kind.rs` for pane payload variants.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs` for pane view-data DTOs.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs` for stable payload structs.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/mod.rs` for payload dispatch.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/module_plugins.rs` for Plugin Manager payload projection.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/build_export.rs` for Build Export payload projection.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_presentation.rs` for `PanePayloadBuildContext` and body presentation assembly.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs` for active pane `PaneData.native_body` assembly.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/floating_windows.rs` for floating-window pane propagation.
- Modify as needed: `zircon_editor/src/ui/layouts/windows/workbench_host_window/shell_presentation.rs` for combined shell presentation.
- Modify as needed: `zircon_editor/src/ui/slint_host/host_contract/data/panes.rs` for Slint-facing pane DTOs.
- Modify as needed: `zircon_editor/src/ui/slint_host/ui/apply_presentation.rs` for host-contract pane projection.
- Modify as needed: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs` only to move existing Plugin Manager and Build Export code into child modules and re-export their narrow functions.
- Create if projection code changes: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/module_plugins.rs` for Plugin Manager row/template projection.
- Create if projection code changes: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/build_export.rs` for Build Export row/template projection.
- Modify as needed: `zircon_editor/src/ui/slint_host/app/host_lifecycle.rs` only to delegate pane data assembly to focused child modules.
- Create if host data assembly changes: `zircon_editor/src/ui/slint_host/app/module_plugins_pane_data.rs` for `SlintEditorHost::module_plugins_pane_data(...)` and Plugin Manager summary helpers.
- Create if host data assembly changes: `zircon_editor/src/ui/slint_host/app/build_export_pane_data.rs` for `SlintEditorHost::build_export_pane_data(...)` and export-target summary helpers.
- Modify as needed: `zircon_editor/src/ui/slint_host/app.rs` for new child module declarations when host data assembly is extracted.
- Modify as needed: `zircon_editor/src/ui/slint_host/app/module_plugin_actions.rs` for host-level Plugin Manager action parsing and backend diagnostics.
- Modify as needed: `zircon_editor/src/ui/slint_host/app/build_export_actions.rs` for host-level Build Export action parsing, queue state, cancellation state, and output-root summary.
- Modify as needed: `zircon_editor/src/ui/slint_host/app/pane_surface_actions.rs` for routing `ModulePluginAction` and `BuildExportAction` button clicks.
- Modify as needed: `zircon_editor/assets/ui/editor/host/module_plugins_body.ui.toml` for Plugin Manager host template anchors.
- Modify as needed: `zircon_editor/assets/ui/editor/host/build_export_desktop_body.ui.toml` for Build Export host template anchors.
- Modify: `zircon_editor/src/tests/workbench/view_model/document_workspace.rs` for default workbench, menu, and idempotent open-route coverage.
- Modify: `zircon_editor/src/tests/host/pane_presentation.rs` for payload/presentation coverage.
- Modify as needed: `zircon_editor/src/ui/slint_host/ui/pane_data_conversion/mod.rs` or extracted child module tests for projection coverage.
- Modify docs: `docs/editor-and-tooling/editor-workbench-shell.md` for workbench shell and host projection behavior.
- Modify docs: `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md` for Plugin Manager host behavior.
- Modify docs: `docs/engine-architecture/runtime-editor-pluginized-export.md` for Build Export host pane/report boundaries.
- Update: `.codex/sessions/20260503-0932-editor-core-convergence.md` after implementation and validation stages.

## Milestone 1: Workbench Route And Descriptor Closure

### Goal

Plugin Manager and Build Export must be stable builtin editor surfaces reachable through descriptors, default layout, View menu actions, operation paths, and content-kind mapping.

### In-Scope Behaviors

- `editor.module_plugins` resolves to `ViewContentKind::ModulePlugins` and title `Plugin Manager`.
- `editor.build_export_desktop` resolves to `ViewContentKind::BuildExport` and title `Desktop Export`.
- Plugin Manager defaults to `ActivityDrawerSlot::LeftBottom` and starts collapsed.
- Build Export defaults to `ActivityDrawerSlot::BottomRight` and starts collapsed alongside Runtime Diagnostics when that subsystem is enabled.
- View menu entries dispatch `MenuAction::OpenView(...)` for both descriptors and expose operation paths `View.PluginManager.Open` and `View.BuildExport.Open`.
- Opening either descriptor twice reuses the stable builtin instance path and must not create duplicate default tabs.

### Dependencies

- Current builtin view registry and default preview fixture must remain the source of truth.
- No plugin package, runtime plugin loader, or export build-plan internals are in scope.

### Implementation Slices

- [ ] Inspect the descriptor chain in `module_plugins_view_descriptor.rs`, `build_export_view_descriptor.rs`, `activity_view_descriptors.rs`, `descriptor_content_kind.rs`, `name_mapping.rs`, `view_menu.rs`, `menu_item_model.rs`, `builtin_shell_view_instances.rs`, and `layout_drawers.rs`.
- [ ] Keep descriptor ids exact: `editor.module_plugins` and `editor.build_export_desktop`.
- [ ] Keep pane body document ids exact: `pane.module_plugins.body` and `pane.build_export_desktop.body`.
- [ ] Keep pane payload kinds exact: `PanePayloadKind::ModulePluginsV1` and `PanePayloadKind::BuildExportV1`.
- [ ] Keep host reflection names exact: `module_plugins` and `build_export` from `content_kind_name(...)`; keep `BuildExportView` from `binding_view_id(...)`.
- [ ] Extend `default_preview_fixture_exposes_hybrid_shell_tool_windows_and_empty_states` in `document_workspace.rs` if any of the following assertions are missing: left-bottom Plugin Manager tab title, bottom-right Desktop Export tab title, non-closeable builtin tabs, View menu Plugin Manager action id, View menu Desktop Export action id.
- [ ] Add `view_menu_operation_paths_route_plugin_manager_and_build_export` to `document_workspace.rs` with these assertions:
  - find the `View` menu from `WorkbenchViewModel::build(&default_preview_fixture().build_chrome()).menu_bar`;
  - find `Plugin Manager` item and assert `operation_path.as_ref().map(|path| path.as_str()) == Some("View.PluginManager.Open")`;
  - find `Desktop Export` item and assert `operation_path.as_ref().map(|path| path.as_str()) == Some("View.BuildExport.Open")`.
- [ ] Add an idempotent default layout assertion in `document_workspace.rs`: count `ViewContentKind::ModulePlugins` in left-bottom tabs and `ViewContentKind::BuildExport` in bottom-right tabs, and assert each count is exactly `1`.

### Testing Stage

- [ ] Run `cargo test -p zircon_editor --lib default_preview_fixture_exposes_hybrid_shell_tool_windows_and_empty_states --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib view_menu_operation_paths_route_plugin_manager_and_build_export --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib default_preview_fixture_projects_drawers_and_document_workspace --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] If a test fails, inspect the lowest route layer first in this order: descriptor id, builtin view registry, builtin shell instance, drawer layout, workbench model, menu projection.

### Lightweight Checks

- [ ] During implementation only, use `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` if route edits change Rust signatures or enum variants.

### Exit Evidence

- [ ] The three focused `zircon_editor` tests in the testing stage pass or the session note records exact unrelated compile blockers.
- [ ] No new descriptor id, alias descriptor, compatibility descriptor, or parallel open route exists for either pane.

## Milestone 2: Shell Presentation And Pane Payload Closure

### Goal

Plugin Manager and Build Export must flow through one shell presentation path from workbench state to `PaneData.native_body`, `PanePayload`, and `ShellPresentation::from_state(...)`.

### In-Scope Behaviors

- `ModulePluginsPaneViewData` carries plugin rows and pane diagnostics.
- `BuildExportPaneViewData` carries export target rows and pane diagnostics.
- `PaneNativeBodyData` contains both `module_plugins` and `build_export` fields.
- `PanePayloadBuildContext` carries both pane view-data references.
- `PanePayload::{ModulePluginsV1, BuildExportV1}` preserves stable body payload data for template projection.
- `ShellPresentation::from_state(...)` accepts both pane view-data references and forwards them into side panes, document panes, and floating windows.

### Dependencies

- Milestone 1 descriptor/content mapping is correct.
- Runtime/plugin/export internals are still outside this milestone.

### Implementation Slices

- [ ] Inspect `host_data.rs`; ensure `ModulePluginStatusViewData`, `ModulePluginsPaneViewData`, `BuildExportTargetViewData`, and `BuildExportPaneViewData` exist and derive `Clone, Default`.
- [ ] Inspect `pane_payload.rs`; ensure `ModulePluginsPanePayload`, `ModulePluginStatusPayload`, `BuildExportPanePayload`, and `BuildExportTargetPayload` contain all fields consumed by their current pane builders.
- [ ] Inspect `pane_payload_builders/module_plugins.rs`; ensure it maps every `ModulePluginStatusViewData` field into `ModulePluginStatusPayload` without plugin package loading.
- [ ] Inspect `pane_payload_builders/build_export.rs`; ensure missing `context.build_export` returns `PanePayload::BuildExportV1(BuildExportPanePayload { diagnostics: String::new(), targets: Vec::new() })` so a body can still be projected.
- [ ] Inspect `pane_presentation.rs`; ensure `PanePayloadBuildContext` has `with_module_plugins(...)` and `with_build_export(...)` builder methods and `build_pane_body_presentation(...)` delegates through `build_payload(...)`.
- [ ] Inspect `pane_projection.rs`; ensure `pane_from_tab(...)`, `document_pane(...)`, and `side_pane(...)` pass both view-data objects and copy them into `PaneNativeBodyData` for every pane.
- [ ] Inspect `floating_windows.rs`; ensure `collect_floating_windows(...)` and `floating_window_data(...)` pass both view-data objects into `pane_from_tab(...)`.
- [ ] Inspect `shell_presentation.rs`; ensure `ShellPresentation::from_state(...)` receives `module_plugins: &ModulePluginsPaneViewData` and `build_export: &BuildExportPaneViewData` and passes both to `collect_floating_windows(...)`, side panes, and `document_pane(...)`.
- [ ] Extend `pane_payload_builders_emit_stable_body_metadata_for_first_wave_views` in `pane_presentation.rs` if either pane lacks assertions for diagnostics, target/plugin count, stable action ids, or target status.
- [ ] Extend `document_pane_projects_first_wave_pane_presentations_alongside_legacy_data` in `pane_presentation.rs` so `editor.module_plugins` uses `module_plugins_fixture()` and asserts `PanePayload::ModulePluginsV1`, and `editor.build_export_desktop` uses `build_export_fixture()` and asserts `PanePayload::BuildExportV1`.

### Testing Stage

- [ ] Run `cargo test -p zircon_editor --lib pane_payload_builders_emit_stable_body_metadata_for_first_wave_views --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib document_pane_projects_first_wave_pane_presentations_alongside_legacy_data --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` after presentation signature edits.
- [ ] If a test fails, diagnose in this order: DTO field shape, payload builder, pane presentation context, pane projection, shell presentation call sites.

### Lightweight Checks

- [ ] Use scoped `cargo check -p zircon_editor --lib ...` only after Rust signature edits; do not run broad workspace validation inside implementation slices.

### Exit Evidence

- [ ] `PanePayloadBuildContext` has one coherent path for both panes.
- [ ] No parallel Plugin Manager or Build Export presentation route exists outside `ShellPresentation::from_state(...)` and pane projection.

## Milestone 3: Slint Host Projection And Host-Level Actions

### Goal

Plugin Manager and Build Export must project into host-contract pane data with empty/diagnostic states and stable host-level action ids, without moving plugin/export business implementation into Slint conversion.

### In-Scope Behaviors

- Host-contract `PaneData` contains `module_plugins` and `build_export` fields.
- `apply_presentation.rs` converts every active pane into host-contract data and includes both panes.
- Plugin Manager row projection emits `ModulePluginRow.<id>` and `ModulePluginAction` nodes with stable action ids.
- Build Export row projection emits `BuildExportRow.<platform>`, `BuildExportCounts.<platform>`, `BuildExportDiagnostics.<platform>`, and `BuildExportAction` nodes with stable action ids.
- Running/queued/cancel-requested export targets expose `BuildExport.Cancel.<profile>`; ready targets expose `BuildExport.Execute.<profile>` unless fatal diagnostics disable the action.
- `pane_surface_actions.rs` routes `ModulePluginAction` to `dispatch_module_plugin_action(...)` and `BuildExportAction` to `dispatch_build_export_action(...)`.

### Dependencies

- Milestone 2 presentation DTOs are stable.
- Host action parsing may call editor-manager facades but must not implement plugin package loading, native library loading, or export materialization in Slint conversion modules.

### Implementation Slices

- [ ] Inspect `host_contract/data/panes.rs`; ensure `ModulePluginStatusData`, `ModulePluginsPaneData`, `BuildExportTargetData`, and `BuildExportPaneData` match the presentation DTO fields.
- [ ] Inspect `apply_presentation.rs`; ensure `to_host_contract_pane(...)` computes `module_plugins` and `build_export` before moving `data.native_body` fields and includes them in host-contract `PaneData`.
- [ ] Before adding projection behavior to `pane_data_conversion/mod.rs`, extract the existing Plugin Manager functions into `pane_data_conversion/module_plugins.rs`: `to_host_contract_module_plugins_pane_from_host_pane(...)`, module plugin status mapping, row node projection, action button projection, and Plugin Manager row constants.
- [ ] Before adding projection behavior to `pane_data_conversion/mod.rs`, extract the existing Build Export functions into `pane_data_conversion/build_export.rs`: `to_host_contract_build_export_pane_from_host_pane(...)`, build export target mapping, row node projection, action button projection, and Build Export row constants.
- [ ] Keep `pane_data_conversion/mod.rs` structural for the extracted panes: `mod module_plugins; mod build_export; pub(crate) use ...` plus shared imports only.
- [ ] Do not move `builtin_host_runtime()` unless both extracted child modules need it; if they do, expose a private `project_pane_template_nodes(...)` helper from `mod.rs` as `pub(super)` and keep it non-mutating.
- [ ] Inspect `module_plugin_actions.rs`; keep parsing coverage for `Plugin.Enable.<id>`, `Plugin.Disable.<id>`, `Plugin.Packaging.Next.<id>`, `Plugin.TargetModes.Next.<id>`, `Plugin.Feature.Enable.<plugin>.<feature>`, `Plugin.Feature.Disable.<plugin>.<feature>`, `Plugin.Feature.EnableDependencies.<plugin>.<feature>`, `Plugin.Unload.<id>`, and `Plugin.HotReload.<id>`.
- [ ] Inspect `build_export_actions.rs`; keep parsing coverage for `BuildExport.Execute.<profile>`, `BuildExport.Cancel.<profile>`, `BuildExport.SetOutput.<profile>|<path>`, and `BuildExport.ClearOutput.<profile>`.
- [ ] Inspect `pane_surface_actions.rs`; keep direct routing for `ModulePluginAction` and `build_export_actions::BUILD_EXPORT_ACTION_CONTROL_ID` before falling back to generic pane dispatch.
- [ ] Add or preserve projection tests for Plugin Manager rows: assert one plugin row, six stable action ids, row actions count, and optional feature text.
- [ ] Add or preserve projection tests for Build Export ready rows: assert one target row, execute action id, counts text contains native package count, and action is enabled.
- [ ] Add or preserve projection tests for Build Export running rows: assert cancel action label and `BuildExport.Cancel.<profile>` action id.

### Testing Stage

- [ ] Run `cargo test -p zircon_editor --lib module_plugins_pane_projects_visual_rows_and_action_buttons --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib build_export_pane_projects_desktop_target_rows --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib build_export_running_target_projects_cancel_action --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib module_plugin_actions_parse_enable_policy_and_target_mode_updates --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib build_export_actions_parse_execute_profile --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib desktop_export_job_queue_starts_and_cancels_pending_jobs --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] If extracted modules cause compile failures, fix visibility and shared helper ownership before changing behavior.

### Lightweight Checks

- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` after extraction or action-routing signature edits.

### Exit Evidence

- [ ] `pane_data_conversion/mod.rs` is smaller or at least no longer owns the extracted Plugin Manager and Build Export row-projection responsibilities.
- [ ] Host action parsing and projection tests pass or the session note records exact unrelated compile blockers.

## Milestone 4: Host Data Assembly, Documentation, And Milestone Validation

### Goal

Close the usable editor loop with explicit diagnostics, docs that map code to behavior, and milestone validation evidence.

### In-Scope Behaviors

- `SlintEditorHost::module_plugins_pane_data(...)` returns fallback Plugin Manager diagnostics when project manifest loading fails and still produces pane view data.
- `SlintEditorHost::build_export_pane_data(...)` returns empty targets plus pane diagnostics when project manifest loading fails and still opens the pane.
- Repeated refresh/open actions rebuild presentation data without duplicating builtin tabs.
- Documentation records the owner split, related code, tests, and validation evidence.

### Dependencies

- Milestones 1 through 3 have either passed their focused tests or recorded scoped blockers.
- Active adjacent sessions remain out of scope unless an editor-only compile failure points to a direct import mismatch.

### Implementation Slices

- [ ] Inspect `host_lifecycle.rs` pane data assembly. If new Plugin Manager assembly code is required, move `module_plugins_pane_data(...)`, `module_plugin_optional_feature_summary(...)`, `module_plugin_feature_action(...)`, `module_plugin_primary_action(...)`, `module_plugin_action_id(...)`, `packaging_label(...)`, and `target_mode_label(...)` into `app/module_plugins_pane_data.rs`.
- [ ] If new Build Export assembly code is required, move `build_export_pane_data(...)`, `prepend_desktop_export_output_diagnostic(...)`, and output-root pane summary helpers into `app/build_export_pane_data.rs`.
- [ ] Keep `host_lifecycle.rs` responsible for lifecycle orchestration only: collect current model/chrome, request pane view data, call `apply_presentation(...)`, sync presenters, and mark dirty flags.
- [ ] Add or preserve host data assembly tests only through public/focused seams already used by editor tests. Do not instantiate a new Slint window solely to test private helper branches.
- [ ] Update `docs/editor-and-tooling/editor-workbench-shell.md` frontmatter and body to mention `editor.build_export_desktop`, `BuildExportPaneViewData`, `build_export_desktop_body.ui.toml`, `build_export_actions.rs`, and the focused tests from this plan.
- [ ] Update `docs/editor-and-tooling/editor-host-minimal-plugin-loading.md` frontmatter and Plugin Manager section if action ids, row projection, unavailable backend diagnostics, or host data assembly changed.
- [ ] Update `docs/engine-architecture/runtime-editor-pluginized-export.md` frontmatter and Build Export section if Build Export pane/report host behavior changed.
- [ ] Update `.codex/sessions/20260503-0932-editor-core-convergence.md` with final touched modules, commands run, pass/fail evidence, unresolved adjacent blockers, and the next step.
- [ ] Run hard-scope searches before validation:
  - `Plugin Manager` and `editor.module_plugins` should appear only in editor host/workbench/docs/tests paths for this milestone.
  - `Desktop Export` and `editor.build_export_desktop` should appear only in editor host/workbench/docs/tests paths plus runtime/export facade calls already owned by `EditorManager`.
  - No new `zircon_plugins/**` file is touched by this plan.

### Testing Stage

- [ ] Run `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never`.
- [ ] Run `cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir target\codex-shared-a --message-format short --color never` if focused editor checks pass and active compile load allows the broader editor lib suite.
- [ ] If the broader editor lib suite is impractical under active compile load, rerun the focused test filters from Milestones 1 through 3 individually and record each result.
- [ ] Run `git diff --check -- docs/superpowers/specs/2026-05-03-editor-core-usable-loop-design.md docs/superpowers/plans/2026-05-03-editor-core-usable-loop.md docs/editor-and-tooling/editor-workbench-shell.md docs/editor-and-tooling/editor-host-minimal-plugin-loading.md docs/engine-architecture/runtime-editor-pluginized-export.md zircon_editor/src/ui/host/builtin_views/activity_views/module_plugins_view_descriptor.rs zircon_editor/src/ui/host/builtin_views/activity_views/build_export_view_descriptor.rs zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs zircon_editor/src/ui/host/builtin_layout/layout_drawers.rs zircon_editor/src/ui/workbench/model/menu/view_menu.rs zircon_editor/src/ui/workbench/model/menu_item_model.rs zircon_editor/src/ui/workbench/snapshot/workbench/descriptor_content_kind.rs zircon_editor/src/ui/workbench/reflection/name_mapping.rs zircon_editor/src/ui/layouts/windows/workbench_host_window zircon_editor/src/ui/slint_host/app.rs zircon_editor/src/ui/slint_host/app zircon_editor/src/ui/slint_host/ui/apply_presentation.rs zircon_editor/src/ui/slint_host/ui/pane_data_conversion zircon_editor/src/tests/host/pane_presentation.rs zircon_editor/src/tests/workbench/view_model/document_workspace.rs`.
- [ ] Run `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -TargetDir target\codex-shared-a -VerboseOutput` if focused editor checks pass and no active Cargo load makes the validator impractical.
- [ ] Do not claim workspace green unless `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose` or the repository validator equivalent ran fresh and passed.

### Lightweight Checks

- [ ] During documentation-only edits, use `git diff --check` instead of Cargo unless Rust code also changed.

### Exit Evidence

- [ ] Focused editor compile and pane tests pass or exact blockers are recorded with file paths and command output summaries.
- [ ] Docs headers include the implementation files, plan source `docs/superpowers/specs/2026-05-03-editor-core-usable-loop-design.md`, plan path `docs/superpowers/plans/2026-05-03-editor-core-usable-loop.md`, and the focused test names.
- [ ] The active session note has enough evidence for another session to avoid duplicate diagnosis.

## Acceptance Checklist

- [ ] `Plugin Manager` opens through `editor.module_plugins` from the workbench route and has descriptor, menu, operation path, default layout, content-kind, payload, presentation, and Slint projection coverage.
- [ ] `Build Export` opens through `editor.build_export_desktop` from the workbench route and has descriptor, menu, operation path, default layout, content-kind, payload, presentation, and Slint projection coverage.
- [ ] `ShellPresentation::from_state(...)` receives both `ModulePluginsPaneViewData` and `BuildExportPaneViewData` and no parallel host-pane presentation route is added.
- [ ] Plugin Manager and Build Export panes produce safe empty or diagnostic data when project/runtime/export capability data is unavailable.
- [ ] Slint conversion stays projection-only: no plugin loading, export materialization, runtime world mutation, or editor state repair happens in `pane_data_conversion`.
- [ ] Refresh/open behavior is idempotent for builtin tabs and does not duplicate `editor.module_plugins#1` or `editor.build_export_desktop#1`.
- [ ] No concrete plugin package implementation, runtime plugin package internals, or export packaging internals are added by this plan.
- [ ] Large touched files are not made worse: pane conversion or host lifecycle behavior added by this milestone is moved into focused child modules first.
- [ ] Documentation and `.codex/sessions/20260503-0932-editor-core-convergence.md` record implementation files, plan sources, tests, and remaining risks.
