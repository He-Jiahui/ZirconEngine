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

    let shell = read_ui_file("shell.slint");
    for snippet in [
        "component HeaderControlSlot inherits Rectangle",
        "slot-height: HubTokens.shell-header-height;",
        "VerticalLayout {",
        "@children",
    ] {
        assert!(
            shell.contains(snippet),
            "shell.slint should provide a shared titlebar control slot that centers existing controls without replacing them; missing {snippet}"
        );
    }
    let header = shell
        .split("export component HubTopHeader")
        .nth(1)
        .and_then(|source| source.split("component NavStatusPanel").next())
        .expect("shell.slint must declare HubTopHeader before NavStatusPanel");
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
        "status-cluster-width: root.status-pill-width * root.header-statuses.length",
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
}
