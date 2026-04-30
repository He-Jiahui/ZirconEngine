use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn workbench_shell_contract_is_rust_owned_and_asset_projected() {
    let slint_host_mod = source("src/ui/slint_host/mod.rs");
    let host_window = source("src/ui/slint_host/host_contract/window.rs");
    let globals = source("src/ui/slint_host/host_contract/globals.rs");
    let host_root = source("src/ui/slint_host/host_contract/data/host_root.rs");
    let template_nodes = source("src/ui/slint_host/host_contract/data/template_nodes.rs");

    let generated_include = ["slint::", "include_modules!()"].concat();
    assert!(!slint_host_mod.contains(&generated_include));
    assert!(slint_host_mod.contains("mod host_contract"));
    assert!(host_window.contains("pub(crate) struct UiHostWindow"));
    assert!(host_window.contains("pub(crate) fn global<T>(&self) -> T"));
    assert!(globals.contains("pub(crate) struct UiHostContext"));
    assert!(globals.contains("pub(crate) struct PaneSurfaceHostContext"));
    assert!(host_root.contains("pub(crate) struct HostWindowPresentationData"));
    assert!(template_nodes.contains("pub(crate) struct TemplatePaneNodeData"));
}

#[test]
fn workbench_shell_assets_replace_deleted_shell_sources() {
    for (relative, markers) in [
        (
            "assets/ui/editor/host/workbench_shell.ui.toml",
            &["UiHostWindow", "root_menu_bar_0"] as &[_],
        ),
        (
            "assets/ui/editor/workbench_menu_chrome.ui.toml",
            &["WorkbenchMenuBarRoot", "MenuSlot0"],
        ),
        (
            "assets/ui/editor/workbench_menu_popup.ui.toml",
            &["WorkbenchMenuPopupRoot", "WorkbenchMenuPopupPanel"],
        ),
        (
            "assets/ui/editor/workbench_activity_rail.ui.toml",
            &["ActivityRailPanel", "ActivityRailButton0"],
        ),
        (
            "assets/ui/editor/workbench_status_bar.ui.toml",
            &["WorkbenchStatusBarRoot", "StatusViewportLabel"],
        ),
    ] {
        let asset = source(relative);
        for marker in markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }
}
