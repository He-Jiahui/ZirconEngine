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
            !source
                .lines()
                .any(|line| line.trim() == "width: root.content-width;"),
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
        "in property <length> page-padding-x: root.page-padding;",
        "in property <length> page-padding-y: root.page-padding;",
        "out property <length> content-width: max(1px, root.width - root.page-padding-x * 2);",
        "out property <length> content-height: max(HubTokens.control-md, root.viewport-height - root.page-padding-y - root.bottom-padding);",
        "in property <bool> compact-stack: true;",
        "in property <int> compact-rows: 2;",
        "in property <length> compact-height-override: 0px;",
        "in property <length> section-padding: HubTokens.space-0;",
        "in property <length> section-padding-left: root.section-padding;",
        "in property <length> section-padding-right: root.section-padding;",
        "in property <length> section-padding-top: root.section-padding;",
        "in property <length> section-padding-bottom: root.section-padding;",
        "preferred-width: 0px;",
        "compact-height: root.row-height * root.compact-row-count + root.spacing-vertical * (root.compact-row-count - 1) + root.section-padding-top + root.section-padding-bottom;",
        "regular-height: root.row-height + root.section-padding-top + root.section-padding-bottom;",
        "section-height: root.compact ? (root.compact-height-override > 0px ? root.compact-height-override : root.compact-height) : root.regular-height;",
        "flex-direction: root.compact && root.compact-stack ? FlexboxLayoutDirection.column : FlexboxLayoutDirection.row;",
        "flex-wrap: root.compact && root.compact-stack ? FlexboxLayoutWrap.no-wrap : FlexboxLayoutWrap.wrap;",
        "padding-left: root.section-padding-left;",
        "padding-right: root.section-padding-right;",
        "padding-top: root.section-padding-top;",
        "padding-bottom: root.section-padding-bottom;",
    ] {
        assert!(
            layout.contains(snippet),
            "WorkspacePanelSection must support compact stacking and compact wrapped metric rows; missing {snippet}"
        );
    }

    let cloud = read_ui_file("cloud.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");
    let cloud_surface = format!("{cloud}\n{cloud_components}");
    assert!(
        cloud_surface.contains("compact-stack: false;")
            && cloud_surface.contains("compact-rows: root.metric-row-count;")
            && cloud_surface.contains("metric-row-height: root.metrics-compact ? HubTokens.list-row-md + HubTokens.space-2 : HubTokens.workspace-row-cloud-metrics;")
            && cloud_surface.contains("row-height: root.metric-row-height;")
            && cloud_surface.contains("metrics-four-columns:")
            && cloud_surface.contains("metrics-two-columns:")
            && cloud_surface.contains("metric-row-count:")
            && (cloud_surface.contains("metric-slot-basis: root.metrics-four-columns ? root.metric-min-width : (root.metrics-two-columns ? HubTokens.panel-min-md : root.content-width);")
                || cloud_surface.contains("metric-slot-basis: root.metrics-four-columns || root.metrics-two-columns ? root.metric-min-width : root.content-width;"))
            && cloud_surface.contains("metric-slot-min-width: root.metrics-four-columns || root.metrics-two-columns ? root.metric-min-width : root.content-width;")
            && cloud_surface.contains("metric-slot-grow: 1;")
            && cloud_surface.contains("export component CloudMetricSlot inherits ResponsiveSlot")
            && cloud_surface.contains("basis: root.metric-slot-basis;")
            && cloud_surface.contains("grow: root.metric-slot-grow;")
            && cloud_surface.contains("min-width: root.metric-slot-min-width;")
            && cloud_surface.contains("height: root.metric-row-height;")
            && cloud_surface.contains("export component CloudPage inherits PageScrollSurface")
            && cloud_surface.contains("in property <bool> collapse-label: false;")
            && cloud_surface.contains("collapse-trailing-label: root.collapse-label;")
            && cloud_surface.contains("label-collapse := ResponsiveCollapse {")
            && cloud_surface.contains("collapse-label: label-collapse.collapsed;")
            && cloud_surface.contains("service-row-slot-height: HubTokens.list-row-lg + HubTokens.space-2;")
            && cloud_surface.contains("service-panel-chrome-height: HubTokens.control-md + HubTokens.toolbar-gap + HubTokens.space-4 * 2;")
            && cloud_surface.contains("service-visible-rows: root.service-count < 4 ? root.service-count : 4;")
            && cloud_surface.contains("service-available-height: max(root.service-panel-chrome-height + root.service-row-slot-height, root.content-height - root.header-height - root.metric-section-height - HubTokens.panel-gap * 2);")
            && cloud_surface.contains("service-available-list-height: max(root.service-row-slot-height, root.service-available-height - root.service-panel-chrome-height);")
            && cloud_surface.contains("service-fit-row-count: Math.floor(root.service-available-list-height / root.service-row-slot-height);")
            && cloud_surface.contains("service-panel-rows: root.service-visible-rows < 1 ? 1 : (root.service-visible-rows < root.service-fit-rows ? root.service-visible-rows : root.service-fit-rows);")
            && cloud_surface.contains("service-list-height: root.service-count == 0 ? HubTokens.list-row-lg + HubTokens.space-4 : root.service-panel-rows * HubTokens.list-row-lg + (root.service-panel-rows - 1) * HubTokens.space-2 + HubTokens.space-1 * 2;")
            && cloud_surface.contains("services-panel-height: root.service-panel-chrome-height + root.service-list-height;")
            && cloud_surface.contains("export component CloudPackageActionRow inherits ActionRow")
            && cloud_surface.contains("export component CloudPackageActionsPanel inherits HubListPanelSlot")
            && cloud_surface.contains("row-count: 2;")
            && cloud_surface.contains("row-height: HubTokens.list-row-sm;")
            && cloud_surface.contains("row-spacing: HubTokens.toolbar-gap;")
            && cloud_surface.contains("CloudPackageActionsPanel {")
            && cloud_surface.contains("height: root.actions-panel-height;")
            && cloud_surface.contains("summary: root.summary;")
            && cloud_surface.contains("package-project => {")
            && cloud_surface.contains("install-device => {")
            && cloud_surface.contains("export component CloudServicesPanel inherits HubListPanelSlot")
            && cloud_surface.contains("CloudServicesPanel {")
            && cloud_surface.contains("height: root.services-panel-height;")
            && cloud_surface.contains("service-scroll-y <=> root.service-scroll-y;")
            && cloud_surface.contains("if root.service-count == 0: EmptyStateBlock")
            && cloud_surface.contains("title: root.ui-text.cloud-local-only;")
            && cloud_surface.contains("center-content: true;"),
        "CloudPage should use wrapped WorkspacePanelSection metrics, compact row labels, PageScrollSurface content height for complete-row list sizing, and an in-panel service empty state"
    );
    assert!(
        !cloud.contains("component CloudMetricSlot")
            && !cloud.contains("component CloudServiceRow")
            && !cloud.contains("component CloudServicesPanel")
            && !cloud.contains("component CloudPackageActionRow")
            && !cloud.contains("component CloudPackageActionsPanel"),
        "cloud.slint should import page-specific Cloud panel/row/metric wrappers instead of defining them inline"
    );
    assert!(
        !cloud.contains("from \"builds_page_components.slint\"")
            && !cloud.contains("BuildControlAction {")
            && cloud_components.matches("CloudPackageActionRow {").count() == 2,
        "CloudPage should not depend on Builds action components for package/install actions"
    );
    assert_eq!(
        cloud.matches("CloudMetricSlot {").count(),
        4,
        "CloudPage should render its four summary metrics through CloudMetricSlot"
    );
    assert_eq!(
        cloud_components.matches("MetricCard {").count(),
        1,
        "cloud_page_components.slint should keep metric card content in CloudMetricSlot instead of repeating it at every metric call site"
    );
    assert_eq!(
        cloud.matches("CloudServicesPanel {").count(),
        1,
        "CloudPage should render its service list through CloudServicesPanel"
    );
    for forbidden in [
        "ResponsiveSlot {\n                basis: root.metric-slot-basis;",
        "metric-card-width",
        "root.content-width - HubTokens.panel-gap",
        "root.content-width - root.metric-gap",
        "root.content-width - root.metric-gap *",
        "service-content-height:",
        "services-panel-height: min(root.service-available-height",
        "PanelSlot {\n        horizontal-stretch: 1;\n        height: root.actions-panel-height;",
        "BuildControlAction {",
        "PanelListViewport {\n            scroll-y <=> root.service-scroll-y;",
        "if root.service-count == 0: EmptyStateBlock",
        "for service in root.services: CloudServiceRow {",
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
    let team_components = read_ui_file("team_page_components.slint");
    let team_surface = format!("{team}\n{team_components}");
    assert!(
        team_surface.contains("row-height: HubTokens.workspace-row-team-summary;")
            && team_surface.contains("export component TeamSummarySlot inherits ResponsiveSlot")
            && team_surface.contains("export component TeamPage inherits PageScrollSurface")
            && team_surface.contains("in property <bool> collapse-label: false;")
            && team_surface.contains("collapse-trailing-label: root.collapse-label;")
            && team_surface.contains("label-collapse := ResponsiveCollapse {")
            && team_surface.contains("collapse-label: label-collapse.collapsed;")
            && team_surface.contains("label: root.ui-text.team-git-identity;")
            && team_surface.contains("primary: root.summary.identity-name;")
            && team_surface.contains("secondary: root.summary.identity-email;")
            && team_surface.contains("label: root.ui-text.team-repository;")
            && team_surface.contains("primary: root.summary.repository-path;")
            && team_surface.contains("secondary: root.ui-text.team-local-only;")
            && team_surface.contains("member-row-slot-height: root.member-row-height + HubTokens.space-2;")
            && team_surface.contains("member-panel-chrome-height: HubTokens.control-md + HubTokens.toolbar-gap + HubTokens.space-4 * 2;")
            && team_surface.contains("member-visible-rows: root.member-count < 6 ? root.member-count : 6;")
            && team_surface.contains("member-available-height: max(root.member-panel-chrome-height + root.member-row-slot-height, root.content-height - root.header-height - root.summary-section-height - HubTokens.panel-gap * 2);")
            && team_surface.contains("member-available-list-height: max(root.member-row-slot-height, root.member-available-height - root.member-panel-chrome-height);")
            && team_surface.contains("member-fit-row-count: Math.floor(root.member-available-list-height / root.member-row-slot-height);")
            && team_surface.contains("member-panel-rows: root.member-visible-rows < 1 ? 1 : (root.member-visible-rows < root.member-fit-rows ? root.member-visible-rows : root.member-fit-rows);")
            && team_surface.contains("member-list-height: root.member-count == 0 ? HubTokens.list-row-lg + HubTokens.space-4 : root.member-panel-rows * root.member-row-height + (root.member-panel-rows - 1) * HubTokens.space-2 + HubTokens.space-1 * 2;")
            && team_surface.contains("members-panel-height: root.member-panel-chrome-height + root.member-list-height;")
            && team_surface.contains("export component TeamMembersPanel inherits HubListPanelSlot")
            && team_surface.contains("TeamMembersPanel {")
            && team_surface.contains("height: root.members-panel-height;")
            && team_surface.contains("member-scroll-y <=> root.member-scroll-y;")
            && team_surface.contains("if root.member-count == 0: EmptyStateBlock")
            && team_surface.contains("for member in root.members: TeamMemberRow {")
            && team_surface.contains("row-height: root.member-row-height;")
            && team_surface.contains("member: member;")
            && team_surface.contains("ui-text: root.ui-text;")
            && team_surface.contains("center-content: true;"),
        "TeamPage summary cards and member list should use compact row labels, tokenized row height, PageScrollSurface content height, complete-row visible budgeting, and an in-panel empty state block"
    );
    assert!(
        !team.contains("component TeamSummarySlot")
            && !team.contains("component TeamMemberRow")
            && !team.contains("component TeamMembersPanel"),
        "team.slint should import page-specific Team panel/row/summary wrappers instead of defining them inline"
    );
    assert_eq!(
        team.matches("TeamSummarySlot {").count(),
        2,
        "TeamPage should render identity and repository summaries through TeamSummarySlot"
    );
    assert_eq!(
        team_components.matches("MetricCard {").count(),
        1,
        "team_page_components.slint should keep summary MetricCard content in TeamSummarySlot instead of repeating it at every summary call site"
    );
    assert_eq!(
        team.matches("TeamMembersPanel {").count(),
        1,
        "TeamPage should render its member list through TeamMembersPanel"
    );
    for forbidden in [
        "ResponsiveSlot {\n                basis: HubTokens.panel-min-sm * 4 / 5;",
        "ResponsiveSlot {\n                basis: HubTokens.panel-min-md;",
        "member-content-height:",
        "members-panel-height: min(root.member-available-height",
        "PanelListViewport {\n            scroll-y <=> root.member-scroll-y;",
        "if root.member-count == 0: EmptyStateBlock",
        "for member in root.members: TeamMemberRow {",
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
    let settings_components = read_ui_file("settings_page_components.slint");
    let shared = read_ui_file("shared.slint");
    let settings_surface = format!("{settings}\n{settings_components}\n{shared}");
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");
    let builds_surface = format!("{builds}\n{builds_components}\n{operation_timeline}");
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
                "order: root.actions-first ? 1 : 0;",
                "flex-order: root.actions-first ? 1 : 0;",
            ][..],
        ),
        (
            "SettingsPage",
            &settings,
            &[
                "SettingsToolchainPanel {",
                "SettingsBuildDefaultsPanel {",
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
        "actions-first: root.compact && root.content-height < root.build-summary-section-height + HubTokens.control-lg;",
        "compact-labels: root.content-width < HubTokens.breakpoint-medium;",
        "side-list-empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
        "compact-height-override: root.build-summary-section-height;",
        "height: root.compact ? root.build-source-summary-height : root.build-summary-height;",
        "height: root.build-summary-height;",
        "order: root.actions-first ? 1 : 0;",
        "order: root.actions-first ? 0 : 1;",
        "min-width: root.compact ? root.content-width : root.overview-min-width;",
        "min-width: root.compact ? root.content-width : root.side-panel-min-width;",
        "collapse-label: root.compact-labels;",
        "export component EditorSideListPanel inherits HubListView",
        "export component EditorSourceEngineListPanel inherits EditorSideListPanel",
        "export component EditorBuildHistoryPanel inherits EditorSideListPanel",
        "in-out property <length> list-scroll-y: 0px;",
        "in property <string> panel-title;",
        "title: root.panel-title;",
        "show-header: true;",
        "body-spacing: HubTokens.toolbar-gap;",
        "scroll-y <=> root.list-scroll-y;",
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
        "HubListPanelSlot,",
        "export component EditorActionsPanel inherits HubListPanelSlot",
        "in property <UiTextData> ui-text;",
        "in property <string> output-path;",
        "callback save-settings();",
        "callback build-engine();",
        "callback open-output();",
        "callback launch-editor();",
        "title: root.ui-text.editor-actions;",
        "row-count: 4;",
        "row-spacing: HubTokens.toolbar-gap;",
        "action-id: \"save-source\";",
        "action-icon: @image-url(\"../assets/icons/ui/settings.svg\");",
        "action-title: root.ui-text.save-source;",
        "action-detail: root.readiness.source-engine-title + \" / \" + root.ui-text.source-checkout-path;",
        "action-id: \"build\";",
        "action-icon: @image-url(\"../assets/icons/actions/build-project.svg\");",
        "action-title: root.ui-text.build;",
        "action-detail: root.readiness.build-enabled ? root.readiness.selected-project-title : root.readiness.build-disabled-reason;",
        "action-id: \"open-output\";",
        "action-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "action-title: root.ui-text.open-output;",
        "action-detail: root.readiness.open-output-enabled ? root.output-path : root.readiness.open-output-disabled-reason;",
        "action-id: \"open-editor\";",
        "action-icon: @image-url(\"../assets/icons/actions/open-editor.svg\");",
        "action-title: root.ui-text.open-editor;",
        "action-detail: root.readiness.open-editor-enabled ? root.readiness.selected-project-title : root.readiness.open-editor-disabled-reason;",
        "action-enabled: root.readiness.open-editor-enabled;",
        "EditorActionsPanel {",
        "readiness: root.readiness;",
        "ui-text: root.ui-text;",
        "output-path: root.source-engine.output-path;",
        "row-height: root.build-row-height;",
        "save-settings => {",
        "build-engine => {",
        "open-output => {",
        "launch-editor => {",
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
        "summary-meta: root.ui-text.last-build-prefix + root.readiness.source-build-summary;",
        "summary-icon: @image-url(\"../assets/brand/zircon-mark.svg\");",
        "summary-status: root.readiness.source-engine-status;",
        "summary-title: root.ui-text.output-prefix;",
        "summary-detail: root.source-engine.output-path;",
        "summary-meta: root.readiness.open-output-enabled ? root.ui-text.profile-prefix + root.source-engine.build-profile : root.readiness.open-output-disabled-reason;",
        "summary-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "summary-status: root.source-engine.jobs + root.ui-text.jobs-suffix;",
        "list-scroll-y <=> root.engine-list-scroll-y;",
        "list-scroll-y <=> root.build-history-scroll-y;",
        "row-count: root.source-engine-count;",
        "row-count: root.source-build-history-count;",
        "empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
        "empty-height: root.side-list-empty-height;",
        "if root.source-engine-count == 0: EmptyStateBlock",
        "height: root.empty-height;",
        "empty-title: root.ui-text.no-source-engines;",
        "if root.source-build-history-count == 0: EmptyStateBlock",
        "empty-title: root.ui-text.no-build-history;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
        "export component EditorSourceSummaryPanel inherits PanelSlot",
        "export component EditorSourceSettingsPanel inherits PanelSlot",
        "EditorSourceSummaryPanel {",
        "EditorSourceSettingsPanel {",
        "readiness: root.readiness;",
        "source-engine: root.source-engine;",
        "ui-text: root.ui-text;",
        "row-height: root.build-row-height;",
        "launch-editor => {",
        "active-engine-name <=> root.active-engine-name;",
        "source-path <=> root.source-path;",
        "output-path <=> root.output-path;",
        "rename-active-engine(name) => {",
        "browse-folder(kind) => {",
    ] {
        assert!(
            editor_surface.contains(snippet),
            "EditorPage is missing tokenized ResponsiveSlot sizing snippet: {snippet}"
        );
    }
    for component in [
        "SourceEngineRow",
        "EditorSideListPanel",
        "EditorSourceEngineListPanel",
        "EditorBuildHistoryPanel",
        "EditorPathFieldRow",
        "EditorActionRow",
        "EditorActionsPanel",
        "EditorSourceSummaryPanel",
        "EditorSourceSettingsPanel",
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
        editor.matches("EditorSourceEngineListPanel {").count(),
        1,
        "EditorPage should render the registered Source Engine list through EditorSourceEngineListPanel"
    );
    assert_eq!(
        editor.matches("EditorBuildHistoryPanel {").count(),
        1,
        "EditorPage should render build history through EditorBuildHistoryPanel"
    );
    assert_eq!(
        editor.matches("EditorActionsPanel {").count(),
        1,
        "EditorPage should render the action column through EditorActionsPanel"
    );
    assert_eq!(
        editor.matches("EditorSourceSummaryPanel {").count(),
        1,
        "EditorPage should render the active source summary through EditorSourceSummaryPanel"
    );
    assert_eq!(
        editor.matches("EditorSourceSettingsPanel {").count(),
        1,
        "EditorPage should render source settings through EditorSourceSettingsPanel"
    );
    assert_eq!(
        editor_components
            .matches("EditorSourceSummaryRow {")
            .count(),
        2,
        "EditorSourceSummaryPanel should own the two source/output summary row call sites"
    );
    assert_eq!(
        editor_components.matches("EditorActionRow {").count(),
        4,
        "EditorActionsPanel should own the four editor action row call sites"
    );
    assert!(
        !editor.contains("SourceEngineRow {")
            && !editor.contains("BuildHistoryRow {")
            && !editor.contains("EditorActionRow {")
            && !editor.contains("EditorSourceSummaryRow {")
            && !editor.contains("EditorPathFieldRow {")
            && !editor.contains("PanelHeader {")
            && !editor.contains("title: root.ui-text.editor-actions;")
            && !editor.contains("action-id: \"save-source\";")
            && !editor.contains("action-id: \"build\";")
            && !editor.contains("action-id: \"open-output\";")
            && !editor.contains("if root.source-engine-count == 0: EmptyStateBlock")
            && !editor.contains("if root.source-build-history-count == 0: EmptyStateBlock"),
        "EditorPage should keep side-list rows, action rows, source summary rows, field rows, headers, and empty-state internals inside typed editor page components"
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
        "export component SettingsToolchainPanel inherits PanelSlot",
        "export component SettingsBuildDefaultsPanel inherits PanelSlot",
        "export component SettingsDefaultPathsPanel inherits HubListPanelSlot",
        "export component SettingsConfigurationHealthPanel inherits HubListPanelSlot",
        "export component SettingsComboChoice inherits Rectangle",
        "export component PathSettingRow inherits Rectangle",
        "export component SettingStatusRow inherits InfoRow",
        "export component SettingsToolchainField inherits HubTextField",
        "export component SettingsSaveActionRow inherits Rectangle",
        "scope: string,",
        "action-id: string,",
        "action-label: string,",
        "disabled-reason: string,",
        "actionable: bool,",
        "callback triggered(string);",
        "callback status-action(string);",
        "detail: root.status.detail;",
        "root.status.disabled-reason == \"\" ? root.status.scope",
        "show-arrow: root.status.actionable;",
        "root.triggered(root.status.action-id);",
        "root.status-action(action-id);",
        "in property <string> label;",
        "in property <string> first-label;",
        "in property <string> first-value;",
        "in property <string> second-label;",
        "in property <string> second-value;",
        "in-out property <string> selected-value;",
        "private property <int> desired-index: root.selected-value == root.second-value ? 1 : 0;",
        "private property <int> selected-index: -1;",
        "private property <[MenuItem]> choice-items:",
        "HubComboBox {",
        "leading-icon: @image-url(\"../assets/icons/ui/settings.svg\");",
        "items: root.choice-items;",
        "current-index: root.selected-index;",
        "selected(index) =>",
        "root.selected-index = index;",
        "root.selected-value = index == 1 ? root.second-value : root.first-value;",
        "init =>",
        "root.selected-index = root.desired-index;",
        "changed selected-value =>",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-lg;",
        "min-width: root.compact ? root.content-width : HubTokens.panel-min-md;",
        "compact-labels: root.content-width < HubTokens.breakpoint-medium;",
        "health-empty-height: HubTokens.list-row-lg + HubTokens.space-4;",
        "panel-title: root.ui-text.toolchain;",
        "panel-title: root.ui-text.build-defaults;",
        "panel-title: root.ui-text.default-paths;",
        "panel-title: root.ui-text.configuration-health;",
        "empty-height: root.health-empty-height;",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "center-content: true;",
        "collapse-trailing-label: root.collapse-label;",
        "collapse-label: root.compact-labels;",
        "save-button-width: min(root.content-width, HubTokens.panel-min-sm);",
        "SettingsSaveActionRow {",
        "button-width: root.save-button-width;",
        "action-label: root.ui-text.save-settings;",
        "root.save-settings();",
        "status-action(action-id) => {",
        "if (action-id == \"save-settings\")",
        "root.browse-folder(\"output\");",
    ] {
        assert!(
            settings_surface.contains(snippet),
            "SettingsPage is missing PanelSlot semantic sizing snippet: {snippet}"
        );
    }
    for component in [
        "PathSettingRow",
        "SettingStatusRow",
        "SettingsToolchainField",
        "SettingsComboChoice",
        "SettingsSaveActionRow",
        "SettingsToolchainPanel",
        "SettingsBuildDefaultsPanel",
        "SettingsDefaultPathsPanel",
        "SettingsConfigurationHealthPanel",
    ] {
        assert!(
            settings_components.contains(&format!("export component {component}")),
            "settings_page_components.slint should export {component}"
        );
        assert!(
            !settings.contains(&format!("component {component}")),
            "settings.slint should import {component} instead of defining it inline"
        );
    }
    assert_eq!(
        settings_surface.matches("SettingsComboChoice {").count(),
        2,
        "SettingsPage should render build profile and language choices through SettingsComboChoice"
    );
    assert_eq!(
        settings_surface.matches("PathSettingRow {").count(),
        4,
        "SettingsPage should render all default paths through PathSettingRow"
    );
    for component in [
        "SettingsToolchainPanel",
        "SettingsBuildDefaultsPanel",
        "SettingsDefaultPathsPanel",
        "SettingsConfigurationHealthPanel",
    ] {
        assert_eq!(
            settings.matches(&format!("{component} {{")).count(),
            1,
            "SettingsPage should render one {component} call site"
        );
    }
    assert_eq!(
        settings.matches("SettingsSaveActionRow {").count(),
        1,
        "SettingsPage should render its save footer through SettingsSaveActionRow"
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
        "PanelSlot {",
        "PanelHeader {",
        "PanelListViewport {",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "for status in root.settings-statuses: SettingStatusRow",
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
        "HubSection,",
        "HubListPanelSlot,",
        "export component BuildSourceSummaryPanel inherits PanelSlot",
        "export component BuildControlsPanel inherits HubListPanelSlot",
        "export component BuildPipelinePanel inherits HubListPanelSlot",
        "export component BuildTaskHistoryPanel inherits PanelSlot",
        "export component OperationTimelinePanel inherits PanelSlot",
        "in property <ProjectDetailData> project;",
        "in property <SourceEngineData> source-engine;",
        "in property <UiTextData> ui-text;",
        "callback build-engine();",
        "callback open-output();",
        "callback launch-editor();",
        "callback package-project();",
        "callback install-device();",
        "in property <[SourceBuildHistoryRowData]> source-build-history;",
        "in property <int> source-build-history-count;",
        "in-out property <length> history-scroll-y: 0px;",
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
        "summary-meta: root.ui-text.last-build-prefix + root.readiness.source-build-summary;",
        "summary-icon: @image-url(\"../assets/icons/nav/builds.svg\");",
        "summary-status: root.readiness.source-engine-status;",
        "summary-title: root.ui-text.output-prefix;",
        "summary-detail: root.source-engine.output-path;",
        "summary-meta: root.ui-text.profile-prefix + root.source-engine.build-profile;",
        "summary-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "summary-status: root.source-engine.jobs + root.ui-text.jobs-suffix;",
        "BuildSourceSummaryPanel {",
        "BuildControlsPanel {",
        "BuildPipelinePanel {",
        "OperationTimelinePanel {",
        "title: root.ui-text.build-controls;",
        "title: root.ui-text.build-pipeline;",
        "row-count: 5;",
        "row-count: 4;",
        "row-spacing: HubTokens.toolbar-gap;",
        "project: root.project;",
        "source-engine: root.source-engine;",
        "status-label: root.status-label;",
        "ui-text: root.ui-text;",
        "row-height: root.build-row-height;",
        "build-engine => { root.build-engine(); }",
        "open-output => { root.open-output(); }",
        "launch-editor => { root.launch-editor(); }",
        "package-project => { root.package-project(); }",
        "install-device => { root.install-device(); }",
        "current-task-title: root.ui-text.current-task;",
        "build-history-title: root.ui-text.build-history;",
        "no-build-history-title: root.ui-text.no-build-history;",
        "source-build-history: root.source-build-history;",
        "source-build-history-count: root.source-build-history-count;",
        "history-scroll-y <=> root.build-history-scroll-y;",
        "HubSection {",
        "current-task-section-height:",
        "history-section-height:",
        "section-height: root.current-task-section-height;",
        "section-height: root.history-section-height;",
        "section-spacing: HubTokens.panel-gap;",
        "title: root.no-build-history-title;",
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
        "BuildSourceSummaryPanel",
        "BuildControlsPanel",
        "BuildPipelinePanel",
        "BuildTaskHistoryPanel",
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
    assert!(
        operation_timeline.contains("export component OperationTimelinePanel inherits PanelSlot"),
        "operation_timeline_components.slint should export the shared OperationTimelinePanel"
    );
    assert!(
        !builds_components.contains("OperationTimelinePanel"),
        "builds_page_components.slint should not own the shared OperationTimelinePanel"
    );
    assert_eq!(
        builds_components.matches("BuildControlAction {").count(),
        5,
        "builds_page_components.slint should render all build controls through BuildControlAction"
    );
    assert_eq!(
        builds.matches("\n            ActionRow {").count(),
        0,
        "BuildsPage should keep ActionRow construction inside BuildControlAction instead of repeating it at each build-control call site"
    );
    assert_eq!(
        builds_components.matches("BuildPipelineStep {").count(),
        4,
        "builds_page_components.slint should render build pipeline rows through BuildPipelineStep"
    );
    assert_eq!(
        builds_components.matches("BuildSourceSummaryRow {").count(),
        2,
        "builds_page_components.slint should render source and output summary rows through BuildSourceSummaryRow"
    );
    assert_eq!(
        builds.matches("BuildTaskHistoryPanel {").count(),
        1,
        "BuildsPage should render current task and build history through BuildTaskHistoryPanel"
    );
    for component in [
        "BuildSourceSummaryPanel",
        "BuildControlsPanel",
        "BuildPipelinePanel",
    ] {
        assert_eq!(
            builds.matches(&format!("{component} {{")).count(),
            1,
            "BuildsPage should render one {component} call site"
        );
    }
    assert!(
        !builds.contains("PanelListViewport {")
            && !builds.contains("for record in root.source-build-history: BuildHistoryRow")
            && !builds.contains("if root.source-build-history-count == 0: EmptyStateBlock")
            && !builds.contains("PanelHeader {")
            && !builds.contains("BuildControlAction {")
            && !builds.contains("BuildPipelineStep {")
            && !builds.contains("BuildSourceSummaryRow {"),
        "BuildsPage should keep panel headers, rows, build-history list, and empty-state internals inside builds_page_components.slint"
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
