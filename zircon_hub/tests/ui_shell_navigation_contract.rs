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
    let button_components = read_ui_file("button_components.slint");
    let collapse_control = shell_sidebar_components
        .split("component HubSidebarCollapseControl")
        .nth(1)
        .and_then(|source| source.split("export component HubNavSidebar").next())
        .expect("shell_sidebar_components.slint must declare HubSidebarCollapseControl before HubNavSidebar");
    let sidebar = shell_sidebar_components
        .split("export component HubNavSidebar")
        .nth(1)
        .and_then(|source| source.split("export component HubNavigationDrawer").next())
        .expect("shell_sidebar_components.slint must export HubNavSidebar");

    assert!(
        shell_sidebar_components.contains("component HubSidebarCollapseControl inherits Rectangle"),
        "shell_sidebar_components.slint must declare HubSidebarCollapseControl as the drawer collapse primitive"
    );
    assert!(
        button_components.contains("export component HubSidebarCollapseButton inherits Rectangle"),
        "button_components.slint must own the sidebar collapse button chrome"
    );
    for snippet in [
        "in property <bool> collapsed: false;",
        "in property <string> text: \"\";",
        "in property <length> button-height: HubTokens.control-md;",
        "StateLayerArea {",
        "border_radius: root.button-radius;",
        "root.clicked();",
        "source-image: root.collapsed ? @image-url(\"../assets/icons/ui/chevron-right.svg\") : @image-url(\"../assets/icons/ui/collapse.svg\");",
        "if !root.collapsed: MaterialText",
    ] {
        assert!(
            button_components.contains(snippet),
            "HubSidebarCollapseButton must own the Material StateLayerArea collapse affordance; missing {snippet}"
        );
    }
    for snippet in [
        "in property <bool> collapsed: false;",
        "in property <string> collapse-label: \"\";",
        "in property <length> control-height: HubTokens.control-md;",
        "HubSidebarCollapseButton {",
        "button-height: root.control-height;",
        "button-radius: HubVisualSpec.panel-radius;",
        "collapsed: root.collapsed;",
        "text: root.collapse-label;",
        "foreground: MaterialPalette.on_surface_variant;",
        "state-layer-color: MaterialPalette.on_surface;",
        "root.clicked();",
    ] {
        assert!(
            collapse_control.contains(snippet),
            "HubSidebarCollapseControl must delegate collapse button chrome through HubSidebarCollapseButton; missing {snippet}"
        );
    }
    for forbidden in [
        "StateLayerArea {",
        "TouchArea",
        "source-image: root.collapsed",
    ] {
        assert!(
            !collapse_control.contains(forbidden),
            "HubSidebarCollapseControl should not keep local collapse-button internals after delegating to HubSidebarCollapseButton: {forbidden}"
        );
    }

    for snippet in [
        "HubSidebarCollapseControl {",
        "control-height: max(HubTokens.control-md, min(HubTokens.control-lg, root.row-height * 3 / 4));",
        "collapsed: root.collapsed;",
        "collapse-label: root.ui-text.collapse;",
        "clicked =>",
        "root.toggle-collapse();",
    ] {
        assert!(
            sidebar.contains(snippet),
            "HubNavSidebar must compose HubSidebarCollapseControl instead of owning collapse hit testing; missing {snippet}"
        );
    }

    for forbidden in [
        "collapse-state := StateLayerArea",
        "StateLayerArea {",
        "collapse-area := TouchArea",
        "collapse-area.has-hover",
    ] {
        assert!(
            !sidebar.contains(forbidden),
            "HubNavSidebar collapse control should not return to local hover/click handling: {forbidden}"
        );
    }

    assert!(
        !ui_dir().join("shell.slint").exists(),
        "shell.slint was a migration-only compatibility note and must stay deleted; sidebar implementation belongs in shell_sidebar_components.slint and window drag TouchArea belongs in shell_header_components.slint"
    );
}

#[test]
fn nav_status_panel_delegates_text_and_update_action_helpers() {
    let shell_sidebar_components = read_ui_file("shell_sidebar_components.slint");
    let summary_stack = shell_sidebar_components
        .split("component NavStatusSummaryStack")
        .nth(1)
        .and_then(|source| source.split("component NavStatusUpdateAction").next())
        .expect(
            "shell_sidebar_components.slint must declare NavStatusSummaryStack before NavStatusUpdateAction",
        );
    let update_action = shell_sidebar_components
        .split("component NavStatusUpdateAction")
        .nth(1)
        .and_then(|source| source.split("export component NavStatusPanel").next())
        .expect(
            "shell_sidebar_components.slint must declare NavStatusUpdateAction before NavStatusPanel",
        );
    let nav_status_panel = shell_sidebar_components
        .split("export component NavStatusPanel")
        .nth(1)
        .and_then(|source| source.split("component HubSidebarCollapseControl").next())
        .expect(
            "shell_sidebar_components.slint must declare NavStatusPanel before HubSidebarCollapseControl",
        );

    for declaration in [
        "component NavStatusSummaryStack inherits VerticalLayout",
        "component NavStatusUpdateAction inherits HubPanel",
    ] {
        assert!(
            shell_sidebar_components.contains(declaration),
            "shell_sidebar_components.slint must keep sidebar status helpers private: {declaration}"
        );
    }
    for snippet in [
        "in property <bool> task-running: false;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "background: root.task-running ? HubVisualSpec.warning-stroke : HubVisualSpec.success-stroke;",
        "MaterialText {",
        "text: root.ui-text.engine-status;",
        "color: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.label_medium;",
        "MutedText { text: root.source-engine.version; }",
        "text: root.source-engine.status;",
        "color: root.task-running ? HubVisualSpec.warning-stroke : HubVisualSpec.success-stroke;",
    ] {
        assert!(
            summary_stack.contains(snippet),
            "NavStatusSummaryStack must own engine status text and running-state summary chrome: {snippet}"
        );
    }
    for snippet in [
        "variant: \"interactive\";",
        "height: HubTokens.control-md;",
        "CenteredIcon {",
        "source-image: @image-url(\"../assets/icons/ui/refresh.svg\");",
        "MaterialText {",
        "text: root.action-text;",
        "color: MaterialPalette.on_surface_variant.with_alpha(0.88);",
        "style: MaterialTypography.label_medium;",
        "vertical_alignment: center;",
    ] {
        assert!(
            update_action.contains(snippet),
            "NavStatusUpdateAction must own the sidebar update action row: {snippet}"
        );
    }
    for snippet in [
        "NavStatusSummaryStack {",
        "task-running: root.task-running;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "if root.project.selected: Badge",
        "text: root.project-context;",
        "NavStatusUpdateAction {",
        "action-text: root.ui-text.check-for-updates;",
    ] {
        assert!(
            nav_status_panel.contains(snippet),
            "NavStatusPanel must keep panel/badge ownership while delegating text helpers: {snippet}"
        );
    }
    for forbidden in [
        "MaterialText {",
        "MutedText {",
        "text: root.ui-text.engine-status;",
        "text: root.source-engine.status;",
        "                    text: root.ui-text.check-for-updates;",
        "source-image: @image-url(\"../assets/icons/ui/refresh.svg\");",
    ] {
        assert!(
            !nav_status_panel.contains(forbidden),
            "NavStatusPanel should not own text/update internals after adopting sidebar status helpers: {forbidden}"
        );
    }
}
