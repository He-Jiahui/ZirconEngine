//! Static contracts for Zircon Hub top header chrome.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_ui_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(ui_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub UI file {name}: {error}");
        }),
    )
}

#[test]
fn top_header_uses_aligned_interactive_titlebar_regions() {
    let app = read_ui_file("app.slint");
    for snippet in [
        "private property <length> shell-drag-height: HubTokens.space-0;",
        "private property <length> shell-row-height: root.shell-header-height;",
    ] {
        assert!(
            app.contains(snippet),
            "Hub top header should not keep an invisible titlebar strip that offsets pointer hit testing; missing {snippet}"
        );
    }

    let shell_header_components = read_ui_file("shell_header_components.slint");
    let shell_header_popup_components = read_ui_file("shell_header_popup_components.slint");
    for snippet in [
        "component HeaderControlSlot inherits Rectangle",
        "slot-height: HubTokens.shell-header-height;",
        "VerticalLayout {",
        "@children",
    ] {
        assert!(
            shell_header_components.contains(snippet),
            "shell_header_components.slint should provide a shared titlebar control slot that centers existing controls without replacing them; missing {snippet}"
        );
    }
    let header = shell_header_components
        .split("export component HubTopHeader")
        .nth(1)
        .expect("shell_header_components.slint must export HubTopHeader");
    assert!(
        !ui_dir().join("shell.slint").exists(),
        "shell.slint was a migration-only compatibility note and must stay deleted after shell chrome extraction"
    );
    assert!(
        shell_header_components
            .contains("import { HeaderEngineSelector } from \"shell_header_popup_components.slint\";")
            && !shell_header_components.contains("export component HeaderEngineSelector")
            && !shell_header_components.contains("export component HeaderEngineOption")
            && shell_header_popup_components.contains("export component HeaderEngineSelector")
            && shell_header_popup_components.contains("export component HeaderEngineOption"),
        "shell_header_components.slint should import Source Engine popup components from shell_header_popup_components.slint instead of defining them inline"
    );
    assert!(
        shell_header_components.contains("HubIconButton,")
            && !shell_header_components.contains("component HeaderPlainIconButton"),
        "HubTopHeader should consume the shared HubIconButton primitive instead of owning a local plain icon button"
    );
    for snippet in [
        "HorizontalLayout {",
        "width: parent.width;",
        "height: root.row-height;",
        "horizontal-stretch: 1;",
        "padding-left: root.pad-x;",
        "padding-right: max(HubTokens.space-2, root.pad-x / 3);",
        "spacing: root.gap;",
        "alignment: stretch;",
    ] {
        assert!(
            header.contains(snippet),
            "HubTopHeader main titlebar row should keep main-axis stretch so compact windows do not center the whole chrome group; missing {snippet}"
        );
    }
    for snippet in [
        "width: root.brand-width;",
        "WindowDragRegion {",
        "status-running-pill-width: HubTokens.control-md * 3;",
        "status-standard-pill-width: HubTokens.control-md * 5 / 2 + HubTokens.space-2;",
        "status-error-pill-width: HubTokens.control-md * 2 + HubTokens.space-2;",
        "status-cluster-width: root.header-statuses.length == 0 ? 0px : root.status-running-pill-width + root.status-standard-pill-width * 2 + root.status-error-pill-width",
        "width: pill.icon == \">\" ? root.status-running-pill-width : (pill.state == \"error\" ? root.status-error-pill-width : root.status-standard-pill-width);",
        "slot-width: root.status-cluster-width;",
        "height: parent.height;",
        "region-height: parent.height;",
    ] {
        assert!(
            header.contains(snippet),
            "HubTopHeader should center titlebar controls and reserve drag hit testing for explicit titlebar regions; missing {snippet}"
        );
    }

    let brand_start = header
        .find("width: root.brand-width;")
        .expect("HubTopHeader must keep a brand slot");
    let selector_start = header
        .find("HeaderEngineSelector {")
        .expect("HubTopHeader must keep an engine selector after the brand slot");
    assert!(
        brand_start < selector_start,
        "HubTopHeader brand drag slot must stay before the engine selector so the selector remains a normal Material button"
    );

    for snippet in [
        "private property <string> brand-subtitle: root.project.selected ? root.project.title : root.ui-text.game-engine;",
        "text: root.brand-subtitle;",
    ] {
        assert!(
            header.contains(snippet),
            "HubTopHeader must show selected-project context when available and fall back to product copy in the empty selection state; missing {snippet}"
        );
    }
    assert!(
        !header.contains("private property <string> brand-subtitle: root.ui-text.game-engine;"),
        "HubTopHeader brand subtitle must not become static game-engine copy when a project is selected"
    );

    let user_menu = shell_header_popup_components
        .split("export component HeaderUserMenu")
        .nth(1)
        .expect("shell_header_popup_components.slint must export HeaderUserMenu");
    for forbidden in ["height: root.width;", "border-radius: root.width / 2;"] {
        assert!(
            !user_menu.contains(forbidden),
            "HeaderUserMenu popup avatar should use token-derived square sizing rather than popup width; found {forbidden}"
        );
    }
}
