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
    let shared_components = read_ui_file("shared.slint");
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
    let brand_title_stack = shell_header_components
        .split("component HeaderBrandTitleStack")
        .nth(1)
        .and_then(|source| source.split("export component HubTopHeader").next())
        .expect(
            "shell_header_components.slint must declare HeaderBrandTitleStack before HubTopHeader",
        );
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
        shell_header_components
            .contains("import { HubTopbarIconButton } from \"icon_button_components.slint\";")
            && !shell_header_components.contains("component HeaderPlainIconButton"),
        "HubTopHeader should consume the shared HubTopbarIconButton primitive instead of owning a local plain icon button"
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
        "HeaderBrandTitleStack {",
        "stack-height: parent.height;",
        "subtitle-text: root.brand-subtitle;",
    ] {
        assert!(
            header.contains(snippet),
            "HubTopHeader must forward selected-project context through the focused brand title stack; missing {snippet}"
        );
    }
    assert!(
        shell_header_components.contains("component HeaderBrandTitleStack inherits Rectangle"),
        "shell_header_components.slint must keep HeaderBrandTitleStack as a private focused helper"
    );
    for snippet in [
        "in property <string> title-text: \"ZIRCON HUB\";",
        "in property <string> subtitle-text;",
        "MaterialText {",
        "text: root.title-text;",
        "color: HubVisualSpec.brand-title-foreground;",
        "style: MaterialTypography.title_medium;",
        "text: root.subtitle-text;",
        "color: HubVisualSpec.brand-subtitle-foreground;",
        "style: MaterialTypography.body_medium;",
    ] {
        assert!(
            brand_title_stack.contains(snippet),
            "HeaderBrandTitleStack must own top-header brand typography: {snippet}"
        );
    }
    assert!(
        !header.contains("private property <string> brand-subtitle: root.ui-text.game-engine;"),
        "HubTopHeader brand subtitle must not become static game-engine copy when a project is selected"
    );
    for forbidden in [
        "MaterialText {",
        "text: \"ZIRCON HUB\";",
        "                        text: root.brand-subtitle;",
        "style: MaterialTypography.title_medium;",
        "style: MaterialTypography.body_medium;",
    ] {
        assert!(
            !header.contains(forbidden),
            "HubTopHeader should not own brand text after adopting HeaderBrandTitleStack: {forbidden}"
        );
    }
    for snippet in [
        "HubTopbarIconButton {",
        "button-size: root.header-button-size;",
        "icon-image: @image-url(\"../assets/icons/ui/bell.svg\");",
        "icon-image: @image-url(\"../assets/icons/ui/help.svg\");",
        "icon-image: @image-url(\"../assets/icons/ui/settings.svg\");",
        "clicked => {",
        "root.settings-clicked();",
    ] {
        assert!(
            header.contains(snippet),
            "HubTopHeader topbar tool buttons should consume HubTopbarIconButton instead of repeating transparent HubIconButton chrome: {snippet}"
        );
    }
    assert!(
        !header.contains("HubIconButton {")
            && !header.contains("idle-background: transparent;")
            && !header.contains("button-border-width: 0px;"),
        "HubTopHeader should not repeat transparent topbar HubIconButton styling after HubTopbarIconButton extraction"
    );
    assert!(
        !header.contains("danger: true;"),
        "HubTopHeader window controls should keep the neutral reference chrome; close must not use the red danger icon state"
    );
    for snippet in [
        "export component BrandMark inherits Rectangle",
        "width: MaterialStyleMetrics.size_40 + MaterialStyleMetrics.size_1;",
        "source: @image-url(\"../assets/brand/zircon-mark.svg\");",
        "y: MaterialStyleMetrics.size_1;",
        "opacity: 0.68;",
    ] {
        assert!(
            shared_components.contains(snippet),
            "BrandMark should keep the topbar mark sized and toned toward the reference header chrome; missing {snippet}"
        );
    }

    let user_menu = shell_header_popup_components
        .split("export component HeaderUserMenu")
        .nth(1)
        .expect("shell_header_popup_components.slint must export HeaderUserMenu");
    for snippet in [
        "HubUserMenuTriggerButton {",
        "avatar-text: root.ui-text.local-user-initials;",
        "user-name: root.ui-text.local-user;",
        "tight: root.tight;",
    ] {
        assert!(
            user_menu.contains(snippet),
            "HeaderUserMenu should route its trigger through the shared button wrapper; missing {snippet}"
        );
    }
    for forbidden in ["StateLayerArea {", "MaterialTypography.label_medium"] {
        assert!(
            !user_menu.contains(forbidden),
            "HeaderUserMenu should not retain local trigger state/text styling after delegating to HubUserMenuTriggerButton: {forbidden}"
        );
    }
    for forbidden in ["height: root.width;", "border-radius: root.width / 2;"] {
        assert!(
            !user_menu.contains(forbidden),
            "HeaderUserMenu popup avatar should use token-derived square sizing rather than popup width; found {forbidden}"
        );
    }
}
