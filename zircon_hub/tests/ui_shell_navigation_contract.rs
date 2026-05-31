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
    let shell_sidebar_components = read_ui_file("shell_sidebar_components.slint");
    let sidebar = shell_sidebar_components
        .split("export component HubNavSidebar")
        .nth(1)
        .expect("shell_sidebar_components.slint must export HubNavSidebar");

    for snippet in [
        "StateLayerArea,",
        "collapse-state := StateLayerArea {",
        "border_radius: HubVisualSpec.panel-radius;",
        "root.toggle-collapse();",
    ] {
        assert!(
            shell_sidebar_components.contains(snippet) || sidebar.contains(snippet),
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
        !ui_dir().join("shell.slint").exists(),
        "shell.slint was a migration-only compatibility note and must stay deleted; sidebar implementation belongs in shell_sidebar_components.slint and window drag TouchArea belongs in shell_header_components.slint"
    );
}
