//! Static contracts for the Zircon Hub Tauri + React + Material UI shell.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_crate_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_file(path: &str) {
    assert!(
        crate_dir().join(path).is_file(),
        "expected Zircon Hub file to exist: {path}"
    );
}

fn assert_no_file(path: &str) {
    assert!(
        !crate_dir().join(path).exists(),
        "Zircon Hub hard-cut Tauri entry must not keep legacy Slint entry file: {path}"
    );
}

#[test]
fn tauri_shell_points_at_vite_react_frontend() {
    for path in [
        "tauri.conf.json",
        "package.json",
        "vite.config.ts",
        "capabilities/default.json",
        "web/index.html",
        "web/src/main.tsx",
        "web/src/App.tsx",
        "src/tauri_app/mod.rs",
        "src/tauri_app/commands.rs",
        "src/tauri_app/runtime_state.rs",
        "src/tauri_app/runtime_state/quick_actions.rs",
        "src/tauri_app/runtime_state/project_delivery_actions.rs",
        "src/tauri_app/view_model.rs",
        "icons/icon.ico",
    ] {
        assert_file(path);
    }

    let tauri_config = read_crate_file("tauri.conf.json");
    for snippet in [
        "\"devUrl\": \"http://localhost:1420\"",
        "\"frontendDist\": \"web/dist\"",
        "\"beforeDevCommand\": \"npm run dev\"",
        "\"beforeBuildCommand\": \"npm run build\"",
        "\"decorations\": false",
        "\"icon\": [\"icons/icon.ico\"]",
        "\"width\": 1568",
        "\"height\": 1003",
    ] {
        assert!(
            tauri_config.contains(snippet),
            "tauri.conf.json must describe the fixed Hub window and Vite frontend handoff; missing {snippet}"
        );
    }

    let default_capability = read_crate_file("capabilities/default.json");
    for snippet in [
        "\"$schema\": \"../gen/schemas/desktop-schema.json\"",
        "\"identifier\": \"default\"",
        "\"local\": true",
        "\"windows\": [\"main\"]",
        "\"core:default\"",
        "\"core:window:allow-minimize\"",
        "\"core:window:allow-toggle-maximize\"",
        "\"core:window:allow-close\"",
    ] {
        assert!(
            default_capability.contains(snippet),
            "capabilities/default.json must bind the local main window to explicit Tauri v2 permissions; missing {snippet}"
        );
    }

    let package_json = read_crate_file("package.json");
    for snippet in [
        "\"dev\": \"vite --host 127.0.0.1 --port 1420\"",
        "\"build\": \"tsc -b && vite build\"",
        "\"tauri:dev\": \"tauri dev\"",
        "\"@tauri-apps/api\": \"2.11.0\"",
        "\"@tauri-apps/cli\": \"2.11.2\"",
    ] {
        assert!(
            package_json.contains(snippet),
            "package.json must keep Vite/Tauri commands and packages aligned with tauri.conf.json; missing {snippet}"
        );
    }

    let tauri_app = read_crate_file("src/tauri_app/mod.rs");
    for snippet in [
        "#[tauri::command]",
        "fn hub_state(state: tauri::State<'_, HubCommandState>) -> Result<HubViewModel, String>",
        "fn hub_action(",
        "request: HubActionRequest",
        "tauri::generate_handler![hub_state, hub_action]",
        ".manage(HubCommandState::load()?)",
    ] {
        assert!(
            tauri_app.contains(snippet),
            "tauri_app/mod.rs must expose a command boundary for React state loading and actions; missing {snippet}"
        );
    }
}

#[test]
fn tauri_cutover_has_no_compiled_slint_entry_path() {
    assert_no_file("src/app/mod.rs");

    let cargo = read_crate_file("Cargo.toml");
    let build_script = read_crate_file("build.rs");
    let lib = read_crate_file("src/lib.rs");
    let main = read_crate_file("src/main.rs");

    for snippet in [
        "tauri = { version = \"2.11.2\"",
        "tauri-build = { version = \"2.6.2\"",
        "build = \"build.rs\"",
    ] {
        assert!(
            cargo.contains(snippet),
            "Cargo manifest must describe the Tauri v2 hard-cut runtime; missing {snippet}"
        );
    }

    for forbidden in [
        "slint",
        "i-slint-compiler",
        "slint-build",
        "slint::include_modules",
        "pub mod app;",
        "mod app;",
    ] {
        assert!(
            !cargo.to_ascii_lowercase().contains(forbidden)
                && !build_script.to_ascii_lowercase().contains(forbidden)
                && !lib.to_ascii_lowercase().contains(forbidden)
                && !main.to_ascii_lowercase().contains(forbidden),
            "compiled Hub entry path must not retain legacy Slint hook: {forbidden}"
        );
    }

    assert!(
        build_script.contains("tauri_build::build()"),
        "build.rs must delegate to tauri_build after the Slint build hard cut"
    );
    assert!(
        main.contains("zircon_hub::tauri_app::run()"),
        "main.rs must launch the Tauri app entrypoint directly"
    );
    assert!(
        main.contains("#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]"),
        "release Hub launcher must use the Windows GUI subsystem so Tauri screenshots are not covered by a console window"
    );
    assert!(
        lib.contains("pub mod tauri_app;") && !lib.contains("pub mod app;"),
        "lib.rs must expose tauri_app and keep the old app module out of the public root"
    );
}

#[test]
fn tauri_commands_project_backend_runtime_state_instead_of_reference_data() {
    let commands = read_crate_file("src/tauri_app/commands.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let action_tasks = read_crate_file("src/tauri_app/runtime_state/action_tasks.rs");
    let scoped_views = read_crate_file("src/tauri_app/runtime_state/scoped_views.rs");
    let build_actions = read_crate_file("src/tauri_app/runtime_state/build_actions.rs");
    let editor_launch_actions =
        read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let quick_actions = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let project_delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");
    let hub_snapshot = read_crate_file("src/state/hub_snapshot.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let action_history_dto = read_crate_file("src/tauri_app/view_model/action_history.rs");
    let tauri_tree = format!(
        "{commands}\n{runtime_state}\n{action_tasks}\n{scoped_views}\n{build_actions}\n{editor_launch_actions}\n{quick_actions}\n{project_delivery_actions}\n{view_model}\n{action_history_dto}"
    );

    for snippet in [
        "struct HubCommandState",
        "Mutex<HubRuntimeSession>",
        "app.emit(\"hub-state-changed\", view_model)",
        "HubConfig::load(&config_path)",
        "load_editor_recent_project_session(&editor_config_path)",
        "merge_recent_projects(config.recent_projects, editor_recent.recent_projects)",
        "HubSnapshot {",
        "HubViewModel::from_snapshot(&self.snapshot())",
        "discover_asset_catalog_for_scope",
        "discover_learn_catalog_for_scope",
        "discover_plugin_catalog_with_project_roots",
        "discover_team_overview",
        "mod build_actions;",
        "mod editor_launch_actions;",
        "mod quick_actions;",
        "mod project_delivery_actions;",
        "mod scoped_views;",
        "pub(super) fn build_selected_project_engine",
        "pub(in crate::tauri_app) struct PendingProjectPackage",
        "pub(in crate::tauri_app) struct PendingDeviceInstall",
        "package_project(&self.request)",
        "install_package_to_device(&install_request)",
        "run_background_worker_loop(request, &session_handle, &emit_state);",
        "pub(in crate::tauri_app) trait BackgroundTask",
        "pub(in crate::tauri_app) fn execute_background_task",
        "pub(in crate::tauri_app) struct PendingEditorLaunch",
        "launch_editor(command)?",
        "let result = pending_build.run()",
        "record_package_success",
        "ProcessMessageId::EditorExecutableUnavailable",
        "pub browser_projects: Vec<HubRecentProject>",
        "pub selected_project: Option<HubProjectDetail>",
        "pub assets: Vec<HubAssetItem>",
        "pub plugins: Vec<HubPluginItem>",
        "pub learn_resources: Vec<HubLearnItem>",
        "pub team: HubTeamSummary",
        "pub action_history: Vec<HubActionHistoryItem>",
        "pub(crate) struct HubProjectDetail",
        "pub(crate) struct HubAssetItem",
        "pub(crate) struct HubPluginItem",
        "pub(crate) struct HubLearnItem",
        "pub(crate) struct HubTeamSummary",
        "pub(crate) struct HubActionHistoryItem",
        "selected_project: selected_project_detail(snapshot)",
        "assets: asset_rows(snapshot)",
        "plugins: plugin_rows(snapshot)",
        "learn_resources: learn_rows(snapshot)",
        "team: team_summary(&snapshot.team, snapshot.settings.language)",
        "action_history: action_history_rows(",
        "snapshot.settings.language",
        "kind: record.action.id().to_string()",
        "let log_excerpt = text.render_message(&record.log_excerpt);",
        "log_excerpt,",
        "command_line: record.command_line.clone()",
    ] {
        assert!(
            tauri_tree.contains(snippet),
            "Tauri shell must derive React JSON from the existing Hub backend state model; missing {snippet}"
        );
    }

    for forbidden in [
        "reference_shell_state",
        "queue_quick_action",
        "Elysium Chronicles",
        "Stellar Outpost",
        "Sands of Time",
        "Whispering Woods",
        "ZirconProjects",
    ] {
        assert!(
            !commands.contains(forbidden)
                && !runtime_state.contains(forbidden)
                && !quick_actions.contains(forbidden)
                && !hub_snapshot.contains(forbidden),
            "Tauri command/runtime state must not keep static dashboard fixtures: {forbidden}"
        );
    }
}

#[test]
fn react_material_components_are_split_from_low_level_to_window_shell() {
    for path in [
        "web/src/theme/tokens.ts",
        "web/src/theme/muiTheme.ts",
        "web/src/types/hub.ts",
        "web/src/data/hubData.ts",
        "web/src/tauri/hubApi.ts",
        "web/src/components/inputs/HubButton.tsx",
        "web/src/components/inputs/HubCheckbox.tsx",
        "web/src/components/inputs/HubComboBox.tsx",
        "web/src/components/inputs/HubIconButton.tsx",
        "web/src/components/inputs/HubSearchField.tsx",
        "web/src/components/inputs/HubSelect.tsx",
        "web/src/components/inputs/HubSwitch.tsx",
        "web/src/components/inputs/HubTabs.tsx",
        "web/src/components/inputs/HubTextField.tsx",
        "web/src/components/inputs/HubToggle.tsx",
        "web/src/components/data/EmptyStateBlock.tsx",
        "web/src/components/data/HubList.tsx",
        "web/src/components/data/HubTreeView.tsx",
        "web/src/components/data/MetricCard.tsx",
        "web/src/components/data/ProjectCard.tsx",
        "web/src/components/data/ProjectTable.tsx",
        "web/src/components/data/QuickActions.tsx",
        "web/src/components/data/SourceEngineList.tsx",
        "web/src/components/data/StatusBadge.tsx",
        "web/src/components/feedback/HubSnackbar.tsx",
        "web/src/components/feedback/HubStatusBanner.tsx",
        "web/src/components/feedback/index.ts",
        "web/src/components/overlays/HubDialog.tsx",
        "web/src/components/overlays/HubMenu.tsx",
        "web/src/components/overlays/HubPopover.tsx",
        "web/src/components/overlays/SourceEnginePopover.tsx",
        "web/src/components/overlays/UserMenuPopover.tsx",
        "web/src/components/shell/NavigationDrawer.tsx",
        "web/src/components/shell/TopBar.tsx",
        "web/src/components/shell/HubWindow.tsx",
        "web/src/pages/ProjectBrowserPage.tsx",
        "web/src/pages/ProjectDetailPage.tsx",
        "web/src/pages/ProjectsDashboard.tsx",
        "web/src/pages/CatalogPage.tsx",
        "web/src/pages/EditorPage.tsx",
        "web/src/pages/BuildsPage.tsx",
        "web/src/pages/CloudPage.tsx",
        "web/src/pages/TeamPage.tsx",
        "web/src/pages/SettingsPage.tsx",
        "web/src/pages/WorkspacePage.tsx",
    ] {
        assert_file(path);
    }

    let inputs = read_crate_file("web/src/components/inputs/index.ts");
    for snippet in [
        "export * from \"./HubButton\";",
        "export * from \"./HubCheckbox\";",
        "export * from \"./HubComboBox\";",
        "export * from \"./HubIconButton\";",
        "export * from \"./HubSearchField\";",
        "export * from \"./HubSelect\";",
        "export * from \"./HubSwitch\";",
        "export * from \"./HubTabs\";",
        "export * from \"./HubTextField\";",
        "export * from \"./HubToggle\";",
    ] {
        assert!(
            inputs.contains(snippet),
            "input layer must centralize low-level control exports; missing {snippet}"
        );
    }

    let data = read_crate_file("web/src/components/data/index.ts");
    for snippet in [
        "export * from \"./EmptyStateBlock\";",
        "export * from \"./HubList\";",
        "export * from \"./HubTreeView\";",
        "export * from \"./MetricCard\";",
        "export * from \"./ProjectCard\";",
        "export * from \"./ProjectTable\";",
        "export * from \"./QuickActions\";",
        "export * from \"./SourceEngineList\";",
        "export * from \"./StatusBadge\";",
        "export * from \"./HubPanel\";",
    ] {
        assert!(
            data.contains(snippet),
            "data-display layer must centralize card, table, list, panel, and badge exports; missing {snippet}"
        );
    }

    let feedback = read_crate_file("web/src/components/feedback/index.ts");
    for snippet in [
        "export * from \"./HubSnackbar\";",
        "export * from \"./HubStatusBanner\";",
    ] {
        assert!(
            feedback.contains(snippet),
            "feedback layer must centralize snackbar and status-banner exports; missing {snippet}"
        );
    }

    let overlays = read_crate_file("web/src/components/overlays/index.ts");
    for snippet in [
        "export * from \"./HubDialog\";",
        "export * from \"./HubMenu\";",
        "export * from \"./HubPopover\";",
        "export * from \"./SourceEnginePopover\";",
        "export * from \"./UserMenuPopover\";",
    ] {
        assert!(
            overlays.contains(snippet),
            "overlay layer must centralize dialog and menu exports; missing {snippet}"
        );
    }

    let shell = read_crate_file("web/src/components/shell/HubWindow.tsx");
    for snippet in [
        "<TopBar state={state} onAction={onAction} />",
        "<NavigationDrawer",
        "onAction={onAction}",
        "projects: ProjectsDashboard,",
        "projects: ProjectsDashboard,",
        "editor: EditorPage,",
        "editor: EditorPage,",
        "builds: BuildsPage,",
        "builds: BuildsPage,",
        "cloud: CloudPage,",
        "cloud: CloudPage,",
        "assets: CatalogPage,",
        "plugins: CatalogPage,",
        "learn: CatalogPage,",
        "assets: CatalogPage,",
        "team: TeamPage,",
        "team: TeamPage,",
        "settings: SettingsPage,",
        "settings: SettingsPage,",
        "<PageComponent state={state} onAction={onAction} />",
    ] {
        assert!(
            shell.contains(snippet),
            "HubWindow must assemble only shell chrome and page regions; missing {snippet}"
        );
    }

    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    for snippet in [
        "useState<HTMLElement | null>",
        "SourceEnginePopover",
        "UserMenuPopover",
        "setEngineAnchor(event.currentTarget)",
        "setUserAnchor(event.currentTarget)",
        "onAction(HUB_ACTION.selectEngine, engineId)",
        "onAction(HUB_ACTION.showPage, \"settings\")",
        "onAction(HUB_ACTION.showPage, \"learn\")",
        "onAction(HUB_ACTION.showPage, \"team\")",
        "const notificationDetail = comingSoonDetail(state, \"notification-center\");",
        "HubIconButton label={state.ui.shell.notifications} tooltip={notificationDetail} disabled",
        "HubIconButton label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, \"learn\")}",
    ] {
        assert!(
            topbar.contains(snippet),
            "TopBar must route source-engine and user-menu popups through reusable overlay components; missing {snippet}"
        );
    }

    let hub_popover = read_crate_file("web/src/components/overlays/HubPopover.tsx");
    for snippet in [
        "<Popover",
        "anchorEl={anchorEl}",
        "open={open}",
        "backgroundColor: \"rgba(25,29,29,0.98)\"",
        "border: `1px solid ${hubTokens.colors.lineStrong}`",
    ] {
        assert!(
            hub_popover.contains(snippet),
            "HubPopover must centralize Material Popover surface styling and anchoring; missing {snippet}"
        );
    }

    let source_popup = read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    for snippet in [
        "HubPopover",
        "{text.activeEngine}",
        "{text.readyFallback}",
        "{text.localDefaults}",
        "EngineRow",
        "PathRow",
        "StatusBadge",
        "onSelect(engine.id)",
        "onManage",
    ] {
        assert!(
            source_popup.contains(snippet),
            "SourceEnginePopover must compose a rich source-engine selector from backend engine/settings state; missing {snippet}"
        );
    }

    let user_popup = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");
    for snippet in [
        "HubPopover",
        "menuItems",
        "text.userAccount",
        "text.preferences",
        "text.documentation",
        "text.signOut",
        "disabled={isDisabled}",
        "if (isDisabled) {",
        "onAction(id)",
    ] {
        assert!(
            user_popup.contains(snippet),
            "UserMenuPopover must centralize topbar user menu content and actions; missing {snippet}"
        );
    }

    let page = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let projects_toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");
    let create_project_dialog =
        read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");
    for snippet in [
        "ProjectBrowserPage",
        "ProjectDetailPage",
        "ProjectCard",
        "ProjectTable",
        "QuickActions",
        "ProjectsToolbar",
        "CreateProjectDialog",
        "EmptyStateBlock",
        "state.projectSubpage === \"project-browser\"",
        "state.projectSubpage === \"project-detail\"",
        "onAction(HUB_ACTION.openProjectDetail, project.id)",
    ] {
        assert!(
            page.contains(snippet),
            "ProjectsDashboard must compose shared components instead of page-local control markup; missing {snippet}"
        );
    }
    for snippet in ["HubSearchField", "HubSelect", "HubToggle"] {
        assert!(
            projects_toolbar.contains(snippet),
            "ProjectsToolbar must own shared project filter controls; missing {snippet}"
        );
    }
    for snippet in [
        "onAction(HUB_ACTION.setProjectFilter, value)",
        "onAction(HUB_ACTION.setProjectSort, value)",
        "onAction(HUB_ACTION.setProjectViewMode, value)",
    ] {
        assert!(
            page.contains(snippet),
            "ProjectsDashboard must route project toolbar actions; missing {snippet}"
        );
    }
    for snippet in ["HubDialog", "HubTextField", "HubComboBox"] {
        assert!(
            create_project_dialog.contains(snippet),
            "CreateProjectDialog must own project creation form controls; missing {snippet}"
        );
    }
    for obsolete in ["ButtonStatesPanel", "text.buttonStates"] {
        assert!(
            !page.contains(obsolete),
            "ProjectsDashboard must not reintroduce visual button-state sample strips; found {obsolete}"
        );
    }

    let project_table = read_crate_file("web/src/components/data/ProjectTable.tsx");
    for snippet in [
        "selectedProjectId: string | null",
        "onSelect?: (project: HubRecentProject) => void",
        "onOpenDetail?: (project: HubRecentProject) => void",
        "selected={selected}",
        "onSelect?.(project)",
        "onOpenDetail?.(project)",
    ] {
        assert!(
            project_table.contains(snippet),
            "ProjectTable must centralize selectable row and detail affordance behavior; missing {snippet}"
        );
    }

    let browser_page = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    for snippet in [
        "state.browserProjects",
        "HubSearchField",
        "HubSelect",
        "HubToggle",
        "ProjectTable",
        "QuickActions",
        "SourceEngineList",
        "onAction(HUB_ACTION.openProjectDetail, project.id)",
        "onAction(HUB_ACTION.selectProject, project.id)",
    ] {
        assert!(
            browser_page.contains(snippet),
            "ProjectBrowserPage must compose shared controls and table routing from backend project state; missing {snippet}"
        );
    }

    let detail_page = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let detail_metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");
    let detail_sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");
    for snippet in [
        "state.selectedProject",
        "HubTabs",
        "ProjectCover",
        "HubList",
        "HubTreeView",
        "ProjectMetricsGrid",
        "ProjectDetailSidebar",
        "StatusBadge",
        "onAction(HUB_ACTION.viewAllProjects)",
        "projectTargetPayload(project)",
    ] {
        assert!(
            detail_page.contains(snippet),
            "ProjectDetailPage must compose shared detail, tree, quick-action, and status components from selectedProject; missing {snippet}"
        );
    }
    for snippet in ["MetricCard"] {
        assert!(
            detail_metrics.contains(snippet),
            "ProjectMetricsGrid must own selected-project metric cards; missing {snippet}"
        );
    }
    for snippet in [
        "QuickActions",
        "SourceEngineList",
        "onAction(HUB_ACTION.packageProject, undefined, projectTarget)",
    ] {
        assert!(
            detail_sidebar.contains(snippet),
            "ProjectDetailSidebar must own selected-project side actions; missing {snippet}"
        );
    }

    let types = read_crate_file("web/src/types/hub.ts");
    for snippet in [
        "export interface HubProjectDetail",
        "export interface HubAssetItem",
        "export interface HubPluginItem",
        "export interface HubLearnItem",
        "export interface HubTeamSummary",
        "export type HubActionHistoryKind",
        "export interface HubActionHistoryItem",
        "kind: HubActionHistoryKind;",
        "browserProjects: HubRecentProject[]",
        "selectedProject: HubProjectDetail | null",
        "assets: HubAssetItem[]",
        "plugins: HubPluginItem[]",
        "learnResources: HubLearnItem[]",
        "team: HubTeamSummary",
        "actionHistory: HubActionHistoryItem[]",
    ] {
        assert!(
            types.contains(snippet),
            "React Hub types must expose project, catalog, team, and action-history DTOs; missing {snippet}"
        );
    }

    let catalog_page = read_crate_file("web/src/pages/CatalogPage.tsx");
    for snippet in [
        "state.assets",
        "state.plugins",
        "state.learnResources",
        "HubSearchField",
        "HubTabs",
        "HubButton",
        "HubList",
        "HubTreeView",
        "MetricCard",
        "EmptyStateBlock",
        "QuickActions",
        "SourceEngineList",
        "StatusBadge",
        "state.ui.actions.openResource",
        "HUB_ACTION.openResource",
        "onAction(HUB_ACTION.openResource, undefined, { resourceId: row.id, path: row.path })",
    ] {
        assert!(
            catalog_page.contains(snippet),
            "CatalogPage must compose Assets, Plugins, and Learn catalogs from shared MUI components; missing {snippet}"
        );
    }

    let team_page = read_crate_file("web/src/pages/TeamPage.tsx");
    for snippet in [
        "state.team",
        "state.actionHistory",
        "HubTabs",
        "HubList",
        "HubTreeView",
        "MetricCard",
        "EmptyStateBlock",
        "QuickActions",
        "SourceEngineList",
        "StatusBadge",
        "action.detailRows",
    ] {
        assert!(
            team_page.contains(snippet),
            "TeamPage must compose team identity, members, source engines, and action history from shared MUI components; missing {snippet}"
        );
    }

    let editor_page = read_crate_file("web/src/pages/EditorPage.tsx");
    for snippet in [
        "state.selectedProject",
        "state.sourceEngines",
        "state.plugins",
        "state.comingSoon.filter((entry) => entry.category === \"plugins\")",
        "state.actionHistory",
        "HubTabs",
        "HubList",
        "HubTreeView",
        "MetricCard",
        "EmptyStateBlock",
        "QuickActions",
        "SourceEngineList",
        "StatusBadge",
        "onAction(HUB_ACTION.openEditor",
        "onAction(HUB_ACTION.selectEngine, engine.id)",
        "HubPanel title={text.pluginComingSoonPanel}",
        "disabled: entry.disabled",
    ] {
        assert!(
            editor_page.contains(snippet),
            "EditorPage must compose editor launch, source-engine, plugin, reserved operation, and activity state from shared MUI components; missing {snippet}"
        );
    }

    let builds_page = read_crate_file("web/src/pages/BuildsPage.tsx");
    for snippet in [
        "state.selectedProject",
        "state.actionHistory",
        "state.settings.defaultBuildOutputDir",
        "HubTabs",
        "HubList",
        "HubTreeView",
        "MetricCard",
        "EmptyStateBlock",
        "QuickActions",
        "SourceEngineList",
        "StatusBadge",
        "LinearProgress",
        "onAction(HUB_ACTION.buildProject",
        "onAction(HUB_ACTION.packageProject",
        "onAction(HUB_ACTION.installDevice",
    ] {
        assert!(
            builds_page.contains(snippet),
            "BuildsPage must compose build/package/install workflows and history from shared MUI components; missing {snippet}"
        );
    }

    let cloud_page = read_crate_file("web/src/pages/CloudPage.tsx");
    for snippet in [
        "state.actionHistory",
        "state.settings.defaultDeviceInstallDir",
        "state.settings.defaultBuildOutputDir",
        "HubTabs",
        "HubList",
        "HubTreeView",
        "MetricCard",
        "EmptyStateBlock",
        "QuickActions",
        "StatusBadge",
        "onAction(HUB_ACTION.packageProject",
        "onAction(HUB_ACTION.installDevice",
    ] {
        assert!(
            cloud_page.contains(snippet),
            "CloudPage must compose local package/install slots and reserved services from shared MUI components; missing {snippet}"
        );
    }

    let settings_page = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    for snippet in [
        "settingsText.heading",
        "settingsDraftState(state)",
        "HubStatusBanner",
        "MetricCard",
        "HubTabs",
        "SettingsSection",
        "onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })",
        "onAction(HUB_ACTION.browseSettingsFolder, field, { field, initialDir, settings: draft })",
    ] {
        assert!(
            settings_page.contains(snippet),
            "SettingsPage must compose the settings shell and route actions; missing {snippet}"
        );
    }
    for snippet in [
        "HubComboBox",
        "HubTextField",
        "HubIconButton",
        "HubSwitch",
        "HubCheckbox",
        "HubList",
        "HubTreeView",
        "SourceEngineList",
        "StatusBadge",
        "LinearProgress",
        "onAction(HUB_ACTION.selectEngine, engine.id)",
        "state.ui.actions.browseFolder",
        "settingsText.buildProfileOptions",
        "settingsText.languageOptions",
    ] {
        assert!(
            settings_section.contains(snippet),
            "SettingsSection must own focused MUI settings controls; missing {snippet}"
        );
    }

    let workspace_page = read_crate_file("web/src/pages/WorkspacePage.tsx");
    for snippet in [
        "HubPanel",
        "QuickActions",
        "SourceEngineList",
        "MetricCard",
        "HubTabs",
        "HubList",
        "HubTreeView",
        "HubSwitch",
        "HubCheckbox",
        "HubStatusBanner",
        "state.settings.defaultProjectDir",
        "onAction(HUB_ACTION.selectEngine, engine.id)",
    ] {
        assert!(
            workspace_page.contains(snippet),
            "WorkspacePage must compose shared panels, source-engine lists, settings rows, and quick actions; missing {snippet}"
        );
    }

    let app = read_crate_file("web/src/App.tsx");
    for snippet in [
        "dispatchHubAction",
        "subscribeHubStateChanged",
        "unlisten?.()",
        "stateRef.current.ui.shell",
        "stateGenerationRef",
        "actionSequenceRef",
        "applyHubState(nextState)",
        "shellText.liveUpdatesUnavailable",
        "shellText.actionFailed",
        "shellText.stateRefreshAfterCommand",
        "shellText.checkActionTarget",
        "setState((current) => ({",
        "<HubWindow state={state} onAction={handleAction} />",
        "<HubSnackbar task={state.taskSummary}",
    ] {
        assert!(
            app.contains(snippet),
            "React shell must refresh the composed window after Tauri actions; missing {snippet}"
        );
    }
    assert!(
        !app.contains("setState(nextState);"),
        "React shell must guard direct Tauri action replies instead of unconditionally replacing state"
    );

    let hub_api = read_crate_file("web/src/tauri/hubApi.ts");
    for snippet in [
        "import { listen } from \"@tauri-apps/api/event\";",
        "import type { UnlistenFn } from \"@tauri-apps/api/event\";",
        "export async function subscribeHubStateChanged",
        "Promise<UnlistenFn>",
        "return () => {};",
        "listen<unknown>(\"hub-state-changed\"",
        "onStateChanged(assertHubShellState(event.payload))",
        "Ignored invalid hub-state-changed payload.",
    ] {
        assert!(
            hub_api.contains(snippet),
            "hubApi.ts must subscribe React to Tauri backend state-change events; missing {snippet}"
        );
    }

    let dispatch = hub_api
        .split("export async function dispatchHubAction")
        .nth(1)
        .expect("hubApi.ts must define dispatchHubAction")
        .split("export async function subscribeHubStateChanged")
        .next()
        .expect("dispatchHubAction must be followed by subscribeHubStateChanged");
    assert!(
        !dispatch.contains("catch"),
        "dispatchHubAction must not fall back to demo state on Tauri action errors; App owns visible error feedback"
    );
}

#[test]
fn hub_visual_assets_are_runtime_assets_not_reference_screenshots() {
    let data = read_crate_file("web/src/data/hubData.ts");
    for snippet in [
        "../../../assets/brand/zircon-mark.svg",
        "../../../assets/covers/reference/project-elysium.png",
        "../../../assets/covers/reference/project-stellar-outpost.png",
        "../../../assets/covers/reference/project-sands-of-time.png",
        "../../../assets/covers/reference/project-whispering-woods.png",
    ] {
        assert!(
            data.contains(snippet),
            "React shell must use Hub runtime asset family for visible project media; missing {snippet}"
        );
    }

    for forbidden in [
        "docs/ui-and-layout/hub.png",
        "hub-ai-drafts",
        "hub-web-reference-1568x1003.png",
    ] {
        assert!(
            !data.contains(forbidden),
            "React shell must not render final/reference screenshots as runtime UI assets: {forbidden}"
        );
    }
}
