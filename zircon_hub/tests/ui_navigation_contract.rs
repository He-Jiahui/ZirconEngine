//! Static contracts for React + Material UI Hub navigation primitives.

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
            "{source_name} should contain navigation contract snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete navigation snippet {snippet:?}"
        );
    }
}

#[test]
fn navigation_drawer_owns_primary_page_list_and_responsive_labels() {
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "Drawer",
            "variant=\"permanent\"",
            "const [collapsed, setCollapsed] = useState(false);",
            "const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;",
            "text.navItems.map(({ id, label }) =>",
            "selected={selected}",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "fontWeight: selected ? 700 : 500",
            "display: collapsed ? \"none\"",
            "@media (max-width: 980px)",
            "hubTokens.window.sidebarCollapsedWidth",
            "{text.engineStatus}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "{collapseLabel}",
            "onClick={() => setCollapsed((current) => !current)}",
        ],
    );
}

#[test]
fn topbar_navigation_routes_source_engine_user_and_settings_actions() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let source_popover = read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    let user_popover = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "const [engineAnchor, setEngineAnchor]",
            "const [userAnchor, setUserAnchor]",
            "activeSourceEngineId",
            "handleUserAction",
            "void onAction(HUB_ACTION.showPage, \"settings\")",
            "void onAction(HUB_ACTION.showPage, \"learn\")",
            "void onAction(HUB_ACTION.showPage, \"team\")",
            "void onAction(HUB_ACTION.selectEngine, engineId)",
            "SourceEnginePopover",
            "UserMenuPopover",
            "const notificationDetail = comingSoonDetail(state, \"notification-center\");",
            "HubIconButton label={state.ui.shell.notifications} tooltip={notificationDetail} disabled",
            "HubIconButton label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, \"learn\")}",
            "HubIconButton label={state.ui.shell.settings}",
            "StatusBadge key={status.id}",
            "gridTemplateColumns: \"222px minmax(0, 1fr) auto\"",
            "gridTemplateColumns: \"78px minmax(0, 1fr) auto\"",
        ],
    );
    assert_contains_all(
        "SourceEnginePopover.tsx",
        &source_popover,
        &[
            "onSelect: (engineId: string) => void",
            "onManage: () => void",
            "onClick={() => onSelect(engine.id)}",
            "{text.manageEngines}",
        ],
    );
    assert_contains_all(
        "UserMenuPopover.tsx",
        &user_popover,
        &[
            "{ id: \"account\", label: text.userAccount",
            "{ id: \"preferences\", label: text.preferences",
            "{ id: \"documentation\", label: text.documentation",
            "onAction(id)",
            "onClose()",
        ],
    );
}

#[test]
fn hub_window_routes_primary_pages_from_one_shell_boundary() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");

    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "TopBar state={state} onAction={onAction}",
            "<NavigationDrawer",
            "activePage={state.activePage}",
            "text={state.ui.shell}",
            "engineVersion={state.engineVersion}",
            "sourceEngines={state.sourceEngines}",
            "activeSourceEngineId={state.activeSourceEngineId}",
            "onAction={onAction}",
            "component=\"main\"",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "projects: ProjectsDashboard,",
            "editor: EditorPage,",
            "builds: BuildsPage,",
            "cloud: CloudPage,",
            "assets: CatalogPage,",
            "team: TeamPage,",
            "settings: SettingsPage,",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "<PageComponent state={state} onAction={onAction} />",
        ],
    );
}

#[test]
fn tabs_and_toggle_wrappers_own_secondary_navigation_controls() {
    let tabs = read_crate_file("web/src/components/inputs/HubTabs.tsx");
    let toggle = read_crate_file("web/src/components/inputs/HubToggle.tsx");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let projects_toolbar = read_crate_file("web/src/components/inputs/ProjectsToolbar.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");

    assert_contains_all(
        "HubTabs.tsx",
        &tabs,
        &[
            "Tabs",
            "Tab",
            "HubTabOption",
            "onChange={(_, nextValue: string) => onChange(nextValue)}",
            "variant=\"scrollable\"",
            "scrollButtons=\"auto\"",
            "iconPosition=\"start\"",
        ],
    );
    assert_contains_all(
        "HubToggle.tsx",
        &toggle,
        &[
            "ToggleButtonGroup",
            "exclusive",
            "nextValue: string | null",
            "Tooltip",
            "aria-label={option.label}",
            "\"&.Mui-selected\"",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "ProjectsToolbar",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "void onAction(HUB_ACTION.viewAllProjects)",
            "void onAction(HUB_ACTION.newProject)",
            "void onAction(HUB_ACTION.openProjectDetail, project.id)",
        ],
    );
    assert_contains_all(
        "ProjectsToolbar.tsx",
        &projects_toolbar,
        &[
            "HubToggle",
            "{ value: \"grid\", label: text.gridView",
            "{ value: \"list\", label: text.listView",
            "onChange={onViewMode}",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &[
            "HubToggle",
            "{ value: \"grid\", label: text.gridView",
            "{ value: \"list\", label: text.listView",
            "void onAction(HUB_ACTION.setProjectViewMode, value)",
            "void onAction(HUB_ACTION.showProjectSubpage, \"dashboard\")",
            "void onAction(HUB_ACTION.newProject)",
        ],
    );
    for (page, source) in [
        ("ProjectDetailPage.tsx", detail),
        ("BuildsPage.tsx", builds),
        ("SettingsPage.tsx", settings),
    ] {
        assert_contains_all(
            page,
            &source,
            &["HubTabs", "const [tab, setTab] = useState"],
        );
    }
}

#[test]
fn tauri_navigation_actions_flow_through_single_action_command() {
    let api = read_crate_file("web/src/tauri/hubApi.ts");
    let app = read_crate_file("web/src/App.tsx");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "hubApi.ts",
        &api,
        &[
            "loadHubState",
            "dispatchHubAction",
            "invoke<unknown>(\"hub_state\")",
            "invoke<unknown>(\"hub_action\"",
            "request: { actionId, targetId, payload }",
            "fallbackShellState",
            "isTauriRuntime",
        ],
    );
    assert_contains_all(
        "App.tsx",
        &app,
        &[
            "const handleAction: HubActionHandler = async (actionId, targetId, payload)",
            "dispatchHubAction(actionId, targetId, payload)",
            "actionSequenceRef",
            "stateGenerationRef",
            "applyHubState(nextState)",
            "HubWindow state={state} onAction={handleAction}",
            "const shellText = stateRef.current.ui.shell;",
            "shellText.actionFailed",
        ],
    );
    assert_not_contains_any("App.tsx", &app, &["setState(nextState);"]);
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "activePage: string;",
            "projectSubpage: string;",
            "projectViewMode: string;",
            "selectedProjectId: string | null;",
            "activeSourceEngineId: string | null;",
        ],
    );
}

#[test]
fn navigation_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_navigation_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_navigation_contract",
            "## Navigation Contract Cutover",
            "React/MUI navigation system",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/inputs/HubTabs.tsx",
            "web/src/components/inputs/HubToggle.tsx",
            "web/src/tauri/hubApi.ts",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_navigation_contract.rs`",
            "React/MUI navigation system",
            "drawer, topbar, page router, tabs, toggle, source-engine popup, and user-menu navigation",
        ],
    );
}

#[test]
fn navigation_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_navigation_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");

    assert_contains_all(
        "ui_navigation_contract.rs",
        &contract,
        &[
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/inputs/HubTabs.tsx",
            "web/src/components/inputs/HubToggle.tsx",
            "web/src/tauri/hubApi.ts",
        ],
    );
    assert_not_contains_any(
        "ui_navigation_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
        ],
    );
}
