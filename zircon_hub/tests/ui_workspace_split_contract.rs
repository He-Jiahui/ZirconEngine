//! Static contracts for shared Hub workspace split layout state.

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
fn workspace_split_state_centralizes_main_side_responsive_sizing() {
    let components = read_ui_file("components.slint");
    let layout = read_ui_file("layout.slint");
    let editor = read_ui_file("editor.slint");
    let builds = read_ui_file("builds.slint");
    let settings = read_ui_file("settings.slint");

    assert!(
        components.contains("HubWorkspaceSplitState,"),
        "components.slint must re-export HubWorkspaceSplitState with layout primitives"
    );

    let split_state = layout
        .split("export component HubWorkspaceSplitState")
        .nth(1)
        .and_then(|source| source.split("export component ResponsiveState").next())
        .expect("layout.slint must declare HubWorkspaceSplitState before ResponsiveState");
    for snippet in [
        "in property <length> content-width;",
        "in property <length> main-basis: HubTokens.panel-min-lg + HubTokens.control-lg * 2;",
        "in property <length> side-basis: HubTokens.panel-min-md + HubTokens.control-lg * 2;",
        "in property <length> gap: HubTokens.panel-gap;",
        "in property <length> compact-label-breakpoint: HubTokens.breakpoint-medium;",
        "in property <float> main-grow: 2;",
        "in property <float> side-grow: 1;",
        "out property <bool> compact: root.content-width < root.main-basis + root.side-basis + root.gap;",
        "out property <bool> compact-labels: root.content-width < root.compact-label-breakpoint;",
        "out property <length> main-min-width: root.compact ? root.content-width : root.main-basis;",
        "out property <length> side-min-width: root.compact ? root.content-width : root.side-basis;",
        "width: 0px;",
        "height: 0px;",
    ] {
        assert!(
            split_state.contains(snippet),
            "HubWorkspaceSplitState must own the shared main/side split rule; missing {snippet}"
        );
    }
    assert!(
        !split_state.contains("PanelSlot {") && !split_state.contains("WorkspacePanelSection {"),
        "HubWorkspaceSplitState should own only split layout state, not panel composition"
    );

    for (page_name, source, state_id) in [
        ("EditorPage", &editor, "workspace-split"),
        ("BuildsPage", &builds, "workspace-split"),
        ("SettingsPage", &settings, "workspace-split"),
    ] {
        for snippet in [
            &format!("{state_id} := HubWorkspaceSplitState {{"),
            "content-width: root.content-width;",
            "compact: workspace-split.compact;",
            "basis: workspace-split.main-basis;",
            "flex-basis: workspace-split.main-basis;",
            "basis: workspace-split.side-basis;",
            "flex-basis: workspace-split.side-basis;",
            "grow: workspace-split.main-grow;",
            "flex-grow: workspace-split.main-grow;",
            "grow: workspace-split.side-grow;",
            "flex-grow: workspace-split.side-grow;",
            "min-width: workspace-split.main-min-width;",
            "min-width: workspace-split.side-min-width;",
        ] {
            assert!(
                source.contains(snippet),
                "{page_name} must consume HubWorkspaceSplitState for main/side workspace sizing; missing {snippet}"
            );
        }
        for forbidden in [
            "side-panel-min-width:",
            "overview-min-width:",
            "root.overview-min-width",
            "root.side-panel-min-width",
            "root.content-width < HubTokens.panel-min-lg + HubTokens.control-lg + HubTokens.panel-min-md + HubTokens.panel-gap",
            "root.content-width < root.overview-min-width + root.side-panel-min-width + HubTokens.panel-gap",
            "min-width: root.compact ? root.content-width",
        ] {
            assert!(
                !source.contains(forbidden),
                "{page_name} should not keep page-local main/side split sizing after HubWorkspaceSplitState extraction: {forbidden}"
            );
        }
    }

    assert!(
        editor.contains("compact-label-breakpoint: HubTokens.breakpoint-wide;")
            && builds.contains("compact-label-breakpoint: HubTokens.breakpoint-medium;")
            && settings.contains("main-basis: HubTokens.panel-min-lg + HubTokens.control-lg;")
            && settings.contains("side-basis: HubTokens.panel-min-md;"),
        "Workspace pages should configure the shared split state for their existing label and panel-density differences"
    );
}
