//! Static contracts for the React/MUI Hub shell window and page slot layout.

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
            "{source_name} should contain shell-window snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete shell-window snippet {snippet:?}"
        );
    }
}

#[test]
fn tauri_config_owns_fixed_hub_window_size_and_web_dist_boundary() {
    let config = read_crate_file("tauri.conf.json");
    let capability = read_crate_file("capabilities/default.json");

    assert_contains_all(
        "tauri.conf.json",
        &config,
        &[
            "\"productName\": \"Zircon Hub\"",
            "\"beforeDevCommand\": \"npm run dev\"",
            "\"beforeBuildCommand\": \"npm run build\"",
            "\"devUrl\": \"http://localhost:1420\"",
            "\"frontendDist\": \"web/dist\"",
            "\"label\": \"main\"",
            "\"title\": \"Zircon Hub\"",
            "\"width\": 1568",
            "\"height\": 1003",
            "\"minWidth\": 960",
            "\"minHeight\": 680",
            "\"resizable\": true",
            "\"decorations\": false",
            "\"transparent\": true",
            "\"center\": true",
        ],
    );
    assert_contains_all(
        "capabilities/default.json",
        &capability,
        &[
            "\"windows\": [\"main\"]",
            "\"core:default\"",
            "\"core:window:allow-minimize\"",
            "\"core:window:allow-toggle-maximize\"",
            "\"core:window:allow-close\"",
        ],
    );
}

#[test]
fn tokens_centralize_window_density_and_shell_dimensions() {
    let tokens = read_crate_file("web/src/theme/tokens.ts");

    assert_contains_all(
        "tokens.ts",
        &tokens,
        &[
            "window: {",
            "width: 1568",
            "height: 1003",
            "topBarHeight: 73",
            "sidebarWidth: 222",
            "sidebarCollapsedWidth: 78",
            "pagePaddingX: 30",
            "pagePaddingY: 28",
            "radius: {",
            "compact: 7",
            "panel: 8",
            "card: 8",
            "pill: 999",
            "colors: {",
            "background: \"#111212\"",
            "accent: \"#21d5cf\"",
            "gradients: {",
            "shadows: {",
        ],
    );
}

#[test]
fn hub_window_owns_viewport_shell_slots_and_page_router_without_page_sizing_leaks() {
    let hub_window = read_crate_file("web/src/components/shell/HubWindow.tsx");

    assert_contains_all(
        "HubWindow.tsx",
        &hub_window,
        &[
            "export interface HubWindowProps",
            "state: HubShellState;",
            "onAction: HubActionHandler;",
            "width: \"100vw\"",
            "height: \"100vh\"",
            "minWidth: 0",
            "minHeight: 0",
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
            "flex: \"1 1 auto\"",
            "backgroundColor: \"rgba(17,17,17,0.55)\"",
            "const pageRoutes: Record<HubPageId, HubPageComponent> = {",
            "projects: ProjectsDashboard,",
            "settings: SettingsPage,",
            "const PageComponent = activeRoute ? pageRoutes[activeRoute] : WorkspacePage;",
            "<PageComponent state={state} onAction={onAction} />",
        ],
    );
    assert_not_contains_any(
        "HubWindow.tsx",
        &hub_window,
        &[
            "position: \"absolute\"",
            "position: \"fixed\"",
            "left:",
            "right:",
            "bottom:",
            "loadHubState",
            "dispatchHubAction",
            "HubSnackbar",
        ],
    );
}

#[test]
fn topbar_window_controls_are_bound_to_tauri_current_window_api() {
    let topbar = read_crate_file("web/src/components/shell/TopBar.tsx");

    assert_contains_all(
        "TopBar.tsx",
        &topbar,
        &[
            "import { getCurrentWindow } from \"@tauri-apps/api/window\";",
            "appWindow.minimize()",
            "appWindow.toggleMaximize()",
            "appWindow.close()",
            "onClick={handleMinimize}",
            "onClick={handleToggleMaximize}",
            "onClick={handleClose}",
            "function runWindowAction(action: (appWindow: TauriWindow) => Promise<void>)",
            "!(\"__TAURI_INTERNALS__\" in window)",
        ],
    );
}

#[test]
fn rust_launcher_enters_tauri_app_without_old_compiled_ui_module() {
    let main_rs = read_crate_file("src/main.rs");
    let lib_rs = read_crate_file("src/lib.rs");

    assert_contains_all(
        "main.rs",
        &main_rs,
        &[
            "#![cfg_attr(not(debug_assertions), windows_subsystem = \"windows\")]",
            "fn main() -> Result<(), zircon_hub::HubError>",
            "zircon_hub::tauri_app::run()",
        ],
    );
    assert_contains_all(
        "lib.rs",
        &lib_rs,
        &["pub mod tauri_app;", "pub use error::HubError;"],
    );
    assert_not_contains_any(
        "lib.rs",
        &lib_rs,
        &["pub mod app;", "include_modules", "slint"],
    );
}

#[test]
fn shell_window_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_shell_window_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_shell_window_contract",
            "## Shell Window Contract Cutover",
            "React/MUI shell window layout",
            "zircon_hub/tauri.conf.json",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/theme/tokens.ts",
            "zircon_hub/src/main.rs",
            "zircon_hub/src/lib.rs",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_shell_window_contract.rs`",
            "React/MUI shell window layout",
            "Tauri window config, explicit self-drawn window-control permissions, shared window tokens, HubWindow viewport shell slots, and Tauri launcher entry",
        ],
    );
}

#[test]
fn shell_window_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_shell_window_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_shell_window_contract.rs",
        &contract,
        &[
            "tauri.conf.json",
            "web/src/components/shell/HubWindow.tsx",
            "web/src/theme/tokens.ts",
            "src/main.rs",
            "src/lib.rs",
        ],
    );
    assert_not_contains_any(
        "ui_shell_window_contract.rs",
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
