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
    let tokens = read_ui_file("tokens.slint");
    let page_header_call = app
        .split("HubPageHeader {")
        .nth(1)
        .and_then(|source| source.split("ProjectsPage {").next())
        .expect("app.slint must compose HubPageHeader before routed pages");
    for snippet in [
        "header-visible: root.selected-page != \"projects\" || root.project-subpage == \"dashboard\";",
        "project-actions-visible: root.project-subpage == \"dashboard\";",
        "page-actions-visible: root.page-header-actions-visible;",
        "secondary-action-label: root.page-header-secondary-action-label;",
        "primary-action-label: root.page-header-primary-action-label;",
        "secondary-action-id: root.page-header-secondary-action-id;",
        "primary-action-id: root.page-header-primary-action-id;",
        "secondary-action-image: root.page-header-secondary-action-image;",
        "primary-action-image: root.page-header-primary-action-image;",
        "ui-text: root.ui-text;",
    ] {
        assert!(
            page_header_call.contains(snippet),
            "HubPageHeader must receive page title and shared page action wiring from HubWindow; missing {snippet}"
        );
    }
    for snippet in [
        "callback page-header-action(string);",
        "private property <string> page-header-secondary-action-id:",
        "\"refresh-sources\"",
        "\"refresh-assets\"",
        "\"refresh-plugins\"",
        "\"refresh-learn\"",
        "\"request-review\"",
        "\"open-output\"",
        "\"package-project\"",
        "\"reset-settings\"",
        "private property <string> page-header-primary-action-id:",
        "\"open-editor\"",
        "\"build-project\"",
        "\"deploy-preview\"",
        "\"open-source-control\"",
        "\"add-asset\"",
        "\"add-plugin\"",
        "\"add-guide\"",
        "\"save-settings\"",
        "page-action(id) => {",
        "root.page-header-action(id);",
    ] {
        assert!(
            app.contains(snippet),
            "HubPageHeader action wiring must use stable action IDs and one HubWindow callback; missing {snippet}"
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
    assert!(
        app.contains("HubStatusBar {")
            && !app.contains("if !(root.selected-page == \"projects\" && root.project-subpage == \"dashboard\"): HubStatusBar"),
        "HubStatusBar must remain visible on the Projects dashboard so selected-project context is not buried in a popup"
    );
    assert!(
        tokens.contains("status-badge-width: root.control-lg * 7;"),
        "HubStatusBar context badges should stay wide enough for selected-project and active-engine labels in the 1568px reference layout"
    );

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
    let components = read_ui_file("components.slint");
    let shell_header_components = read_ui_file("shell_header_components.slint");
    let shell_header_popup_components = read_ui_file("shell_header_popup_components.slint");
    let shell_layout_components = read_ui_file("shell_layout_components.slint");
    let shell_sidebar_components = read_ui_file("shell_sidebar_components.slint");
    let shell_page_components = read_ui_file("shell_page_components.slint");
    let button_components = read_ui_file("button_components.slint");
    let shell_surface = format!(
        "{shell_header_components}\n{shell_header_popup_components}\n{shell_layout_components}\n{shell_sidebar_components}\n{shell_page_components}"
    );
    assert!(
        !ui_dir().join("shell.slint").exists(),
        "shell.slint was a migration-only compatibility note and must stay deleted after focused shell component extraction"
    );
    assert!(
        components.contains("HubTopHeader,")
            && components.contains("} from \"shell_header_components.slint\";"),
        "components.slint should publicly export top-header chrome from shell_header_components.slint"
    );
    assert!(
        components.contains("HubShellRootSurface,")
            && components.contains("HubShellBody,")
            && components.contains("HubWorkspaceSurface")
            && components.contains("} from \"shell_layout_components.slint\";"),
        "components.slint should publicly export shell layout slots from shell_layout_components.slint"
    );
    for snippet in [
        "export component HubShellRootSurface inherits Rectangle",
        "export component HubShellStack inherits Rectangle",
        "export component HubShellBody inherits Rectangle",
        "export component HubWorkspaceSurface inherits Rectangle",
        "export component HubPageContentSlot inherits Fill",
    ] {
        assert!(
            shell_layout_components.contains(snippet),
            "shell_layout_components.slint should own shared AppBar/Drawer/workspace layout slots; missing {snippet}"
        );
    }
    assert!(
        !components.contains("shell_header_popup_components.slint")
            && shell_header_components
                .contains("HeaderEngineSelector")
            && shell_header_components.contains("from \"shell_header_popup_components.slint\";"),
        "shell_header_popup_components.slint should stay an internal top-header helper imported by shell_header_components.slint"
    );
    assert!(
        components.contains("HubNavSidebar,")
            && components.contains("} from \"shell_sidebar_components.slint\";"),
        "components.slint should publicly export sidebar chrome from shell_sidebar_components.slint"
    );
    assert!(
        components.contains("HubPageHeader,")
            && components.contains("HubStatusBar,")
            && components.contains("} from \"shell_page_components.slint\";"),
        "components.slint should publicly export page-title and bottom-status chrome from shell_page_components.slint"
    );
    let page_header = shell_page_components
        .split("export component HubPageHeader")
        .nth(1)
        .and_then(|source| source.split("export component HubStatusBar").next())
        .expect("shell_page_components.slint must export HubPageHeader before HubStatusBar");
    for snippet in [
        "in property <bool> header-visible: true;",
        "in property <bool> project-actions-visible: true;",
        "in property <bool> page-actions-visible: false;",
        "in property <string> secondary-action-label: \"\";",
        "in property <string> primary-action-label: \"\";",
        "in property <string> secondary-action-id: \"\";",
        "in property <string> primary-action-id: \"\";",
        "in property <image> secondary-action-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "in property <image> primary-action-image: @image-url(\"../assets/icons/ui/plus.svg\");",
        "private property <bool> projects-action-strip-visible: root.selected-page == \"projects\" && root.project-actions-visible;",
        "private property <bool> action-strip-visible: root.projects-action-strip-visible || root.page-actions-visible;",
        "height: root.header-visible ? root.page-title-height : 0px;",
        "clip: true;",
        "content-offset-y: HubTokens.space-3 + MaterialStyleMetrics.size_1;",
        "if root.action-strip-visible: VerticalLayout",
        "padding-bottom: MaterialStyleMetrics.size_8;",
        "height: root.action-height;",
        "height: root.action-height;",
        "text: root.projects-action-strip-visible ? root.ui-text.import-project : root.secondary-action-label;",
        "source-image: root.projects-action-strip-visible ? @image-url(\"../assets/icons/ui/import.svg\") : root.secondary-action-image;",
        "callback page-action(string);",
        "root.page-action(root.secondary-action-id);",
        "text: root.projects-action-strip-visible ? root.ui-text.create-project : root.primary-action-label;",
        "source-image: root.projects-action-strip-visible ? @image-url(\"../assets/icons/ui/plus.svg\") : root.primary-action-image;",
        "with-menu: root.projects-action-strip-visible || root.primary-action-with-menu;",
        "root.page-action(root.primary-action-id);",
        "text: root.selected-page-title;",
        "style: MaterialTypography.headline_medium;",
        "text: root.selected-page-subtitle;",
        "style: MaterialTypography.body_medium;",
        "HubCommandButton {",
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
    assert!(
        shell_page_components
            .contains("import { HubCommandButton } from \"button_components.slint\";")
            && !shell_page_components.contains("component HeaderActionButton"),
        "shell_page_components.slint should consume the shared HubCommandButton instead of owning HeaderActionButton locally"
    );
    assert!(
        button_components
            .contains("private property <color> primary-command-fill: HubVisualSpec.command-primary-fill;")
            && button_components
                .contains("private property <color> primary-command-stroke: HubVisualSpec.command-primary-stroke;")
            && button_components.contains(
                "background: root.primary ? root.primary-command-fill : HubVisualSpec.panel-background;",
            )
            && button_components
                .contains("color: root.primary ? root.primary-command-stroke : MaterialPalette.on_surface;")
            && button_components.contains("icon-size: HubTokens.icon-md;")
            && button_components.contains("style: MaterialTypography.label_medium;"),
        "HubCommandButton primary Projects action color should stay desaturated toward the fixed hub.png New Project button"
    );

    let status_bar = shell_page_components
        .split("export component HubStatusBar")
        .nth(1)
        .expect("shell_page_components.slint must export HubStatusBar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "in property <bool> compact: false;",
        "in property <length> context-badge-width: HubTokens.status-badge-width;",
        "preferred-height: root.status-height;",
        "max-height: root.status-height;",
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

    let engine_selector = shell_header_popup_components
        .split("export component HeaderEngineSelector")
        .nth(1)
        .expect("shell_header_popup_components.slint must export HeaderEngineSelector");
    for snippet in [
        "MaterialText {",
        "color: MaterialPalette.on_surface_variant;",
        "color: MaterialPalette.on_surface_variant;",
        "padding-left: HubTokens.space-3;",
        "text: root.ui-text.source-engines;",
        "style: MaterialTypography.label_large;",
        "text: root.ui-text.registered;",
        "style: MaterialTypography.label_medium;",
        "private property <length> popup-height: root.selector-height * 7;",
        "private property <length> popup-header-height: HubTokens.icon-lg;",
        "private property <length> popup-list-height: max(HubTokens.list-row-lg, root.popup-height - HubTokens.toolbar-gap * 2 - root.popup-header-height - HubTokens.space-2);",
        "private property <length> engine-popup-scroll-y: 0px;",
        "popup := HubAnchoredPopupPanel",
        "popup-height: root.popup-height;",
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
    assert!(
        !shell_header_components.contains("export component HeaderEngineSelector")
            && !shell_header_components.contains("export component HeaderEngineOption"),
        "Source Engine popup components should live in shell_header_popup_components.slint instead of shell_header_components.slint"
    );
    assert!(
        shell_header_popup_components.contains("import { HubAnchoredPopupPanel } from \"overlays.slint\";")
            && shell_header_popup_components.matches("HubAnchoredPopupPanel {").count() == 2
            && shell_header_popup_components.contains("user-menu-popup := HubAnchoredPopupPanel")
            && !shell_header_popup_components
                .lines()
                .any(|line| line.trim() == "PopupPanel {")
            && !shell_header_popup_components
                .lines()
                .any(|line| line.trim() == "HubPopupWindow {")
            && !shell_header_popup_components
                .lines()
                .any(|line| line.trim() == "HubPanel {"),
        "top-header popups should consume the shared anchored popup panel instead of instantiating popup window/panel shells directly"
    );
    assert!(
        !shell_header_popup_components.contains("popup := PopupWindow")
            && !shell_header_popup_components.contains("user-menu-popup := PopupWindow")
            && !shell_header_popup_components.contains("popup := HubPopupWindow")
            && !shell_header_popup_components.contains("user-menu-popup := HubPopupWindow"),
        "top-header popups should route PopupWindow ownership through HubAnchoredPopupPanel"
    );
    assert!(
        shell_header_popup_components
            .contains("import { HubMenuRow, PanelListViewport } from \"data_display.slint\";")
            && shell_header_popup_components
                .contains("export component HeaderEngineOption inherits HubMenuRow")
            && shell_header_popup_components.contains("callback picked(string);")
            && shell_header_popup_components.contains(
                "detail: root.engine.last-build == \"\" ? root.engine.status : root.engine.status + \" / \" + root.engine.last-build;"
            )
            && shell_header_popup_components.contains("selected: root.engine.active;")
            && shell_header_popup_components.contains("selected-border-width: HubTokens.border-width;")
            && shell_header_popup_components.contains("idle-background: transparent;")
            && shell_header_popup_components
                .contains("enabled-avatar-background: HubVisualSpec.neutral-icon-background;")
            && shell_header_popup_components
                .contains("enabled-avatar-foreground: HubVisualSpec.neutral-icon-foreground;"),
        "top-header Source Engine rows should consume the shared HubMenuRow primitive while preserving active/inactive popup row chrome"
    );
    let engine_option = shell_header_popup_components
        .split("export component HeaderEngineOption")
        .nth(1)
        .and_then(|source| source.split("export component HeaderEngineSelector").next())
        .expect("shell_header_popup_components.slint must declare HeaderEngineOption before HeaderEngineSelector");
    assert!(
        !engine_option.contains("ListTile {")
            && !engine_option.contains("border-radius: HubVisualSpec.panel-radius;")
            && !engine_option.contains("inherits InfoRow")
            && !engine_option
                .contains("background: root.engine.active ? MaterialPalette.secondary_container"),
        "HeaderEngineOption should not keep a local Material row shell after moving to HubMenuRow"
    );
    assert!(
        shell_header_popup_components
            .contains("import { HubMenuRow, PanelListViewport } from \"data_display.slint\";")
            && shell_header_popup_components
                .contains("component HeaderUserMenuAction inherits HubMenuRow")
            && shell_header_popup_components.contains("dense: true;")
            && shell_header_popup_components.contains("leading-image: root.icon-image;")
            && shell_header_popup_components.contains("has-leading-image: true;"),
        "top-header user menu actions should consume the shared HubMenuRow primitive while preserving compact popup row chrome"
    );
    let user_menu_action = shell_header_popup_components
        .split("component HeaderUserMenuAction")
        .nth(1)
        .and_then(|source| source.split("export component HeaderEngineOption").next())
        .expect("shell_header_popup_components.slint must declare HeaderUserMenuAction before HeaderEngineOption");
    assert!(
        !user_menu_action.contains("ListTile {")
            && !user_menu_action.contains("StateLayerArea {")
            && !user_menu_action.contains("inherits ActionRow")
            && !user_menu_action.contains("show-trailing")
            && !user_menu_action.contains("disabled-shell-opacity")
            && !user_menu_action.contains("border-color: HubVisualSpec.outline-muted;"),
        "HeaderUserMenuAction should not keep a local Material row shell after moving to HubMenuRow"
    );

    let top_header = shell_header_components
        .split("export component HubTopHeader")
        .nth(1)
        .expect("shell_header_components.slint must export HubTopHeader");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "in property <bool> tight: false;",
        "in property <bool> minimal: false;",
        "alignment: start;",
        "MaterialText {",
        "text: \"ZIRCON HUB\";",
        "style: MaterialTypography.title_medium;",
        "color: HubVisualSpec.brand-title-foreground;",
        "private property <string> brand-subtitle: root.project.selected ? root.project.title : root.ui-text.game-engine;",
        "text: root.brand-subtitle;",
        "color: HubVisualSpec.brand-subtitle-foreground;",
        "style: MaterialTypography.body_medium;",
        "idle-foreground: HubVisualSpec.topbar-icon-foreground;",
        "state-layer-color: HubVisualSpec.topbar-icon-foreground;",
    ] {
        assert!(
            top_header.contains(snippet),
            "HubTopHeader brand chrome should use MaterialText typography; missing {snippet}"
        );
    }
    for snippet in [
        "text: root.ui-text.local-user-initials;",
        "background: HubVisualSpec.user-avatar-background;",
        "color: HubVisualSpec.user-avatar-foreground;",
        "style: MaterialTypography.label_medium_prominent;",
        "text: root.ui-text.local-user;",
        "color: HubVisualSpec.user-name-foreground;",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            top_header.contains(snippet) || shell_header_popup_components.contains(snippet),
            "HubTopHeader user chrome or its popup helper should use MaterialText typography; missing {snippet}"
        );
    }
    assert!(
        !top_header.contains("responsive-state := ResponsiveState")
            && !top_header.contains("private property <bool> tight: responsive-state.medium;")
            && !top_header.contains("private property <bool> minimal: responsive-state.compact;"),
        "HubTopHeader must consume semantic tight/minimal state from app.slint instead of instantiating its own ResponsiveState"
    );
    assert!(
        shell_header_components.contains("export component HubTopHeader")
            && shell_header_components.contains("component HeaderControlSlot")
            && shell_header_components.contains("component WindowDragRegion")
            && shell_header_components.contains(
                "tone: pill.icon == \">\" ? \"running\" : (pill.state == \"ok\" ? \"success\" : (pill.state == \"warn\" ? \"warning\" : pill.state));"
            ),
        "top-header implementation and drag/control-slot helpers belong in shell_header_components.slint"
    );

    let nav_status_panel = shell_sidebar_components
        .split("export component NavStatusPanel inherits HubPanel")
        .nth(1)
        .and_then(|source| source.split("export component HubNavSidebar").next())
        .expect("shell_sidebar_components.slint must export NavStatusPanel");
    for snippet in [
        "in property <bool> task-running: false;",
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "in property <length> nav-width: HubTokens.nav-width-expanded-min;",
        "in property <length> nav-status-height:",
        "height: root.nav-status-height;",
        "padding-left: HubTokens.space-3;",
        "padding-right: HubTokens.space-3;",
        "padding-top: HubTokens.space-6;",
        "padding-bottom: HubTokens.space-4 + MaterialStyleMetrics.size_2;",
        "MaterialText {",
        "text: root.ui-text.engine-status;",
        "color: MaterialPalette.on_surface_variant;",
        "style: MaterialTypography.label_medium;",
        "text: root.source-engine.status;",
        "color: root.task-running ? HubVisualSpec.warning-stroke : HubVisualSpec.success-stroke;",
        "private property <string> project-context: root.ui-text.current-project + \": \" + root.project.title;",
        "if root.project.selected: Badge",
        "text: root.project-context;",
        "tone: \"accent\";",
        "badge-width: root.nav-width - HubTokens.space-6;",
        "variant: \"interactive\";",
        "source-image: @image-url(\"../assets/icons/ui/refresh.svg\");",
        "text: root.ui-text.check-for-updates;",
        "color: MaterialPalette.on_surface_variant.with_alpha(0.88);",
        "style: MaterialTypography.label_medium;",
    ] {
        assert!(
            nav_status_panel.contains(snippet),
            "NavStatusPanel must own the sidebar project/source-engine status card chrome; missing {snippet}"
        );
    }
    let nav_sidebar = shell_sidebar_components
        .split("export component HubNavSidebar")
        .nth(1)
        .expect("shell_sidebar_components.slint must export HubNavSidebar");
    for snippet in [
        "in property <ProjectDetailData> project;",
        "if !root.collapsed && !root.compact-height: NavStatusPanel {",
        "task-running: root.task-running;",
        "project: root.project;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "nav-width: root.nav-width;",
        "nav-status-height: root.nav-status-height;",
    ] {
        assert!(
            nav_sidebar.contains(snippet),
            "HubNavSidebar should compose NavStatusPanel instead of owning status-card internals; missing {snippet}"
        );
    }
    for snippet in [
        "in property <length> bottom-reserved-height: 0px;",
        "if root.bottom-reserved-height > 0px: Rectangle",
        "height: root.bottom-reserved-height;",
    ] {
        assert!(
            nav_sidebar.contains(snippet),
            "HubNavSidebar should reserve bottom chrome space for dashboard reference overlays; missing {snippet}"
        );
    }
    assert!(
        app.contains("private property <length> nav-status-height: (root.project-detail.selected ? root.shell-row-height * 3 : root.shell-row-height * 2) + HubTokens.space-6;"),
        "app.slint must reserve token-derived sidebar status height for engine-only and selected-project context states"
    );
    assert!(
        !nav_sidebar.contains("if !root.collapsed && !root.compact-height: HubPanel")
            && !nav_sidebar.contains("private property <string> project-title"),
        "HubNavSidebar should not keep project/source-engine status-card layout internals after NavStatusPanel extraction"
    );
    assert!(
        shell_sidebar_components.contains("component NavStatusPanel inherits HubPanel")
            && shell_sidebar_components.contains("export component HubNavSidebar")
            && shell_page_components.contains("root.ui-text.current-project"),
        "sidebar components belong directly in shell_sidebar_components.slint"
    );
    assert!(
        shell_page_components.contains("export component HubPageHeader")
            && shell_page_components.contains("export component HubStatusBar"),
        "page-title and bottom-status chrome belong directly in shell_page_components.slint"
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
        line_count <= 560,
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
        "window chrome implementation details belong in shell chrome component files"
    );

    for component in [
        "HeaderEngineOption",
        "HubTopHeader",
        "WindowDragRegion",
        "HeaderControlSlot",
        "NavStatusPanel",
        "HubNavSidebar",
        "HubPageHeader",
        "HubStatusBar",
        "HeaderEngineSelector",
        "NavRail",
    ] {
        assert!(
            shell_surface.contains(component),
            "shell chrome surface must declare or compose {component}"
        );
    }

    let header_engine_option = shell_header_popup_components
        .split("export component HeaderEngineOption")
        .nth(1)
        .and_then(|source| source.split("export component HeaderEngineSelector").next())
        .expect("shell_header_popup_components.slint must export HeaderEngineOption before HeaderEngineSelector");
    for snippet in [
        "inherits HubMenuRow {",
        "title: root.engine.title;",
        "detail: root.engine.last-build == \"\" ? root.engine.status : root.engine.status + \" / \" + root.engine.last-build;",
        "leading-image: @image-url(\"../assets/brand/zircon-mark.svg\");",
        "has-leading-image: true;",
        "selected: root.engine.active;",
        "row-height: HubTokens.list-row-md;",
        "selected-border-width: HubTokens.border-width;",
        "idle-background: transparent;",
        "enabled-avatar-background: HubVisualSpec.neutral-icon-background;",
        "enabled-avatar-foreground: HubVisualSpec.neutral-icon-foreground;",
        "clicked =>",
    ] {
        assert!(
            header_engine_option.contains(snippet),
            "HeaderEngineOption must route source-engine popup rows through HubMenuRow while preserving row bindings; missing {snippet}"
        );
    }
    assert!(
        !header_engine_option.contains("ListTile {")
            && !header_engine_option.contains("area := TouchArea")
            && !header_engine_option.contains("inherits InfoRow"),
        "HeaderEngineOption should not keep a local ListTile/TouchArea shell now that HubMenuRow owns row interaction"
    );

    let header_engine_selector = shell_header_popup_components
        .split("export component HeaderEngineSelector")
        .nth(1)
        .expect("shell_header_popup_components.slint must export HeaderEngineSelector");
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
