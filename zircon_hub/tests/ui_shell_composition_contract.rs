//! Static contracts for Zircon Hub shell component composition.

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
fn app_window_routes_shell_chrome_through_components() {
    let app = read_ui_file("app.slint");
    let page_header_call = app
        .split("HubPageHeader {")
        .nth(1)
        .and_then(|source| source.split("ProjectsPage {").next())
        .expect("app.slint must compose HubPageHeader before routed pages");
    for snippet in [
        "header-visible: root.selected-page != \"projects\" || root.project-subpage == \"dashboard\";",
        "project-actions-visible: root.project-subpage == \"dashboard\";",
        "ui-text: root.ui-text;",
    ] {
        assert!(
            page_header_call.contains(snippet),
            "HubPageHeader must receive page title and Projects action wiring from HubWindow; missing {snippet}"
        );
    }
    for removed_snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "status-label: root.status-label;",
        "context-compact: root.status-compact;",
        "context-badge-width: HubTokens.status-badge-width;",
    ] {
        assert!(
            !page_header_call.contains(removed_snippet),
            "HubPageHeader should not duplicate project/engine/status context already owned by HubStatusBar; found {removed_snippet}"
        );
    }

    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "compact: root.status-compact;",
        "context-badge-width: HubTokens.status-badge-width;",
    ] {
        assert!(
            app.contains(snippet),
            "HubStatusBar must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }

    let top_header_call = app
        .split("HubTopHeader {")
        .nth(1)
        .and_then(|source| source.split("HubNavSidebar {").next())
        .expect("app.slint must compose HubTopHeader before HubNavSidebar");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "tight: root.header-tight;",
        "minimal: root.header-minimal;",
    ] {
        assert!(
            top_header_call.contains(snippet),
            "HubTopHeader must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }

    let nav_sidebar_call = app
        .split("HubNavSidebar {")
        .nth(1)
        .and_then(|source| source.split("clicked(id) =>").next())
        .expect("app.slint must compose HubNavSidebar before routed pages");
    for snippet in [
        "project: root.project-detail;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
    ] {
        assert!(
            nav_sidebar_call.contains(snippet),
            "HubNavSidebar must receive selected project and active Source Engine context from HubWindow; missing {snippet}"
        );
    }
    assert!(
        app.contains("private property <length> nav-status-height: root.shell-row-height * 4;"),
        "app.slint must leave enough token-derived sidebar status height for project and engine context"
    );

    let shell = read_ui_file("shell.slint");
    let page_header = shell
        .split("export component HubPageHeader")
        .nth(1)
        .and_then(|source| source.split("export component HubStatusBar").next())
        .expect("shell.slint must declare HubPageHeader before HubStatusBar");
    for snippet in [
        "in property <bool> header-visible: true;",
        "in property <bool> project-actions-visible: true;",
        "height: root.header-visible ? root.page-title-height : 0px;",
        "clip: true;",
        "if root.selected-page == \"projects\" && root.project-actions-visible: VerticalLayout",
        "height: root.action-height;",
        "height: root.action-height;",
        "text: root.selected-page-title;",
        "style: MaterialTypography.headline_medium;",
    ] {
        assert!(
            page_header.contains(snippet),
            "HubPageHeader must surface page title and Projects actions without duplicating bottom status context; missing {snippet}"
        );
    }
    for removed_snippet in [
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <string> status-label;",
        "in property <bool> task-running:",
        "in property <bool> context-compact:",
        "in property <length> context-badge-width:",
        "if root.selected-page != \"projects\": VerticalLayout",
        "text: root.status-label;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "root.source-engine.title == \"\" ? root.ui-text.no-source-engines : root.source-engine.title",
    ] {
        assert!(
            !page_header.contains(removed_snippet),
            "HubPageHeader must stay a clean page-title/action component and leave status context to HubStatusBar; found {removed_snippet}"
        );
    }
    assert!(
        !page_header.contains("root.width <") && !page_header.contains("root.width /"),
        "HubPageHeader must use app-level responsive state instead of deriving layout from its own width"
    );
    assert!(
        !page_header.contains("font-size:") && !page_header.contains("font-weight:"),
        "HubPageHeader typography should stay on MaterialText styles instead of raw Text font bindings"
    );

    let status_bar = shell
        .split("export component HubStatusBar")
        .nth(1)
        .expect("shell.slint must declare HubStatusBar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "in property <bool> compact: false;",
        "in property <length> context-badge-width: HubTokens.status-badge-width;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "root.ui-text.active-source-engine + \": \" + root.source-engine.title",
        "if !root.compact: Badge",
        "badge-width: root.context-badge-width;",
        "tone: root.project.selected ? \"accent\" : \"neutral\";",
        "MaterialText {",
        "text: root.status-detail;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            status_bar.contains(snippet),
            "HubStatusBar must surface selected-project and active Source Engine badges without crowding compact widths; missing {snippet}"
        );
    }
    assert!(
        !status_bar.contains("root.width <") && !status_bar.contains("root.width /"),
        "HubStatusBar must not derive responsive layout from its own resolved width; app.slint should pass compact state and token badge width"
    );
    assert!(
        !status_bar.lines().any(|line| line.trim() == "Text {")
            && !status_bar.contains("font-size:"),
        "HubStatusBar status detail should stay on MaterialText typography instead of raw Text font bindings"
    );

    let engine_selector = shell
        .split("component HeaderEngineSelector")
        .nth(1)
        .and_then(|source| source.split("export component HubTopHeader").next())
        .expect("shell.slint must declare HeaderEngineSelector before HubTopHeader");
    for snippet in [
        "MaterialText {",
        "text: root.ui-text.source-engines;",
        "style: MaterialTypography.label_large;",
        "text: root.ui-text.registered;",
        "style: MaterialTypography.label_medium;",
        "private property <length> popup-height: root.selector-height * 7;",
        "private property <length> popup-header-height: HubTokens.icon-lg;",
        "private property <length> popup-list-height: max(HubTokens.list-row-lg, root.popup-height - HubTokens.toolbar-gap * 2 - root.popup-header-height - HubTokens.space-2);",
        "private property <length> engine-popup-scroll-y: 0px;",
        "height: root.popup-height;",
        "engine-list := PanelListViewport {",
        "height: root.popup-list-height;",
        "scroll-y <=> root.engine-popup-scroll-y;",
        "row-count: root.engine-count;",
        "row-height: HubTokens.list-row-md;",
        "empty-height: HubTokens.list-row-lg;",
    ] {
        assert!(
            engine_selector.contains(snippet),
            "HeaderEngineSelector popup chrome should use MaterialText typography; missing {snippet}"
        );
    }

    let top_header = shell
        .split("export component HubTopHeader")
        .nth(1)
        .and_then(|source| source.split("component NavStatusPanel").next())
        .expect("shell.slint must declare HubTopHeader before NavStatusPanel");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <bool> tight: false;",
        "in property <bool> minimal: false;",
        "alignment: center;",
        "MaterialText {",
        "text: \"ZIRCON HUB\";",
        "style: MaterialTypography.title_medium;",
        "root.project.selected ? root.project.title : root.ui-text.game-engine",
        "text: root.brand-subtitle;",
        "text: root.ui-text.local-user-initials;",
        "style: MaterialTypography.label_medium_prominent;",
        "text: root.ui-text.local-user;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            top_header.contains(snippet),
            "HubTopHeader brand and user chrome should use MaterialText typography; missing {snippet}"
        );
    }
    assert!(
        !top_header.contains("responsive-state := ResponsiveState")
            && !top_header.contains("private property <bool> tight: responsive-state.medium;")
            && !top_header.contains("private property <bool> minimal: responsive-state.compact;"),
        "HubTopHeader must consume semantic tight/minimal state from app.slint instead of instantiating its own ResponsiveState"
    );

    let nav_status_panel = shell
        .split("component NavStatusPanel inherits HubPanel")
        .nth(1)
        .and_then(|source| source.split("export component HubNavSidebar").next())
        .expect("shell.slint must declare NavStatusPanel before HubNavSidebar");
    for snippet in [
        "in property <bool> task-running: false;",
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "in property <length> nav-status-height:",
        "height: root.nav-status-height;",
        "MaterialText {",
        "text: root.ui-text.engine-status;",
        "style: MaterialTypography.label_large;",
        "Badge {",
        "text: root.source-engine.status;",
        "text: root.ui-text.current-project;",
        "root.project.selected ? root.project.title : root.ui-text.no-project-selected",
        "text: root.project-title;",
        "color: root.project.selected ? MaterialPalette.on_surface : MaterialPalette.on_surface_variant;",
        "variant: \"interactive\";",
        "source-image: @image-url(\"../assets/icons/ui/refresh.svg\");",
        "text: root.ui-text.check-for-updates;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            nav_status_panel.contains(snippet),
            "NavStatusPanel must own the sidebar project/source-engine status card chrome; missing {snippet}"
        );
    }

    let nav_sidebar = shell
        .split("export component HubNavSidebar")
        .nth(1)
        .and_then(|source| source.split("export component HubPageHeader").next())
        .expect("shell.slint must declare HubNavSidebar before HubPageHeader");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "if !root.collapsed && !root.compact-height: NavStatusPanel {",
        "task-running: root.task-running;",
        "project: root.project;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "nav-status-height: root.nav-status-height;",
    ] {
        assert!(
            nav_sidebar.contains(snippet),
            "HubNavSidebar should compose NavStatusPanel instead of owning status-card internals; missing {snippet}"
        );
    }
    assert!(
        !nav_sidebar.contains("if !root.collapsed && !root.compact-height: HubPanel")
            && !nav_sidebar.contains("private property <string> project-title"),
        "HubNavSidebar should not keep project/source-engine status-card layout internals after NavStatusPanel extraction"
    );

    for (name, source) in [
        ("HeaderEngineSelector", engine_selector),
        ("HubTopHeader", top_header),
        ("NavStatusPanel", nav_status_panel),
        ("HubNavSidebar", nav_sidebar),
    ] {
        assert!(
            !source.lines().any(|line| {
                let trimmed = line.trim();
                trimmed == "Text {" || trimmed.ends_with(": Text {")
            }) && !source.contains("font-size:")
                && !source.contains("font-weight:"),
            "{name} shell typography should not return to raw Text font bindings"
        );
    }

    let line_count = app.lines().count();
    assert!(
        line_count <= 520,
        "app.slint should keep shell composition thin; found {line_count} lines"
    );

    for component in [
        "HubTopHeader",
        "HubNavSidebar",
        "HubPageHeader",
        "HubStatusBar",
    ] {
        assert!(
            app.contains(component),
            "app.slint must route shell chrome through {component}"
        );
    }

    assert!(
        !app.contains("component HeaderEngineOption")
            && !app.contains("component HeaderEngineSelector")
            && !app.contains("for item in root.nav-items: NavButton"),
        "window chrome implementation details belong in shell.slint"
    );

    for component in [
        "HubTopHeader",
        "NavStatusPanel",
        "HubNavSidebar",
        "HubPageHeader",
        "HubStatusBar",
        "HeaderEngineSelector",
        "NavRail",
    ] {
        assert!(
            shell.contains(component),
            "shell.slint must declare or compose {component}"
        );
    }

    let header_engine_option = shell
        .split("component HeaderEngineOption")
        .nth(1)
        .and_then(|source| source.split("component HeaderEngineSelector").next())
        .expect("shell.slint must declare HeaderEngineOption before HeaderEngineSelector");
    for snippet in [
        "height: HubTokens.list-row-md;",
        "ListTile {",
        "text: root.engine.title;",
        "supporting_text: root.engine.status + \" / \" + root.engine.last-build;",
        "avatar_icon:",
        "avatar_background:",
        "avatar_foreground:",
        "clicked =>",
    ] {
        assert!(
            header_engine_option.contains(snippet),
            "HeaderEngineOption must use Material ListTile for source-engine popup rows; missing {snippet}"
        );
    }
    assert!(
        !header_engine_option.contains("area := TouchArea"),
        "HeaderEngineOption should not keep a custom full-row TouchArea now that ListTile owns row interaction"
    );

    let header_engine_selector = shell
        .split("component HeaderEngineSelector")
        .nth(1)
        .and_then(|source| source.split("component WindowDragRegion").next())
        .expect("shell.slint must declare HeaderEngineSelector before WindowDragRegion");
    for snippet in [
        "engine-list := PanelListViewport {",
        "height: root.popup-list-height;",
        "scroll-y <=> root.engine-popup-scroll-y;",
        "row-count: root.engine-count;",
        "row-height: HubTokens.list-row-md;",
        "empty-height: HubTokens.list-row-lg;",
        "if root.engine-count == 0: EmptyStateBlock",
        "height: HubTokens.list-row-lg;",
        "title: root.ui-text.no-source-engines;",
        "body-padding: HubTokens.space-3;",
        "center-content: true;",
    ] {
        assert!(
            header_engine_selector.contains(snippet),
            "HeaderEngineSelector should use the shared empty-state block for empty source-engine popup content; missing {snippet}"
        );
    }
    assert!(
        !header_engine_selector.contains("if root.engine-count == 0: MutedText"),
        "HeaderEngineSelector should not render empty source-engine popup content as a loose MutedText row"
    );
}
