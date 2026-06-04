use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .to_string_lossy()
            .into_owned()
    }))
}

fn read_ui_file(name: &str) -> String {
    fs::read_to_string(crate_dir().join("ui").join(name))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {name}: {error}"))
}

#[test]
fn panel_slot_wraps_responsive_slot_and_material_panel_body() {
    let surfaces = read_ui_file("surfaces.slint");
    let components = read_ui_file("components.slint");

    for snippet in [
        "import { Divider, ResponsiveSlot } from \"layout.slint\";",
        "export component PanelSlot inherits ResponsiveSlot",
        "in property <length> body-padding: HubTokens.space-4;",
        "in property <length> body-spacing: HubTokens.toolbar-gap;",
        "HubPanel {",
        "variant: root.variant;",
        "VerticalLayout {",
        "padding: root.body-padding;",
        "spacing: root.body-spacing;",
        "@children",
    ] {
        assert!(
            surfaces.contains(snippet),
            "PanelSlot should own responsive panel fill and body layout; missing {snippet}"
        );
    }

    assert!(
        components.contains("PanelSlot,"),
        "components.slint must re-export PanelSlot for page modules"
    );

    let panel_slot = surfaces
        .split("export component PanelSlot")
        .nth(1)
        .and_then(|source| source.split("export component HeaderGroup").next())
        .expect("surfaces.slint must declare PanelSlot before HeaderGroup");
    assert!(
        !panel_slot.contains("TouchArea"),
        "PanelSlot must not add a page-level pointer layer above nested controls"
    );
}

#[test]
fn overview_panel_wraps_cloud_summary_header_and_team_uses_metric_first_overview() {
    let surfaces = read_ui_file("surfaces.slint");
    let components = read_ui_file("components.slint");
    let cloud = read_ui_file("cloud.slint");
    let team = read_ui_file("team.slint");

    for snippet in [
        "export component OverviewPanel inherits HubPanel",
        "in property <length> body-padding-horizontal: MaterialStyleMetrics.padding_16;",
        "in property <length> body-padding-vertical: MaterialStyleMetrics.padding_14;",
        "PanelHeader {",
        "title: root.title;",
        "subtitle: root.subtitle;",
        "badge-text: root.badge-text;",
        "badge-width: root.badge-width;",
    ] {
        assert!(
            surfaces.contains(snippet),
            "OverviewPanel should own the Material-backed summary panel shell; missing {snippet}"
        );
    }
    assert!(
        components.contains("OverviewPanel,"),
        "components.slint must re-export OverviewPanel for page modules"
    );

    let cloud_top_region = cloud
        .split("WorkspacePanelSection {")
        .next()
        .expect("CloudPage should declare its overview before WorkspacePanelSection");
    assert!(
        cloud.contains("OverviewPanel,") && cloud_top_region.contains("OverviewPanel {"),
        "CloudPage should route the top summary card through OverviewPanel"
    );
    for snippet in [
        "height: root.header-height;",
        "title: root.overview-title;",
        "badge-text: root.summary.status;",
    ] {
        assert!(
            cloud_top_region.contains(snippet),
            "CloudPage OverviewPanel should preserve summary header bindings: {snippet}"
        );
    }
    assert!(
        !cloud_top_region.contains("HubPanel {")
            && !cloud_top_region.contains("padding-left: MaterialStyleMetrics.padding_16;")
            && !cloud_top_region.contains("padding-top: MaterialStyleMetrics.padding_14;"),
        "CloudPage should not repeat top summary HubPanel/VerticalLayout padding boilerplate"
    );

    assert!(
        !team.contains("OverviewPanel,")
            && !team.contains("OverviewPanel {")
            && team.contains("summary-metrics := HubMetricSectionState {")
            && team.contains("allow-two-columns: false;")
            && team.contains("wide-breakpoint: HubTokens.panel-min-sm * 3 + HubTokens.panel-gap * 3;")
            && team.contains("compact-rows: summary-metrics.row-count;")
            && team.matches("TeamSummarySlot {").count() == 4
            && team.contains("TeamMembersPanel {")
            && team.contains("TeamActionsPanel {"),
        "TeamPage should use the HTML-reference metric-first layout instead of a sparse top OverviewPanel"
    );
}

#[test]
fn empty_state_primitives_wrap_project_and_team_empty_states() {
    let surfaces = read_ui_file("surfaces.slint");
    let components = read_ui_file("components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let team = read_ui_file("team.slint");
    let team_components = read_ui_file("team_page_components.slint");

    for snippet in [
        "export component EmptyStateBlock inherits Rectangle",
        "export component EmptyStatePanel inherits HubPanel",
        "in property <string> title;",
        "in property <string> detail;",
        "in property <string> extra-detail;",
        "in property <length> body-padding: HubTokens.space-4;",
        "in property <bool> title-prominent: false;",
        "in property <bool> center-content: false;",
        "MaterialText {",
        "text: root.title;",
        "if root.detail != \"\": MutedText",
        "if root.extra-detail != \"\": MutedText",
        "EmptyStateBlock {",
    ] {
        assert!(
            surfaces.contains(snippet),
            "EmptyStatePanel should own the reusable Material empty-state panel; missing {snippet}"
        );
    }
    assert!(
        components.contains("EmptyStateBlock,") && components.contains("EmptyStatePanel,"),
        "components.slint must re-export empty-state primitives for page and data-display modules"
    );

    let project_flow = project_card_flow_components
        .split("export component ProjectFlow")
        .nth(1)
        .and_then(|source| {
            source
                .split("export component DashboardProjectCardsSection")
                .next()
        })
        .expect(
            "project_card_flow_components.slint must export ProjectFlow before DashboardProjectCardsSection",
        );
    for snippet in [
        "if root.project-card-count == 0: EmptyStatePanel",
        "title: root.empty-title;",
        "detail: root.empty-detail;",
        "body-padding: HubTokens.space-6;",
        "title-prominent: true;",
    ] {
        assert!(
            project_flow.contains(snippet),
            "ProjectFlow empty state should route through EmptyStatePanel: {snippet}"
        );
    }
    assert!(
        !project_flow.contains("if root.project-card-count == 0: HubPanel")
            && !project_flow.contains("VerticalLayout {\n            width: parent.width;"),
        "ProjectFlow should not repeat empty-state HubPanel/VerticalLayout boilerplate"
    );

    let member_empty = team_components
        .split("if root.member-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("for member in root.members").next())
        .expect(
            "team_page_components.slint must declare member EmptyStateBlock before member rows",
        );
    for snippet in [
        "title: root.ui-text.no-team-members-found;",
        "detail: root.members-empty-detail;",
        "extra-detail: root.repository-path;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            member_empty.contains(snippet),
            "TeamPage empty state should preserve bindings through EmptyStateBlock: {snippet}"
        );
    }
    assert!(
        team_components.contains("EmptyStateBlock,")
            && team.contains("TeamMembersPanel {")
            && !team.contains("if root.member-count == 0: EmptyStateBlock")
            && !member_empty.contains("MaterialText {")
            && !member_empty.contains("MutedText {")
            && !member_empty.contains("HubPanel {"),
        "TeamPage should consume TeamMembersPanel while the component uses shared EmptyStateBlock instead of local text/panel boilerplate"
    );
}

#[test]
fn builds_page_uses_panel_slot_for_workspace_panels() {
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let operation_timeline = read_ui_file("operation_timeline_components.slint");

    assert!(
        builds_components.contains("HubContentPanelSlot,")
            && builds_components.contains("HubListPanelSlot,")
            && !builds_components.contains("\n    PanelSlot,")
            && !builds.contains("PanelSlot,"),
        "BuildsPage should leave shared content/list panel primitives to builds_page_components.slint"
    );
    assert_eq!(
        builds_components.matches("inherits PanelSlot").count()
            + builds_components
                .matches("inherits HubContentPanelSlot")
                .count()
            + builds_components.matches("inherits HubListPanelSlot").count()
            + operation_timeline
                .matches("export component OperationTimelinePanel inherits HubListPanelSlot")
                .count(),
        5,
        "BuildsPage should route its five workspace panels through focused panel/list-slot-backed components, with the shared operation timeline hosted by operation_timeline_components.slint"
    );
    assert!(
        builds.contains("BuildTaskHistoryPanel {")
            && builds_components
                .contains("export component BuildTaskHistoryPanel inherits HubContentPanelSlot"),
        "BuildsPage should route current-task/build-history chrome through the HubContentPanelSlot-backed BuildTaskHistoryPanel"
    );
    for (component, base, message) in [
        (
            "BuildSourceSummaryPanel",
            "HubContentPanelSlot",
            "source/output summary chrome through the HubContentPanelSlot-backed BuildSourceSummaryPanel",
        ),
        (
            "BuildControlsPanel",
            "HubListPanelSlot",
            "build controls through the HubListPanelSlot-backed BuildControlsPanel",
        ),
        (
            "BuildPipelinePanel",
            "HubListPanelSlot",
            "pipeline rows through the HubListPanelSlot-backed BuildPipelinePanel",
        ),
    ] {
        assert!(
            builds.contains(&format!("{component} {{"))
                && builds_components
                    .contains(&format!("export component {component} inherits {base}")),
            "BuildsPage should route {message}"
        );
    }
    assert!(
        builds.contains("OperationTimelinePanel {")
            && operation_timeline
                .contains("export component OperationTimelinePanel inherits HubListPanelSlot"),
        "BuildsPage should route operation timeline chrome through the shared operation-timeline OperationTimelinePanel"
    );
    for forbidden in [
        "PanelSlot {",
        "ResponsiveSlot {",
        "HubPanel {",
        "PanelHeader {",
        "BuildControlAction {",
        "BuildPipelineStep {",
        "BuildSourceSummaryRow {",
        "VerticalLayout {\n                        width: parent.width;",
        "VerticalLayout {\n                    width: parent.width;",
    ] {
        assert!(
            !builds.contains(forbidden),
            "BuildsPage should not repeat responsive panel fill/body boilerplate after PanelSlot migration: {forbidden}"
        );
    }
}

#[test]
fn editor_page_uses_panel_slot_for_standard_workspace_panels() {
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let editor_surface = format!("{editor}\n{editor_components}");

    assert!(
        editor_components.contains("PanelSlot,") && !editor.contains("PanelSlot,"),
        "EditorPage should leave the shared PanelSlot primitive import to editor_page_components.slint"
    );
    assert_eq!(
        editor.matches("PanelSlot {").count(),
        0,
        "EditorPage should route overview/actions/config panels through typed PanelSlot-backed components"
    );
    assert_eq!(
        editor.matches("ResponsiveSlot {").count(),
        1,
        "EditorPage should keep ResponsiveSlot only for the split side slot that contains two nested panels"
    );
    assert!(
        editor_components.contains("export component EditorSideListPanel inherits HubListView"),
        "EditorPage should consolidate the split side-list HubListView shell in an exported page component"
    );
    assert_eq!(
        editor_components
            .matches("export component EditorSideListPanel inherits HubListView")
            .count(),
        1,
        "editor_page_components.slint should own one side-list panel shell instead of repeating list boilerplate"
    );
    assert!(
        !editor.contains("component EditorSideListPanel"),
        "editor.slint should import EditorSideListPanel instead of defining it inline"
    );
    assert_eq!(
        editor.matches("EditorSourceEngineListPanel {").count(),
        1,
        "EditorPage split side slot should use EditorSourceEngineListPanel for Source Engines"
    );
    assert_eq!(
        editor.matches("EditorBuildHistoryPanel {").count(),
        1,
        "EditorPage split side slot should use EditorBuildHistoryPanel for Build History"
    );
    for component in ["EditorSourceEngineListPanel", "EditorBuildHistoryPanel"] {
        assert!(
            editor_components.contains(&format!(
                "export component {component} inherits EditorSideListPanel"
            )),
            "editor_page_components.slint should export typed side-list component {component}"
        );
        assert!(
            !editor.contains(&format!("component {component}")),
            "editor.slint should import {component} instead of defining it inline"
        );
    }
    assert!(
        !editor.contains("SourceEngineRow {")
            && !editor.contains("BuildHistoryRow {")
            && !editor.contains("if root.source-engine-count == 0: EmptyStateBlock")
            && !editor.contains("if root.source-build-history-count == 0: EmptyStateBlock"),
        "EditorPage split side slot should keep row loops and empty states inside typed side-list components"
    );
    assert!(
        editor.contains("EditorSourceSummaryPanel {")
            && editor_components
                .contains("export component EditorSourceSummaryPanel inherits HubContentPanelSlot"),
        "EditorPage should route the active source summary through the HubContentPanelSlot-backed EditorSourceSummaryPanel"
    );
    assert!(
        editor.contains("EditorSourceSettingsPanel {")
            && editor_components
                .contains("export component EditorSourceSettingsPanel inherits HubFormPanelSlot"),
        "EditorPage should route the source settings form through the HubFormPanelSlot-backed EditorSourceSettingsPanel"
    );
    assert!(
        editor.contains("EditorActionsPanel {")
            && editor_components
                .contains("export component EditorActionsPanel inherits HubListPanelSlot"),
        "EditorPage should route the actions panel through the HubListPanelSlot-backed EditorActionsPanel"
    );
    assert!(
        editor_components.contains("export component EditorActionRow inherits ActionRow"),
        "EditorPage should consolidate editor action row construction in an exported ActionRow wrapper"
    );
    assert!(
        !editor.contains("component EditorActionRow"),
        "editor.slint should import EditorActionRow instead of defining it inline"
    );
    assert_eq!(
        editor_components.matches("EditorActionRow {").count(),
        4,
        "EditorActionsPanel should route Save Source, Build, Open Output, and Open Editor actions through EditorActionRow"
    );
    assert_eq!(
        editor.matches("EditorActionRow {").count(),
        0,
        "EditorPage should keep action rows inside EditorActionsPanel instead of repeating them in the page"
    );

    for snippet in [
        "body-spacing: HubTokens.space-3;",
        "actions-first: workspace-split.compact && root.content-height < root.build-summary-section-height + HubTokens.control-lg;",
        "editor-config-height: HubTokens.workspace-row-editor-config - HubTokens.list-row-md - HubTokens.control-lg;",
        "side-list-empty-height: HubTokens.list-row-sm + HubTokens.space-4;",
        "order: root.actions-first ? 1 : 0;",
        "flex-order: root.actions-first ? 1 : 0;",
        "order: root.actions-first ? 0 : 1;",
        "flex-order: root.actions-first ? 0 : 1;",
        "row-height: root.editor-config-height;",
        "height: root.editor-config-height;",
        "export component EditorSourceSummaryPanel inherits HubContentPanelSlot",
        "export component EditorSourceSettingsPanel inherits HubFormPanelSlot",
        "EditorSourceSummaryPanel {",
        "EditorSourceSettingsPanel {",
        "source-engine: root.source-engine;",
        "launch-editor => {",
        "active-engine-name <=> root.active-engine-name;",
        "source-path <=> root.source-path;",
        "output-path <=> root.output-path;",
        "rename-active-engine(name) => {",
        "browse-folder(kind) => {",
        "title: root.readiness.selected-project-title;",
        "export component EditorActionsPanel inherits HubListPanelSlot",
        "row-count: 4;",
        "row-spacing: HubTokens.toolbar-gap;",
        "title: root.ui-text.editor-actions;",
        "title: root.ui-text.source-engine;",
        "panel-title: root.ui-text.source-engines;",
        "panel-title: root.ui-text.build-history;",
        "badge-text: root.source-engine-count + \"\";",
        "badge-text: root.source-build-history-count + \"\";",
        "source-engines: root.source-engines;",
        "source-engine-count: root.source-engine-count;",
        "source-build-history: root.source-build-history;",
        "source-build-history-count: root.source-build-history-count;",
        "empty-title: root.ui-text.no-source-engines;",
        "empty-title: root.ui-text.no-build-history-short;",
        "ui-text: root.ui-text;",
        "output-path: root.source-engine.output-path;",
        "row-height: root.build-row-height;",
        "save-settings => {",
        "build-engine => {",
        "open-output => {",
        "action-id: \"save-source\";",
        "action-icon: @image-url(\"../assets/icons/ui/settings.svg\");",
        "action-title: root.ui-text.save-source;",
        "action-detail: root.readiness.source-engine-title + \" / \" + root.ui-text.source-checkout-path;",
        "root.save-settings();",
        "action-id: \"build\";",
        "action-icon: @image-url(\"../assets/icons/actions/build-project.svg\");",
        "action-title: root.ui-text.build;",
        "action-detail: root.readiness.build-enabled ? root.readiness.selected-project-title : root.readiness.build-disabled-reason;",
        "root.build-engine();",
        "action-id: \"open-output\";",
        "action-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "action-title: root.ui-text.open-output;",
        "action-detail: root.readiness.open-output-enabled ? root.output-path : root.readiness.open-output-disabled-reason;",
        "root.open-output();",
        "action-id: \"open-editor\";",
        "action-icon: @image-url(\"../assets/icons/actions/open-editor.svg\");",
        "action-title: root.ui-text.open-editor;",
        "action-detail: root.readiness.open-editor-enabled ? root.readiness.selected-project-title : root.readiness.open-editor-disabled-reason;",
        "root.launch-editor();",
    ] {
        assert!(
            editor_surface.contains(snippet),
            "EditorPage PanelSlot migration should preserve panel content and spacing: {snippet}"
        );
    }
    for forbidden in [
        "PanelSlot {",
        "PanelHeader {",
        "EditorSourceSummaryRow {",
        "EditorPathFieldRow {",
        "EditorActionRow {",
        "title: root.ui-text.editor-actions;",
        "action-id: \"save-source\";",
        "action-id: \"build\";",
        "action-id: \"open-output\";",
    ] {
        assert!(
            !editor.contains(forbidden),
            "EditorPage should keep standard panel internals inside typed editor panel components: {forbidden}"
        );
    }
}

#[test]
fn project_dashboard_lower_panels_use_panel_slot() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    let lower_grid = dashboard
        .split("PanelGrid {")
        .nth(1)
        .expect("ProjectDashboardPage must keep a lower PanelGrid");

    assert!(
        dashboard_components.contains("HubTableView,")
            && dashboard_components.contains("HubListPanelSlot,"),
        "Project Dashboard page components should import the shared table and list panel primitives"
    );
    assert_eq!(
        lower_grid.matches("PanelSlot {").count()
            + dashboard_components.matches("inherits PanelSlot").count()
            + dashboard_components.matches("inherits HubTableView").count()
            + dashboard_components.matches("inherits HubListPanelSlot").count(),
        2,
        "ProjectDashboardPage lower Recent/Quick panels should route through shared panel-backed components"
    );
    assert_eq!(
        lower_grid.matches("DashboardRecentProjectsPanel {").count(),
        1,
        "ProjectDashboardPage lower grid should compose one typed Recent Projects panel"
    );
    assert_eq!(
        lower_grid.matches("DashboardQuickActionsPanel {").count(),
        1,
        "ProjectDashboardPage lower grid should compose one typed Quick Actions panel"
    );
    assert_eq!(
        lower_grid.matches("ResponsiveSlot {").count(),
        0,
        "ProjectDashboardPage lower PanelGrid should leave ResponsiveSlot boilerplate to PanelSlot"
    );
    for snippet in [
        "export component DashboardQuickActionRow inherits ActionRow",
        "export component DashboardRecentProjectsPanel inherits HubTableView",
        "export component DashboardQuickActionsPanel inherits HubListPanelSlot",
        "HubTableView,",
        "HubListPanelSlot,",
        "ActionRow,",
        "DashboardRecentProjectsPanel {",
        "DashboardQuickActionsPanel {",
        "panel-title: root.dashboard-project-title;",
        "action-text: root.ui-text.view-all-projects;",
        "project-rows: root.dashboard-project-rows;",
        "project-row-count: root.dashboard-project-row-count;",
        "show-header: true;",
        "title: root.panel-title;",
        "show-divider: false;",
        "minimum-row-height: root.table-row-height;",
        "DataTable {",
        "HubPanelNavigationCommand {",
        "text: root.action-text;",
        "source-image: @image-url(\"../assets/icons/nav/projects.svg\");",
        "panel-title: root.ui-text.quick-actions;",
        "quick-actions: root.quick-actions;",
        "quick-action-count: root.quick-action-count;",
        "action: root.quick-action-data;",
        "detail-override: root.visual-detail;",
        "compact-shell: true;",
        "plain-avatar: true;",
        "plain-trailing: true;",
        "leading-shell-size-override: MaterialStyleMetrics.size_24;",
        "leading-icon-size-override: MaterialStyleMetrics.size_24;",
        "root.triggered(id);",
        "header-height: HubTokens.control-md * 2 / 3;",
        "quick-actions-scroll-y <=> root.quick-actions-scroll-y;",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "row-count: root.quick-action-count;",
        "row-height: root.quick-action-row-height;",
        "row-spacing: root.quick-action-row-gap;",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "quick-action-data: action;",
        "triggered(id) => { root.triggered(id); }",
        "if root.quick-action-count == 0: EmptyStateBlock {",
        "empty-title: root.ui-text.no-quick-actions;",
        "empty-detail: root.ui-text.quick-actions-empty-detail;",
    ] {
        assert!(
            lower_grid.contains(snippet) || dashboard_surface.contains(snippet),
            "ProjectDashboardPage lower PanelSlot migration should preserve panel content: {snippet}"
        );
    }
    assert_eq!(
        dashboard_components
            .matches("export component DashboardQuickActionRow inherits ActionRow")
            .count(),
        1,
        "project_dashboard_components.slint should own one exported Quick Actions ActionRow wrapper"
    );
    assert!(
        !dashboard.contains("component DashboardQuickActionRow inherits"),
        "ProjectDashboardPage should import DashboardQuickActionRow instead of defining it inline"
    );
    assert_eq!(
        dashboard_components
            .matches("DashboardQuickActionRow {")
            .count(),
        1,
        "Quick Actions should route repeated action rows through DashboardQuickActionRow inside the typed panel component"
    );
    for forbidden in [
        "HubPanel {\n                            width: parent.width;",
        "VerticalLayout {\n                                width: parent.width;",
        "DataTable {",
        "action-list := PanelListViewport {",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "if root.quick-action-count == 0: EmptyStateBlock {",
        "for action in root.quick-actions: ActionRow {",
    ] {
        assert!(
            !lower_grid.contains(forbidden),
            "ProjectDashboardPage lower PanelGrid should not repeat panel shell boilerplate after PanelSlot migration: {forbidden}"
        );
    }
}

#[test]
fn cloud_and_team_lower_list_panels_use_panel_slot_shells() {
    let cloud = read_ui_file("cloud.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");
    let team = read_ui_file("team.slint");
    let team_components = read_ui_file("team_page_components.slint");

    assert!(
        !cloud.contains("from \"builds_page_components.slint\"")
            && !cloud.contains("BuildControlAction {")
            && cloud_components.contains("export component CloudPackageActionRow inherits ActionRow")
            && cloud_components
                .contains("export component CloudPackageActionsPanel inherits HubListPanelSlot")
            && cloud.contains("CloudPackageActionsPanel {")
            && cloud.contains("row-height: root.workflow-row-height;")
            && cloud.contains("height: root.actions-panel-height;")
            && cloud.contains("Column {")
            && cloud.contains("OperationTimelinePanel {")
            && cloud.contains("height: root.workflow-timeline-height;")
            && cloud.contains("summary: root.summary;")
            && cloud.contains("ui-text: root.ui-text;")
            && cloud.contains("package-project => {")
            && cloud.contains("install-device => {"),
        "CloudPage should own package/install actions through a HubListPanelSlot-backed cloud component instead of importing Builds page components"
    );

    for (
        page,
        source,
        component_source,
        component_call,
        component_export,
        sizing_snippet,
        height_snippet,
        scroll_snippet,
        title_snippet,
    ) in [
        (
            "CloudPage",
            &cloud,
            &cloud_components,
            "CloudServicesPanel {",
            "export component CloudServicesPanel inherits HubListPanelSlot",
            "flex-basis: HubTokens.panel-min-lg;",
            "height: root.workflow-row-height;",
            "service-scroll-y <=> root.service-scroll-y;",
            "title: root.ui-text.cloud-services;",
        ),
        (
            "TeamPage",
            &team,
            &team_components,
            "TeamMembersPanel {",
            "export component TeamMembersPanel inherits HubTabbedListPanelSlot",
            "flex-basis: HubTokens.panel-min-lg;",
            "height: root.overview-section-height;",
            "member-scroll-y <=> root.member-scroll-y;",
            "title: \"Team Overview\";",
        ),
    ] {
        assert!(
            !source.contains("PanelSlot,") && component_source.contains("HubListPanelSlot"),
            "{page} should import typed page components while its component module imports HubListPanelSlot"
        );
        assert_eq!(
            source.matches("PanelSlot {").count(),
            0,
            "{page} should not keep lower panels as direct PanelSlot calls in the page"
        );
        assert!(
            source.contains(component_call) && component_source.contains(component_export),
            "{page} should route its lower list panel through a typed PanelSlot-backed component"
        );
        for snippet in [sizing_snippet, height_snippet, scroll_snippet] {
            assert!(
                source.contains(snippet),
                "{page} lower typed panel call should preserve list panel sizing and scroll forwarding: {snippet}"
            );
        }
        for snippet in [
            "body-padding: MaterialStyleMetrics.padding_16;",
            "body-spacing: HubTokens.toolbar-gap;",
            title_snippet,
            "HubListPanelSlot,",
            "show-badge: true;",
        ] {
            assert!(
                component_source.contains(snippet),
                "{page} lower typed panel should preserve list panel content: {snippet}"
            );
        }
    }

    assert!(
        team.contains("TeamActionsPanel {")
            && team.contains("action-row-height: root.action-row-height;")
            && team_components.contains("export component TeamActionsPanel inherits HubListPanelSlot")
            && team_components.contains("export component TeamActionRow inherits ActionRow")
            && team_components.contains("HubToggleRow {")
            && team_components.contains("HubCheckBoxRow {"),
        "TeamPage should pair the compact Team Overview panel with a HubListPanelSlot-backed actions panel that reuses shared action/toggle/checkbox rows"
    );

    for forbidden in [
        "PanelListViewport {\n            scroll-y <=> root.service-scroll-y;",
        "PanelListViewport {\n            scroll-y <=> root.member-scroll-y;",
        "if root.service-count == 0: EmptyStateBlock",
        "if root.member-count == 0: EmptyStateBlock",
        "for service in root.services: CloudServiceRow {",
        "for member in root.members: TeamMemberRow {",
        "BuildControlAction {",
        "HubPanel {\n            horizontal-stretch: 1;\n            height: root.services-panel-height;",
        "HubPanel {\n            horizontal-stretch: 1;\n            height: root.members-panel-height;",
        "VerticalLayout {\n                width: parent.width;\n                height: parent.height;\n                padding: MaterialStyleMetrics.padding_16;",
        "for member in root.members: Rectangle {",
    ] {
        assert!(
            !cloud.contains(forbidden) && !team.contains(forbidden),
            "Cloud/Team lower list panels should not repeat outer HubPanel body boilerplate after PanelSlot migration: {forbidden}"
        );
    }
    assert!(
        team.contains("TeamMembersPanel {")
            && team.contains("member-row-height: root.member-row-height;")
            && team.contains("member-scroll-y <=> root.member-scroll-y;")
            && team.contains("collapse-label: label-collapse.collapsed;")
            && team_components.contains("for member in root.members: TeamMemberRow {")
            && team_components.contains("row-height: root.member-row-height;")
            && team_components.contains("member: member;")
            && team_components.contains("ui-text: root.ui-text;")
            && team_components.contains("export component TeamMemberRow inherits InfoRow")
            && !team.contains("component TeamMemberRow"),
        "TeamPage member list should route TeamMemberRow repetition through TeamMembersPanel while preserving row height, data, copy, and compact trailing-label bindings"
    );
}

#[test]
fn project_secondary_pages_use_panel_slot_for_standard_new_and_detail_panels() {
    let project_pages = read_ui_file("project_pages.slint");
    let new_page = read_ui_file("project_new_page.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let detail_components = read_ui_file("project_detail_components.slint");
    let browser_page = read_ui_file("project_browser_page.slint");
    let detail_page = read_ui_file("project_detail_page.slint");
    let project_new_surface = format!("{new_page}\n{project_components}");
    let project_detail_surface = format!("{detail_page}\n{detail_components}");

    assert!(
        !new_page.contains("PanelSlot")
            && project_components.contains("PanelSlot")
            && !detail_page.contains("PanelSlot"),
        "Project New and Detail pages should leave shared panel primitives to typed component modules"
    );
    assert_eq!(
        new_page.matches("PanelSlot {").count(),
        0,
        "ProjectNewPage should route settings, compact summary, and template panels through typed shared panel-shell components"
    );
    assert!(
        new_page.contains("ProjectCreateSettingsPanel {")
            && project_components.contains(
                "export component ProjectCreateSettingsPanel inherits HubContentPanelSlot"
            ),
        "ProjectNewPage should route the settings form through the shared content-panel component"
    );
    assert!(
        new_page.contains("ProjectCreateCompactSummaryPanel {")
            && project_components
                .contains("export component ProjectCreateCompactSummaryPanel inherits HubContentPanelSlot"),
        "ProjectNewPage should route the compact summary through a typed HubContentPanelSlot-backed component"
    );
    assert!(
        new_page.contains("ProjectTemplateRailPanel {")
            && project_components
                .contains("export component ProjectTemplateRailPanel inherits HubListPanelSlot"),
        "ProjectNewPage should route the template rail through the shared list PanelSlot-backed component"
    );
    assert_eq!(
        detail_page.matches("PanelSlot {").count(),
        0,
        "ProjectDetailPage should route the main detail and actions columns through typed panel components"
    );
    assert!(
        detail_page.contains("ProjectDetailMainPanel {")
            && detail_components.contains(
                "export component ProjectDetailMainPanel inherits HubMediaContentPanelSlot"
            ),
        "ProjectDetailPage should route the main media/status/info column through the shared media content-panel component"
    );
    assert!(
        detail_page.contains("ProjectDetailActionsSection {")
            && detail_components.contains("export component ProjectDetailActionsSection inherits HubContentPanelSlot"),
        "ProjectDetailPage should route the actions column through the shared content-panel component"
    );
    assert_eq!(
        browser_page.matches("PanelSlot {").count(),
        0,
        "ProjectBrowserPage should keep its explicit list panel because it owns a custom inner scroll viewport"
    );
    assert!(
        project_pages
            .contains("export { ProjectBrowserPage } from \"project_browser_page.slint\";"),
        "project_pages.slint should keep ProjectBrowserPage as a stable re-export"
    );
    assert!(
        project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";")
            && !project_pages.contains("export component ProjectNewPage inherits"),
        "project_pages.slint should keep ProjectNewPage as a stable re-export"
    );
    assert!(
        project_pages.contains("export { ProjectDetailPage } from \"project_detail_page.slint\";")
            && !project_pages.contains("export component ProjectDetailPage inherits"),
        "project_pages.slint should keep ProjectDetailPage as a stable re-export"
    );

    for snippet in [
        "body-spacing: root.page-gap;",
        "export component ProjectCreateSettingsPanel inherits HubContentPanelSlot",
        "export component ProjectCreateCompactSummaryPanel inherits HubContentPanelSlot",
        "export component ProjectTemplateRailPanel inherits HubListPanelSlot",
        "HubListPanelSlot,",
        "title: root.ui-text.project-settings-title;",
        "content-spacing: root.page-gap;",
        "ProjectCreateField {",
        "body-spacing: 0px;",
        "content-spacing: 0px;",
        "show-header: false;",
        "export component ProjectCreateSummary inherits HubSection",
        "ProjectCreateSummary {",
        "panel-title: root.ui-text.templates-title;",
        "scroll-y <=> root.list-scroll-y;",
        "row-count: root.template-count;",
        "empty-height: HubTokens.list-row-lg;",
        "for template in root.templates: TemplateChoiceRow {",
        "ProjectCreateActionRow {",
        "row-height: root.create-action-row-height;",
        "height: root.detail-main-height;",
        "height: root.detail-action-height;",
        "export component ProjectDetailMainPanel inherits HubMediaContentPanelSlot",
        "media-height: root.cover-height;",
        "media-background: root.project.accent == 0 ? MaterialPalette.primary_container",
        "has-media-source: root.project.has-cover;",
        "ProjectDetailMainPanel {",
        "cover-height: root.cover-height;",
        "content-stack-spacing: root.page-gap;",
        "ProjectDetailStatusStrip {",
        "row-height: root.status-row-height;",
        "ProjectDetailInfoSection {",
        "section-height: root.detail-info-section-height;",
        "export component ProjectDetailActionsSection inherits HubContentPanelSlot",
        "title: root.copy.project-actions-title;",
        "content-spacing: root.panel-spacing;",
        "detail-engine-scroll-y: 0px;",
        "ProjectEngineChoiceList {",
        "list-scroll-y <=> root.engine-scroll-y;",
        "ProjectDetailEngineSection {",
        "section-height: root.engine-section-height;",
    ] {
        assert!(
            project_new_surface.contains(snippet) || project_detail_surface.contains(snippet),
            "Projects PanelSlot migration should preserve panel content and sizing: {snippet}"
        );
    }

    for forbidden in [
        "HubPanel {",
        "template-list := PanelListViewport",
        "for template in root.templates: TemplateChoiceRow",
        "PanelHeader {\n                                title: root.ui-text.project-settings-title;",
        "ProjectCreateField {",
        "ProjectCreateActionRow {",
        "ProjectEngineChoiceList {",
        "VerticalLayout {\n                                width: parent.width;",
        "VerticalLayout {\n                            width: parent.width;",
    ] {
        assert!(
            !new_page.contains(forbidden) && !detail_page.contains(forbidden),
            "ProjectNewPage and ProjectDetailPage should not repeat panel shell boilerplate after PanelSlot migration: {forbidden}"
        );
    }
}

#[test]
fn settings_page_uses_panel_slot_for_semantic_panel_shells() {
    let settings = read_ui_file("settings.slint");
    let settings_components = read_ui_file("settings_page_components.slint");
    let settings_surface = format!("{settings}\n{settings_components}");

    assert!(
        settings_components.contains("PanelSlot,")
            && settings_components.contains("HubContentPanelSlot,")
            && settings_components.contains("HubFormPanelSlot,")
            && settings_components.contains("HubListPanelSlot,")
            && !settings.contains("PanelSlot,"),
        "SettingsPage should leave shared panel primitives to settings_page_components.slint"
    );
    assert_eq!(
        settings_components.matches("inherits PanelSlot").count(),
        0,
        "SettingsPage should not keep semantic panels on direct PanelSlot after shared content/form/list shell extraction"
    );
    assert_eq!(
        settings_components
            .matches("inherits HubContentPanelSlot")
            .count(),
        1,
        "SettingsPage should route the overview panel through the shared content panel slot"
    );
    assert_eq!(
        settings_components
            .matches("inherits HubFormPanelSlot")
            .count(),
        2,
        "SettingsPage should route Toolchain and Build Defaults through the shared form panel slot"
    );
    assert_eq!(
        settings_components
            .matches("inherits HubListPanelSlot")
            .count(),
        2,
        "SettingsPage should route default paths and configuration health through the shared list panel slot"
    );

    for snippet in [
        "export component SettingsConfigurationOverviewPanel inherits HubContentPanelSlot",
        "export component SettingsToolchainPanel inherits HubFormPanelSlot",
        "export component SettingsBuildDefaultsPanel inherits HubFormPanelSlot",
        "export component SettingsDefaultPathsPanel inherits HubListPanelSlot",
        "export component SettingsConfigurationHealthPanel inherits HubListPanelSlot",
        "title: root.panel-title;",
        "form-spacing: HubTokens.space-2;",
        "private property <length> paths-scroll-y: 0px;",
        "private property <length> health-scroll-y: 0px;",
        "SettingsConfigurationOverviewPanel {",
        "SettingsToolchainPanel {",
        "SettingsBuildDefaultsPanel {",
        "SettingsDefaultPathsPanel {",
        "SettingsConfigurationHealthPanel {",
        "panel-title: \"Configuration Overview\";",
        "panel-title: root.ui-text.toolchain;",
        "panel-title: root.ui-text.build-defaults;",
        "panel-title: root.ui-text.default-paths;",
        "panel-title: root.ui-text.configuration-health;",
        "scroll-y <=> root.paths-scroll-y;",
        "scroll-y <=> root.health-scroll-y;",
        "callback triggered(string);",
        "callback status-action(string);",
        "detail: root.status.detail;",
        "root.status.disabled-reason == \"\" ? root.status.scope",
        "show-arrow: root.status.actionable;",
        "root.triggered(root.status.action-id);",
        "root.status-action(action-id);",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "center-content: true;",
        "collapse-label: root.compact-labels;",
        "SettingsSaveActionRow {",
        "button-width: root.save-button-width;",
        "action-label: root.ui-text.save-settings;",
        "root.save-settings();",
        "status-action(action-id) => {",
        "if (action-id == \"save-settings\")",
    ] {
        assert!(
            settings_surface.contains(snippet),
        "SettingsPage should keep Settings panel shells in typed PanelSlot-backed components: {snippet}"
        );
    }
    for (component, base) in [
        ("SettingsConfigurationOverviewPanel", "HubContentPanelSlot"),
        ("SettingsToolchainPanel", "HubFormPanelSlot"),
        ("SettingsBuildDefaultsPanel", "HubFormPanelSlot"),
        ("SettingsDefaultPathsPanel", "HubListPanelSlot"),
        ("SettingsConfigurationHealthPanel", "HubListPanelSlot"),
    ] {
        assert!(
            settings.contains(&format!("{component} {{"))
                && settings_components
                    .contains(&format!("export component {component} inherits {base}")),
            "SettingsPage should compose the typed {component} wrapper"
        );
    }
    assert!(
        settings_components.contains("export component SettingsSaveActionRow inherits HubFormActionRow")
            && settings_components.contains("action-width: root.button-width;")
            && settings_components.contains("action-icon: @image-url(\"../assets/icons/ui/chevron-right.svg\");")
            && !settings.contains("PillButton {"),
        "SettingsPage should import SettingsSaveActionRow so the footer primary action consumes HubFormActionRow"
    );

    for forbidden in [
        "PanelSlot {",
        "ResponsiveSlot {",
        "inherits HubPanel",
        "component ToolchainPanel",
        "component BuildDefaultsPanel",
        "component DefaultPathsPanel",
        "component ConfigurationHealthPanel",
        "MaterialStyleMetrics.padding_16",
        "PillButton {",
        "PanelHeader {",
        "PanelListViewport {",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "SettingsToolchainField {",
        "SettingsComboChoice {",
        "PathSettingRow {",
        "SettingStatusRow {",
    ] {
        assert!(
            !settings.contains(forbidden),
            "SettingsPage should not keep semantic panel internals after typed Settings panel extraction: {forbidden}"
        );
    }
}
