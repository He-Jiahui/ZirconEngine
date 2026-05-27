//! Static contracts for Zircon Hub sidebar navigation chrome.

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
fn sidebar_collapse_uses_material_state_layer() {
    let shell = read_ui_file("shell.slint");
    let sidebar = shell
        .split("export component HubNavSidebar")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect("shell.slint must declare HubNavSidebar before HubPageHeader");

    for snippet in [
        "StateLayerArea,",
        "collapse-state := StateLayerArea {",
        "border_radius: MaterialStyleMetrics.border_radius_12;",
        "root.toggle-collapse();",
    ] {
        assert!(
            shell.contains(snippet) || sidebar.contains(snippet),
            "HubNavSidebar collapse control must use Material StateLayerArea; missing {snippet}"
        );
    }

    for forbidden in ["collapse-area := TouchArea", "collapse-area.has-hover"] {
        assert!(
            !sidebar.contains(forbidden),
            "HubNavSidebar collapse control should not return to custom hover/click handling: {forbidden}"
        );
    }

    assert!(
        shell.matches("TouchArea").count() <= 1 && shell.contains("drag-area := TouchArea"),
        "shell.slint should reserve TouchArea for window dragging after Materializing collapse controls"
    );
}
