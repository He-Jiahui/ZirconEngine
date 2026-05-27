//! Static contracts for Hub workspace-page Taffy layout usage.

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
fn workspace_pages_use_workspace_panel_section() {
    for page in [
        "editor.slint",
        "builds.slint",
        "settings.slint",
        "cloud.slint",
        "team.slint",
    ] {
        let source = read_ui_file(page);
        assert!(
            source.contains("WorkspacePanelSection"),
            "{page} must compose workspace panels through WorkspacePanelSection"
        );
        assert!(
            !source.contains("ResponsivePanelFlow"),
            "{page} should not directly use low-level ResponsivePanelFlow"
        );
        assert!(
            !source.contains("flow-height: root.compact"),
            "{page} should not hand-write compact flow-height formulas"
        );
        let direct_page_root = match page {
            "builds.slint" => Some("BuildsPage"),
            "cloud.slint" => Some("CloudPage"),
            "editor.slint" => Some("EditorPage"),
            "settings.slint" => Some("SettingsPage"),
            "team.slint" => Some("TeamPage"),
            _ => None,
        };
        if let Some(component) = direct_page_root {
            let direct_inherits =
                format!("export component {component} inherits PageScrollSurface");
            assert!(
                source.contains(&direct_inherits)
                    && !source.contains("page-surface := PageScrollSurface")
                    && !source.contains("content-width: page-surface.content-width;")
                    && !source.contains("content-height: page-surface.content-height;")
                    && !source.contains("page-surface.content-height")
                    && !source.contains("page-surface.viewport-height"),
                "{page} should inherit PageScrollSurface directly so the page root owns content sizing"
            );
        } else {
            assert!(
                source.contains("page-surface := PageScrollSurface")
                    && source.contains("content-width: page-surface.content-width;"),
                "{page} should derive content width from PageScrollSurface instead of recomputing page padding"
            );
        }
        assert!(
            !source.contains("root.width - HubTokens.page-padding * 2"),
            "{page} should not return to page-local content-width subtraction"
        );
        assert!(
            !source.contains("width: root.content-width;"),
            "{page} should let PageScrollSurface and stretch layout size workspace rows instead of assigning width directly"
        );
        for forbidden in ["width: root.width;", "height: root.height;"] {
            assert!(
                !source.contains(forbidden),
                "{page} should let PageScrollSurface inherit page geometry instead of binding {forbidden}"
            );
        }
    }

    let layout = read_ui_file("layout.slint");
    for snippet in [
        "out property <length> content-width: max(1px, root.width - root.page-padding * 2);",
        "out property <length> content-height: max(HubTokens.control-md, root.viewport-height - root.page-padding * 2);",
        "in property <bool> compact-stack: true;",
        "in property <int> compact-rows: 2;",
        "in property <length> compact-height-override: 0px;",
        "preferred-width: 0px;",
        "section-height: root.compact ? (root.compact-height-override > 0px ? root.compact-height-override : root.compact-height) : root.row-height;",
        "flex-direction: root.compact && root.compact-stack ? FlexboxLayoutDirection.column : FlexboxLayoutDirection.row;",
        "flex-wrap: root.compact && root.compact-stack ? FlexboxLayoutWrap.no-wrap : FlexboxLayoutWrap.wrap;",
    ] {
        assert!(
            layout.contains(snippet),
            "WorkspacePanelSection must support compact stacking and compact wrapped metric rows; missing {snippet}"
        );
    }

    let cloud = read_ui_file("cloud.slint");
    assert!(
        cloud.contains("compact-stack: false;")
            && cloud.contains("compact-rows: root.metric-row-count;")
            && cloud.contains("metric-row-height: root.metrics-compact ? HubTokens.list-row-md + HubTokens.space-2 : HubTokens.workspace-row-cloud-metrics;")
            && cloud.contains("row-height: root.metric-row-height;")
            && cloud.contains("metrics-four-columns:")
            && cloud.contains("metrics-two-columns:")
            && cloud.contains("metric-row-count:")
            && (cloud.contains("metric-slot-basis: root.metrics-four-columns ? root.metric-min-width : (root.metrics-two-columns ? HubTokens.panel-min-md : root.content-width);")
                || cloud.contains("metric-slot-basis: root.metrics-four-columns || root.metrics-two-columns ? root.metric-min-width : root.content-width;"))
            && cloud.contains("metric-slot-min-width: root.metrics-four-columns || root.metrics-two-columns ? root.metric-min-width : root.content-width;")
            && cloud.contains("metric-slot-grow: 1;")
            && cloud.contains("component CloudMetricSlot inherits ResponsiveSlot")
            && cloud.contains("basis: root.metric-slot-basis;")
            && cloud.contains("grow: root.metric-slot-grow;")
            && cloud.contains("min-width: root.metric-slot-min-width;")
            && cloud.contains("height: root.metric-row-height;")
            && cloud.contains("export component CloudPage inherits PageScrollSurface")
            && cloud.contains("in property <bool> collapse-label: false;")
            && cloud.contains("collapse-trailing-label: root.collapse-label;")
            && cloud.contains("collapse-label: root.content-width < HubTokens.breakpoint-medium;")
            && cloud.contains("service-row-slot-height: HubTokens.list-row-lg + HubTokens.space-2;")
            && cloud.contains("service-panel-chrome-height: HubTokens.control-md + HubTokens.toolbar-gap + HubTokens.space-4 * 2;")
            && cloud.contains("service-visible-rows: root.service-count < 4 ? root.service-count : 4;")
            && cloud.contains("service-available-height: max(root.service-panel-chrome-height + root.service-row-slot-height, root.content-height - root.header-height - root.metric-section-height - HubTokens.panel-gap * 2);")
            && cloud.contains("service-available-list-height: max(root.service-row-slot-height, root.service-available-height - root.service-panel-chrome-height);")
            && cloud.contains("service-fit-row-count: Math.floor(root.service-available-list-height / root.service-row-slot-height);")
            && cloud.contains("service-panel-rows: root.service-visible-rows < 1 ? 1 : (root.service-visible-rows < root.service-fit-rows ? root.service-visible-rows : root.service-fit-rows);")
            && cloud.contains("service-list-height: root.service-count == 0 ? HubTokens.list-row-lg + HubTokens.space-4 : root.service-panel-rows * HubTokens.list-row-lg + (root.service-panel-rows - 1) * HubTokens.space-2 + HubTokens.space-1 * 2;")
            && cloud.contains("services-panel-height: root.service-panel-chrome-height + root.service-list-height;")
            && cloud.contains("if root.service-count == 0: EmptyStateBlock")
            && cloud.contains("title: root.ui-text.cloud-local-only;")
            && cloud.contains("center-content: true;"),
        "CloudPage should use wrapped WorkspacePanelSection metrics, compact row labels, PageScrollSurface content height for complete-row list sizing, and an in-panel service empty state"
    );
    assert_eq!(
        cloud.matches("CloudMetricSlot {").count(),
        4,
        "CloudPage should render its four summary metrics through CloudMetricSlot"
    );
    assert_eq!(
        cloud.matches("MetricCard {").count(),
        1,
        "CloudPage should keep metric card content in CloudMetricSlot instead of repeating it at every metric call site"
    );
    for forbidden in [
        "ResponsiveSlot {\n                basis: root.metric-slot-basis;",
        "metric-card-width",
        "root.content-width - HubTokens.panel-gap",
        "root.content-width - root.metric-gap",
        "root.content-width - root.metric-gap *",
        "service-content-height:",
        "services-panel-height: min(root.service-available-height",
        "root.height - HubTokens.page-padding",
        "root.height - HubTokens.page-padding * 2",
        "root.height - HubTokens.page-padding * 2 - HubTokens.bottom-status-height",
    ] {
        assert!(
            !cloud.contains(forbidden),
            "CloudPage should not return to page-local metric/list sizing formulas: {forbidden}"
        );
    }

    let team = read_ui_file("team.slint");
    assert!(
        team.contains("row-height: HubTokens.workspace-row-team-summary;")
            && team.contains("component TeamSummarySlot inherits ResponsiveSlot")
            && team.contains("export component TeamPage inherits PageScrollSurface")
            && team.contains("in property <bool> collapse-label: false;")
            && team.contains("collapse-trailing-label: root.collapse-label;")
            && team.contains("collapse-label: root.content-width < HubTokens.breakpoint-medium;")
            && team.contains("label: root.ui-text.team-git-identity;")
            && team.contains("primary: root.summary.identity-name;")
            && team.contains("secondary: root.summary.identity-email;")
            && team.contains("label: root.ui-text.team-repository;")
            && team.contains("primary: root.summary.repository-path;")
            && team.contains("secondary: root.ui-text.team-local-only;")
            && team.contains("member-row-slot-height: root.member-row-height + HubTokens.space-2;")
            && team.contains("member-panel-chrome-height: HubTokens.control-md + HubTokens.toolbar-gap + HubTokens.space-4 * 2;")
            && team.contains("member-visible-rows: root.member-count < 6 ? root.member-count : 6;")
            && team.contains("member-available-height: max(root.member-panel-chrome-height + root.member-row-slot-height, root.content-height - root.header-height - root.summary-section-height - HubTokens.panel-gap * 2);")
            && team.contains("member-available-list-height: max(root.member-row-slot-height, root.member-available-height - root.member-panel-chrome-height);")
            && team.contains("member-fit-row-count: Math.floor(root.member-available-list-height / root.member-row-slot-height);")
            && team.contains("member-panel-rows: root.member-visible-rows < 1 ? 1 : (root.member-visible-rows < root.member-fit-rows ? root.member-visible-rows : root.member-fit-rows);")
            && team.contains("member-list-height: root.member-count == 0 ? HubTokens.list-row-lg + HubTokens.space-4 : root.member-panel-rows * root.member-row-height + (root.member-panel-rows - 1) * HubTokens.space-2 + HubTokens.space-1 * 2;")
            && team.contains("members-panel-height: root.member-panel-chrome-height + root.member-list-height;")
            && team.contains("if root.member-count == 0: EmptyStateBlock")
            && team.contains("for member in root.members: TeamMemberRow {")
            && team.contains("row-height: root.member-row-height;")
            && team.contains("member: member;")
            && team.contains("ui-text: root.ui-text;")
            && team.contains("center-content: true;"),
        "TeamPage summary cards and member list should use compact row labels, tokenized row height, PageScrollSurface content height, complete-row visible budgeting, and an in-panel empty state block"
    );
    assert_eq!(
        team.matches("TeamSummarySlot {").count(),
        2,
        "TeamPage should render identity and repository summaries through TeamSummarySlot"
    );
    assert_eq!(
        team.matches("MetricCard {").count(),
        1,
        "TeamPage should keep summary MetricCard content in TeamSummarySlot instead of repeating it at every summary call site"
    );
    for forbidden in [
        "ResponsiveSlot {\n                basis: HubTokens.panel-min-sm * 4 / 5;",
        "ResponsiveSlot {\n                basis: HubTokens.panel-min-md;",
        "member-content-height:",
        "members-panel-height: min(root.member-available-height",
        "root.height - HubTokens.page-padding",
        "root.height - HubTokens.page-padding * 2",
        "root.height - HubTokens.page-padding * 2 - HubTokens.bottom-status-height",
        "for member in root.members: Rectangle {",
    ] {
        assert!(
            !team.contains(forbidden),
            "TeamPage should not return to page-local member-list height formulas: {forbidden}"
        );
    }

    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let editor_surface = format!("{editor}\n{editor_components}");
    let settings = read_ui_file("settings.slint");
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds_surface = format!("{builds}\n{builds_components}");
    for (page, source) in [
        ("CloudPage", &cloud),
        ("TeamPage", &team),
        ("EditorPage", &editor),
        ("SettingsPage", &settings),
        ("BuildsPage", &builds),
    ] {
        assert_semantic_taffy_properties_have_slint_flex_pairs(page, source);
    }
    for (page, source, snippets) in [
        (
            "EditorPage",
            &editor,
            &[
                "basis: root.overview-min-width;",
                "flex-basis: root.overview-min-width;",
                "grow: 2;",
                "flex-grow: 2;",
                "shrink: 1;",
                "flex-shrink: 1;",
            ][..],
        ),
        (
            "SettingsPage",
            &settings,
            &[
                "basis: HubTokens.panel-min-md + HubTokens.control-sm;",
                "flex-basis: HubTokens.panel-min-md + HubTokens.control-sm;",
                "grow: 1;",
                "flex-grow: 1;",
                "shrink: 1;",
                "flex-shrink: 1;",
            ][..],
        ),
        (
            "BuildsPage",
            &builds,
            &[
                "basis: root.overview-min-width;",
                "flex-basis: root.overview-min-width;",
                "grow: 2;",
                "flex-grow: 2;",
                "order: root.controls-first ? 1 : 0;",
                "flex-order: root.controls-first ? 1 : 0;",
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
        "side-panel-min-width: HubTokens.panel-min-md + HubTokens.control-lg * 2;",
        "overview-min-width: HubTokens.panel-min-lg + HubTokens.control-lg * 2;",
        "basis: root.overview-min-width;",
        "basis: root.side-panel-min-width;",
        "grow: 2;",
        "grow: 1;",
        "build-source-summary-height: HubTokens.list-row-sm + HubTokens.border-width + root.build-row-height * 2 + HubTokens.toolbar-gap * 3 + HubTokens.space-4 * 2;",
        "build-summary-section-height: root.build-source-summary-height + root.build-summary-height + HubTokens.panel-gap;",
        "compact-labels: root.content-width < HubTokens.breakpoint-medium;",
        "side-list-empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
        "compact-height-override: root.build-summary-section-height;",
        "height: root.compact ? root.build-source-summary-height : root.build-summary-height;",
        "height: root.build-summary-height;",
        "min-width: root.compact ? root.content-width : root.overview-min-width;",
        "min-width: root.compact ? root.content-width : root.side-panel-min-width;",
        "collapse-label: root.compact-labels;",
        "export component EditorSideListPanel inherits HubPanel",
        "in-out property <length> list-scroll-y: 0px;",
        "in property <string> panel-title;",
        "in property <string> badge-text;",
        "export component EditorActionRow inherits ActionRow",
        "in property <string> action-id;",
        "in property <image> action-icon;",
        "in property <string> action-title;",
        "in property <string> action-detail;",
        "id: root.action-id,",
        "icon-image: root.action-icon,",
        "has-icon-image: true,",
        "title: root.action-title,",
        "detail: root.action-detail,",
        "enabled: root.action-enabled,",
        "root.triggered(id);",
        "export component EditorSourceSummaryRow inherits InfoRow",
        "in property <image> summary-icon;",
        "in property <string> summary-title;",
        "in property <string> summary-detail;",
        "in property <string> summary-meta;",
        "in property <string> summary-status;",
        "in property <string> summary-tone: \"neutral\";",
        "title: root.summary-title;",
        "detail: root.summary-detail;",
        "meta: root.summary-meta;",
        "leading-image: root.summary-icon;",
        "trailing-text: root.summary-status;",
        "trailing-tone: root.summary-tone;",
        "collapse-trailing-label: root.collapse-label;",
        "summary-title: root.ui-text.source-prefix;",
        "summary-detail: root.source-engine.source-path;",
        "summary-meta: root.ui-text.last-build-prefix + root.source-engine.last-build;",
        "summary-icon: @image-url(\"../assets/brand/zircon-mark.svg\");",
        "summary-status: root.source-engine.status;",
        "summary-title: root.ui-text.output-prefix;",
        "summary-detail: root.source-engine.output-path;",
        "summary-meta: root.ui-text.profile-prefix + root.source-engine.build-profile;",
        "summary-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "summary-status: root.source-engine.jobs + root.ui-text.jobs-suffix;",
        "list-scroll-y <=> root.engine-list-scroll-y;",
        "list-scroll-y <=> root.build-history-scroll-y;",
        "row-count: root.row-count;",
        "empty-height: root.empty-height;",
        "empty-height: root.side-list-empty-height;",
        "if root.source-engine-count == 0: EmptyStateBlock",
        "height: root.side-list-empty-height;",
        "title: root.ui-text.no-source-engines;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-build-history;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            editor_surface.contains(snippet),
            "EditorPage is missing tokenized ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for component in [
        "SourceEngineRow",
        "EditorSideListPanel",
        "EditorPathFieldRow",
        "EditorActionRow",
        "EditorSourceSummaryRow",
    ] {
        assert!(
            editor_components.contains(&format!("export component {component}")),
            "editor_page_components.slint should export {component}"
        );
        assert!(
            !editor.contains(&format!("component {component}")),
            "editor.slint should import {component} instead of defining it inline"
        );
    }
    assert_eq!(
        editor.matches("EditorSourceSummaryRow {").count(),
        2,
        "EditorPage should render source and output summary rows through EditorSourceSummaryRow"
    );
    for forbidden in [
        "summary-main-width",
        "summary-side-width",
        "root.content-width - root.summary-side-width",
        "split-content-width",
        "overview-width",
        "actions-width",
        "config-width",
        "root.split-content-width",
        "root.content-width - root.side-panel-min-width",
    ] {
        assert!(
            !editor.contains(forbidden),
            "EditorPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }
    for snippet in [
        "basis: HubTokens.panel-min-md + HubTokens.control-sm;",
        "row-preferred-width: HubTokens.input-width + HubTokens.control-md * 3 + HubTokens.toolbar-gap;",
        "basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "basis: HubTokens.panel-min-md;",
        "grow: 2;",
        "grow: 1;",
        "component SettingsSegmentChoice inherits Rectangle",
        "in property <string> label;",
        "in property <string> first-label;",
        "in property <string> first-value;",
        "in property <string> second-label;",
        "in property <string> second-value;",
        "in-out property <string> selected-value;",
        "text: root.first-label;",
        "active: root.selected-value == root.first-value;",
        "root.selected-value = root.first-value;",
        "text: root.second-label;",
        "active: root.selected-value == root.second-value;",
        "root.selected-value = root.second-value;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-lg;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
        "compact-labels: root.content-width < HubTokens.breakpoint-medium;",
        "health-empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
        "empty-height: root.health-empty-height;",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "center-content: true;",
        "collapse-trailing-label: root.collapse-label;",
        "collapse-label: root.compact-labels;",
    ] {
        assert!(
            settings.contains(snippet),
            "SettingsPage is missing PanelSlot semantic sizing snippet: {snippet}"
        );
    }
    assert_eq!(
        settings.matches("SettingsSegmentChoice {").count(),
        2,
        "SettingsPage should render build profile and language choices through SettingsSegmentChoice"
    );
    for forbidden in [
        "detail-side-width",
        "detail-primary-width",
        "detail-path-list-width",
        "root.content-width - root.detail-side-width",
        "path-list-width: root.detail-path-list-width",
        "path-list-width:",
        "row-preferred-width: root.path-list-width",
        "root.width - MaterialStyleMetrics.padding_16",
    ] {
        assert!(
            !settings.contains(forbidden),
            "SettingsPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }
    for snippet in [
        "side-panel-min-width: HubTokens.panel-min-md + HubTokens.control-lg * 2;",
        "overview-min-width: HubTokens.panel-min-lg + HubTokens.control-lg * 2;",
        "controls-first: root.compact && root.content-height < root.build-summary-section-height + HubTokens.control-lg;",
        "compact-labels: root.content-width < HubTokens.breakpoint-medium;",
        "export component BuildControlAction inherits ActionRow",
        "in property <string> action-id;",
        "in property <image> action-icon;",
        "in property <string> action-title;",
        "in property <string> action-detail;",
        "in property <bool> action-enabled: true;",
        "callback triggered(string);",
        "id: root.action-id,",
        "icon-image: root.action-icon,",
        "has-icon-image: true,",
        "title: root.action-title,",
        "detail: root.action-detail,",
        "enabled: root.action-enabled,",
        "activate(id) => {",
        "root.triggered(id);",
        "export component BuildPipelineStep inherits InfoRow",
        "in property <image> step-icon;",
        "in property <string> step-title;",
        "in property <string> step-detail;",
        "in property <string> step-status;",
        "in property <string> step-tone: \"neutral\";",
        "title: root.step-title;",
        "detail: root.step-detail;",
        "leading-image: root.step-icon;",
        "trailing-text: root.step-status;",
        "trailing-tone: root.step-tone;",
        "export component BuildSourceSummaryRow inherits InfoRow",
        "in property <image> summary-icon;",
        "in property <string> summary-title;",
        "in property <string> summary-detail;",
        "in property <string> summary-meta;",
        "in property <string> summary-status;",
        "in property <string> summary-tone: \"neutral\";",
        "title: root.summary-title;",
        "detail: root.summary-detail;",
        "meta: root.summary-meta;",
        "leading-image: root.summary-icon;",
        "trailing-text: root.summary-status;",
        "trailing-tone: root.summary-tone;",
        "collapse-trailing-label: root.collapse-label;",
        "summary-title: root.ui-text.source-prefix;",
        "summary-detail: root.source-engine.source-path;",
        "summary-meta: root.ui-text.last-build-prefix + root.source-engine.last-build;",
        "summary-icon: @image-url(\"../assets/icons/nav/builds.svg\");",
        "summary-status: root.source-engine.status;",
        "summary-title: root.ui-text.output-prefix;",
        "summary-detail: root.source-engine.output-path;",
        "summary-meta: root.ui-text.profile-prefix + root.source-engine.build-profile;",
        "summary-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "summary-status: root.source-engine.jobs + root.ui-text.jobs-suffix;",
        "compact-height-override: root.build-summary-section-height;",
        "basis: root.overview-min-width;",
        "basis: root.side-panel-min-width;",
        "grow: 2;",
        "grow: 1;",
        "order: root.controls-first ? 1 : 0;",
        "order: root.controls-first ? 0 : 1;",
        "height: root.compact ? root.build-source-summary-height : root.build-summary-height;",
        "height: root.build-summary-height;",
        "min-width: root.compact ? root.content-width : root.overview-min-width;",
        "min-width: root.compact ? root.content-width : root.side-panel-min-width;",
        "collapse-label: root.compact-labels;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "height: HubTokens.list-row-lg + HubTokens.space-4;",
        "title: root.ui-text.no-build-history;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            builds_surface.contains(snippet),
            "BuildsPage is missing tokenized ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for component in [
        "BuildControlAction",
        "BuildPipelineStep",
        "BuildSourceSummaryRow",
    ] {
        assert!(
            builds_components.contains(&format!("export component {component}")),
            "builds_page_components.slint should export {component}"
        );
        assert!(
            !builds.contains(&format!("component {component}")),
            "builds.slint should import {component} instead of defining it inline"
        );
    }
    assert_eq!(
        builds.matches("BuildControlAction {").count(),
        5,
        "BuildsPage should render all build controls through BuildControlAction"
    );
    assert_eq!(
        builds.matches("\n            ActionRow {").count(),
        0,
        "BuildsPage should keep ActionRow construction inside BuildControlAction instead of repeating it at each build-control call site"
    );
    assert_eq!(
        builds.matches("BuildPipelineStep {").count(),
        4,
        "BuildsPage should render build pipeline rows through BuildPipelineStep"
    );
    assert_eq!(
        builds.matches("BuildSourceSummaryRow {").count(),
        2,
        "BuildsPage should render source and output summary rows through BuildSourceSummaryRow"
    );
    for forbidden in [
        "detail-main-width",
        "detail-side-width",
        "root.content-width - root.detail-side-width",
        "split-content-width",
        "overview-width",
        "side-panel-width",
        "root.split-content-width",
    ] {
        assert!(
            !builds.contains(forbidden),
            "BuildsPage should not return to page-local remaining width formulas: {forbidden}"
        );
    }
}
