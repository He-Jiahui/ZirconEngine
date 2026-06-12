//! Static contracts for React/MUI Hub overlay primitives.

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
            "{source_name} should contain overlay primitive snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete overlay primitive snippet {snippet:?}"
        );
    }
}

#[test]
fn overlay_barrel_exports_dialog_menu_popover_and_business_popups() {
    let index = read_crate_file("web/src/components/overlays/index.ts");

    assert_contains_all(
        "components/overlays/index.ts",
        &index,
        &[
            "export * from \"./HubDialog\";",
            "export * from \"./HubMenu\";",
            "export * from \"./HubPopover\";",
            "export * from \"./CreateProjectDialog\";",
            "export * from \"./SourceEnginePopover\";",
            "export * from \"./UserMenuPopover\";",
        ],
    );
}

#[test]
fn dialog_menu_and_popover_wrap_material_overlay_primitives_with_hub_api() {
    let dialog = read_crate_file("web/src/components/overlays/HubDialog.tsx");
    let menu = read_crate_file("web/src/components/overlays/HubMenu.tsx");
    let popover = read_crate_file("web/src/components/overlays/HubPopover.tsx");

    assert_contains_all(
        "HubDialog.tsx",
        &dialog,
        &[
            "import { Dialog, DialogActions, DialogContent, DialogTitle } from \"@mui/material\";",
            "export interface HubDialogProps extends PropsWithChildren",
            "open: boolean;",
            "title: string;",
            "actions?: ReactNode;",
            "onClose: () => void;",
            "<Dialog",
            "open={open}",
            "onClose={onClose}",
            "maxWidth=\"sm\"",
            "fullWidth",
            "DialogTitle",
            "DialogContent",
            "DialogActions",
            "backgroundColor: \"rgba(28,28,28,0.98)\"",
        ],
    );
    assert_contains_all(
        "HubMenu.tsx",
        &menu,
        &[
            "import { Menu, MenuItem, Typography } from \"@mui/material\";",
            "export interface HubMenuItem",
            "id: string;",
            "label: string;",
            "icon?: ReactNode;",
            "export interface HubMenuProps",
            "anchorEl: HTMLElement | null;",
            "items: HubMenuItem[];",
            "onSelect: (id: string) => void;",
            "<Menu",
            "slotProps",
            "MenuItem",
            "onSelect(item.id);",
            "onClose();",
        ],
    );
    assert_contains_all(
        "HubPopover.tsx",
        &popover,
        &[
            "import { Box, Popover } from \"@mui/material\";",
            "export interface HubPopoverProps extends PropsWithChildren",
            "anchorEl: HTMLElement | null;",
            "open: boolean;",
            "width?: number;",
            "align?: \"left\" | \"right\";",
            "onClose: () => void;",
            "width = 340",
            "align = \"left\"",
            "anchorOrigin",
            "transformOrigin",
            "maxWidth: \"calc(100vw - 32px)\"",
            "backgroundColor: \"rgba(25,29,29,0.98)\"",
            "boxShadow: \"0 24px 60px rgba(0,0,0,0.46), 0 0 0 1px rgba(45,212,207,0.08)\"",
        ],
    );
}

#[test]
fn source_engine_popover_composes_engine_rows_defaults_and_manage_action() {
    let source = read_crate_file("web/src/components/overlays/SourceEnginePopover.tsx");

    assert_contains_all(
        "SourceEnginePopover.tsx",
        &source,
        &[
            "export interface SourceEnginePopoverProps",
            "engines: HubSourceEngineSummary[];",
            "activeEngineId?: string | null;",
            "settings: HubSettingsSummary;",
            "text: HubShellText;",
            "onSelect: (engineId: string) => void;",
            "onManage: () => void;",
            "<HubPopover anchorEl={anchorEl} open={open} width={388} onClose={onClose}>",
            "const activeId = activeEngineId ?? engines.find((engine) => engine.active)?.id ?? engines[0]?.id;",
            "const activeEngines = engines.filter((engine) => engine.id === activeId);",
            "const fallbackEngines = engines.filter((engine) => engine.id !== activeId);",
            "{text.activeEngine}",
            "{text.readyFallback}",
            "{text.localDefaults}",
            "{text.noSourceEngineRegistered}",
            "{text.noFallbackEngineConfigured}",
            "PathRow label={text.source} value={settings.defaultSourceDir}",
            "PathRow label={text.buildOutput} value={settings.defaultBuildOutputDir}",
            "onClick={onManage}",
            "{text.manageEngines}",
            "function EngineRow",
            "onClick={() => onSelect(engine.id)}",
            "StatusBadge label={activeLabel} tone=\"success\"",
        ],
    );
}

#[test]
fn user_menu_popover_composes_profile_header_menu_actions_and_close() {
    let user = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");

    assert_contains_all(
        "UserMenuPopover.tsx",
        &user,
        &[
            "export interface UserMenuPopoverProps",
            "anchorEl: HTMLElement | null;",
            "open: boolean;",
            "userName: string;",
            "initials: string;",
            "text: HubShellText;",
            "signOutDetail: string;",
            "onClose: () => void;",
            "onAction: (actionId: string) => void;",
            "const menuItems = [",
            "{ id: \"account\", label: text.userAccount",
            "{ id: \"preferences\", label: text.preferences",
            "{ id: \"documentation\", label: text.documentation",
            "{ id: \"sign-out\", label: text.signOut, detail: signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }",
            "<HubPopover anchorEl={anchorEl} open={open} width={284} align=\"right\" onClose={onClose}>",
            "{text.workspaceProfile}",
            "const isDisabled = Boolean(disabled);",
            "disabled={isDisabled}",
            "if (isDisabled) {",
            "onAction(id);",
            "onClose();",
            "danger ? hubTokens.colors.error",
            "\"&.Mui-disabled\"",
        ],
    );
}

#[test]
fn shell_and_project_pages_consume_shared_overlay_components() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let create_project = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "const [engineAnchor, setEngineAnchor] = useState<HTMLElement | null>(null);",
            "const [userAnchor, setUserAnchor] = useState<HTMLElement | null>(null);",
            "SourceEnginePopover",
            "anchorEl={engineAnchor}",
            "open={Boolean(engineAnchor)}",
            "engines={state.sourceEngines}",
            "activeEngineId={state.activeSourceEngineId}",
            "settings={state.settings}",
            "text={state.ui.shell}",
            "setEngineAnchor(null);",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "void onAction(HUB_ACTION.showPage, \"settings\");",
            "UserMenuPopover",
            "anchorEl={userAnchor}",
            "open={Boolean(userAnchor)}",
            "onAction={handleUserAction}",
            "text={state.ui.shell}",
            "signOutDetail={signOutDetail}",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "import { CreateProjectDialog, HubMenu, type HubMenuItem } from \"../components/overlays\";",
            "<CreateProjectDialog",
            "open={state.projectSubpage === \"new-project\"}",
            "onClose={() => void onAction(HUB_ACTION.viewAllProjects)}",
            "onCreate={(payload) => void onAction(HUB_ACTION.createProject, undefined, payload)}",
        ],
    );
    assert_contains_all(
        "CreateProjectDialog.tsx",
        &create_project,
        &[
            "import { HubDialog } from \"./HubDialog\";",
            "<HubDialog",
            "title={text.newProjectDialog}",
            "actions={",
            "HubTextField label={text.projectName}",
            "HubTextField label={text.location}",
            "HubComboBox",
        ],
    );
}

#[test]
fn overlay_primitives_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_overlay_primitives_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_overlay_primitives_contract",
            "## Overlay Primitive Contract Cutover",
            "React/MUI overlay primitives",
            "web/src/components/overlays/HubDialog.tsx",
            "web/src/components/overlays/HubMenu.tsx",
            "web/src/components/overlays/HubPopover.tsx",
            "web/src/components/overlays/CreateProjectDialog.tsx",
            "web/src/components/overlays/SourceEnginePopover.tsx",
            "web/src/components/overlays/UserMenuPopover.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/pages/ProjectsDashboard.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_overlay_primitives_contract.rs`",
            "React/MUI overlay primitives",
            "Dialog, Menu, Popover, CreateProjectDialog, SourceEnginePopover, and UserMenuPopover",
            "TopBar and ProjectsDashboard consume shared overlay wrappers",
        ],
    );
}

#[test]
fn overlay_primitives_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_overlay_primitives_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_overlay_primitives_contract.rs",
        &contract,
        &[
            "web/src/components/overlays/index.ts",
            "web/src/components/overlays/HubDialog.tsx",
            "web/src/components/overlays/HubMenu.tsx",
            "web/src/components/overlays/HubPopover.tsx",
            "web/src/components/overlays/CreateProjectDialog.tsx",
            "web/src/components/overlays/SourceEnginePopover.tsx",
            "web/src/components/overlays/UserMenuPopover.tsx",
            "web/src/components/shell/TopBar.tsx",
            "web/src/pages/ProjectsDashboard.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_overlay_primitives_contract.rs",
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
