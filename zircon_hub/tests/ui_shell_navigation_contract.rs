//! Static contracts for React/MUI Hub shell navigation chrome.

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
            "{source_name} should contain shell-navigation snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete shell-navigation snippet {snippet:?}"
        );
    }
}

#[test]
fn navigation_drawer_owns_page_list_selected_state_and_responsive_collapse() {
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "import { Box, ButtonBase, Drawer, List, ListItemButton, ListItemIcon, Tooltip, Typography } from \"@mui/material\";",
            "import KeyboardDoubleArrowRightIcon from \"@mui/icons-material/KeyboardDoubleArrowRight\";",
            "import { useState } from \"react\";",
            "const navIcons: Record<HubPageId, typeof FolderOutlinedIcon> = {",
            "projects: FolderOutlinedIcon,",
            "editor: WebAssetOutlinedIcon,",
            "assets: Inventory2OutlinedIcon,",
            "builds: ConstructionOutlinedIcon,",
            "plugins: ExtensionOutlinedIcon,",
            "cloud: CloudOutlinedIcon,",
            "team: GroupsOutlinedIcon,",
            "learn: AutoStoriesOutlinedIcon,",
            "settings: SettingsOutlinedIcon,",
            "export interface NavigationDrawerProps",
            "activePage: string;",
            "text: HubShellText;",
            "engineVersion: string;",
            "onAction: HubActionHandler;",
            "const [collapsed, setCollapsed] = useState(false);",
            "const drawerWidth = collapsed ? hubTokens.window.sidebarCollapsedWidth : hubTokens.window.sidebarWidth;",
            "const collapseLabel = collapsed ? text.expand : text.collapse;",
            "const CollapseIcon = collapsed ? KeyboardDoubleArrowRightIcon : KeyboardDoubleArrowLeftIcon;",
            "variant=\"permanent\"",
            "width: drawerWidth",
            "text.navItems.map(({ id, label }) =>",
            "const Icon = navIcons[id];",
            "const selected = activePage === id;",
            "selected={selected}",
            "onClick={() => void onAction(HUB_ACTION.showPage, id)}",
            "fontWeight: selected ? 700 : 500",
            "display: collapsed ? \"none\"",
            "@media (max-width: 980px)",
        ],
    );
}

#[test]
fn navigation_drawer_keeps_status_panel_and_collapse_affordance_in_sidebar() {
    let drawer = read_crate_file("web/src/components/shell/NavigationDrawer.tsx");

    assert_contains_all(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "{text.engineStatus}",
            "{engineVersion}",
            "{text.upToDate}",
            "{text.checkForUpdates}",
            "{text.checkForUpdatesDetail}",
            "disabled",
            "Tooltip title={text.checkForUpdatesDetail}",
            "\"&.Mui-disabled\"",
            "backgroundColor: hubTokens.colors.success",
            "ButtonBase",
            "<CollapseIcon fontSize=\"small\" />",
            "{collapseLabel}",
            "aria-label={collapseLabel}",
            "onClick={() => setCollapsed((current) => !current)}",
            "borderTop: `1px solid ${hubTokens.colors.line}`",
            "\"@media (max-width: 980px)\": { display: \"none\" }",
        ],
    );
    assert_not_contains_any(
        "NavigationDrawer.tsx",
        &drawer,
        &[
            "position: \"absolute\"",
            "height: \"100vh\"",
            "width: \"100vw\"",
        ],
    );
}

#[test]
fn hub_window_places_navigation_drawer_between_topbar_and_page_router() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");

    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "<TopBar state={state} onAction={onAction} />",
            "height: `calc(100vh - ${hubTokens.window.topBarHeight}px)`",
            "<NavigationDrawer activePage={state.activePage} text={state.ui.shell} engineVersion={state.engineVersion} onAction={onAction} />",
            "component=\"main\"",
            "overflow: \"hidden\"",
            "state.activePage === \"projects\"",
            "ProjectsDashboard state={state} onAction={onAction}",
            "state.activePage === \"assets\" || state.activePage === \"plugins\" || state.activePage === \"learn\"",
            "CatalogPage state={state} onAction={onAction}",
            "WorkspacePage state={state} onAction={onAction}",
        ],
    );
}

#[test]
fn rust_navigation_ids_match_drawer_items_and_tauri_show_page_action() {
    let navigation = read_crate_file("src/state/navigation.rs");
    let action_request = read_crate_file("src/tauri_app/action_request.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "navigation.rs",
        &navigation,
        &[
            "pub enum HubPage",
            "Projects",
            "Editor",
            "Assets",
            "Builds",
            "Plugins",
            "Cloud",
            "Team",
            "Learn",
            "Settings",
            "Self::Projects => \"projects\"",
            "Self::Editor => \"editor\"",
            "Self::Assets => \"assets\"",
            "Self::Builds => \"builds\"",
            "Self::Plugins => \"plugins\"",
            "Self::Cloud => \"cloud\"",
            "Self::Team => \"team\"",
            "Self::Learn => \"learn\"",
            "Self::Settings => \"settings\"",
            "pub fn from_id(id: &str) -> Option<Self>",
        ],
    );
    assert_contains_all(
        "action_request.rs",
        &action_request,
        &[
            "\"show-page\" | \"page\" => Ok(HubAction::ShowPage",
            "target_id: self.required_target()?.to_string()",
        ],
    );
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "HubAction::ShowPage { target_id } => self.select_page_by_id(&target_id)?",
            "fn select_page_by_id(&mut self, page_id: &str) -> Result<(), HubError>",
            "HubPage::from_id(page_id)",
            "self.selected_page = page;",
            "self.persist_hub_config()",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "activePage: string;",
            "pageTitle: string;",
            "pageSubtitle: string;",
        ],
    );
}

#[test]
fn shell_navigation_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_shell_navigation_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_navigation_contract",
            "## Shell Navigation Contract Cutover",
            "React/MUI shell navigation chrome",
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "src/state/navigation.rs",
            "src/tauri_app/runtime_state.rs",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_shell_navigation_contract.rs`",
            "React/MUI shell navigation chrome",
            "permanent drawer page list, status panel, collapse affordance, HubWindow routing, and Tauri show-page action",
        ],
    );
}

#[test]
fn shell_navigation_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_shell_navigation_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_shell_navigation_contract.rs",
        &contract,
        &[
            "web/src/components/shell/NavigationDrawer.tsx",
            "web/src/components/shell/HubWindow.tsx",
            "src/state/navigation.rs",
            "src/tauri_app/runtime_state.rs",
            "web/src/types/hub.ts",
        ],
    );
    assert_not_contains_any(
        "ui_shell_navigation_contract.rs",
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
