use std::fs;
use std::path::Path;

fn source(relative: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn workbench_shell_contract_is_rust_owned_and_asset_projected() {
    let retained_host_mod = source("src/ui/retained_host/mod.rs");
    let host_window = source("src/ui/retained_host/host_contract/window.rs");
    let host_window_lifecycle = source("src/ui/retained_host/host_contract/window/lifecycle.rs");
    let globals = source("src/ui/retained_host/host_contract/globals.rs");
    let ui_context = source("src/ui/retained_host/host_contract/globals/ui_context.rs");
    let pane_context = source("src/ui/retained_host/host_contract/globals/pane_context.rs");
    let host_root = source("src/ui/retained_host/host_contract/data/host_root.rs");
    let template_nodes = source("src/ui/retained_host/host_contract/data/template_nodes.rs");
    let template_node_data =
        source("src/ui/retained_host/host_contract/data/template_nodes/node.rs");

    let generated_include = [
        "crate::ui::retained_host::primitives::",
        "include_modules!()",
    ]
    .concat();
    assert!(!retained_host_mod.contains(&generated_include));
    assert!(retained_host_mod.contains("mod host_contract"));
    assert!(host_window.contains("pub(crate) struct UiHostWindow"));
    assert!(host_window_lifecycle.contains("pub(crate) fn global<T>(&self) -> T"));
    assert!(globals.contains("pub(crate) use ui_context::UiHostContext"));
    assert!(ui_context.contains("pub(crate) struct UiHostContext"));
    assert!(globals.contains("pub(crate) use pane_context::PaneSurfaceHostContext"));
    assert!(pane_context.contains("pub(crate) struct PaneSurfaceHostContext"));
    assert!(host_root.contains("pub(crate) struct HostWindowPresentationData"));
    assert!(template_nodes.contains("pub(crate) use node::*"));
    assert!(template_node_data.contains("pub(crate) struct TemplatePaneNodeData"));
}

#[test]
fn workbench_shell_assets_replace_deleted_shell_sources() {
    for (relative, markers) in [
        (
            "assets/ui/editor/host/workbench_shell.zui",
            &[
                "UiHostWindowRoot",
                "WorkbenchBody",
                "editor_workbench_strict.zui",
                "res://ui/editor/components/workbench/primitives/inputs/workbench_icon_button.zui#WorkbenchIconButton",
                "res://ui/editor/components/workbench/primitives/chrome/workbench_rail_button.zui#WorkbenchRailButton",
                "res://ui/editor/components/workbench/primitives/feedback/workbench_status_item.zui#WorkbenchStatusItem",
                "WorkbenchScaffold",
                "StatusBarRoot",
            ] as &[_],
        ),
        (
            "assets/ui/editor/workbench_menu_chrome.zui",
            &["WorkbenchMenuBarRoot", "MenuSlot0"],
        ),
        (
            "assets/ui/editor/workbench_menu_popup.zui",
            &["WorkbenchMenuPopupRoot", "WorkbenchMenuPopupPanel"],
        ),
        (
            "assets/ui/editor/workbench_activity_rail.zui",
            &["ActivityRailPanel", "ActivityRailButton0"],
        ),
        (
            "assets/ui/editor/workbench_status_bar.zui",
            &["WorkbenchStatusBarRoot", "StatusViewportLabel"],
        ),
    ] {
        let asset = source(relative);
        for marker in markers {
            assert!(asset.contains(marker), "{relative} missing `{marker}`");
        }
    }

    let shell = source("assets/ui/editor/host/workbench_shell.zui");
    for forbidden in [
        "WorkbenchShellReferenceImage",
        "ui/editor/reference/workbench.png",
        "docs/ui-and-layout/workbench.png",
        "component = \"IconButton\"",
        "component = \"Label\"",
    ] {
        assert!(
            !shell.contains(forbidden),
            "workbench shell must stay componentized instead of rendering `{forbidden}`"
        );
    }
}
