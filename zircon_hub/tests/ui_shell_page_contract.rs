//! Static contracts for Zircon Hub page header and status chrome.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

#[test]
fn status_bar_delegates_detail_text_to_focused_helper() {
    let shell_page_components = read_ui_file("shell_page_components.slint");
    let status_detail_text = shell_page_components
        .split("component HubStatusDetailText")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect(
            "shell_page_components.slint must declare HubStatusDetailText before HubPageHeader",
        );
    let status_bar = shell_page_components
        .split("export component HubStatusBar")
        .nth(1)
        .expect("shell_page_components.slint must declare HubStatusBar");

    assert!(
        shell_page_components.contains("component HubStatusDetailText inherits MaterialText"),
        "shell_page_components.slint must keep HubStatusDetailText as a private focused helper"
    );
    for snippet in [
        "in property <string> detail-text;",
        "text: root.detail-text;",
        "color: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.label_medium;",
        "vertical_alignment: center;",
        "overflow: elide;",
        "horizontal-stretch: 1;",
    ] {
        assert!(
            status_detail_text.contains(snippet),
            "HubStatusDetailText must own bottom-status detail typography: {snippet}"
        );
    }
    for snippet in [
        "HubStatusDetailText {",
        "detail-text: root.status-detail;",
        "if !root.compact: Badge {",
        "text: root.project-context;",
        "text: root.engine-context;",
    ] {
        assert!(
            status_bar.contains(snippet),
            "HubStatusBar must keep status badges while delegating detail text: {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "            text: root.status-detail;",
        "style: MaterialTypography.label_medium;",
        "font-size:",
        "font-weight:",
    ] {
        assert!(
            !status_bar.contains(forbidden),
            "HubStatusBar should not own detail typography after adopting HubStatusDetailText: {forbidden}"
        );
    }
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
fn page_header_delegates_title_typography_to_focused_stack() {
    let shell_page_components = read_ui_file("shell_page_components.slint");
    let title_stack = shell_page_components
        .split("component HubPageHeaderTitleStack")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect(
            "shell_page_components.slint must declare HubPageHeaderTitleStack before HubPageHeader",
        );
    let page_header = shell_page_components
        .split("export component HubPageHeader")
        .nth(1)
        .and_then(|source| source.split("export component HubStatusBar").next())
        .expect("shell_page_components.slint must declare HubPageHeader before HubStatusBar");

    assert!(
        shell_page_components.contains("component HubPageHeaderTitleStack inherits Rectangle"),
        "shell_page_components.slint must keep HubPageHeaderTitleStack as a private focused helper"
    );
    for snippet in [
        "in property <string> title-text;",
        "in property <string> subtitle-text;",
        "in property <length> stack-height: HubTokens.shell-header-height;",
        "in property <length> stack-spacing: MaterialStyleMetrics.spacing_6;",
        "MaterialText {",
        "text: root.title-text;",
        "color: MaterialPalette.on_surface;",
        "style: MaterialTypography.headline_medium;",
        "text: root.subtitle-text;",
        "color: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.body_medium;",
        "overflow: elide;",
    ] {
        assert!(
            title_stack.contains(snippet),
            "HubPageHeaderTitleStack must own page header title/subtitle typography: {snippet}"
        );
    }
    for snippet in [
        "HubPageHeaderTitleStack {",
        "stack-height: parent.height;",
        "title-text: root.selected-page-title;",
        "subtitle-text: root.selected-page-subtitle;",
        "HubHeaderCommandGroup {",
        "root.page-action(root.primary-action-id);",
    ] {
        assert!(
            page_header.contains(snippet),
            "HubPageHeader must keep page action wiring while delegating title text: {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "                text: root.selected-page-title;",
        "                text: root.selected-page-subtitle;",
        "style: MaterialTypography.headline_medium;",
        "style: MaterialTypography.body_medium;",
        "font-size:",
        "font-weight:",
    ] {
        assert!(
            !page_header.contains(forbidden),
            "HubPageHeader should not own page header typography after adopting HubPageHeaderTitleStack: {forbidden}"
        );
    }
}
