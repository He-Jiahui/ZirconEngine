//! Static contracts for Projects-page Taffy layout usage.

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

fn assert_semantic_taffy_properties_have_slint_flex_pairs(page: &str, source: &str) {
    let lines = source.lines().collect::<Vec<_>>();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some((semantic, flex_property)) = trimmed
            .strip_prefix("basis: ")
            .map(|value| (value, "flex-basis"))
            .or_else(|| {
                trimmed
                    .strip_prefix("grow: ")
                    .map(|value| (value, "flex-grow"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("shrink: ")
                    .map(|value| (value, "flex-shrink"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("order: ")
                    .map(|value| (value, "flex-order"))
            })
        else {
            continue;
        };
        let expected = format!("{flex_property}: {semantic}");
        let next = lines
            .get(index + 1)
            .map(|line| line.trim())
            .unwrap_or_default();
        assert_eq!(
            next, expected,
            "{page} must keep Slint-required {flex_property} directly paired with semantic ResponsiveSlot sizing"
        );
    }

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some((flex_property, semantic_property)) = trimmed
            .strip_prefix("flex-basis: ")
            .map(|_| ("flex-basis", "basis"))
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-grow: ")
                    .map(|_| ("flex-grow", "grow"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-shrink: ")
                    .map(|_| ("flex-shrink", "shrink"))
            })
            .or_else(|| {
                trimmed
                    .strip_prefix("flex-order: ")
                    .map(|_| ("flex-order", "order"))
            })
        else {
            continue;
        };
        let previous = index
            .checked_sub(1)
            .and_then(|previous| lines.get(previous))
            .map(|line| line.trim())
            .unwrap_or_default();
        assert!(
            previous.starts_with(&format!("{semantic_property}: ")),
            "{page} must not hand-code {flex_property} without the matching semantic ResponsiveSlot {semantic_property}"
        );
    }
}

#[test]
fn project_pages_use_responsive_taffy_sizing() {
    let app = read_ui_file("app.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let projects = read_ui_file("projects.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let project_surface = format!("{project_pages}\n{project_components}");
    let browser_page = project_pages
        .split("export component ProjectBrowserPage")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailPage").next())
        .expect("project_pages.slint must define ProjectBrowserPage before ProjectDetailPage");
    for (page, source) in [
        ("ProjectDashboardPage", &dashboard),
        ("Project secondary pages", &project_pages),
    ] {
        assert_semantic_taffy_properties_have_slint_flex_pairs(page, source);
    }
    for (page, source, snippets) in [
        (
            "ProjectDashboardPage",
            &dashboard,
            &[
                "basis: root.dashboard-toolbar-search-basis;",
                "flex-basis: root.dashboard-toolbar-search-basis;",
                "grow: root.dashboard-toolbar-wrap ? 1 : 0;",
                "flex-grow: root.dashboard-toolbar-wrap ? 1 : 0;",
                "basis: root.compact ? root.content-width : root.dashboard-main-basis;",
                "flex-basis: root.compact ? root.content-width : root.dashboard-main-basis;",
            ][..],
        ),
        (
            "Project secondary pages",
            &project_pages,
            &[
                "basis: root.toolbar-search-basis;",
                "flex-basis: root.toolbar-search-basis;",
                "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
                "flex-basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
                "grow: 2;",
                "flex-grow: 2;",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{page} should keep ResponsiveSlot/PanelSlot semantic sizing and mirror it to Slint flex properties at the direct Flexbox child use site: {snippet}"
            );
        }
    }
    for snippet in [
        "for card in root.project-cards: ProjectCard",
        "min-width: max(HubTokens.panel-min-sm * 2 / 3, root.card-basis);",
        "card-width-basis: root.card-basis;",
        "raw-card-columns: Math.floor((root.flow-width + root.card-gap-x) / (root.card-basis + root.card-gap-x));",
        "card-column-count: root.raw-card-columns < 1 ? 1 : root.raw-card-columns;",
        "card-row-count: (root.project-card-count + root.card-column-count - 1) / root.card-column-count;",
        "expanded-visible-rows: root.card-row-count < 3 ? root.card-row-count : 3;",
        "flow-visible-height: root.visible-row-count * root.card-height + (root.visible-row-count - 1) * root.card-gap-y;",
        "flow-content-height: root.card-row-count * root.card-height + (root.card-row-count - 1) * root.card-gap-y;",
        "viewport_y <=> root.card-scroll-y;",
        "viewport_height: root.expanded ? root.flow-content-height : root.flow-visible-height;",
        "project: card;",
        "flow-width: parent.width;",
        "card-basis: max(HubTokens.panel-min-sm * 2 / 3, root.flow-width * HubTokens.project-dashboard-card-ratio);",
        "dashboard-toolbar-search-basis: root.dashboard-toolbar-wrap ? root.content-width",
        "dashboard-toolbar-select-basis: root.dashboard-toolbar-wrap ? root.toolbar-control-min-width",
        "basis: root.dashboard-toolbar-search-basis;",
        "basis: root.dashboard-toolbar-select-basis;",
        "min-width: root.dashboard-toolbar-select-basis;",
        "dashboard-table-width: max(root.toolbar-height * 8, root.content-width * HubTokens.project-dashboard-table-ratio);",
        "dashboard-main-basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "dashboard-side-basis: HubTokens.panel-min-md;",
        "quick-action-count: root.quick-actions.length;",
        "component DashboardQuickActionRow inherits ActionRow",
        "action: root.quick-action-data;",
        "action-list := PanelListViewport {",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
        "row-height: root.quick-action-row-height;",
        "row-spacing: root.quick-action-row-gap;",
        "empty-height: HubTokens.list-row-lg;",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "quick-action-data: action;",
        "triggered(id) => { root.quick-action(id); }",
        "if root.quick-action-count == 0: EmptyStateBlock {",
        "title: root.ui-text.no-quick-actions;",
        "detail: root.ui-text.quick-actions-empty-detail;",
        "center-content: true;",
        "basis: root.compact ? root.content-width : root.dashboard-main-basis;",
        "basis: root.compact ? root.content-width : root.dashboard-side-basis;",
        "grow: 2;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-sm;",
    ] {
        assert!(
            dashboard.contains(snippet),
            "ProjectDashboardPage is missing dashboard Taffy sizing snippet: {snippet}"
        );
    }
    assert!(
        !dashboard.contains("dashboard-column-width"),
        "ProjectDashboardPage lower panels should not return to page-local remaining width formulas"
    );
    for forbidden in [
        "dashboard-toolbar-select-width",
        "root.content-width - root.toolbar-height",
        "root.content-width - root.page-gap * 3",
        "root.content-width / 4",
        "root.content-width / 9",
        "root.card-basis * 4 / 5",
        "root.content-width * 23 / 100",
        "root.available-content * 23 / 100",
        "available-content:",
        "card-height-token:",
        "dashboard-card-basis",
        "dashboard-card-height",
        "dashboard-flow-height",
        "root.content-width * 58 / 100",
        "page-surface.viewport-height * 3 / 5",
        "page-surface.viewport-height * 7 / 20",
    ] {
        assert!(
            !dashboard.contains(forbidden),
            "ProjectDashboardPage should not return to toolbar/card remaining-width formulas: {forbidden}"
        );
    }
    assert!(
        app.contains("viewport-compact: responsive-state.compact;"),
        "HubWindow must pass viewport compact state from shared ResponsiveState into ProjectsPage"
    );
    assert!(
        projects.contains("in property <bool> viewport-compact: false;"),
        "ProjectsPage must expose viewport compact state for secondary project pages"
    );
    assert!(
        projects
            .matches("viewport-compact: root.viewport-compact;")
            .count()
            >= 3,
        "ProjectsPage must forward viewport compact state into ProjectNewPage, ProjectBrowserPage, and ProjectDetailPage"
    );
    for snippet in [
        "in property <bool> viewport-compact: false;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
    ] {
        assert!(
            browser_page.contains(snippet),
            "ProjectBrowserPage must use viewport compact state for its secondary-page spacing; missing {snippet}"
        );
    }
    for snippet in [
        "narrow-flow: root.content-width < HubTokens.panel-min-lg + HubTokens.panel-min-md + root.page-gap;",
        "flex-wrap: root.narrow-flow ? FlexboxLayoutWrap.wrap : FlexboxLayoutWrap.no-wrap;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-lg;",
        "basis: root.narrow-flow ? root.content-width : HubTokens.panel-min-md;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
        "toolbar-search-basis: root.toolbar-wrap ? root.content-width : root.content-width * HubTokens.project-browser-toolbar-search-ratio;",
        "basis: root.toolbar-search-basis;",
        "toolbar-select-basis: root.toolbar-wrap ? root.toolbar-control-min-width",
        "basis: root.toolbar-select-basis;",
        "min-width: root.toolbar-select-basis;",
        "compact-page: root.viewport-compact;",
        "page-pad: root.compact-page ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "page-gap: root.compact-page ? HubTokens.toolbar-gap : HubTokens.panel-gap;",
        "form-panel-height: HubTokens.space-4 * 2 + HubTokens.list-row-sm + root.field-height * 2 + root.engine-section-height + root.create-action-row-height + root.page-gap * 4;",
        "component ProjectCreateField inherits Rectangle",
        "ProjectCreateField {",
        "field-label: root.ui-text.project-name;",
        "field-text <=> root.project-name;",
        "field-label: root.ui-text.location;",
        "field-text <=> root.project-location;",
        "show-browse: true;",
        "root.browse-folder(\"new-project-location\");",
        "component ProjectCreateActionRow inherits Rectangle",
        "in property <string> action-label;",
        "in property <bool> action-enabled;",
        "callback action-clicked();",
        "clicked => { root.action-clicked(); }",
        "ProjectCreateActionRow {",
        "row-height: root.create-action-row-height;",
        "row-spacing: root.page-gap;",
        "action-label: root.ui-text.create;",
        "action-enabled: root.form-ready;",
        "action-clicked => { root.create-project(); }",
        "summary-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "summary-section-height: root.summary-header-height + root.summary-row-height * 2 + root.page-gap;",
        "summary-panel-padding: root.narrow-flow ? HubTokens.space-3 : HubTokens.space-4;",
        "engine-panel-rows: root.engine-count < 1 ? 1 : (root.engine-count > 3 ? 3 : root.engine-count);",
        "engine-row-gap: MaterialStyleMetrics.spacing_8;",
        "engine-list-height: root.engine-count == 0 ? root.choice-row-height : root.engine-panel-rows * root.choice-row-height + (root.engine-panel-rows - 1) * root.engine-row-gap;",
        "new-engine-scroll-y: 0px;",
        "component ProjectEngineChoiceList inherits Rectangle",
        "in-out property <length> list-scroll-y: 0px;",
        "engine-list := PanelListViewport {",
        "width: parent.width;",
        "scroll-y <=> root.list-scroll-y;",
        "for engine in root.engines: EngineChoiceRow {",
        "selected(id) => { root.selected(id); }",
        "ProjectEngineChoiceList {",
        "list-height: root.engine-list-height;",
        "list-scroll-y <=> root.new-engine-scroll-y;",
        "row-count: root.engine-count;",
        "row-height: root.choice-row-height;",
        "empty-height: root.choice-row-height;",
        "empty-title: root.ui-text.register-source-engine-before-create;",
        "summary-panel-height: root.summary-panel-padding * 2 + root.summary-section-height;",
        "project-settings-panel-height: root.narrow-flow ? root.form-panel-height : root.form-panel-height + root.summary-section-height + root.page-gap;",
        "template-panel-rows: root.template-count < 1 ? 1 : (root.template-count > 4 ? 4 : root.template-count);",
        "template-list-height: root.template-count == 0 ? HubTokens.list-row-lg : root.template-panel-rows * root.template-row-height + (root.template-panel-rows - 1) * root.page-gap;",
        "template-panel-height: HubTokens.space-4 * 2 + HubTokens.control-md + root.template-list-height + root.page-gap;",
        "template-scroll-y: 0px;",
        "template-list := PanelListViewport",
        "height: root.template-list-height;",
        "scroll-y <=> root.template-scroll-y;",
        "row-count: root.template-count;",
        "row-height: root.template-row-height;",
        "empty-height: HubTokens.list-row-lg;",
        "flow-visible-height: max(root.content-height - root.header-height - root.page-gap, root.project-settings-panel-height);",
        "flow-height: root.narrow-flow ? root.project-settings-panel-height + root.page-gap + root.summary-panel-height + root.page-gap + root.template-panel-height : max(root.flow-visible-height, root.template-panel-height);",
        "height: root.choice-row-height;",
        "title: root.ui-text.register-source-engine-before-create;",
        "height: root.narrow-flow ? root.project-settings-panel-height : root.flow-height;",
        "height: root.narrow-flow ? root.summary-panel-height : 0px;",
        "height: root.narrow-flow ? root.template-panel-height : root.flow-height;",
        "browser-row-slot-height: root.browser-row-height + root.browser-row-gap;",
        "browser-panel-chrome-height: HubTokens.control-md + root.page-gap * 3;",
        "browser-fit-row-count: Math.floor(root.browser-available-list-height / root.browser-row-slot-height);",
        "browser-list-height: root.row-count == 0 ? HubTokens.list-row-lg : root.browser-panel-rows * root.browser-row-height + (root.browser-panel-rows - 1) * root.browser-row-gap;",
        "browser-panel-height: root.browser-panel-chrome-height + root.browser-list-height;",
        "browser-list := PanelListViewport",
        "height: root.browser-list-height;",
        "empty-height: HubTokens.list-row-lg;",
        "if root.row-count == 0: EmptyStateBlock",
        "height: HubTokens.list-row-lg;",
        "title: root.ui-text.no-projects-match;",
        "body-padding: HubTokens.space-4;",
        "center-content: true;",
        "detail-panel-padding: root.narrow-flow ? HubTokens.space-3 : HubTokens.space-4;",
        "cover-height: root.narrow-flow ? HubTokens.list-row-lg",
        "detail-title-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "detail-info-header-height: root.narrow-flow ? HubTokens.control-md : HubTokens.list-row-sm;",
        "detail-info-section-height: root.detail-info-header-height + root.info-row-height * 5 + root.detail-info-row-gap * 5;",
        "component ProjectDetailStatusStrip inherits Rectangle",
        "in property <ProjectDetailData> detail;",
        "in property <UiTextData> copy;",
        "text: root.detail.version;",
        "text: root.detail.pinned ? root.copy.pinned-label : root.copy.not-pinned-label;",
        "text: root.copy.modified-prefix + root.detail.modified;",
        "ProjectDetailStatusStrip {",
        "row-height: root.status-row-height;",
        "row-spacing: root.status-gap;",
        "version-badge-width: root.version-badge-width;",
        "pin-badge-width: root.pin-badge-width;",
        "detail: root.project;",
        "copy: root.ui-text;",
        "component ProjectDetailInfoSection inherits Rectangle",
        "in property <ProjectDetailData> project;",
        "in property <UiTextData> ui-text;",
        "ProjectDetailInfoSection {",
        "section-height: root.detail-info-section-height;",
        "row-height: root.info-row-height;",
        "row-spacing: root.detail-info-row-gap;",
        "header-subtitle: root.narrow-flow ? \"\" : root.ui-text.project-info-subtitle;",
        "project: root.project;",
        "ui-text: root.ui-text;",
        "detail-engine-list-height: root.engine-count == 0 ? HubTokens.list-row-lg : root.detail-engine-panel-rows * root.detail-choice-row-height + (root.detail-engine-panel-rows - 1) * root.detail-engine-row-gap;",
        "component ProjectDetailActionButton inherits PillButton",
        "height: root.action-height;",
        "ProjectDetailActionButton {",
        "action-height: root.action-row-height;",
        "text: root.ui-text.confirm-delete;",
        "text: root.ui-text.cancel-delete;",
        "text: root.ui-text.open;",
        "text: root.project.pinned ? root.ui-text.unpin-project : root.ui-text.pin-project;",
        "text: root.ui-text.remove-from-hub;",
        "text: root.ui-text.delete-project;",
        "list-height: root.detail-engine-list-height;",
        "list-scroll-y <=> root.detail-engine-scroll-y;",
        "row-height: root.detail-choice-row-height;",
        "empty-height: HubTokens.list-row-lg;",
        "component ProjectDetailEngineSection inherits Rectangle",
        "in property <ProjectDetailData> detail;",
        "in property <UiTextData> copy;",
        "title: root.copy.change-source-engine;",
        "subtitle: root.copy.bound-source-engine + \": \" + root.detail.engine-label;",
        "selected-label: root.copy.selected-label;",
        "registered-label: root.copy.registered;",
        "empty-title: root.copy.no-source-engine-available;",
        "ProjectDetailEngineSection {",
        "section-height: root.detail-engine-section-height;",
        "section-spacing: root.page-gap;",
        "detail: root.project;",
        "copy: root.ui-text;",
        "list-scroll-y <=> root.detail-engine-scroll-y;",
        "selected(id) => { root.select-engine(id); }",
        "detail-main-panel-height: root.detail-panel-padding * 2 + root.cover-height + root.detail-title-header-height + root.status-row-height + root.detail-info-section-height + root.page-gap * 3;",
        "detail-action-panel-height: root.detail-panel-padding * 2 + (root.project.pending-delete ? root.detail-action-delete-height : root.detail-action-standard-height);",
        "detail-visible-height: max(root.content-height - root.detail-header-height - root.page-gap, root.detail-main-panel-height);",
        "detail-flow-height: root.narrow-flow ? root.detail-main-height + root.page-gap + root.detail-action-height : max(root.detail-main-height, root.detail-action-height);",
        "scroll-y <=> root.browser-scroll-y;",
    ] {
        assert!(
            project_surface.contains(snippet),
            "Project secondary pages are missing ResponsiveSlot/scroll sizing snippet: {snippet}"
        );
    }
    for component_name in [
        "ProjectCreateField",
        "ProjectCreateActionRow",
        "ProjectEngineChoiceList",
        "ProjectDetailActionButton",
        "ProjectDetailStatusStrip",
        "ProjectDetailInfoSection",
        "ProjectDetailEngineSection",
    ] {
        assert!(
            project_components.contains(&format!("export component {component_name}")),
            "project_page_components.slint should own the exported Projects workflow component {component_name}"
        );
        assert!(
            !project_pages.contains(&format!("component {component_name} inherits")),
            "project_pages.slint should import {component_name} instead of declaring it locally"
        );
    }
    for forbidden in [
        "column-width",
        "toolbar-select-width",
        "root.content-width - root.page-gap",
        "page-surface.viewport-height - root.page-pad - root.page-pad",
        "page-surface.viewport-height - root.page-pad * 2",
        "page-surface.viewport-height - root.page-pad - root.page-pad - root.action-row-height",
        "page-surface.content-height",
        "page-surface.viewport-height",
        "viewport_y <=> root.scroll-y;",
        "scroll-y <=> root.page-scroll-y;",
        "root.content-width / 8",
        "root.content-width * 2 / 5",
        "root.flow-height * 3 / 5",
        "root.flow-height * 2 / 5",
        "root.content-height / 4",
        "root.content-width / 10",
        "root.content-width / 14",
        "root.content-width / 2",
        "root.content-width / 4",
        "root.content-height * 3 / 5",
        "root.content-height * 2 / 5",
        "if root.engine-count > 0: PanelListViewport",
        "detail-engine-list := PanelListViewport",
        "HubTextField {\n                                height: root.field-height;\n                                label: root.ui-text.project-name;",
        "if root.project.pending-delete: PillButton",
        "if !root.project.pending-delete: PillButton",
        "Badge { text: root.project.version; tone: \"accent\"; badge-width: root.version-badge-width; }",
        "root.project.pinned ? root.ui-text.pinned-label : root.ui-text.not-pinned-label",
        "text: root.ui-text.modified-prefix + root.project.modified;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.project-status;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.project-root-path;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.source-engine;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.engine-version-column;",
        "row-height: root.info-row-height;\n                            label: root.ui-text.last-modified-column;",
        "height: root.create-action-row-height;\n                                vertical-stretch: 0;\n                                alignment: center;",
        "PillButton {\n                                    text: root.ui-text.create;",
        "if !root.project.pending-delete: VerticalLayout {\n                        horizontal-stretch: 1;\n                        height: root.detail-engine-section-height;",
        "title: root.ui-text.change-source-engine;\n                            subtitle: root.ui-text.bound-source-engine + \": \" + root.project.engine-label;",
    ] {
        assert!(
            !project_pages.contains(forbidden),
            "Project secondary pages should not return to page-local remaining width/height formulas: {forbidden}"
        );
    }
    assert_eq!(
        project_pages.matches("ProjectEngineChoiceList {").count(),
        1,
        "ProjectNewPage should route Source Engine choices through ProjectEngineChoiceList directly"
    );
    assert_eq!(
        project_components
            .matches("ProjectEngineChoiceList {")
            .count(),
        1,
        "ProjectDetailEngineSection should reuse ProjectEngineChoiceList for detail engine choices"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailActionButton {").count(),
        6,
        "ProjectDetailPage should route all six visible action buttons through ProjectDetailActionButton"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailInfoSection {").count(),
        1,
        "ProjectDetailPage should route the five project-info rows through one ProjectDetailInfoSection"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailStatusStrip {").count(),
        1,
        "ProjectDetailPage should route version, pinned state, and modified time through one ProjectDetailStatusStrip"
    );
    assert_eq!(
        project_pages.matches("ProjectCreateActionRow {").count(),
        1,
        "ProjectNewPage should route its create button row through one ProjectCreateActionRow"
    );
    assert_eq!(
        project_pages.matches("ProjectDetailEngineSection {").count(),
        1,
        "ProjectDetailPage should route Change Source Engine controls through one ProjectDetailEngineSection"
    );
}
