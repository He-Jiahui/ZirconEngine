//! Static contracts that real Hub pages are covered by React/MUI surfaces.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {path}: {error}");
        }),
    )
}

fn assert_contains_all(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{label} must contain page-surface snippet: {snippet}"
        );
    }
}

fn assert_not_contains_any(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{label} must not contain obsolete page-surface snippet: {snippet}"
        );
    }
}

fn page_source(name: &str) -> String {
    read_crate_file(&format!("web/src/pages/{name}.tsx"))
}

#[test]
fn react_window_routes_every_primary_page_surface() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        &hub_window,
        &[
            "import { BuildsPage } from \"../../pages/BuildsPage\";",
            "import { CatalogPage } from \"../../pages/CatalogPage\";",
            "import { CloudPage } from \"../../pages/CloudPage\";",
            "import { EditorPage } from \"../../pages/EditorPage\";",
            "import { ProjectsDashboard } from \"../../pages/ProjectsDashboard\";",
            "import { SettingsPage } from \"../../pages/SettingsPage\";",
            "import { TeamPage } from \"../../pages/TeamPage\";",
            "import { WorkspacePage } from \"../../pages/WorkspacePage\";",
            "projects: ProjectsDashboard,",
            "editor: EditorPage,",
            "builds: BuildsPage,",
            "cloud: CloudPage,",
            "assets: CatalogPage,",
            "plugins: CatalogPage,",
            "learn: CatalogPage,",
            "team: TeamPage,",
            "settings: SettingsPage,",
            "<PageComponent state={state} onAction={onAction} />",
            "component=\"main\"",
            "height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`",
        ],
        "HubWindow",
    );

    assert_contains_all(
        &drawer,
        &[
            "text.navItems.map(({ id, label }) =>",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "const [collapsed, setCollapsed] = useState(false);",
            "const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;",
            "width: drawerWidth",
            "{text.engineStatus}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "onClick={() => setCollapsed((current) => !current)}",
        ],
        "NavigationDrawer",
    );
}

#[test]
fn projects_dashboard_does_not_render_visual_button_state_reference_strip() {
    let dashboard = page_source("ProjectsDashboard");

    assert_not_contains_any(
        &dashboard,
        &[
            "ButtonStatesPanel",
            "text.buttonStates",
            "buttonStatePrimary",
            "buttonStateSecondary",
            "buttonStateTertiary",
            "buttonStateIcon",
        ],
        "ProjectsDashboard",
    );
}

#[test]
fn project_surfaces_cover_dashboard_browser_detail_and_new_project_dialog() {
    let dashboard = page_source("ProjectsDashboard");
    let projects_toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");
    let create_project = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");
    let browser = page_source("ProjectBrowserPage");
    let detail = page_source("ProjectDetailPage");
    let metrics_grid = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");
    let detail_sidebar = read_crate_file("web/src/components/data/ProjectDetailSidebar.tsx");

    assert_contains_all(
        &dashboard,
        &[
            "if (state.projectSubpage === \"project-browser\")",
            "return <ProjectBrowserPage state={state} onAction={onAction} />;",
            "if (state.projectSubpage === \"project-detail\")",
            "return <ProjectDetailPage state={state} onAction={onAction} />;",
            "ProjectsToolbar",
            "ProjectCard",
            "ProjectTable",
            "QuickActions",
            "EmptyStateBlock",
            "CreateProjectDialog",
            "open={state.projectSubpage === \"new-project\"}",
            "onAction(HUB_ACTION.newProject)",
            "onAction(HUB_ACTION.openProjectDetail, project.id)",
            "onAction(HUB_ACTION.setProjectViewMode, value)",
        ],
        "ProjectsDashboard",
    );
    assert_contains_all(
        &projects_toolbar,
        &[
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "{ value: \"grid\", label: text.gridView",
            "{ value: \"list\", label: text.listView",
        ],
        "ProjectsToolbar",
    );
    assert_contains_all(
        &create_project,
        &[
            "HubDialog",
            "open={open}",
            "title={text.newProjectDialog}",
            "HubTextField label={text.projectName}",
            "HubTextField label={text.location}",
            "HubComboBox",
        ],
        "CreateProjectDialog",
    );
    assert_contains_all(
        &browser,
        &[
            "export function ProjectBrowserPage",
            "HubStatusBanner",
            "HubSearchField",
            "HubSelect",
            "HubToggle",
            "ProjectTable",
            "SourceEngineList",
            "QuickActions",
            "EmptyStateBlock title={text.noProjectsFound}",
            "onAction(HUB_ACTION.searchProjects, undefined, { query: value })",
            "onAction(HUB_ACTION.setProjectFilter, value)",
            "onAction(HUB_ACTION.setProjectSort, value)",
            "onAction(HUB_ACTION.selectProject, project.id)",
            "onAction(HUB_ACTION.openProjectDetail, project.id)",
            "onAction(HUB_ACTION.selectEngine, engine.id)",
        ],
        "ProjectBrowserPage",
    );
    assert_contains_all(
        &detail,
        &[
            "export function ProjectDetailPage",
            "const project = state.selectedProject ?? null;",
            "HubStatusBanner",
            "ProjectMetricsGrid",
            "ProjectDetailSidebar",
            "HubTabs",
            "ProjectCover",
            "HubList",
            "HubTreeView",
            "StatusBadge",
            "QuickActions",
            "EmptyStateBlock title={text.noProjectSelected}",
            "{ value: \"overview\", label: text.overview }",
            "{ value: \"files\", label: text.files }",
            "{ value: \"actions\", label: text.actions }",
            "projectTargetPayload(project)",
            "onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
        ],
        "ProjectDetailPage",
    );
    assert_contains_all(
        &metrics_grid,
        &[
            "MetricCard",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
        ],
        "ProjectMetricsGrid",
    );
    assert_contains_all(
        &detail_sidebar,
        &[
            "QuickActions",
            "SourceEngineList",
            "onAction(HUB_ACTION.packageProject, undefined, projectTarget)",
            "onAction(HUB_ACTION.installDevice, undefined, projectTarget)",
        ],
        "ProjectDetailSidebar",
    );
}

#[test]
fn workspace_pages_cover_editor_build_catalog_cloud_team_and_settings_states() {
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    for (page, snippets) in [
        (
            "EditorPage",
            &[
                "HubStatusBanner",
                "MetricCard",
                "HubTabs",
                "HubPanel title={text.launchTarget}",
                "HubPanel title={text.editorPluginScope}",
                "HubPanel title={text.editorActivity}",
                "HubPanel title={text.launchReadiness}",
                "SourceEngineList",
                "QuickActions",
                "HubTreeView",
                "HubSwitch",
                "HubCheckbox",
                "EmptyStateBlock title={text.noProjectSelectedTitle}",
                "EmptyStateBlock title={text.noEditorPluginsTitle}",
                "EmptyStateBlock title={text.noEditorActivityTitle}",
                "projectTargetPayload(project)",
                "onAction(HUB_ACTION.openEditor, undefined, projectTarget)",
            ][..],
        ),
        (
            "BuildsPage",
            &[
                "HubStatusBanner",
                "LinearProgress",
                "MetricCard",
                "HubTabs",
                "HubPanel title={text.buildWorkflow}",
                "HubPanel title={common.selectedProject}",
                "HubPanel title={text.buildHistory}",
                "HubPanel title={text.outputTree}",
                "BuildActionDetail",
                "HubTreeView",
                "QuickActions",
                "SourceEngineList",
                "EmptyStateBlock title={common.noProjectSelected}",
                "EmptyStateBlock title={text.noBuildHistory}",
                "EmptyStateBlock title={text.noWorkflowSelected}",
                "workflowProjectTargetPayload(state)",
                "workflowTargetProject(state)",
                "onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)",
                "onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)",
                "onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)",
            ][..],
        ),
        (
            "CatalogPage",
            &[
                "HubStatusBanner",
                "HubSearchField",
                "MetricCard",
                "HubTabs",
                "HubPanel title={catalogPanelTitle(mode, text)}",
                "HubPanel title={text.selectedEntry}",
                "HubPanel title={text.catalogTree}",
                "QuickActions",
                "SourceEngineList",
                "HubTreeView",
                "StatusBadge",
                "EmptyStateBlock title={text.noEntriesFound}",
                "EmptyStateBlock title={text.noCatalogEntrySelected}",
                "state.plugins.map",
                "state.learnResources.map",
                "state.assets.map",
            ][..],
        ),
        (
            "CloudPage",
            &[
                "HubStatusBanner",
                "MetricCard",
                "HubTabs",
                "HubPanel title={text.packageOutputs}",
                "HubPanel title={text.deviceInstalls}",
                "HubPanel title={text.reservedServices}",
                "HubPanel title={text.localDeliveryTree}",
                "HubPanel title={text.currentStatus}",
                "HubSwitch",
                "HubCheckbox",
                "HubTreeView",
                "QuickActions",
                "StatusBadge",
                "EmptyStateBlock title={text.noPackagesRecorded}",
                "EmptyStateBlock title={text.noInstallsRecorded}",
                "workflowProjectTargetPayload(state)",
                "workflowTargetProject(state)",
                "onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)",
                "onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)",
            ][..],
        ),
        (
            "TeamPage",
            &[
                "HubStatusBanner",
                "MetricCard",
                "HubTabs",
                "HubPanel title={text.teamMembers}",
                "HubPanel title={text.repositoryIdentity}",
                "HubPanel title={text.teamTree}",
                "HubPanel title={text.actionHistory}",
                "HubPanel title={text.latestAction}",
                "HubPanel title={common.sourceEngines}",
                "QuickActions",
                "SourceEngineList",
                "HubTreeView",
                "ActionDetail",
                "StatusBadge",
                "EmptyStateBlock title={text.noTeamMembersFound}",
                "EmptyStateBlock title={text.noRecentActions}",
                "EmptyStateBlock title={text.noActionSelected}",
            ][..],
        ),
        (
            "SettingsPage",
            &[
                "HubStatusBanner",
                "MetricCard",
                "HubTabs",
                "SettingsSection",
                "onAction(HUB_ACTION.saveSettings",
            ][..],
        ),
        (
            "WorkspacePage",
            &[
                "HubStatusBanner",
                "MetricCard",
                "HubTabs",
                "HubPanel title={common.sourceEngines}",
                "HubPanel title={settingsText.heading}",
                "HubPanel title={settingsText.advancedConfigurationPanel}",
                "HubPanel title={state.ui.editor.workspaceTree}",
                "HubList",
                "HubTreeView",
                "QuickActions",
                "SourceEngineList",
                "HubSwitch",
                "HubCheckbox",
                "onAction(HUB_ACTION.showPage, \"settings\")",
            ][..],
        ),
    ] {
        let source = page_source(page);
        assert_contains_all(&source, snippets, page);
        assert!(
            source.contains("@media (max-width:"),
            "{page} must keep responsive page-surface constraints"
        );
    }
    assert_contains_all(
        &settings_section,
        &[
            "LinearProgress",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            "HubPanel title={settingsText.configurationPathsPanel}",
            "HubPanel title={settingsText.sourceEnginesPanel}",
            "HubPanel title={settingsText.pathDefaultsPanel}",
            "HubPanel title={settingsText.advancedConfigurationPanel}",
            "HubPanel title={settingsText.configurationHealthPanel}",
            "HubComboBox",
            "HubTextField",
            "HubSwitch",
            "HubCheckbox",
            "HubTreeView",
            "SourceEngineList",
            "StatusBadge",
        ],
        "SettingsSection",
    );
}

#[test]
fn workspace_fallback_routes_to_settings_instead_of_saving_without_draft() {
    let workspace = page_source("WorkspacePage");
    let settings = page_source("SettingsPage");

    assert_contains_all(
        &workspace,
        &[
            "startIcon={<SettingsOutlinedIcon />}",
            "onClick={() => void onAction(HUB_ACTION.showPage, \"settings\")}",
            "{state.ui.shell.settings}",
        ],
        "WorkspacePage",
    );
    assert_not_contains_any(
        &workspace,
        &[
            "SaveOutlinedIcon",
            "onAction(HUB_ACTION.saveSettings)",
            "{settingsText.saveButton}",
        ],
        "WorkspacePage",
    );
    assert_contains_all(
        &settings,
        &["onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })"],
        "SettingsPage",
    );
}

#[test]
fn feedback_popups_and_state_surfaces_cover_menu_empty_loading_and_error_cases() {
    let app = read_crate_file(&["web/src", "App.tsx"].join("/"));
    let snackbar = read_crate_file("web/src/components/feedback/HubSnackbar.tsx");
    let status_banner = read_crate_file("web/src/components/feedback/HubStatusBanner.tsx");
    let empty_state = read_crate_file("web/src/components/data/EmptyStateBlock.tsx");
    let dialog = read_crate_file("web/src/components/overlays/HubDialog.tsx");
    let popover = read_crate_file("web/src/components/overlays/HubPopover.tsx");
    let source_engine_popover =
        read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    let user_menu = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");
    let top_bar = read_crate_file("web/src/components/shell/TopBar.tsx");

    assert_contains_all(
        &app,
        &[
            "HubSnackbar",
            "state.taskSummary.running || state.taskSummary.tone !== \"neutral\" || state.taskSummary.recovery",
            "setSnackbarOpen(true)",
            "label: shellText.actionFailed",
            "tone: \"error\"",
            "recovery: shellText.checkActionTarget",
            "operation: shellText.actionFailed",
            "taskStatus: current.taskStatus.map",
        ],
        "App",
    );
    assert_contains_all(
        &snackbar,
        &[
            "import { Alert, Box, Snackbar, Typography } from \"@mui/material\";",
            "task.tone === \"neutral\" || task.tone === \"running\" ? \"info\" : task.tone",
            "autoHideDuration={4200}",
            "anchorOrigin={{ vertical: \"bottom\", horizontal: \"right\" }}",
            "variant=\"filled\"",
            "<Typography variant=\"subtitle2\">{task.label}</Typography>",
            "<Typography variant=\"body2\">{task.detail}</Typography>",
            "{task.recovery ? (",
            "{task.recovery}",
        ],
        "HubSnackbar",
    );
    assert_contains_all(
        &status_banner,
        &[
            "import { Alert, Box, LinearProgress, Typography } from \"@mui/material\";",
            "task.tone === \"neutral\" || task.tone === \"running\" ? \"info\" : task.tone",
            "variant=\"outlined\"",
            "const shouldShowProgress = task.running || task.progressPercent > 0;",
            "variant=\"determinate\"",
            "value={task.progressPercent}",
            "{task.operation}",
            "{task.recovery}",
        ],
        "HubStatusBanner",
    );
    assert_contains_all(
        &empty_state,
        &[
            "minHeight: 148",
            "placeItems: \"center\"",
            "border: `1px dashed ${hubTokens.colors.lineStrong}`",
            "Typography variant=\"body2\"",
            "Typography variant=\"caption\"",
        ],
        "EmptyStateBlock",
    );
    assert_contains_all(
        &dialog,
        &[
            "import { Dialog, DialogActions, DialogContent, DialogTitle } from \"@mui/material\";",
            "open={open}",
            "onClose={onClose}",
            "maxWidth=\"sm\"",
            "fullWidth",
            "DialogTitle",
            "DialogContent",
            "DialogActions",
        ],
        "HubDialog",
    );
    assert_contains_all(
        &popover,
        &[
            "import { Box, Popover } from \"@mui/material\";",
            "anchorEl={anchorEl}",
            "open={open}",
            "onClose={onClose}",
            "width = 340",
            "maxWidth: \"calc(100vw - 32px)\"",
            "backgroundColor: \"rgba(25,29,29,0.98)\"",
        ],
        "HubPopover",
    );
    assert_contains_all(
        &source_engine_popover,
        &[
            "HubPopover anchorEl={anchorEl} open={open} width={388}",
            "{text.activeEngine}",
            "{text.readyFallback}",
            "{text.localDefaults}",
            "{text.noSourceEngineRegistered}",
            "{text.noFallbackEngineConfigured}",
            "{text.manageEngines}",
            "StatusBadge label={activeLabel} tone=\"success\"",
            "onSelect(engine.id)",
        ],
        "SourceEnginePopover",
    );
    assert_contains_all(
        &user_menu,
        &[
            "HubPopover anchorEl={anchorEl} open={open} width={284} align=\"right\"",
            "{ id: \"account\", label: text.userAccount",
            "{ id: \"preferences\", label: text.preferences",
            "{ id: \"documentation\", label: text.documentation",
            "{ id: \"sign-out\", label: text.signOut, detail: signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }",
            "const isDisabled = Boolean(disabled);",
            "disabled={isDisabled}",
            "if (isDisabled) {",
            "onAction(id);",
            "onClose();",
        ],
        "UserMenuPopover",
    );
    assert_contains_all(
        &top_bar,
        &[
            "SourceEnginePopover",
            "UserMenuPopover",
            "setEngineAnchor(event.currentTarget)",
            "setUserAnchor(event.currentTarget)",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "void onAction(HUB_ACTION.showPage, \"settings\");",
            "void onAction(HUB_ACTION.showPage, \"learn\");",
            "void onAction(HUB_ACTION.showPage, \"team\");",
            "signOutDetail={signOutDetail}",
        ],
        "TopBar",
    );
}

#[test]
fn pages_stay_composition_surfaces_instead_of_redeclaring_low_level_controls() {
    let pages = [
        "ProjectsDashboard",
        "ProjectBrowserPage",
        "ProjectDetailPage",
        "EditorPage",
        "BuildsPage",
        "CatalogPage",
        "CloudPage",
        "TeamPage",
        "SettingsPage",
        "WorkspacePage",
    ];

    for page in pages {
        let source = page_source(page);
        assert_contains_all(
            &source,
            &[
                "import { Box",
                "from \"@mui/material\";",
                "from \"../components/data\"",
                "from \"../theme/tokens\"",
                "import type",
                "export interface",
                &format!("export function {page}"),
                "HubShellState",
                "@media (max-width:",
            ],
            page,
        );
        assert_not_contains_any(
            &source,
            &[
                "from \"../data/hubData\"",
                "import { Dialog",
                "import { Popover",
                "import { Snackbar",
                "import { Menu",
                "function Local",
                "const styles =",
            ],
            page,
        );
    }
}

#[test]
fn page_surface_contract_is_cut_over_to_react_sources() {
    let source = read_crate_file("tests/ui_page_surface_coverage_contract.rs");
    let obsolete_ui_suffix = [".", "slint"].concat();
    let obsolete_reader = ["read", "_ui", "_file"].concat();
    let obsolete_root_reader = ["ui", "_dir"].concat();
    let obsolete_app_path = ["src", "app"].join("/");

    for obsolete in [
        obsolete_ui_suffix.as_str(),
        obsolete_reader.as_str(),
        obsolete_root_reader.as_str(),
        obsolete_app_path.as_str(),
    ] {
        assert!(
            !source.contains(obsolete),
            "page-surface contract must not inspect removed UI-file or app-module surfaces: {obsolete}"
        );
    }

    assert_contains_all(
        &source,
        &[
            "web/src/pages/",
            "HubWindow.tsx",
            "NavigationDrawer.tsx",
            "HubSnackbar.tsx",
            "HubStatusBanner.tsx",
            "HubDialog.tsx",
            "CreateProjectDialog.tsx",
            "HubPopover.tsx",
            "SourceEnginePopover.tsx",
            "UserMenuPopover.tsx",
            "EmptyStateBlock.tsx",
            "ProjectsToolbar.tsx",
            "ProjectMetricsGrid.tsx",
            "ProjectDetailSidebar.tsx",
            "SettingsSection.tsx",
            "ProjectsDashboard",
            "ProjectBrowserPage",
            "ProjectDetailPage",
            "SettingsPage",
            "BuildsPage",
            "CatalogPage",
            "CloudPage",
            "TeamPage",
            "EditorPage",
            "WorkspacePage",
        ],
        "page-surface contract",
    );
}
