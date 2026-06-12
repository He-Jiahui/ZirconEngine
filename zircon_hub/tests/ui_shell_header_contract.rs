//! Static contracts for React/MUI Zircon Hub top header chrome.

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
            "{source_name} should contain shell-header snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete shell-header snippet {snippet:?}"
        );
    }
}

#[test]
fn topbar_owns_brand_engine_status_user_and_window_control_regions() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "import { Avatar, Box, ButtonBase, Divider, Typography } from \"@mui/material\";",
            "import { brandMark } from \"../../data/hubData\";",
            "import { getCurrentWindow } from \"@tauri-apps/api/window\";",
            "import { StatusBadge } from \"../data\";",
            "import { HubIconButton } from \"../inputs\";",
            "import { SourceEnginePopover, UserMenuPopover } from \"../overlays\";",
            "export interface TopBarProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "component=\"header\"",
            "height: hubTokens.window.topBarHeight",
            "gridTemplateColumns: \"222px minmax(0, 1fr) auto\"",
            "gridTemplateColumns: \"78px minmax(0, 1fr) auto\"",
            "src={brandMark}",
            "{state.productName}",
            "{state.ui.shell.productCategory}",
            "activeEngine?.name ?? state.engineVersion",
            "state.taskStatus.map((status) =>",
            "<StatusBadge key={status.id} label={status.label} tone={status.tone} />",
            "const userName = state.team.identityName || state.ui.common.notConfigured;",
            "const userInitials = initialsFromName(userName);",
            "const handleMinimize = () => runWindowAction((appWindow) => appWindow.minimize());",
            "const handleToggleMaximize = () => runWindowAction((appWindow) => appWindow.toggleMaximize());",
            "const handleClose = () => runWindowAction((appWindow) => appWindow.close());",
            "{userName}",
            "HubIconButton label={state.ui.shell.minimize} onClick={handleMinimize}",
            "HubIconButton label={state.ui.shell.maximize} onClick={handleToggleMaximize}",
            "HubIconButton label={state.ui.shell.close} onClick={handleClose}",
        ],
    );
}

#[test]
fn topbar_routes_engine_user_settings_and_help_regions_through_shared_actions() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "const [engineAnchor, setEngineAnchor] = useState<HTMLElement | null>(null);",
            "const [userAnchor, setUserAnchor] = useState<HTMLElement | null>(null);",
            "const handleUserAction = (actionId: string) => {",
            "if (actionId === \"preferences\")",
            "void onAction(HUB_ACTION.showPage, \"settings\")",
            "if (actionId === \"documentation\")",
            "void onAction(HUB_ACTION.showPage, \"learn\")",
            "if (actionId === \"account\")",
            "void onAction(HUB_ACTION.showPage, \"team\")",
            "const notificationDetail = comingSoonDetail(state, \"notification-center\");",
            "const signOutDetail = comingSoonDetail(state, \"sign-out\");",
            "HubIconButton label={state.ui.shell.notifications} tooltip={notificationDetail} disabled",
            "HubIconButton label={state.ui.shell.help} onClick={() => void onAction(HUB_ACTION.showPage, \"learn\")}",
            "HubIconButton label={state.ui.shell.settings} onClick={() => void onAction(HUB_ACTION.showPage, \"settings\")}",
            "\"&.Mui-disabled\"",
            "onClick={(event) => setEngineAnchor(event.currentTarget)}",
            "onClick={(event) => setUserAnchor(event.currentTarget)}",
            "void onAction(HUB_ACTION.selectEngine, engineId);",
            "void onAction(HUB_ACTION.showPage, \"settings\");",
            "onAction={handleUserAction}",
        ],
    );
    assert_not_contains_any(
        "TopBar.tsx",
        &topbar,
        &[
            "HubIconButton label={state.ui.shell.notifications} sx={topIconSx}",
            "HubIconButton label={state.ui.shell.help} sx={topIconSx}",
        ],
    );
}

#[test]
fn status_badge_and_icon_button_own_reusable_header_chrome() {
    let status_badge = read_crate_file("web/src/components/data/StatusBadge.tsx");
    let icon_button = read_crate_file("web/src/components/inputs/HubIconButton.tsx");

    assert_contains_all(
        "StatusBadge.tsx",
        &status_badge,
        &[
            "export interface StatusBadgeProps",
            "label: string;",
            "tone: StatusTone;",
            "const toneMap: Record<StatusTone",
            "running:",
            "success:",
            "warning:",
            "error:",
            "neutral:",
            "Icon: PlayArrowIcon",
            "Icon: CheckCircleIcon",
            "Icon: WarningIcon",
            "Icon: ErrorIcon",
            "height: 36",
            "minWidth: 112",
            "borderRadius: `${hubTokens.radius.compact}px`",
            "tone === \"running\"",
        ],
    );
    assert_contains_all(
        "HubIconButton.tsx",
        &icon_button,
        &[
            "export interface HubIconButtonProps extends IconButtonProps",
            "selected?: boolean;",
            "label: string;",
            "tooltip?: string;",
            "Tooltip title={tooltip ?? label}",
            "aria-label={label}",
            "width: 50",
            "height: 42",
            "backgroundColor: selected ?",
            "\"&.Mui-disabled\"",
            "...asSxArray(sx)",
        ],
    );
}

#[test]
fn brand_asset_and_fallback_header_state_stay_centralized() {
    let data = read_crate_file("web/src/data/hubData.ts");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "hubData.ts",
        &data,
        &[
            "import brandMarkAsset from \"../../../assets/brand/zircon-mark.svg\";",
            "export const brandMark = brandMarkAsset;",
            "productName: \"Zircon Hub\"",
            "engineVersion: \"Zircon Engine 1.8.2\"",
            "taskStatus: []",
            "activeSourceEngineId: null",
            "id: \"notification-center\"",
            "detail: \"桌面通知为预留能力；v1 在 Hub 窗口内显示本地任务反馈。\"",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "productName: string;",
            "engineVersion: string;",
            "activeSourceEngineId: string | null;",
            "taskStatus: HubStatusPill[];",
            "sourceEngines: HubSourceEngineSummary[];",
            "settings: HubSettingsSummary;",
        ],
    );
    assert_not_contains_any(
        "hubData.ts",
        &data,
        &["docs/ui-and-layout", "hub-ai-drafts", "hub.png"],
    );
}

#[test]
fn user_menu_keeps_local_v1_sign_out_reserved_and_disabled() {
    let user_menu = read_crate_file("web/src/components/overlays/UserMenuPopover.tsx");
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let coming_soon = read_crate_file("src/tauri_app/view_model/coming_soon.rs");
    let fallback_data = read_crate_file("web/src/data/hubData.ts");

    assert_contains_all(
        "UserMenuPopover.tsx",
        &user_menu,
        &[
            "signOutDetail: string;",
            "{ id: \"sign-out\", label: text.signOut, detail: signOutDetail, Icon: LogoutOutlinedIcon, danger: true, disabled: true }",
            "menuItems.map(({ id, label, detail, Icon, danger, disabled }) => {",
            "const isDisabled = Boolean(disabled);",
            "disabled={isDisabled}",
            "if (isDisabled) {",
            "return;",
            "onAction(id);",
            "onClose();",
            "color: isDisabled ? hubTokens.colors.textMuted : danger ? hubTokens.colors.error : hubTokens.colors.text",
            "backgroundColor: isDisabled ? \"transparent\"",
            "\"&.Mui-disabled\"",
            "cursor: \"not-allowed\"",
        ],
    );
    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "const signOutDetail = comingSoonDetail(state, \"sign-out\");",
            "signOutDetail={signOutDetail}",
        ],
    );
    assert_not_contains_any(
        "TopBar.tsx",
        &topbar,
        &[
            "if (actionId === \"sign-out\")",
            "state.ui.shell.signOutDetail",
        ],
    );
    assert_contains_all(
        "coming_soon.rs",
        &coming_soon,
        &[
            "\"notification-center\"",
            "Desktop notifications are reserved; v1 shows local task feedback in the Hub window.",
            "桌面通知为预留能力；v1 在 Hub 窗口内显示本地任务反馈。",
            "\"sign-out\"",
            "Remote accounts are disabled for the local-only Hub.",
            "本地版 Hub 不启用远程账号。",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &fallback_data,
        &[
            "id: \"sign-out\"",
            "detail: \"本地版 Hub 不启用远程账号。\"",
        ],
    );
}

#[test]
fn frameless_window_controls_call_tauri_current_window_actions() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");
    let capability = read_crate_file("capabilities/default.json");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "import { getCurrentWindow } from \"@tauri-apps/api/window\";",
            "const handleMinimize = () => runWindowAction((appWindow) => appWindow.minimize());",
            "const handleToggleMaximize = () => runWindowAction((appWindow) => appWindow.toggleMaximize());",
            "const handleClose = () => runWindowAction((appWindow) => appWindow.close());",
            "HubIconButton label={state.ui.shell.minimize} onClick={handleMinimize}",
            "HubIconButton label={state.ui.shell.maximize} onClick={handleToggleMaximize}",
            "HubIconButton label={state.ui.shell.close} onClick={handleClose}",
            "type TauriWindow = ReturnType<typeof getCurrentWindow>;",
            "function runWindowAction(action: (appWindow: TauriWindow) => Promise<void>)",
            "if (typeof window === \"undefined\" || !(\"__TAURI_INTERNALS__\" in window))",
            "void action(getCurrentWindow());",
        ],
    );
    assert_not_contains_any(
        "TopBar.tsx",
        &topbar,
        &[
            "HubIconButton label={state.ui.shell.minimize} sx={windowIconSx}",
            "HubIconButton label={state.ui.shell.maximize} sx={windowIconSx}",
            "HubIconButton label={state.ui.shell.close} sx={windowIconSx}",
            "void onAction(HUB_ACTION.minimize",
        ],
    );
    assert_contains_all(
        "capabilities/default.json",
        &capability,
        &[
            "\"core:window:allow-minimize\"",
            "\"core:window:allow-toggle-maximize\"",
            "\"core:window:allow-close\"",
        ],
    );
}

#[test]
fn shell_header_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_shell_header_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_header_contract",
            "## Shell Header Contract Cutover",
            "React/MUI shell header chrome",
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/data/StatusBadge.tsx",
            "web/src/components/inputs/HubIconButton.tsx",
            "web/src/data/hubData.ts",
            "disabled local-v1 account-service reservation",
            "disabled local-v1 notification-service reservation",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_shell_header_contract.rs`",
            "React/MUI shell header chrome",
            "brand, engine selector, status badges, user menu, settings/help tools, disabled local-v1 notification-service reservation, and window controls",
            "disabled local-v1 account-service reservation",
            "disabled local-v1 notification-service reservation",
        ],
    );
}

#[test]
fn shell_header_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_shell_header_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_shell_header_contract.rs",
        &contract,
        &[
            "web/src/components/shell/TopBar.tsx",
            "web/src/components/data/StatusBadge.tsx",
            "web/src/components/inputs/HubIconButton.tsx",
            "web/src/data/hubData.ts",
            "web/src/types/hub.ts",
        ],
    );
    assert_not_contains_any(
        "ui_shell_header_contract.rs",
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
