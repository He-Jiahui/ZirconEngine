---
related_code:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/hub.toml
  - zircon_hub/tauri.conf.json
  - zircon_hub/capabilities/default.json
  - zircon_hub/package.json
  - zircon_hub/vite.config.ts
  - zircon_hub/src/main.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/catalog.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/src/engines/source_engine_paths.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/tauri/projectTarget.ts
  - zircon_hub/web/src/main.tsx
  - zircon_hub/web/src/theme/tokens.ts
  - zircon_hub/web/src/theme/muiTheme.ts
  - zircon_hub/web/src/components/inputs/HubButton.tsx
  - zircon_hub/web/src/components/inputs/HubCheckbox.tsx
  - zircon_hub/web/src/components/inputs/HubComboBox.tsx
  - zircon_hub/web/src/components/inputs/HubIconButton.tsx
  - zircon_hub/web/src/components/inputs/HubSearchField.tsx
  - zircon_hub/web/src/components/inputs/HubSelect.tsx
  - zircon_hub/web/src/components/inputs/HubSwitch.tsx
  - zircon_hub/web/src/components/inputs/HubTabs.tsx
  - zircon_hub/web/src/components/inputs/HubTextField.tsx
  - zircon_hub/web/src/components/inputs/HubToggle.tsx
  - zircon_hub/web/src/components/data/EmptyStateBlock.tsx
  - zircon_hub/web/src/components/data/HubList.tsx
  - zircon_hub/web/src/components/data/HubTreeView.tsx
  - zircon_hub/web/src/components/data/MetricCard.tsx
  - zircon_hub/web/src/components/data/ProjectCard.tsx
  - zircon_hub/web/src/components/data/ProjectTable.tsx
  - zircon_hub/web/src/components/data/QuickActions.tsx
  - zircon_hub/web/src/components/data/SourceEngineList.tsx
  - zircon_hub/web/src/components/data/StatusBadge.tsx
  - zircon_hub/web/src/components/feedback/HubSnackbar.tsx
  - zircon_hub/web/src/components/feedback/HubStatusBanner.tsx
  - zircon_hub/web/src/components/overlays/HubDialog.tsx
  - zircon_hub/web/src/components/overlays/HubMenu.tsx
  - zircon_hub/web/src/components/overlays/HubPopover.tsx
  - zircon_hub/web/src/components/overlays/SourceEnginePopover.tsx
  - zircon_hub/web/src/components/overlays/UserMenuPopover.tsx
  - zircon_hub/web/src/components/shell/NavigationDrawer.tsx
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/web/src/components/shell/HubWindow.tsx
  - zircon_hub/web/src/pages/ProjectBrowserPage.tsx
  - zircon_hub/web/src/pages/ProjectDetailPage.tsx
  - zircon_hub/web/src/pages/ProjectsDashboard.tsx
  - zircon_hub/web/src/pages/CatalogPage.tsx
  - zircon_hub/web/src/pages/EditorPage.tsx
  - zircon_hub/web/src/pages/BuildsPage.tsx
  - zircon_hub/web/src/pages/CloudPage.tsx
  - zircon_hub/web/src/pages/TeamPage.tsx
  - zircon_hub/web/src/pages/SettingsPage.tsx
  - zircon_hub/web/src/pages/WorkspacePage.tsx
  - zircon_hub/web/src/data/hubData.ts
  - zircon_hub/web/src/settings/options.ts
  - zircon_hub/web/src/types/hub.ts
  - zircon_hub/tests/app_error_recovery_contract.rs
  - zircon_hub/tests/project_management_contract.rs
  - zircon_hub/tests/project_path_scope_contract.rs
  - zircon_hub/tests/project_quick_actions_contract.rs
  - zircon_hub/tests/project_source_engine_contract.rs
  - zircon_hub/tests/project_workflow_contract.rs
  - zircon_hub/tests/project_page_copy_contract.rs
  - zircon_hub/tests/project_cloud_local_delivery_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/ui_data_container_primitives_contract.rs
  - zircon_hub/tests/ui_data_display_contract.rs
  - zircon_hub/tests/ui_foundation_contract.rs
  - zircon_hub/tests/ui_global_rules_contract.rs
  - zircon_hub/tests/ui_input_navigation_api_contract.rs
  - zircon_hub/tests/ui_input_primitives_contract.rs
  - zircon_hub/tests/ui_inputs_contract.rs
  - zircon_hub/tests/ui_material_usage_contract.rs
  - zircon_hub/tests/ui_metric_section_contract.rs
  - zircon_hub/tests/ui_navigation_contract.rs
  - zircon_hub/tests/ui_overlay_primitives_contract.rs
  - zircon_hub/tests/ui_page_surface_coverage_contract.rs
  - zircon_hub/tests/ui_panel_slot_contract.rs
  - zircon_hub/tests/ui_project_browser_table_contract.rs
  - zircon_hub/tests/ui_project_layout_contract.rs
  - zircon_hub/tests/ui_project_navigation_contract.rs
  - zircon_hub/tests/ui_project_scope_contract.rs
  - zircon_hub/tests/ui_selected_project_catalog_contract.rs
  - zircon_hub/tests/ui_selected_project_runtime_contract.rs
  - zircon_hub/tests/ui_shell_composition_contract.rs
  - zircon_hub/tests/ui_shell_header_contract.rs
  - zircon_hub/tests/ui_shell_navigation_contract.rs
  - zircon_hub/tests/ui_shell_page_contract.rs
  - zircon_hub/tests/ui_shell_window_contract.rs
  - zircon_hub/tests/ui_table_view_contract.rs
  - zircon_hub/tests/ui_typography_contract.rs
  - zircon_hub/tests/ui_visual_standard_contract.rs
  - zircon_hub/tests/ui_workspace_layout_contract.rs
  - zircon_hub/tests/ui_workspace_split_contract.rs
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/SKILL.md
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/compare-hub-tauri-references.ps1
implementation_files:
  - zircon_hub/Cargo.toml
  - zircon_hub/build.rs
  - zircon_hub/hub.toml
  - zircon_hub/tauri.conf.json
  - zircon_hub/capabilities/default.json
  - zircon_hub/package.json
  - zircon_hub/vite.config.ts
  - zircon_hub/src/main.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/scoped_views.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/learn_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/output_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/quick_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/settings_actions.rs
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/src/tauri_app/view_model/action_history.rs
  - zircon_hub/src/tauri_app/view_model/catalog.rs
  - zircon_hub/src/tauri_app/view_model/coming_soon.rs
  - zircon_hub/src/tauri_app/view_model/localized.rs
  - zircon_hub/src/tauri_app/view_model/project_templates.rs
  - zircon_hub/src/tauri_app/view_model/settings_dto.rs
  - zircon_hub/src/tauri_app/view_model/ui_text.rs
  - zircon_hub/src/engines/source_engine_paths.rs
  - zircon_hub/src/process/folder_picker.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/web/src
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/SKILL.md
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-window.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/compare-hub-tauri-references.ps1
plan_sources:
  - user: 2026-06-05 switch Zircon Hub to Tauri + React frontend, use real Material UI, and build bottom-up component layers
  - docs/ui-and-layout/hub.png
  - docs/ui-and-layout/hub-web-reference
  - docs/ui-and-layout/hub-ai-drafts
  - dev/material-ui
tests:
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_management_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_path_scope_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml team_overview_ -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_quick_actions_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_source_engine_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml background_build
  - cargo test --manifest-path zircon_hub/Cargo.toml background_actions_queue_while_worker_is_active_and_dequeue_fifo -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml background_editor_launch
  - cargo test --manifest-path zircon_hub/Cargo.toml background_package
  - cargo test --manifest-path zircon_hub/Cargo.toml background_install
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_workflow_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml settings_draft_folder_changes_wait_for_save_settings
  - cargo test --manifest-path zircon_hub/Cargo.toml save_settings_refreshes_source_scoped_catalogs_in_returned_view_model -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml keeps_first_source_engine_root_before_fallback_limit -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml settings_health_includes_rustup_path_status -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml import_project_missing_manifest_failure_localizes_task_summary -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml project_view_action_status_localizes_in_chinese_view_model -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml chinese_bundle_localizes_page_and_status_copy -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml build_completion_localizes_success_history_detail -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml build_precondition_failure_localizes_unbound_source_engine_detail -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml open_resource_missing_catalog_file_failure_localizes_task_summary -- --nocapture
  - cargo test --manifest-path zircon_hub/Cargo.toml --test app_error_recovery_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_page_copy_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test project_cloud_local_delivery_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test tauri_react_shell_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --lib tauri_app
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_data_container_primitives_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_data_display_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_foundation_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_global_rules_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_input_navigation_api_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_input_primitives_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_inputs_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_material_usage_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_metric_section_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_navigation_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_overlay_primitives_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_page_surface_coverage_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_panel_slot_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_browser_table_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_layout_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_navigation_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_project_scope_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_catalog_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_runtime_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_composition_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_header_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_navigation_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_page_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_window_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_table_view_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_typography_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_visual_standard_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_workspace_layout_contract
  - cargo test --manifest-path zircon_hub/Cargo.toml --test ui_workspace_split_contract
  - npm run typecheck
  - npm run build
  - npm run tauri:build
  - npm run tauri:dev -- --no-watch --no-dev-server-wait --config target/hub-visual-check/tauri-dev/tauri-dev-override.json
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-project-pages.ps1 -OutputDir target/hub-visual-check/tauri-project-pages-full-matrix -CapturePendingDelete -CaptureBrowserMenus
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1
  - .codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/compare-hub-tauri-references.ps1
doc_type: ui-architecture
---

# Zircon Hub Tauri React Shell

This slice starts the hard cut from the Slint launcher to a Tauri v2 desktop shell with a React + Material UI frontend. The Rust binary now enters through `zircon_hub::tauri_app::run()`, `build.rs` delegates to `tauri_build::build()`, and `tauri.conf.json` points Tauri at the Vite frontend on port 1420 with a `1568x1003` undecorated Hub window. On Windows release builds, `src/main.rs` uses `cfg_attr(not(debug_assertions), windows_subsystem = "windows")` so `target/release/zircon_hub.exe` opens as a desktop GUI window without a console window covering the WebView or visual capture. `capabilities/default.json` binds the local `main` window to the explicit Tauri v2 `core:default` permission set plus `core:window:allow-minimize`, `core:window:allow-toggle-maximize`, and `core:window:allow-close`, so the shell has a concrete IPC capability boundary for its self-drawn window controls instead of relying on an absent capability directory.

The Tauri boundary is folder-backed. `src/tauri_app/mod.rs` stays as the launcher and command wrapper, `commands.rs` owns shared Tauri command state plus the `hub-state-changed` event emission, `runtime_state.rs` loads and persists the Slint-independent Hub backend state and routes actions, `runtime_state/scoped_views.rs` owns selected-project and Source Engine scoped Assets/Learn/Plugins/Team refreshes, `runtime_state/build_actions.rs` owns editor/runtime build preparation and completion, `runtime_state/editor_launch_actions.rs` owns open-editor preparation and completion, `runtime_state/project_actions.rs` owns project create/import/lifecycle actions including the localized Import Project folder picker titles, `runtime_state/project_delivery_actions.rs` owns package/install preparation and completion, `runtime_state/quick_actions.rs` owns shared action-history persistence helpers, `runtime_state/output_actions.rs` owns output-folder shell handoff, `runtime_state/settings_actions.rs` owns Settings folder-picker draft updates, and `view_model.rs` converts `HubSnapshot` into serde DTOs that match `web/src/types/hub.ts`.

`hub_state` now returns a backend-derived `HubViewModel`: Hub configuration, editor recent-project sync, source-engine registry repair, selected project state, project filters, catalog discovery, team discovery, action history, persisted settings, editable `settingsDraft`, and localized `comingSoon` entries are all projected from Rust state instead of a static dashboard fixture. The same projection now exposes `browserProjects` for the full filtered browser table, `selectedProject` for detail-page metadata such as path, platform, engine binding, stable template id, localized template label, pinned state, and missing-path status, plus `assets`, `plugins`, `learnResources`, `team`, and `actionHistory` for the remaining Hub workspaces. Project modified-time strings, selected-project available/missing status, and selected-project template labels are localized before they reach React, so Project cards, tables, detail metrics, and Editor launch-target rows do not infer English fallback labels or render raw template ids. Project templates are projected by `src/tauri_app/view_model/project_templates.rs`, including the localized `optionLabel` that the New Project dialog renders for enabled and disabled template rows plus the selected-project `templateLabel` used by Project Detail and Editor; React no longer joins template title/status with page-local punctuation for “敬请期待” rows and no longer uses `templateId` as visible copy. The `comingSoon` DTO uses stable categories (`assets`, `plugins`, `local-delivery`, and `team`) with localized title/detail/status/meta text and `disabled = true`; Catalog, Cloud, Team, and Editor plugin surfaces filter those entries and render `entry.meta` instead of declaring page-local "敬请期待" category/status business copy. Plugin rows keep localized `maturity` as visible text but expose `maturityTone` as the status-badge tone, so Catalog does not parse English words such as `stable` out of display copy. Catalog search placeholders also use Rust-projected `HubCatalogText` prefix, separator, and suffix fields, so React does not decide language spacing or punctuation around the current page title. Settings text includes localized tab labels, field labels, build-profile options, and language options, so the React Settings page no longer owns option copy even though it still submits stable values such as `debug`, `release`, `Chinese`, and `English`; Settings metrics, switches, checkboxes, and path trees derive visible option labels from those DTOs instead of echoing the saved machine values. The fallback Workspace localized UI checkbox follows the same saved-language semantics as Settings: `Chinese` is the checked localized UI state, while the label/detail still come from localized Settings DTO fields. Import Project and Settings folder-picker dialog titles, task feedback, cancel text, failure text, recovery text, and operation target text are localized from the current Hub or draft language before the native picker or React feedback surfaces appear. Known internal operation targets such as `Output Folder` and `Hub settings` are translated by `HubTextBundle::operation_target`, while project names and filesystem paths remain unchanged. Project filter, sort, view-all, project selection, Source Engine selection, Source Engine validation/binding errors, build success payload details, package/install success details, delivery log excerpts, package/install failure details, Learn open-resource missing-file details, and visual loading task labels/details also stay as stable English runtime status internally and are translated by `HubTextBundle` at the DTO edge before React renders the header/banner/snackbar feedback. Shell text now also owns Source Engine popup labels, user menu labels/details, shared Source Engine empty-state copy, and fallback Workspace labels; `SourceEnginePopover`, `UserMenuPopover`, `SourceEngineList`, `TopBar`, and fallback `WorkspacePage` consume those DTO fields instead of carrying page-local English business copy. Source Engine row status is localized in Rust as active/registered display text before React renders it. Action-history DTOs include a stable `kind` and row `id` derived from `HubActionKind::id()`, while `action`, `status`, detail, log excerpt, and recovery are localized through `HubTextBundle` at the DTO boundary; Action-history DTOs include `detailRows` for target, finished time, output, recovery, command, and log display. Builds and Team render those rows directly without page-local command/log/output fallback wording, Builds, Cloud, and Editor filter history by `kind` instead of parsing localized labels, and `open-output-folder` resolves history rows by the same stable id before falling back to an explicit path. The DTO also carries status tone, output path, localized log excerpt, process id, and command-line data so React can render a real activity surface without owning backend language rules. `hub_action` routes navigation, project search/filter/sort/view changes, project selection, project subpage changes, source-engine selection, settings save, `browse-settings-folder`, `open-resource`, `open-output-folder`, and quick actions through the same runtime session before returning the refreshed window model to React. The `search-projects` action sends typed `{ query }` payloads from Projects dashboard and browser search fields, while Rust still accepts the old target-id fallback only as a compatibility parse path. The Learn catalog sends `open-resource` with a typed `{ resourceId, path }` payload from the selected `learnResources` row; the Rust action owner accepts only resources that still exist in the current discovered catalog before opening the containing folder, and missing catalog-file failures are translated in task feedback plus action history before React renders them. The Settings browse action applies any typed draft payload, invokes the native folder picker, updates only `settingsDraft`, and waits for `save-settings` before persisting HubConfig or refreshing registered Source Engine defaults. `save-settings` registers or updates the active Source Engine, refreshes source-scoped Assets, Plugins, Learn, and Team views, and returns a `HubViewModel` whose catalog rows already include the configured source root; Assets and Learn rank that configured root ahead of fallback roots before catalog limits truncate results. Build, package, install, and open-editor now prepare work under the Hub state lock, release that lock while `tools/zircon_build.py`, project package/device-install file copies, or editor process spawn runs, then reacquire the lock only to record action history and emit the completed `HubViewModel`. Open-editor still supports a selected project or empty editor launch and records recoverable failures without forcing the React shell back to demo data. React action dispatch no longer falls back to demo data when a Tauri action errors; `App` keeps the current backend state and replaces only task feedback with a visible error summary. React-synthesized live-update and command-failure task operations also use localized `HubShellText` fields instead of exposing raw event names or action ids.

`src/tauri_app/view_model/coming_soon.rs` now owns the disabled future-capability DTO separately from the general UI text bundle. Each row keeps the stable `category` key for React filtering and adds localized `categoryLabel` plus `meta` for visible grouping/status text, so Catalog, Cloud, Editor, Team, and the Chinese fallback state can show the future capability category and “敬请期待” state without parsing machine keys or joining labels in page code. Cloud Reserved Services renders only disabled `local-delivery` `comingSoon` entries; package and install roots stay in the Packages/Installs local workflow panels instead of being counted as reserved remote/service capabilities.

`web/src/tauri/projectTarget.ts` centralizes the React project workflow target payload. Project Detail and Editor use `projectTargetPayload(project)` for visible project-scoped actions, while Builds and Cloud use `workflowProjectTargetPayload`: selected project first and the latest recent project second. Builds and Cloud target panels render the same workflow project name/path that their main Build, Package, and Install buttons submit, so the UI cannot say "no selected project" while the backend runs against a latest-recent fallback. Project QuickActions remain stricter and only forward a selected project payload when that project still exists; disabled quick-action DTO rows stay disabled instead of inventing a React-side fallback. The Rust action parser keeps `targetId` only as a compatibility fallback for project-target actions, while runtime project-target resolution prefers `projectPath` before project id so management and background workflow actions match the intended project even when legacy ids collide.

Builds history detail now renders every recorded backend workflow field needed for local recovery: stable action target, localized finished time, output directory, recovery hint, command line, and localized log excerpt. The command-line row uses `common.command` and `common.noCommandRecorded` from the Rust-projected UI text bundle, so the Builds page displays real build/package/install history without adding page-local English fallback copy. Builds output-root and device-install folder rows still display the localized settings labels, but their `open-output-folder` action sends typed `{ outputDir }` from `state.settings.defaultBuildOutputDir` or `state.settings.defaultDeviceInstallDir` instead of reusing visible row detail text as a path. Cloud local-delivery package/install history rows read their visible output line from the backend `outputDir` field, render localized action detail and localized package/install log excerpts, and fall back only to localized `common.noOutputDirectory`; the local package and device-install handoff can open the correct output folder without depending on detail-row ordering. Cloud package-target output rows also dispatch `open-output-folder` with typed settings `outputDir`, keeping local delivery folder handoff on DTO path fields rather than UI text. The Editor active Source Engine output button follows that same typed folder handoff by sending `{ outputDir }` from `activeSourceEngine.outputPath` instead of a generic visible-path payload. React's `OpenOutputFolderPayload` exposes only `outputDir` and `historyId`; Rust still resolves `OpenOutputFolderPayload.outputDir` before the legacy `path` compatibility field for older callers, so a stale visible-path payload cannot shadow the explicit output-directory target. Editor no-project empty-state copy is also Rust-projected: it tells users they can choose a project from the browser or open an empty editor, matching the `open-editor` action path that launches with or without a selected project.

Settings Configuration Health is computed from the editable toolchain and path settings in `settings_dto.rs`. Python, Cargo, and Rustup each produce executable health rows, and project/source/build-output/device-install defaults produce directory rows, so the Settings page cannot report completeness from only part of the saved toolchain. The React `fallbackShellState` mirrors that full row set, including the Rustup executable row, so first-paint and backend-failure Settings health does not omit an editable toolchain field before Rust DTOs arrive.

`web/src/tauri/hubApi.ts` also exposes `subscribeHubStateChanged`, which listens for the backend `hub-state-changed` event and returns Tauri's unlisten cleanup function. `App.tsx` mounts that listener once, applies pushed `HubViewModel` payloads directly to window state, cleans the listener up on unmount, and surfaces a warning only if live updates cannot be registered. Build, package, install, and open-editor use that event path for both the immediate running state and the final success/error state while long-running work executes without holding the shared session lock.

App-level subscription and command failures render localized `HubShellText` label/detail/recovery fields (`liveUpdatesUnavailableDetail`, `actionFailedDetail`, `stateRefreshAfterCommand`, and `checkActionTarget`) instead of showing raw thrown error strings in the snackbar or status banner. `App.tsx` keeps the latest shell DTO in a ref so fallback-mode, Chinese default, and any saved English setting all use the active language; raw caught errors are logged to the console for diagnosis without becoming user-facing copy.

The React fallback state is treated as a first-paint and backend-failure surface, not as an English demo fixture. `web/src/data/hubData.ts` keeps only stable ids, paths, command tokens, asset ids, and product/engine names in their machine-readable form; visible sample project names, Source Engine status, asset kinds/sources, plugin names/descriptions/categories/maturity/scope, Learn resource titles/categories/summaries, action-history targets, and Settings health rows are localized to Chinese. The first launch language stays Chinese by both Rust default and bundled `hub.toml`, so a missing user config and the checked-in developer config project the same default language. `ui_foundation_contract.rs` now guards those fallback labels and the Rustup health row so an unavailable Tauri runtime or failed `hub_state` load still opens on Chinese default copy instead of leaking old English sample text or incomplete toolchain health before Rust DTOs arrive.

The React shell renders Projects as a dashboard/browser/detail subpage router for `activePage = "projects"`, renders Editor through `EditorPage`, Builds through `BuildsPage`, Assets/Plugins/Learn through a shared `CatalogPage`, Cloud through `CloudPage`, Team through `TeamPage`, and Settings through a focused `SettingsPage`. `WorkspacePage` remains only as a fallback workspace composition for unknown or future page ids, but it still consumes localized settings/common/shell DTO labels for metrics, tabs, panels, tree nodes, and actions. Workspace fallback page labels use Rust-projected navigation labels instead of stable route ids. Its header Settings button routes to the focused Settings page instead of dispatching `save-settings` without an editable draft; `SettingsPage` is the only React page that persists Settings by sending `{ settings: draft }`. Those pages use `SourceEngineList`, `HubPanel`, `ProjectTable`, `MetricCard`, `HubList`, `HubTreeView`, `StatusBadge`, `EmptyStateBlock`, `HubTabs`, `HubButton`, `HubSwitch`, `HubCheckbox`, `LinearProgress`, and `QuickActions` rather than page-local rows.

## Component Order

The React side is intentionally bottom-up:

- `web/src/theme` owns density, color, radius, shadow, typography, and shared MUI component overrides.
- `web/src/components/inputs` owns low-level buttons, icon buttons, text fields, search fields, combo boxes, selects, checkboxes, switches, tabs, and toggles.
- `web/src/components/data` owns reusable cards, cover media, lists, tree views, tables, metric cards, empty states, panels, source-engine lists, quick-action lists, status badges, and button-state samples.
- `web/src/components/feedback` owns snackbar and status-banner feedback surfaces fed by the Rust task summary.
- `web/src/components/overlays` owns dialog, menu, popover, source-engine popup, and user-menu popup surfaces.
- `web/src/components/shell` owns drawer, topbar, and window composition.
- `web/src/pages` only assembles shared components into the Projects dashboard, project browser, project detail, Editor, Builds, catalog, Cloud, Team, Settings, and fallback workspace layouts.

The visual asset policy uses `zircon_hub/assets/brand` and `zircon_hub/assets/covers/reference` at runtime. It must not render `docs/ui-and-layout/hub.png`, `hub-web-reference-1568x1003.png`, or AI draft PNGs as application UI. Reference project covers remain React-side assets, while project names, paths, selection, filters, and source-engine labels come from the backend snapshot.

## Visual State Matrix

The real-window Tauri visual matrix now extends project-page capture coverage beyond Dashboard, New Project, Browser, Detail, delete confirmation, browser filter, and browser sort screenshots. `.codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/capture-hub-visual-state-matrix.ps1` captures Editor, Assets, Builds, Plugins, Cloud, Team, Learn, Settings, Source Engine popup, user menu, Project Browser empty state, loading/running feedback, and error feedback from the Hub binary at the reference `1568x1003` window size. The script requires a titled `Zircon Hub` native window and valid window bounds, then captures pixels through WebView2 DevTools `Page.captureScreenshot` because automated Windows desktop copy can return a black composed WebView surface even when the React DOM has rendered correctly. It rejects mostly white captures and low-accent screenshots so helper windows or blank WebViews cannot be accepted as visual evidence.

The matrix seeds English runtime config and every capture now waits for state-specific WebView text before saving. Editor waits for `Launch Target`, Assets for `Assets Catalog`, Builds for `Build Workflow`, Plugins for `Plugins Catalog`, Cloud for `Package Outputs`, Team for `Team Members`, Learn for `Learn Catalog`, Settings for `Build Defaults`, the empty browser for `No projects found`, loading for `Loading Hub state`, and error for `Visual verification error state`. This fails fast if React is still showing the localized fallback first paint instead of the backend `hub_state` page selected by the isolated config.

The Source Engine popup and user menu captures now use WebView text actions instead of fixed header coordinates. The matrix clicks `Zircon Engine 1.8.2` and `He-Jiahui`, then waits for popup-only text candidates such as `Manage engines`/`管理引擎` and `Preferences`/`偏好设置` before saving. Project Detail and delete-confirm captures use the same WebView action helper to open the seeded Elysium project and scroll/click `Delete Project`/`删除项目`, while still requiring screenshot-difference gates against the prior page. This prevents a focused but closed topbar button or a scroll-only movement from passing as popup/delete evidence.

The loading and error captures use the diagnostic `ZIRCON_HUB_VISUAL_TASK_STATE` override in `src/tauri_app/runtime_state.rs` to seed `TaskStatus` while still rendering through the production `hub_state` view-model, `HubStatusBanner`, and `HubSnackbar` components. This is a visual verification hook only; ordinary user sessions keep task state from the real quick-action/backend flow.

The Project Browser menu capture now uses the Recent Projects `View All Projects` entry as the stable dashboard-to-browser transition on wide windows, then refuses to save any menu capture whose title is not `Zircon Hub` or whose dimensions are less than 90% of the requested Hub window. This prevents native file pickers or popup-only helper windows from being mislabeled as Browser filter/sort evidence.

`.codex/skills/zircon-project-skills/capture-hub-window-screenshot/scripts/compare-hub-tauri-references.ps1` now compares the real Tauri Dashboard plus all 19 exported reference pages/states against the HTML/CSS-finalized `docs/ui-and-layout` PNG references. It emits `target/hub-visual-check/tauri-reference-comparison/hub-tauri-reference-comparison.json` and `.md`; the 2026-06-06 run sampled 20 comparisons at step 4 with zero fatal failures and zero similarity warnings after the popup text gates were added. AI draft PNGs remain inventory support for the same page ids, while final metrics are computed against the HTML/CSS references.

## Open Migration Work

The old Slint source tree and historical `src/app` helper tree have now been removed from `zircon_hub`. The compiled Rust module root remains hard-cut: `lib.rs` does not expose `pub mod app`, `main.rs` enters only through `zircon_hub::tauri_app::run()`, `build.rs` delegates to Tauri, and the remaining UI contracts inspect React/MUI sources rather than old `.slint` files.

The long-running backend slices now move build execution, package/install file-copy work, and editor process launch out of the shared Hub-state lock. `commands.rs` still returns an immediate running model for background actions, while `runtime_state/action_tasks.rs` owns an in-process FIFO queue so a second build/package/install/open-editor request waits behind the active worker instead of overwriting its running status or spawning a parallel workflow. `run_background_build_action` prepares a `PendingEditorRuntimeBuild` through `runtime_state/build_actions.rs`, `run_background_package_action` and `run_background_install_action` prepare delivery requests through `runtime_state/project_delivery_actions.rs`, and `run_background_editor_action` prepares `PendingEditorLaunch` through `runtime_state/editor_launch_actions.rs`. Each path emits the running `HubViewModel`, runs the process or file-copy work outside the lock, then completes action history and task status through the matching completion method before `continue_background_queue` starts the next queued request.

## Component Coverage Continuation

The follow-up React/MUI coverage slice extends the shell beyond buttons/cards/tables. The input layer now includes checkbox, switch, tabs, text field, and combo box primitives; the data layer now includes list, tree view, metric card, and empty-state primitives; the feedback layer exposes snackbar and status banner components; and the overlay layer exposes shared dialog, menu, popover, source-engine popup, and user-menu popup components. `TopBar` anchors `SourceEnginePopover` and `UserMenuPopover` while routing settings/learn/team actions through the same `onAction` dispatcher, `ProjectsDashboard` keeps the card dashboard plus New Project dialog and routes `project-browser`/`project-detail` subpages to focused React pages, `ProjectBrowserPage` composes shared search/select/toggle/table/source-engine components against `browserProjects`, `ProjectDetailPage` composes cover, metric, list, tree, status, and quick-action components against `selectedProject`, `EditorPage` composes launch target, Source Engine, editor-plugin, reserved plugin operation, readiness, and editor activity panels against selected project, plugin, `comingSoon`, source-engine, and action-history DTOs, `BuildsPage` composes build/package/install workflows, output trees, action history, progress, and Source Engine panels against backend action and settings DTOs, `CatalogPage` composes Assets/Plugins/Learn catalog rows, reserved capability rows, tabs, status, trees, quick actions, and source-engine panels against backend catalog and `comingSoon` DTOs, `CloudPage` composes local package outputs, device install handoff, reserved service slots, and task feedback against settings, action history, and `comingSoon`, `TeamPage` composes Git identity, member lists, reserved collaboration rows, action-history detail, source-engine panels, and quick actions against `team`, `actionHistory`, and `comingSoon`, `SettingsPage` composes build defaults, path defaults, health, source-engine lists, and advanced path trees from shared components, `WorkspacePage` remains a fallback layout, and `App` displays real Rust task feedback through `HubSnackbar`.

## Material Contract Cutover

`ui_material_usage_contract.rs` has now been hard-cut from the old static Slint component tree to the React/MUI source tree. The contract checks the real `@mui/material` and `@mui/icons-material` package wiring, `web/src/theme/tokens.ts`, `web/src/theme/muiTheme.ts`, every low-level input wrapper, the list/tree/table/status/data wrappers, feedback surfaces, dialog/menu/popover wrappers, drawer/topbar/window composition, and the page files that assemble them. The same contract also guards against reintroducing old UI-file readers or old `src/app` binding paths into this Material usage slice.

## Data Contract Cutover

`ui_data_container_primitives_contract.rs` has also been rewritten to inspect the React/MUI data layer directly. It now locks the `components/data` barrel exports, MUI-backed `HubList`, `HubTreeView`, `ProjectTable`, `HubPanel`, `ProjectCard`, `ProjectCover`, `MetricCard`, `StatusBadge`, `EmptyStateBlock`, `QuickActions`, and `SourceEngineList`, then verifies that routed pages consume those shared components rather than importing raw Material data primitives such as `Table`, `ListItemButton`, `Card`, or `Chip` from the page layer. It also guards that the old dashboard button-state reference sample is absent from the data barrel and Projects product page.

`ui_data_display_contract.rs` now follows the same React/MUI cutover for display semantics. Instead of reading the historical UI component files, it checks the shared display atoms and row-like components directly: `StatusBadge`, `EmptyStateBlock`, `MetricCard`, `ProjectCover`, `ProjectCard`, `ProjectTable`, `HubList`, `HubTreeView`, `QuickActions`, `SourceEngineList`, and `HubPanel`. It also checks that Projects, Browser, Detail, Catalog, Editor, Builds, Cloud, Team, Settings, and fallback Workspace pages assemble those display atoms from the shared data barrel while TypeScript DTOs continue to feed project rows, task status, source engines, quick actions, and action history.

## Input Contract Cutover

`ui_inputs_contract.rs` has been rewritten against the React/MUI input layer. The contract now locks the `components/inputs` barrel exports, the MUI-backed `HubButton`, `HubIconButton`, `HubSearchField`, `HubTextField`, `HubSelect`, `HubComboBox`, `HubCheckbox`, `HubSwitch`, `HubToggle`, and `HubTabs` wrappers, and the shared theme/token density used by those controls. It also checks that routed pages consume the shared input wrappers instead of importing raw Material input primitives such as `Button`, `TextField`, `Select`, `Checkbox`, `Switch`, `Tabs`, or `ToggleButton` from the page layer.

## Input Primitive Contract Cutover

`ui_input_primitives_contract.rs` now checks the React/MUI input primitive system instead of historical low-level Slint control wrappers. The contract locks `web/src/components/inputs/HubButton.tsx`, `HubIconButton.tsx`, `HubSearchField.tsx`, `HubTextField.tsx`, `HubSelect.tsx`, `HubComboBox.tsx`, `HubCheckbox.tsx`, `HubSwitch.tsx`, `HubTabs.tsx`, and `HubToggle.tsx` as direct Material UI wrappers with Hub tokens. It also checks `web/src/pages` so routed pages consume the shared input wrappers and do not import raw Material button, text-field, select, checkbox, switch, tabs, toggle, or icon-button primitives. `HubCheckbox` and `HubSwitch` now treat a missing `onChange` callback as read-only state: they derive `isDisabled` from `disabled || !onChange`, keep labels readable through the shared muted text color, and remove hover/edit affordances unless a page supplies a real state mutation.

## Input Navigation API Contract Cutover

`ui_input_navigation_api_contract.rs` now checks the React/MUI input/navigation API instead of historical exported input structs and Slint navigation adapters. The contract locks `web/src/components/inputs/index.ts` plus each typed input wrapper around TypeScript props and callback names, then checks `web/src/components/shell/NavigationDrawer.tsx`, `TopBar.tsx`, `HubWindow.tsx`, `web/src/App.tsx`, and `web/src/tauri/hubApi.ts` so primary navigation, topbar menus, routed pages, filters, tabs, toggles, and backend actions share the same typed `onAction(actionId, targetId, payload)` dispatcher.

## Overlay Primitive Contract Cutover

`ui_overlay_primitives_contract.rs` now checks React/MUI overlay primitives instead of historical popup, dropdown, drawer, and window wrappers. The contract locks `web/src/components/overlays/HubDialog.tsx`, `HubMenu.tsx`, and `HubPopover.tsx` as the Material Dialog, Menu, and Popover wrappers; `SourceEnginePopover.tsx` and `UserMenuPopover.tsx` as the business popup bodies; `web/src/components/shell/TopBar.tsx` as the anchor/action owner for source-engine and user popups; and `web/src/pages/ProjectsDashboard.tsx` as the New Project dialog consumer. `UserMenuPopover` now keeps the sign-out row as a disabled local-v1 account-service reservation with localized detail text, disabled MUI styling, and a click guard before `onAction`.

## Visual Standard Contract Cutover

`ui_visual_standard_contract.rs` now checks the React/MUI visual standard instead of the historical UI token files. It locks `web/src/theme/tokens.ts`, `web/src/theme/muiTheme.ts`, `web/src/styles.css`, shell chrome, drawer/topbar, shared input/data components, overlay and feedback styling, responsive page surfaces, runtime image assets, and the design reference PNG/AI draft/web-reference artifact matrix. It also rejects old UI-file readers and old `src/app` visual-contract assumptions, so this slice follows the Tauri/React hard cut without preserving Slint-era visual assertions as a compatibility layer.

## Shell Composition Contract Cutover

`ui_shell_composition_contract.rs` now checks React/MUI shell composition instead of historical shell component files. The contract locks `web/src/App.tsx` as the state-loading, state-event subscription, and single-dispatch entry, `web/src/components/shell/HubWindow.tsx` as the window surface and page router, `web/src/components/shell/TopBar.tsx` as the brand/source-engine/status/user/window-control chrome, `web/src/components/shell/NavigationDrawer.tsx` as the permanent MUI drawer, and `web/src/components/overlays/SourceEnginePopover.tsx` plus `web/src/components/overlays/UserMenuPopover.tsx` as shared popup bodies. It also checks that `HubWindow`, `TopBar`, `NavigationDrawer`, `SourceEnginePopover`, and `UserMenuPopover` exchange only `HubShellState` plus the shared `onAction` dispatcher instead of reintroducing page-local state loading or old UI-file shell paint.

## Shell Navigation Contract Cutover

`ui_shell_navigation_contract.rs` now checks React/MUI shell navigation chrome instead of historical sidebar helper components. The contract locks `web/src/components/shell/NavigationDrawer.tsx` as the permanent drawer page list, selected-row owner, status panel, responsive collapsed-label surface, and collapse affordance; `web/src/components/shell/HubWindow.tsx` as the slot that places the drawer between topbar and routed pages; `src/state/navigation.rs` as the Rust page id owner; `src/tauri_app/view_model/localized.rs` and `src/tauri_app/view_model/ui_text.rs` as the localized title, subtitle, and drawer-copy projection; and `src/tauri_app/runtime_state.rs` as the Tauri `show-page` action handler. The drawer collapse row now toggles a local React collapsed state and switches between localized Collapse/Expand labels without entering `hub_action`, while the status-card update check stays disabled with localized tooltip/detail copy because remote update service is not part of local v1.

## Shell Page Contract Cutover

`ui_shell_page_contract.rs` now checks React/MUI page chrome and routed page surfaces instead of historical shell page header/status helpers. The contract locks `web/src/components/shell/HubWindow.tsx` as the routed main surface, `src/state/navigation.rs`, `src/tauri_app/view_model/localized.rs`, and `src/tauri_app/view_model.rs` as the localized page title/subtitle projection, `web/src/components/feedback/HubStatusBanner.tsx` and `web/src/components/feedback/HubSnackbar.tsx` as shared status feedback, and `web/src/pages` as the page-header/status-banner consumers.

## Shell Header Contract Cutover

`ui_shell_header_contract.rs` now checks React/MUI shell header chrome instead of historical top-header helpers. The contract locks `web/src/components/shell/TopBar.tsx` as the brand, source-engine selector, status badges, user trigger, settings/help tools, disabled local-v1 notification-service reservation, window controls, and popup anchor owner; `web/src/components/data/StatusBadge.tsx` as the shared status-pill atom; `web/src/components/inputs/HubIconButton.tsx` as the shared icon tool button; and `web/src/data/hubData.ts` as the brand asset and fallback header state owner. The Help icon routes to the Learn page through the shared Hub action dispatcher, while the Notifications icon stays disabled and uses localized tooltip text to explain that notification center is a future local-v1 reservation rather than an active service. The self-drawn minimize, maximize, and close buttons call Tauri's current-window `minimize`, `toggleMaximize`, and `close` APIs directly, with a non-Tauri preview guard, so they do not enter the Hub business `hub_action` router. It also locks the user-menu sign-out row as a disabled local-v1 account-service reservation, matching the Cloud reserved account service instead of exposing a no-op sign-out action.

## Shell Window Contract Cutover

`ui_shell_window_contract.rs` now checks React/MUI shell window layout instead of historical shell layout slots. The contract locks `zircon_hub/tauri.conf.json` as the Tauri webview window and Vite build boundary, `zircon_hub/capabilities/default.json` as the local main-window permission owner for the self-drawn window controls, `web/src/theme/tokens.ts` as the shared window and shell density owner, `web/src/components/shell/HubWindow.tsx` as the viewport-sized shell slot and page router, and `zircon_hub/src/main.rs` plus `zircon_hub/src/lib.rs` as the hard-cut Tauri launcher entry without the old compiled UI module.

## Typography Contract Cutover

`ui_typography_contract.rs` now checks the React/MUI typography system instead of historical MaterialText usage. The contract locks the shared scale in `web/src/theme/muiTheme.ts`, global font inheritance in `web/src/styles.css`, dense card/list/table/status text in `web/src/components/data`, label/detail text in `web/src/components/inputs`, popup row and section labels in `web/src/components/overlays`, product/navigation/status text in `web/src/components/shell`, and routed title/subtitle usage in `web/src/pages`. Page files keep high-level copy on MUI `Typography` while delegating repeated row, card, table, status, and popup text to shared components.

## Panel Slot Contract Cutover

`ui_panel_slot_contract.rs` now checks React/MUI panel composition instead of historical PanelSlot and HubPanel Slint wrappers. The contract locks `web/src/components/data/HubPanel.tsx` as the shared card-backed section shell, `web/src/components/data/EmptyStateBlock.tsx` as the shared empty panel body, `web/src/components/data/MetricCard.tsx` and `ProjectCard.tsx` as reusable panel atoms, and `web/src/pages` as responsive composition surfaces. It also checks that pages use shared `HubPanel`, metric, list, tree, table, quick-action, and source-engine components instead of importing raw Material `Card`, `Paper`, `Table`, list-row, or drawer containers.

## Metric Section Contract Cutover

`ui_metric_section_contract.rs` now checks the React/MUI metric section system instead of historical shared metric sizing state. The contract locks `web/src/components/data/MetricCard.tsx` as the shared tone/icon/text atom, `web/src/pages/ProjectDetailPage.tsx` as the four-card project metric surface, and `web/src/pages/BuildsPage.tsx`, `CatalogPage.tsx`, `CloudPage.tsx`, and `TeamPage.tsx` as the three-card workspace metric grids. Page files project the data and grid count while `MetricCard` owns the repeated visual chrome.

## Project Layout Contract Cutover

`ui_project_layout_contract.rs` now checks the React/MUI Projects layout instead of historical Projects Taffy sizing. The contract locks `web/src/pages/ProjectsDashboard.tsx` as the dashboard/browser/detail route split and dashboard toolbar/card/table/dialog surface, `web/src/pages/ProjectBrowserPage.tsx` as the browser toolbar/table/sidebar layout, `web/src/pages/ProjectDetailPage.tsx` as the metric/tab/media/detail/sidebar layout, and `web/src/types/hub.ts` as the project DTO contract. Projects pages keep responsive grid proportions and state branching while shared project card, table, metric, tree, list, quick-action, source-engine, dialog, and input components own the repeated UI chrome. The dashboard card rail intentionally renders the first four visible project summaries so the `1568x1003` Hub window matches the reference first viewport, while dashboard and Browser tables consume the Rust-projected `browserProjects`/`recentProjects` row DTOs directly. React no longer strips an English `Modified` prefix or reconstructs table locations from card summaries, so localized modified-time display remains a backend DTO responsibility.

## Project Scope Contract Cutover

`ui_project_scope_contract.rs` now checks React/MUI project scope projection instead of historical passive native DTO fields. The contract locks `src/state/scope.rs`, `src/state/hub_snapshot.rs`, `src/settings/hub_config.rs`, and `src/projects/metadata.rs` as the Rust project-scope and metadata owners; `src/tauri_app/view_model.rs` as the visible project, selected-project, and quick-action DTO projection; `src/tauri_app/runtime_state/quick_actions.rs` as the action target resolver; and `web/src/components/data/ProjectCard.tsx`, `QuickActions.tsx`, plus `web/src/pages/ProjectDetailPage.tsx` as React consumers of those DTOs. Project-scoped workflow buttons now dispatch the same selected-project DTO through `projectTargetPayload(project)` instead of passing only a project id.

## Project Management Contract Cutover

`project_management_contract.rs` now checks React/MUI project management state and registry repair through the current Rust owners instead of historical runtime helper modules. The contract locks `src/projects/metadata.rs`, create-project requests, templates, recycle-bin command construction, editor-recent sync, and `src/settings/hub_config.rs` registry repair as the project data layer; `src/tauri_app/runtime_state.rs` owns HubRuntimeSession persistence into HubConfig plus editor recent last-project handoff before React projection.

## Project Source Engine Contract Cutover

`project_source_engine_contract.rs` now checks React/MUI Source Engine registration and selection through current source owners. The contract locks `src/engines/source_engine_paths.rs` to the shared project filesystem key, `src/engines/registry.rs` plus validation/build-history modules as the Source Engine data layer, `src/tauri_app/runtime_state.rs` as the owner of active-engine selection, settings registration, metadata migration, and New Project engine-default repair, `view_model.rs` as the localized Source Engine status projector, and `web/src/components/data/SourceEngineList.tsx` as the Material UI Source Engine row renderer with caller-supplied localized empty-state copy. Source Engine rows are only interactive when the caller supplies `onSelect`; read-only summary panels disable the underlying `ButtonBase` while preserving normal opacity and status-badge readability, so shared Source Engine lists no longer imply an active selection route where none exists. Source Engine build-history DTOs are projected in `src/tauri_app/view_model/source_engines.rs`: status, detail, log excerpt, finished time, output directory, and `secondaryDetail` are localized or path-normalized before `EditorPage` renders the Source Engine build history panel. The visible row still shows `record.outputDir`, but the `open-output-folder` action sends the explicit typed `{ outputDir }` payload from the matching Source Engine history record instead of using the list row's visible `detail` field as the action path. The active Source Engine output button also sends `{ outputDir }` from `activeSourceEngine.outputPath`, so every Editor Source Engine folder-open route uses the output-directory payload field. The Editor row also renders the backend `secondaryDetail` string for the recorded command line plus localized log excerpt, so page code does not own the command/log separator or fallback wording.

## Project Path Scope Contract Cutover

`project_path_scope_contract.rs` now checks React/MUI project path scope and path display instead of historical runtime root-path helper files. The contract locks `src/projects/metadata.rs`, `src/projects/validation.rs`, `src/tauri_app/runtime_state.rs`, `src/tauri_app/runtime_state/scoped_views.rs`, and `src/tauri_app/view_model.rs` as the Rust owners for shared path keys, project validation, selected-project path persistence, scoped root de-duplication, and DTO path strings; `web/src/pages/ProjectDetailPage.tsx`, `web/src/components/data/ProjectTable.tsx`, and related React pages render those DTO strings without recreating filesystem normalization helpers.

## Project Quick Actions Contract Cutover

`project_quick_actions_contract.rs` now checks React/MUI scope-derived quick-action DTOs instead of historical Slint page-header callback wiring. The contract locks `src/tauri_app/view_model.rs` as the owner of selected/latest/stale/empty project quick-action copy and enabled quick-action DTO state, `src/tauri_app/runtime_state/build_actions.rs` as the build preparation/completion owner, `src/tauri_app/runtime_state/editor_launch_actions.rs` as the open-editor preparation/completion owner, `src/tauri_app/runtime_state/project_delivery_actions.rs` as the package/install preparation/completion owner, `src/tauri_app/runtime_state/quick_actions.rs` as the shared action-history persistence path, and `web/src/components/data/QuickActions.tsx` as the Material UI list that renders disabled actions from DTO state without recomputing scope in React. The same contract now locks `workflowProjectTargetPayload` for Builds/Cloud main workflow buttons and target panels: selected project first and the latest recent project second. All routed QuickActions panels still forward `projectTargetPayload(state.selectedProject)` only when a selected project exists, so stale selected-project cards do not send stale paths while backend no-payload fallback remains available for non-QuickAction workflow routes.

## Project Workflow Contract Cutover

`project_workflow_contract.rs` now checks React/MUI project workflow routing instead of historical Slint callback/runtime helpers. The contract locks `src/tauri_app/runtime_state.rs` as the single Tauri session action router for page, project, filter, sort, selection, detail, Source Engine, settings, and workflow commands; `src/tauri_app/runtime_state/build_actions.rs` as the build preparation/completion owner; `src/tauri_app/runtime_state/editor_launch_actions.rs` as the open-editor preparation/completion owner; `src/tauri_app/runtime_state/project_actions.rs` as the create/import/lifecycle owner that localizes the Import Project folder picker; `src/tauri_app/runtime_state/project_delivery_actions.rs` as the package/install preparation/completion owner; `src/tauri_app/runtime_state/quick_actions.rs` as the shared action-history helper owner; `src/tauri_app/runtime_state/output_actions.rs` as the `open-output-folder` handoff owner; `src/tauri_app/runtime_state/settings_actions.rs` as the `browse-settings-folder` owner that mutates `settingsDraft` without persisting until `save-settings` and localizes native folder-picker titles plus task feedback from the draft language; `src/tauri_app/commands.rs`, `web/src/tauri/hubApi.ts`, and `web/src/App.tsx` as the IPC path; and `web/src/pages/ProjectsDashboard.tsx`, Browser, Detail, Builds, Cloud, and Settings pages as consumers of one single action dispatcher and refreshed HubViewModel. Project Detail, Editor, and QuickActions use `projectTargetPayload(project)` for selected-project-only actions, while Builds and Cloud main workflow buttons use `workflowProjectTargetPayload(state)` so selected project wins before the latest recent fallback; Rust `targetId` support remains only for compatibility.

## Page Copy Contract Cutover

`project_page_copy_contract.rs` now checks React/MUI Hub page copy and runtime labels instead of historical localized native page-copy files. The contract locks `src/state/navigation.rs` and `src/state/task_status.rs` for Rust-owned navigation/status labels, `src/tauri_app/view_model.rs` for project labels and quick-action text, `src/tauri_app/view_model/project_templates.rs` for localized New Project template option labels and selected-project template labels, `src/tauri_app/view_model/localized.rs` for task labels plus static/dynamic task-detail localization such as import cancellation, project filter/sort/view-all feedback, import validation path errors, Source Engine validation and binding errors, build success payload details, project lifecycle action-history prefixes, action-history log excerpts, open-output target/path/history details, editor-launch executable/process details, package/install file-count suffixes plus delivery failure details, and disabled project templates, `src/tauri_app/view_model/ui_text.rs` for Catalog search placeholder prefix/separator/suffix ownership, `src/tauri_app/view_model/settings_dto.rs` for Settings tab/field/option labels, and `web/src/pages/ProjectsDashboard.tsx`, `ProjectBrowserPage.tsx`, `ProjectDetailPage.tsx`, Builds, Catalog, Cloud, Team, and Settings pages for the routed React copy visible in the Hub surface. The same contract guards that Projects dashboard no longer renders the old button-state reference sample strip.

## Project Browser Table Contract Cutover

`ui_project_browser_table_contract.rs` now checks the React/MUI Project Browser table instead of historical native table row components. The contract locks `web/src/components/data/ProjectTable.tsx` as the shared Material table column model, row selection owner, cover/name cell, body-cell typography owner, and trailing detail icon action; `web/src/pages/ProjectBrowserPage.tsx` as the responsive filter toolbar and table/sidebar composition; `web/src/pages/ProjectsDashboard.tsx` as the dashboard table consumer of the same backend row DTOs; and `src/tauri_app/view_model.rs` plus `web/src/types/hub.ts` as the `browserProjects` and `recentProjects` DTO projection.

## Table View Contract Cutover

`ui_table_view_contract.rs` now checks the React/MUI table/list/tree view system instead of historical native table-view component files. The contract locks `web/src/components/data/ProjectTable.tsx`, `web/src/components/data/HubList.tsx`, and `web/src/components/data/HubTreeView.tsx` as the shared ProjectTable column model, HubList row model, and HubTreeView recursive tree model, then checks `web/src/pages` so Projects, Browser, Detail, Editor, Builds, Catalog, Cloud, Team, Settings, and fallback Workspace pages consume the shared table/list/tree components rather than rebuilding raw Material table or list primitives at the page layer. `HubList` now derives item disabled state from both row DTOs and the presence of an `onSelect` handler, and `HubTreeView` only exposes pointer/hover behavior for expandable branches or callers with a real leaf-selection handler. Read-only detail rows, action-history detail rows, output trees, and settings health rows therefore stay visually readable without presenting empty click affordances.

## Project Navigation Contract Cutover

`ui_project_navigation_contract.rs` now checks React/MUI Projects navigation instead of historical row-hit and subpage routing files. The contract locks `web/src/pages/ProjectsDashboard.tsx` as the dashboard, browser, detail, and new-project subpage router, `web/src/pages/ProjectBrowserPage.tsx` as the browser command/filter/detail route surface, `web/src/pages/ProjectDetailPage.tsx` as the project-scoped action route surface, `web/src/components/data/ProjectTable.tsx` and `ProjectCard.tsx` as the shared selection/open-detail interaction owners, `web/src/tauri/hubApi.ts` as the single frontend action wrapper, and `src/tauri_app/runtime_state.rs` as the backend project navigation state owner. The project-card corner action opens Project Detail instead of an empty menu, so visible card controls lead to the existing local project-management surface.

## Selected Project Catalog Contract Cutover

`ui_selected_project_catalog_contract.rs` now checks React/MUI selected-project catalog scope instead of historical Assets/Plugins/Learn/Team/Cloud page copy files. The contract locks `src/tauri_app/runtime_state.rs` as the selected-project/active-engine action owner, `src/tauri_app/runtime_state/scoped_views.rs` as the selected project and Source Engine catalog refresh owner, `src/assets/catalog.rs`, `src/plugins/catalog.rs`, and `src/learn/catalog.rs` as discovery modules that prioritize selected-project entries before engine entries, `src/tauri_app/runtime_state/learn_actions.rs` as the Learn open-resource owner, `src/team/local_git.rs` plus `scoped_views.rs` as the Team Git repository discovery path, `src/tauri_app/view_model.rs`, `src/tauri_app/view_model/catalog.rs`, and `web/src/types/hub.ts` as the catalog DTO projection, `web/src/pages/CatalogPage.tsx` as the shared Assets/Plugins/Learn mode surface, and `web/src/pages/TeamPage.tsx`, `CloudPage.tsx`, and `BuildsPage.tsx` as selected-project-aware workspace surfaces. Builds and Cloud workflow buttons and target panels use `workflowProjectTargetPayload`; QuickActions panels keep the selected-project-only payload path. Asset row detail text is projected by the Rust/fallback asset DTO, including localized punctuation between asset kind and path, so CatalogPage renders `asset.detail` instead of constructing row copy locally. Editor plugin scope is a stable `editorScoped` DTO flag, derived from manifest targets, capabilities, and module metadata rather than localized plugin description or scope copy. It also locks the Learn `open-resource` action wiring so CatalogPage can send row id and path through the typed payload while Rust resolves only live Learn catalog entries, falls back from stale ids to the supplied path, and keeps the existence and catalog-membership gate. Behavior tests in `scoped_views.rs` create local Git repositories to prove Team prefers the selected project's repository and falls back to the Source Engine repository when no project is selected.

Source Engine asset roots such as `Editor` and `Runtime` keep those precise display source labels, while `src/tauri_app/view_model/catalog.rs` normalizes their `sourceKey` to `engine` so the Assets page Engine tab filters Source Engine assets without parsing localized or implementation-specific copy.

## Selected Project Runtime Contract Cutover

`ui_selected_project_runtime_contract.rs` now checks the React/MUI selected-project runtime scope instead of historical app binding and native UI DTO files. The contract locks `src/state/scope.rs` and `src/state/hub_snapshot.rs` as the canonical selected-project and Source Engine scope resolver, `src/tauri_app/runtime_state.rs` as the Tauri selected-project persistence/action-refresh owner, `src/tauri_app/view_model.rs` plus `web/src/types/hub.ts` as the `selectedProjectId` and `selectedProject` DTO projection, and `web/src/pages/ProjectDetailPage.tsx` plus the Projects, Browser, Editor, Builds, and Cloud pages as passive consumers of that selected-project state.

## Workspace Layout Contract Cutover

`ui_workspace_layout_contract.rs` now checks the React/MUI workspace layout instead of historical workspace-page Taffy sizing. The contract locks `web/src/pages/EditorPage.tsx`, `BuildsPage.tsx`, `CatalogPage.tsx`, `CloudPage.tsx`, `TeamPage.tsx`, `SettingsPage.tsx`, and `WorkspacePage.tsx` around the same shared page shell, metric row, tabs, main/sidebar grid, and `HubPanel` composition. Each workspace page owns only its state projection and tab branching while shared data, input, feedback, and panel components own repeated row, empty-state, tree, status, action, and readiness chrome.

## Workspace Split Contract Cutover

`ui_workspace_split_contract.rs` now checks React/MUI workspace main/sidebar split geometry instead of historical split-state layout components. The contract locks the shared two-column `minmax(0, 1fr) minmax(...)` grids, the `@media (max-width: 1180px)` one-column collapse rule, and support-panel grouping across `web/src/pages/ProjectsDashboard.tsx`, `ProjectBrowserPage.tsx`, `ProjectDetailPage.tsx`, `EditorPage.tsx`, `BuildsPage.tsx`, `CatalogPage.tsx`, `CloudPage.tsx`, `TeamPage.tsx`, `SettingsPage.tsx`, and `WorkspacePage.tsx`.

## Global Rules Contract Cutover

`ui_global_rules_contract.rs` now checks React/MUI global guardrails instead of scanning every historical UI file. The contract scans `web/src/App.tsx`, `web/src/components`, `web/src/pages`, `web/src/theme`, and runtime data assets to keep raw Material primitive ownership in the matching wrapper family, ensure pages remain composition surfaces, keep absolute positioning out of page layouts, centralize theme/typography/global CSS, and keep the shell as the only viewport-sized routing surface.

## Navigation Contract Cutover

`ui_navigation_contract.rs` now checks the React/MUI navigation system instead of historical navigation primitives. The contract locks `web/src/components/shell/NavigationDrawer.tsx` as the permanent primary page drawer, `web/src/components/shell/TopBar.tsx` as the source-engine/user/settings navigation chrome, `web/src/components/shell/HubWindow.tsx` as the single page router, `web/src/components/inputs/HubTabs.tsx` and `HubToggle.tsx` as secondary navigation controls, and `web/src/tauri/hubApi.ts` plus `App.tsx` as the single backend action dispatch path.

## Foundation Contract Cutover

`ui_foundation_contract.rs` now anchors the Hub foundation on the Tauri/React/MUI sources instead of the historical UI component files. It checks that `Cargo.toml`, `build.rs`, `main.rs`, and `lib.rs` hard-cut the launcher to `zircon_hub::tauri_app::run()`, that `tauri.conf.json`, `capabilities/default.json`, and `vite.config.ts` keep the fixed webview window and local IPC boundary, and that `web/src/main.tsx`, `App.tsx`, theme tokens, MUI overrides, component-family barrels, shell composition, routed pages, Rust command state, quick-action routing, view-model DTOs, and runtime visual assets all line up with the bottom-up React implementation.

The same contract guards the cutover itself: it no longer defines old UI-file reader helpers, no longer points at the removed compiled app module, and proves that the old `src/app/mod.rs` and one-file Tauri launcher surfaces remain absent. Foundation coverage now treats `web/src/theme`, `components/inputs`, `components/data`, `components/feedback`, `components/overlays`, `components/shell`, `web/src/pages`, and `src/tauri_app` as the authoritative Hub UI and state boundary.

## Page Surface Contract Cutover

`ui_page_surface_coverage_contract.rs` now checks the React/MUI page surface instead of the historical page files. It locks `HubWindow` and `NavigationDrawer` routing for Projects, Editor, Assets, Builds, Plugins, Cloud, Team, Learn, Settings, and fallback workspaces; verifies Projects dashboard/browser/detail/new-project dialog coverage; and verifies Editor, Builds, Catalog, Cloud, Team, Settings, and fallback Workspace pages all compose shared data, input, feedback, overlay, and shell components.

The contract also covers target-state surfaces named in the migration plan: `HubSnackbar` and `HubStatusBanner` for running/error/status feedback, `HubDialog` for New Project, `HubPopover` plus `SourceEnginePopover` and `UserMenuPopover` for popup menus, `EmptyStateBlock` for empty states, `LinearProgress` for loading/progress surfaces, and page-level responsive constraints. It guards pages against reverting to local low-level dialog, popover, snackbar, menu, or sample-surface implementations.
