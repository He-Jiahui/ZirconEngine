//! Static contracts for Zircon Hub shell window sizing and page slot layout.

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
fn app_shell_uses_preferred_window_size_and_remaining_content_width() {
    let app = read_ui_file("app.slint");
    let surfaces = read_ui_file("surfaces.slint");
    for snippet in [
        "import { Fill, HubTokens,",
        "resize-border-width: HubTokens.window-resize-border;",
        "min-width: HubTokens.window-min-width;",
        "min-height: HubTokens.window-min-height;",
        "preferred-width: HubTokens.window-preferred-width;",
        "preferred-height: HubTokens.window-preferred-height;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint must use native window constraints instead of fixed startup size; missing {snippet}"
        );
    }
    for forbidden in ["\n    width: 1600px;", "\n    height: 1024px;"] {
        assert!(
            !app.contains(forbidden),
            "HubWindow root must not set fixed {forbidden:?}; use preferred dimensions"
        );
    }
    for snippet in [
        "horizontal-stretch: 1;",
        "vertical-stretch: 1;",
        "min-width: 1px;",
        "preferred-width: 0px;",
        "min-height: 0px;",
        "preferred-height: 0px;",
        "responsive-state := ResponsiveState",
        "private property <bool> header-compact: !responsive-state.wide;",
        "private property <bool> header-tight: responsive-state.medium;",
        "private property <bool> header-minimal: responsive-state.compact;",
        "private property <bool> status-compact: responsive-state.medium;",
        "private property <bool> sidebar-compact-height: responsive-state.short;",
    ] {
        assert!(
            app.contains(snippet),
            "app.slint is missing required responsive shell contract snippet: {snippet}"
        );
    }
    assert!(
        app.contains("Fill {") && app.contains("clip: true;"),
        "app.slint must route selected pages through the shared Fill slot instead of a hand-sized page Rectangle"
    );
    let top_header_call = app
        .split("HubTopHeader {")
        .nth(1)
        .and_then(|source| source.split("Rectangle {").next())
        .expect("app.slint must declare HubTopHeader before shell body");
    for snippet in ["horizontal-stretch: 1;", "width: parent.width;"] {
        assert!(
            top_header_call.contains(snippet),
            "HubTopHeader must span the full window width so compact window buttons stay right-aligned; missing {snippet}"
        );
    }
    let header_group = surfaces
        .split("export component HeaderGroup")
        .nth(1)
        .and_then(|source| source.split("export component Badge").next())
        .expect("surfaces.slint must declare HeaderGroup before Badge");
    for snippet in [
        "horizontal-stretch: 1;",
        "min-width: 1px;",
        "preferred-width: 0px;",
    ] {
        assert!(
            header_group.contains(snippet),
            "HeaderGroup must stretch horizontally when hosted by shell layouts; missing {snippet}"
        );
    }
    let fill_slot = app
        .split("Fill {")
        .nth(1)
        .and_then(|source| source.split("HubStatusBar {").next())
        .expect("app.slint must declare the selected-page Fill slot before HubStatusBar");
    for forbidden in ["VerticalLayout {", "spacing: HubTokens.space-0;"] {
        assert!(
            !fill_slot.contains(forbidden),
            "selected-page Fill slot should inherit the layout primitive's internal child layout instead of declaring {forbidden}"
        );
    }
    for (marker, next_marker) in [
        (
            "if root.selected-page == \"projects\": ProjectsPage {",
            "if root.selected-page == \"editor\": EditorPage {",
        ),
        (
            "if root.selected-page == \"editor\": EditorPage {",
            "if root.selected-page == \"settings\": SettingsPage {",
        ),
        (
            "if root.selected-page == \"settings\": SettingsPage {",
            "if root.selected-page == \"builds\": BuildsPage {",
        ),
        (
            "if root.selected-page == \"builds\": BuildsPage {",
            "if root.selected-page == \"assets\": AssetsPage {",
        ),
        (
            "if root.selected-page == \"assets\": AssetsPage {",
            "if root.selected-page == \"plugins\": PluginsPage {",
        ),
        (
            "if root.selected-page == \"plugins\": PluginsPage {",
            "if root.selected-page == \"cloud\": CloudPage {",
        ),
        (
            "if root.selected-page == \"cloud\": CloudPage {",
            "if root.selected-page == \"team\": TeamPage {",
        ),
        (
            "if root.selected-page == \"team\": TeamPage {",
            "if root.selected-page == \"learn\": LearnPage {",
        ),
        (
            "if root.selected-page == \"learn\": LearnPage {",
            "HubStatusBar {",
        ),
    ] {
        let block = app
            .split(marker)
            .nth(1)
            .and_then(|source| source.split(next_marker).next())
            .unwrap_or_else(|| panic!("app.slint is missing selected-page block {marker}"));
        for snippet in ["horizontal-stretch: 1;", "vertical-stretch: 1;"] {
            assert!(
                block.contains(snippet),
                "selected-page block {marker} must fill the shared page slot with stretch constraints; missing {snippet}"
            );
        }
        for forbidden in ["width: parent.width;", "height: parent.height;"] {
            assert!(
                !block.contains(forbidden),
                "selected-page block {marker} should inherit geometry from the Fill slot instead of binding {forbidden}"
            );
        }
    }
    for forbidden in [
        "parent.width - root.nav-width",
        "max-width: max(1px, parent.width",
        "height: root.height - root.shell-header-height;",
    ] {
        assert!(
            !app.contains(forbidden),
            "app.slint must let Taffy/Slint allocate remaining shell space instead of hand-written subtraction: {forbidden}"
        );
    }
}
