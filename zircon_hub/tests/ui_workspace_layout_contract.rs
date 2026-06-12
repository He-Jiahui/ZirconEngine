//! Static contracts for React + Material UI workspace page layouts.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain workspace layout contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete or page-local workspace layout snippet {snippet:?}"
        );
    }
}

#[test]
fn workspace_pages_share_responsive_mui_page_shells() {
    for (page, metric_grid, main_grid, minimum_panel_count) in [
        (
            "EditorPage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            7,
        ),
        (
            "BuildsPage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            7,
        ),
        (
            "CatalogPage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            5,
        ),
        (
            "CloudPage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            8,
        ),
        (
            "TeamPage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            7,
        ),
        (
            "SettingsPage.tsx",
            "gridTemplateColumns: \"repeat(4, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.42fr)\"",
            7,
        ),
        (
            "WorkspacePage.tsx",
            "gridTemplateColumns: \"repeat(3, minmax(0, 1fr))\"",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
            7,
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &[
                "height: \"100%\"",
                "minHeight: 0",
                "overflow: \"auto\"",
                "hubTokens.window.pagePaddingX",
                "hubTokens.window.pagePaddingY",
                "Typography variant=\"h4\"",
                "Typography variant=\"body1\" color=\"text.secondary\"",
                "HubStatusBanner task={state.taskSummary}",
                "HubTabs",
                "MetricCard",
                metric_grid,
                main_grid,
                "@media (max-width: 980px)",
                "@media (max-width: 1180px)",
            ],
        );
        let panel_source = if page == "SettingsPage.tsx" {
            read_crate_file("web/src/components/data/SettingsSection.tsx")
        } else {
            source.clone()
        };
        assert_contains_all(page, &panel_source, &["HubPanel"]);
        let panel_count = panel_source.matches("<HubPanel").count();
        assert!(
            panel_count >= minimum_panel_count,
            "{page} should keep workspace panels on shared HubPanel; expected at least {minimum_panel_count}, found {panel_count}"
        );
    }
}

#[test]
fn editor_builds_and_settings_pages_preserve_workspace_specific_state_projection() {
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");

    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "const [tab, setTab] = useState(\"overview\")",
            "const project = state.selectedProject",
            "const editorPlugins = useMemo",
            "state.plugins.filter((plugin) => plugin.editorScoped)",
            "const editorActivity = useMemo",
            "state.actionHistory.filter",
            "action.kind === \"open-editor\" || action.kind === \"build-editor-runtime\"",
            "const editorTree = useMemo",
            "HubPanel title={text.launchTarget}",
            "HubPanel title={common.sourceEngines}",
            "HubPanel title={common.quickActions}",
            "HubPanel title={text.editorPluginScope}",
            "HubPanel title={text.workspaceTree}",
            "HubPanel title={text.editorActivity}",
            "HubPanel title={text.launchReadiness}",
            "HubSwitch checked={Boolean(project?.exists)}",
            "HubCheckbox checked={state.sourceEngines.length > 0}",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const buildActionKinds: HubActionHistoryItem[\"kind\"][]",
            "buildActionKinds.includes(action.kind)",
            "action.kind === \"install-project\"",
            "const buildHistory = useMemo",
            "const latestAction = buildHistory[0]",
            "const workflowRows = [",
            "const buildTree = useMemo",
            "LinearProgress",
            "HubPanel title={text.buildWorkflow}",
            "HubPanel title={common.selectedProject}",
            "HubPanel title={common.sourceEngines}",
            "HubPanel title={text.buildHistory}",
            "HubPanel title={text.latestWorkflow}",
            "HubPanel title={text.outputTree}",
            "HubPanel title={common.quickActions}",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "void onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)",
            "void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)",
        ],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "const healthRows = useMemo",
            "const pathTree = useMemo",
            "HubStatusBanner",
            "SettingsSection",
            "void onAction(HUB_ACTION.saveSettings, undefined, { settings: draft })",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "HubComboBox",
            "HubTextField",
            "HubSwitch",
            "HubCheckbox",
            "SourceEngineList",
            "HubTreeView",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            "HubPanel title={settingsText.configurationPathsPanel}",
            "HubPanel title={settingsText.sourceEnginesPanel}",
            "HubPanel title={settingsText.pathDefaultsPanel}",
            "HubPanel title={settingsText.advancedConfigurationPanel}",
            "HubPanel title={settingsText.configurationHealthPanel}",
            "HubPanel title={settingsText.activeSourceEnginePanel}",
        ],
    );
}

#[test]
fn catalog_cloud_team_and_fallback_pages_use_shared_workspace_patterns() {
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let workspace = read_crate_file("web/src/pages/WorkspacePage.tsx");

    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "const mode: \"assets\" | \"plugins\" | \"learn\"",
            "const rows = useMemo(() => catalogRows(state, mode, text), [mode, state, text])",
            "const visibleRows = useMemo(() => filterRows(rows, mode, tab, query)",
            "groupBy(rows, (row) => row.category)",
            "HubSearchField value={query}",
            "HubPanel title={catalogPanelTitle(mode, text)}",
            "HubPanel title={text.selectedEntry}",
            "HubPanel title={text.catalogTree}",
            "HubPanel title={common.quickActions}",
            "HubPanel title={common.sourceEngines}",
            "function catalogRows",
            "function filterRows",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const reservedServices = useMemo",
            "state.comingSoon.filter((entry) => entry.category === \"local-delivery\")",
            "const packageActions = useMemo",
            "const installActions = useMemo",
            "action.kind === \"package-project\"",
            "action.kind === \"install-project\"",
            "const outputTree = useMemo",
            "detail: formatCountText(common.reservedCountTemplate, reservedServices.length)",
            "HubPanel title={text.packageOutputs}",
            "HubPanel title={text.packageTarget}",
            "HubPanel title={common.quickActions}",
            "HubPanel title={text.deviceInstalls}",
            "HubPanel title={text.installReadiness}",
            "HubPanel title={text.reservedServices}",
            "HubPanel title={text.localDeliveryTree}",
            "HubPanel title={text.currentStatus}",
            "workflowProjectTargetPayload(state)",
            "workflowTargetProject(state)",
            "HubSwitch checked={Boolean(workflowProject && (!(\"exists\" in workflowProject) || workflowProject.exists))}",
            "HubCheckbox checked={state.settings.defaultDeviceInstallDir !== common.notConfigured}",
        ],
    );
    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &[
            "const memberRows = useMemo",
            "const actionRows = useMemo",
            "const teamTree = useMemo",
            "HubPanel title={text.teamMembers}",
            "HubPanel title={text.repositoryIdentity}",
            "HubPanel title={text.teamTree}",
            "HubPanel title={text.actionHistory}",
            "HubPanel title={text.latestAction}",
            "HubPanel title={common.sourceEngines}",
            "HubPanel title={common.quickActions}",
            "function ActionDetail",
        ],
    );
    assert_contains_all(
        "WorkspacePage.tsx",
        &workspace,
        &[
            "const settingsRows = useMemo",
            "const sourceTree = useMemo",
            "const common = state.ui.common",
            "const labels = state.settings.text.labels",
            "const settingsText = state.settings.text",
            "HubPanel title={common.sourceEngines}",
            "HubPanel title={settingsText.heading}",
            "HubPanel title={common.quickActions}",
            "HubPanel title={settingsText.advancedConfigurationPanel}",
            "HubPanel title={settingsText.configurationPathsPanel}",
            "HubPanel title={state.ui.editor.workspaceTree}",
            "HubSwitch checked={state.settings.buildProfile === \"release\"}",
            "HubCheckbox checked={state.settings.language === \"Chinese\"} label={labels.localizedUi}",
        ],
    );
}

#[test]
fn workspace_pages_use_shared_data_and_input_wrappers_not_raw_material_containers() {
    for page in [
        "EditorPage.tsx",
        "BuildsPage.tsx",
        "CatalogPage.tsx",
        "CloudPage.tsx",
        "TeamPage.tsx",
        "SettingsPage.tsx",
        "WorkspacePage.tsx",
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &["../components/data", "../components/inputs"],
        );
        for import_line in source.lines().filter(|line| line.contains("@mui/material")) {
            assert_not_contains_any(
                page,
                import_line,
                &[
                    "Card",
                    "Paper",
                    "Table",
                    "ListItemButton",
                    "Drawer",
                    "Button",
                    "TextField",
                    "Select",
                    "Checkbox",
                    "Switch",
                    "Tabs",
                ],
            );
        }
        assert_not_contains_any(
            page,
            &source,
            &[
                "<Card",
                "<Paper",
                "<Table",
                "<ListItemButton",
                "<Drawer",
                "<TextField",
                "<Select",
                "<Checkbox",
                "<Switch",
            ],
        );
    }
}

#[test]
fn workspace_layout_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_workspace_layout_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_workspace_layout_contract",
            "## Workspace Layout Contract Cutover",
            "React/MUI workspace layout",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/components/data/SettingsSection.tsx",
            "web/src/pages/WorkspacePage.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_workspace_layout_contract.rs`",
            "React/MUI workspace layout",
            "shared page shell, metric row, tabs, main/sidebar grid, and HubPanel composition",
            "Editor, Builds, Catalog, Cloud, Team, Settings, and fallback Workspace pages",
        ],
    );
}

#[test]
fn workspace_layout_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_workspace_layout_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_workspace_layout_contract.rs",
        &contract,
        &[
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/components/data/SettingsSection.tsx",
            "web/src/pages/WorkspacePage.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_workspace_layout_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_taffy_name.as_str(),
        ],
    );
}
