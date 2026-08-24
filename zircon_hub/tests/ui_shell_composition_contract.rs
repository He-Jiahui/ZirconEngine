//! Static contracts for the Hub React/MUI shell composition.

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

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read repository file {path}: {error}");
        }),
    )
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub must live under the repository root")
        .to_path_buf()
}

fn assert_contains_all(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{label} must contain shell-composition snippet: {snippet}"
        );
    }
}

fn assert_not_contains_any(source: &str, snippets: &[&str], label: &str) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{label} must not contain obsolete shell-composition snippet: {snippet}"
        );
    }
}

#[test]
fn app_installs_hub_window_and_keeps_state_flow_outside_page_chrome() {
    let app = read_crate_file("web/src/App.tsx");
    let shell_index = read_crate_file("web/src/components/shell/index.ts");

    assert_contains_all(
        &shell_index,
        &[
            "export * from \"./HubWindow\";",
            "export * from \"./NavigationDrawer\";",
            "export * from \"./TopBar\";",
        ],
        "web/src/components/shell/index.ts",
    );
    assert_contains_all(
        &app,
        &[
            "import { useEffect, useRef, useState } from \"react\";",
            "import { HubErrorBoundary, HubSnackbar } from \"./components/feedback\";",
            "import { HubWindow } from \"./components/shell\";",
            "import { fallbackShellState } from \"./data/hubData\";",
            "import { dispatchHubAction, loadHubState, subscribeHubStateChanged } from \"./tauri/hubApi\";",
            "import type { HubActionHandler, HubShellState } from \"./types/hub\";",
            "useState<HubShellState>(fallbackShellState)",
            "const stateRef = useRef(state);",
            "const stateGenerationRef = useRef(0);",
            "const actionSequenceRef = useRef(0);",
            "function applyHubState(nextState: HubShellState) {",
            "loadHubState().then((nextState) =>",
            "subscribeHubStateChanged((nextState) =>",
            "unlisten?.();",
            "const shellText = stateRef.current.ui.shell;",
            "label: shellText.liveUpdatesUnavailable",
            "const handleAction: HubActionHandler = async (actionId, targetId, payload) =>",
            "const actionSequence = actionSequenceRef.current + 1;",
            "const stateGenerationAtDispatch = stateGenerationRef.current;",
            "const nextState = await dispatchHubAction(actionId, targetId, payload);",
            "applyHubState(nextState);",
            "taskSummary: {",
            "label: shellText.actionFailed",
            "operation: shellText.actionFailed",
            "<HubWindow state={state} onAction={handleAction} />",
            "<HubSnackbar task={state.taskSummary} open={snackbarOpen} onClose={() => setSnackbarOpen(false)} />",
        ],
        "web/src/App.tsx",
    );
    assert_not_contains_any(&app, &["setState(nextState);"], "web/src/App.tsx");
    assert_not_contains_any(
        &app,
        &[
            "TopBar",
            "NavigationDrawer",
            "ProjectsDashboard",
            "ProjectBrowserPage",
            "SourceEnginePopover",
            "UserMenuPopover",
        ],
        "web/src/App.tsx",
    );
}

#[test]
fn hub_window_owns_root_layout_drawer_slot_and_page_router() {
    let window = read_crate_file("web/src/components/shell/HubWindow.tsx");

    assert_contains_all(
        &window,
        &[
            "import { Box } from \"@mui/material\";",
            "import { BuildsPage } from \"../../pages/BuildsPage\";",
            "import { CatalogPage } from \"../../pages/CatalogPage\";",
            "import { CloudPage } from \"../../pages/CloudPage\";",
            "import { EditorPage } from \"../../pages/EditorPage\";",
            "import { ProjectsDashboard } from \"../../pages/ProjectsDashboard\";",
            "import { SettingsPage } from \"../../pages/SettingsPage\";",
            "import { TeamPage } from \"../../pages/TeamPage\";",
            "import { WorkspacePage } from \"../../pages/WorkspacePage\";",
            "import { hubTokens } from \"../../theme/tokens\";",
            "import type { ComponentType } from \"react\";",
            "import type { HubActionHandler, HubPageId, HubShellState } from \"../../types/hub\";",
            "import { NavigationDrawer } from \"./NavigationDrawer\";",
            "import { TopBar } from \"./TopBar\";",
            "export interface HubWindowProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "type HubPageComponent = ComponentType<HubWindowProps>;",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "width: \"100vw\"",
            "height: \"100vh\"",
            "overflow: \"hidden\"",
            "color: hubTokens.colors.text",
            "background: hubTokens.gradients.window",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "borderRadius: \"10px\"",
            "<TopBar state={state} onAction={onAction} />",
            "height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`",
            "<NavigationDrawer",
            "activePage={state.activePage}",
            "text={state.ui.shell}",
            "engineVersion={state.engineVersion}",
            "sourceEngines={state.sourceEngines}",
            "activeSourceEngineId={state.activeSourceEngineId}",
            "onAction={onAction}",
            "component=\"main\"",
            "backgroundColor: \"rgba(17,17,17,0.55)\"",
        ],
        "HubWindow",
    );

    for (page_id, page_component) in [
        ("projects", "ProjectsDashboard"),
        ("editor", "EditorPage"),
        ("builds", "BuildsPage"),
        ("cloud", "CloudPage"),
        ("assets", "CatalogPage"),
        ("plugins", "CatalogPage"),
        ("learn", "CatalogPage"),
        ("team", "TeamPage"),
        ("settings", "SettingsPage"),
        ("fallback", "WorkspacePage"),
    ] {
        let snippet = if page_id == "fallback" {
            format!(": {page_component};")
        } else {
            format!("{page_id}: {page_component},")
        };
        assert!(
            window.contains(&snippet),
            "HubWindow must route {page_id} through {page_component}"
        );
    }

    assert_not_contains_any(
        &window,
        &[
            "useState(",
            "loadHubState",
            "dispatchHubAction",
            "HubSnackbar",
            "SourceEnginePopover",
            "UserMenuPopover",
            "import { Drawer",
        ],
        "HubWindow",
    );
}

#[test]
fn top_bar_composes_brand_engine_status_user_menu_and_window_controls() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");

    assert_contains_all(
        &topbar,
        &[
            "import { Avatar, Box, ButtonBase, Divider, Typography } from \"@mui/material\";",
            "import { getCurrentWindow } from \"@tauri-apps/api/window\";",
            "import { useState } from \"react\";",
            "import { brandMark } from \"../../data/hubData\";",
            "import type { HubActionHandler, HubShellState } from \"../../types/hub\";",
            "import { HUB_ACTION } from \"../../types/hub\";",
            "import { StatusBadge } from \"../data\";",
            "import { HubIconButton } from \"../inputs\";",
            "import { SourceEnginePopover, UserMenuPopover } from \"../overlays\";",
            "const [engineAnchor, setEngineAnchor] = useState<HTMLElement | null>(null);",
            "const [userAnchor, setUserAnchor] = useState<HTMLElement | null>(null);",
            "state.sourceEngines.find((engine) => engine.id === state.activeSourceEngineId)",
            "const notificationDetail = comingSoonDetail(state, \"notification-center\");",
            "const signOutDetail = comingSoonDetail(state, \"sign-out\");",
            "void onAction(HUB_ACTION.showPage, \"settings\");",
            "void onAction(HUB_ACTION.showPage, \"learn\");",
            "void onAction(HUB_ACTION.showPage, \"team\");",
            "component=\"header\"",
            "height: hubTokens.window.topBarHeight",
            "gridTemplateColumns: \"222px minmax(0, 1fr) auto\"",
            "gridTemplateColumns: \"78px minmax(0, 1fr) auto\"",
            "src={brandMark}",
            "{state.productName}",
            "{state.ui.shell.productCategory}",
            "onClick={(event) => setEngineAnchor(event.currentTarget)}",
            "border: `1px solid ${engineAnchor ? \"rgba(45,212,207,0.48)\" : hubTokens.colors.lineStrong}`",
            "{engineLabel}",
            "state.taskStatus.map((status) =>",
            "<StatusBadge key={status.id} label={status.label} tone={status.tone} />",
            "label={state.ui.shell.notifications} tooltip={notificationDetail} disabled",
            "label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, \"learn\")}",
            "onClick={() => void onAction(HUB_ACTION.showPage, \"settings\")}",
            "onClick={(event) => setUserAnchor(event.currentTarget)}",
            "const userName = state.team.identityName || state.ui.common.notConfigured;",
            "const userInitials = initialsFromName(userName);",
            "runWindowAction(\"minimize\", windowActionSchedulerRef.current!",
            "runWindowAction(\"toggle-maximize\", windowActionSchedulerRef.current!",
            "runWindowAction(\"close\", windowActionSchedulerRef.current!",
            "{userName}",
            "onClick={handleMinimize}",
            "onClick={handleToggleMaximize}",
            "onClick={handleClose}",
            "<SourceEnginePopover",
            "engines={state.sourceEngines}",
            "activeEngineId={state.activeSourceEngineId}",
            "settings={state.settings}",
            "text={state.ui.shell}",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "<UserMenuPopover",
            "initials={userInitials}",
            "userName={userName}",
            "text={state.ui.shell}",
            "signOutDetail={signOutDetail}",
            "onAction={handleUserAction}",
            "function comingSoonDetail(state: HubShellState, id: string): string",
            "createWindowActionScheduler",
            "scheduler.run(actionKind, () => action(getCurrentWindow()))",
            "const topIconSx =",
            "const windowIconSx =",
        ],
        "TopBar",
    );
    assert_not_contains_any(
        &topbar,
        &[
            "loadHubState",
            "dispatchHubAction",
            "ProjectsDashboard",
            "ProjectBrowserPage",
            "Drawer",
        ],
        "TopBar",
    );
}

#[test]
fn navigation_drawer_uses_permanent_mui_drawer_and_forwards_page_actions() {
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        &drawer,
        &[
            "import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Tooltip, Typography } from \"@mui/material\";",
            "import KeyboardDoubleArrowRightIcon from \"@mui/icons-material/KeyboardDoubleArrowRight\";",
            "import { useState } from \"react\";",
            "import type { HubActionHandler, HubPageId, HubShellText, HubSourceEngineSummary } from \"../../types/hub\";",
            "import { HUB_ACTION } from \"../../types/hub\";",
            "const navIcons: Record<HubPageId, typeof FolderOutlinedIcon>",
            "projects: FolderOutlinedIcon",
            "editor: WebAssetOutlinedIcon",
            "assets: Inventory2OutlinedIcon",
            "builds: ConstructionOutlinedIcon",
            "plugins: ExtensionOutlinedIcon",
            "cloud: CloudOutlinedIcon",
            "team: GroupsOutlinedIcon",
            "learn: AutoStoriesOutlinedIcon",
            "settings: SettingsOutlinedIcon",
            "text: HubShellText;",
            "engineVersion: string;",
            "sourceEngines: HubSourceEngineSummary[];",
            "activeSourceEngineId: string | null;",
            "const [collapsed, setCollapsed] = useState(false);",
            "const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "const CollapseIcon = collapsed ? KeyboardDoubleArrowRightIcon : KeyboardDoubleArrowLeftIcon;",
            "sourceEngines.find((engine) => engine.id === activeSourceEngineId)",
            "const statusLabel = activeEngine?.status ?? text.noSourceEngineRegistered;",
            "const engineLabel = activeEngine?.name ?? engineVersion;",
            "variant=\"permanent\"",
            "width: drawerWidth",
            "backgroundColor: \"rgba(16,16,16,0.96)\"",
            "borderRight: `1px solid ${hubTokens.colors.line}`",
            "text.navItems.map(({ id, label }) =>",
            "const Icon = navIcons[id];",
            "const selected = activePage === id;",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "height: 49",
            "backgroundColor: selected ? \"rgba(15,99,96,0.56)\" : \"transparent\"",
            "<ListItemIcon sx={{ minWidth: collapsed ? 0 : 40, color: \"inherit\", justifyContent: \"center\" }}>",
            "{text.engineStatus}",
            "{engineLabel}",
            "{statusLabel}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "<CollapseIcon fontSize=\"small\" />",
            "{collapseLabel}",
            "onClick={() => setCollapsed((current) => !current)}",
        ],
        "NavigationDrawer",
    );
    assert_not_contains_any(
        &drawer,
        &[
            "state.",
            "HubWindow",
            "TopBar",
            "SourceEnginePopover",
            "UserMenuPopover",
            "loadHubState",
            "dispatchHubAction",
        ],
        "NavigationDrawer",
    );
}

#[test]
fn topbar_popups_are_shared_overlay_components_not_inline_shell_paint() {
    let hub_popover = read_crate_file("web/src/components/overlays/HubPopover.tsx");
    let source_engine = read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");
    let user_menu = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");
    let overlay_index = read_crate_file("web/src/components/overlays/index.ts");

    assert_contains_all(
        &overlay_index,
        &[
            "export * from \"./HubDialog\";",
            "export * from \"./HubMenu\";",
            "export * from \"./HubPopover\";",
            "export * from \"./SourceEnginePopover\";",
            "export * from \"./UserMenuPopover\";",
        ],
        "web/src/components/overlays/index.ts",
    );
    assert_contains_all(
        &hub_popover,
        &[
            "import { Box, Popover } from \"@mui/material\";",
            "export interface HubPopoverProps",
            "anchorEl: HTMLElement | null;",
            "open: boolean;",
            "width?: number;",
            "align?: \"left\" | \"right\";",
            "onClose: () => void;",
            "width = 340",
            "maxWidth: \"calc(100vw - 32px)\"",
            "backgroundColor: \"rgba(25,29,29,0.98)\"",
            "border: `1px solid ${hubTokens.colors.lineStrong}`",
            "borderRadius: `${hubTokens.radius.panel}px`",
            "boxShadow: \"0 24px 60px rgba(0,0,0,0.46), 0 0 0 1px rgba(45,212,207,0.08)\"",
        ],
        "HubPopover",
    );
    assert_contains_all(
        &source_engine,
        &[
            "import type { HubSettingsSummary, HubShellText, HubSourceEngineSummary } from \"../../types/hub\";",
            "import { StatusBadge } from \"../data\";",
            "import { HubPopover } from \"./HubPopover\";",
            "export interface SourceEnginePopoverProps",
            "engines: HubSourceEngineSummary[];",
            "settings: HubSettingsSummary;",
            "text: HubShellText;",
            "onSelect: (engineId: string) => void;",
            "onManage: () => void;",
            "<HubPopover anchorEl={anchorEl} open={open} width={388} onClose={onClose}>",
            "{text.activeEngine}",
            "{text.readyFallback}",
            "{text.localDefaults}",
            "gridTemplateColumns: \"34px minmax(0, 1fr) auto\"",
            "StatusBadge label={activeLabel} tone=\"success\"",
            "{text.manageEngines}",
        ],
        "SourceEnginePopover",
    );
    assert_contains_all(
        &user_menu,
        &[
            "import { HubPopover } from \"./HubPopover\";",
            "export interface UserMenuPopoverProps",
            "signOutDetail: string;",
            "onAction: (actionId: string) => void;",
            "const menuItems = [",
            "{ id: \"account\", label: text.userAccount, detail: text.userAccountDetail, Icon: AccountCircleOutlinedIcon }",
            "{ id: \"preferences\", label: text.preferences, detail: text.preferencesDetail, Icon: SettingsOutlinedIcon }",
            "{ id: \"documentation\", label: text.documentation, detail: text.documentationDetail, Icon: AutoStoriesOutlinedIcon }",
            "{ id: \"sign-out\", label: text.signOut, detail: signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }",
            "<HubPopover anchorEl={anchorEl} open={open} width={284} align=\"right\" onClose={onClose}>",
            "gridTemplateColumns: \"42px minmax(0, 1fr)\"",
            "{text.workspaceProfile}",
            "const isDisabled = Boolean(disabled);",
            "disabled={isDisabled}",
            "if (isDisabled) {",
            "onAction(id);",
            "onClose();",
        ],
        "UserMenuPopover",
    );
}

#[test]
fn shell_state_type_matches_shell_component_contracts() {
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        &types,
        &[
            "export interface HubShellState",
            "productName: string;",
            "engineVersion: string;",
            "activePage: string;",
            "pageTitle: string;",
            "pageSubtitle: string;",
            "projectSubpage: string;",
            "taskSummary: HubTaskSummary;",
            "taskStatus: HubStatusPill[];",
            "selectedProject: HubProjectDetail | null;",
            "quickActions: HubQuickAction[];",
            "sourceEngines: HubSourceEngineSummary[];",
            "activeSourceEngineId: string | null;",
            "team: HubTeamSummary;",
            "kind: HubActionHistoryKind;",
            "actionHistory: HubActionHistoryItem[];",
            "settings: HubSettingsSummary;",
            "workspaceProfile: string;",
            "noSourceEngineRegistered: string;",
            "manageEngines: string;",
        ],
        "HubShellState",
    );
}

#[test]
fn shell_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let component_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        &shell_doc,
        &[
            "Zircon Hub Tauri React Shell",
            "ui_shell_composition_contract.rs",
            "React/MUI shell composition",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/overlays/SourceEnginePopover.tsx",
            "web/src/components/overlays/UserMenuPopover.tsx",
        ],
        "tauri-react-shell.md",
    );
    assert_contains_all(
        &component_doc,
        &[
            "React/MUI shell composition",
            "`ui_shell_composition_contract.rs`",
            "HubWindow, TopBar, NavigationDrawer, SourceEnginePopover, UserMenuPopover",
            "live `hub-state-changed` subscription cleanup",
            "one `onAction` dispatcher",
        ],
        "responsive-component-system.md",
    );
}

#[test]
fn shell_composition_contract_is_cut_over_to_react_sources() {
    let source = read_crate_file("tests/ui_shell_composition_contract.rs");
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
            "shell-composition contract must not inspect removed UI-file or app-module surfaces: {obsolete}"
        );
    }

    assert_contains_all(
        &source,
        &[
            "web/src/App.tsx",
            "web/src/components/shell/index.ts",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/overlays/HubPopover.tsx",
            "web/src/components/overlays/SourceEnginePopover.tsx",
            "web/src/components/overlays/UserMenuPopover.tsx",
            "web/src/types/hub.ts",
        ],
        "shell-composition contract",
    );
}
