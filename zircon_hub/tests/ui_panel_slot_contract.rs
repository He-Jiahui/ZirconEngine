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
        "import { ResponsiveSlot } from \"layout.slint\";",
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
fn overview_panel_wraps_cloud_and_team_summary_headers() {
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

    for (page, source, title_snippet, badge_snippet) in [
        (
            "CloudPage",
            &cloud,
            "title: root.overview-title;",
            "badge-text: root.summary.status;",
        ),
        (
            "TeamPage",
            &team,
            "title: root.workspace-title;",
            "badge-text: root.member-count + \" \" + root.ui-text.team-members-found;",
        ),
    ] {
        let top_region = source
            .split("WorkspacePanelSection {")
            .next()
            .expect("Cloud/Team pages should declare their overview before WorkspacePanelSection");
        assert!(
            source.contains("OverviewPanel,"),
            "{page} should import the shared OverviewPanel primitive"
        );
        assert!(
            top_region.contains("OverviewPanel {"),
            "{page} should route the top summary card through OverviewPanel"
        );
        for snippet in ["height: root.header-height;", title_snippet, badge_snippet] {
            assert!(
                top_region.contains(snippet),
                "{page} OverviewPanel should preserve summary header bindings: {snippet}"
            );
        }
        assert!(
            !top_region.contains("HubPanel {")
                && !top_region.contains("padding-left: MaterialStyleMetrics.padding_16;")
                && !top_region.contains("padding-top: MaterialStyleMetrics.padding_14;"),
            "{page} should not repeat top summary HubPanel/VerticalLayout padding boilerplate"
        );
    }
}

#[test]
fn empty_state_primitives_wrap_project_and_team_empty_states() {
    let surfaces = read_ui_file("surfaces.slint");
    let components = read_ui_file("components.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let team = read_ui_file("team.slint");

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

    let project_flow = dashboard
        .split("component ProjectFlow")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDashboardPage").next())
        .expect("project_dashboard.slint must declare ProjectFlow before ProjectDashboardPage");
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

    let member_empty = team
        .split("if root.member-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("for member in root.members").next())
        .expect("team.slint must declare member EmptyStateBlock before member rows");
    for snippet in [
        "title: root.ui-text.no-team-members-found;",
        "detail: root.members-empty-detail;",
        "extra-detail: root.summary.repository-path;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            member_empty.contains(snippet),
            "TeamPage empty state should preserve bindings through EmptyStateBlock: {snippet}"
        );
    }
    assert!(
        team.contains("EmptyStateBlock,")
            && !member_empty.contains("MaterialText {")
            && !member_empty.contains("MutedText {")
            && !member_empty.contains("HubPanel {"),
        "TeamPage should consume the shared in-panel EmptyStateBlock instead of local text/panel boilerplate"
    );
}

#[test]
fn builds_page_uses_panel_slot_for_workspace_panels() {
    let builds = read_ui_file("builds.slint");

    assert!(
        builds.contains("PanelSlot,"),
        "BuildsPage should import the shared PanelSlot primitive"
    );
    assert_eq!(
        builds.matches("PanelSlot {").count(),
        4,
        "BuildsPage should route its four workspace panels through PanelSlot"
    );
    for forbidden in [
        "ResponsiveSlot {",
        "HubPanel {",
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

    assert!(
        editor.contains("PanelSlot,"),
        "EditorPage should import the shared PanelSlot primitive"
    );
    assert_eq!(
        editor.matches("PanelSlot {").count(),
        3,
        "EditorPage should route standard overview/actions/config panels through PanelSlot"
    );
    assert_eq!(
        editor.matches("ResponsiveSlot {").count(),
        1,
        "EditorPage should keep ResponsiveSlot only for the split side slot that contains two nested panels"
    );
    assert!(
        editor_components.contains("export component EditorSideListPanel inherits HubPanel"),
        "EditorPage should consolidate the split side-list HubPanel shell in an exported page component"
    );
    assert_eq!(
        editor_components
            .matches("export component EditorSideListPanel inherits HubPanel")
            .count(),
        1,
        "editor_page_components.slint should own one side-list panel shell instead of repeating HubPanel list boilerplate"
    );
    assert!(
        !editor.contains("component EditorSideListPanel"),
        "editor.slint should import EditorSideListPanel instead of defining it inline"
    );
    assert_eq!(
        editor.matches("EditorSideListPanel {").count(),
        2,
        "EditorPage split side slot should use EditorSideListPanel for Source Engines and Build History"
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
        editor.matches("EditorActionRow {").count(),
        3,
        "EditorPage should route Save Source, Build, and Open Output actions through EditorActionRow"
    );
    assert_eq!(
        editor.matches("\n                ActionRow {").count(),
        0,
        "EditorPage should keep ActionRow construction inside EditorActionRow instead of repeating it in the actions panel"
    );

    for snippet in [
        "body-spacing: HubTokens.space-3;",
        "title: root.source-engine.title;",
        "title: root.ui-text.editor-actions;",
        "title: root.ui-text.source-engine;",
        "panel-title: root.ui-text.source-engines;",
        "panel-title: root.ui-text.build-history;",
        "badge-text: root.source-engine-count + \"\";",
        "badge-text: root.source-build-history-count + \"\";",
        "action-id: \"save-source\";",
        "action-icon: @image-url(\"../assets/icons/ui/settings.svg\");",
        "action-title: root.ui-text.save-source;",
        "action-detail: root.ui-text.source-checkout-path;",
        "root.save-settings();",
        "action-id: \"build\";",
        "action-icon: @image-url(\"../assets/icons/actions/build-project.svg\");",
        "action-title: root.ui-text.build;",
        "action-detail: root.ui-text.compile-editor;",
        "root.build-engine();",
        "action-id: \"open-output\";",
        "action-icon: @image-url(\"../assets/icons/ui/folder.svg\");",
        "action-title: root.ui-text.open-output;",
        "action-detail: root.source-engine.output-path;",
        "root.open-output();",
    ] {
        assert!(
            editor.contains(snippet),
            "EditorPage PanelSlot migration should preserve panel content and spacing: {snippet}"
        );
    }
}

#[test]
fn project_dashboard_lower_panels_use_panel_slot() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let lower_grid = dashboard
        .split("PanelGrid {")
        .nth(1)
        .expect("ProjectDashboardPage must keep a lower PanelGrid");

    assert!(
        dashboard.contains("PanelSlot,"),
        "ProjectDashboardPage should import the shared PanelSlot primitive"
    );
    assert_eq!(
        lower_grid.matches("PanelSlot {").count(),
        2,
        "ProjectDashboardPage lower Recent/Quick panels should route through PanelSlot"
    );
    assert_eq!(
        lower_grid.matches("ResponsiveSlot {").count(),
        0,
        "ProjectDashboardPage lower PanelGrid should leave ResponsiveSlot boilerplate to PanelSlot"
    );
    for snippet in [
        "component DashboardQuickActionRow inherits ActionRow",
        "title: root.dashboard-project-title;",
        "action-text: root.ui-text.view-all-projects;",
        "DataTable {",
        "title: root.ui-text.quick-actions;",
        "action-list := PanelListViewport {",
        "scroll-y <=> root.quick-actions-scroll-y;",
        "for action in root.quick-actions: DashboardQuickActionRow {",
        "quick-action-data: action;",
        "triggered(id) => { root.quick-action(id); }",
        "if root.quick-action-count == 0: EmptyStateBlock {",
        "title: root.ui-text.no-quick-actions;",
        "detail: root.ui-text.quick-actions-empty-detail;",
    ] {
        assert!(
            lower_grid.contains(snippet) || dashboard.contains(snippet),
            "ProjectDashboardPage lower PanelSlot migration should preserve panel content: {snippet}"
        );
    }
    assert_eq!(
        dashboard
            .matches("component DashboardQuickActionRow inherits ActionRow")
            .count(),
        1,
        "ProjectDashboardPage should own one local Quick Actions row wrapper"
    );
    assert_eq!(
        lower_grid.matches("DashboardQuickActionRow {").count(),
        1,
        "Quick Actions should route repeated action rows through DashboardQuickActionRow"
    );
    for forbidden in [
        "HubPanel {\n                            width: parent.width;",
        "VerticalLayout {\n                                width: parent.width;",
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
    let team = read_ui_file("team.slint");

    for (page, source, height_snippet, scroll_snippet, title_snippet) in [
        (
            "CloudPage",
            &cloud,
            "height: root.services-panel-height;",
            "scroll-y <=> root.service-scroll-y;",
            "title: root.ui-text.cloud-services;",
        ),
        (
            "TeamPage",
            &team,
            "height: root.members-panel-height;",
            "scroll-y <=> root.member-scroll-y;",
            "title: root.ui-text.team-members-found;",
        ),
    ] {
        assert!(
            source.contains("PanelSlot,"),
            "{page} should import the shared PanelSlot primitive"
        );
        assert_eq!(
            source.matches("PanelSlot {").count(),
            1,
            "{page} should route its lower typed list panel through PanelSlot"
        );
        for snippet in [
            "horizontal-stretch: 1;",
            height_snippet,
            "body-padding: MaterialStyleMetrics.padding_16;",
            "body-spacing: HubTokens.toolbar-gap;",
            title_snippet,
            "PanelListViewport {",
            scroll_snippet,
        ] {
            assert!(
                source.contains(snippet),
                "{page} lower PanelSlot should preserve list panel sizing and content: {snippet}"
            );
        }
    }

    for forbidden in [
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
        team.contains("for member in root.members: TeamMemberRow {")
            && team.contains("row-height: root.member-row-height;")
            && team.contains("member: member;")
            && team.contains("ui-text: root.ui-text;")
            && team.contains("collapse-label: root.content-width < HubTokens.breakpoint-medium;"),
        "TeamPage member list should repeat TeamMemberRow directly in PanelListViewport while preserving row height, data, copy, and compact trailing-label bindings"
    );
}

#[test]
fn project_secondary_pages_use_panel_slot_for_standard_new_and_detail_panels() {
    let project_pages = read_ui_file("project_pages.slint");
    let new_page = project_pages
        .split("export component ProjectNewPage")
        .nth(1)
        .and_then(|source| source.split("export component ProjectBrowserPage").next())
        .expect("project_pages.slint must declare ProjectNewPage before ProjectBrowserPage");
    let browser_page = project_pages
        .split("export component ProjectBrowserPage")
        .nth(1)
        .and_then(|source| source.split("export component ProjectDetailPage").next())
        .expect("project_pages.slint must declare ProjectBrowserPage before ProjectDetailPage");
    let detail_page = project_pages
        .split("export component ProjectDetailPage")
        .nth(1)
        .expect("project_pages.slint must declare ProjectDetailPage");

    assert!(
        project_pages.contains("PanelSlot,"),
        "Projects secondary pages should import the shared PanelSlot primitive"
    );
    assert_eq!(
        new_page.matches("PanelSlot {").count(),
        3,
        "ProjectNewPage should route project settings, compact summary, and template rail through PanelSlot"
    );
    assert_eq!(
        detail_page.matches("PanelSlot {").count(),
        2,
        "ProjectDetailPage should route detail and actions columns through PanelSlot"
    );
    assert_eq!(
        browser_page.matches("PanelSlot {").count(),
        0,
        "ProjectBrowserPage should keep its explicit list panel because it owns a custom inner scroll viewport"
    );

    for snippet in [
        "body-spacing: root.page-gap;",
        "title: root.ui-text.project-settings-title;",
        "title: root.ui-text.templates-title;",
        "ProjectCreateActionRow {",
        "row-height: root.create-action-row-height;",
        "height: root.detail-main-height;",
        "height: root.detail-action-height;",
        "ProjectDetailStatusStrip {",
        "row-height: root.status-row-height;",
        "ProjectDetailInfoSection {",
        "section-height: root.detail-info-section-height;",
        "title: root.ui-text.project-actions-title;",
        "detail-engine-scroll-y: 0px;",
        "ProjectEngineChoiceList {",
        "list-scroll-y <=> root.detail-engine-scroll-y;",
        "ProjectDetailEngineSection {",
        "section-height: root.detail-engine-section-height;",
    ] {
        assert!(
            new_page.contains(snippet) || detail_page.contains(snippet),
            "Projects PanelSlot migration should preserve panel content and sizing: {snippet}"
        );
    }

    for forbidden in [
        "HubPanel {",
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

    assert!(
        settings.contains("PanelSlot,"),
        "SettingsPage should import the shared PanelSlot primitive"
    );
    assert_eq!(
        settings.matches("PanelSlot {").count(),
        4,
        "SettingsPage should route its four semantic panels through PanelSlot"
    );

    for snippet in [
        "PanelSlot {\n            basis: HubTokens.panel-min-md + HubTokens.control-sm;",
        "PanelSlot {\n            basis: HubTokens.panel-min-lg + HubTokens.control-lg;",
        "private property <length> paths-scroll-y: 0px;",
        "private property <length> health-scroll-y: 0px;",
        "title: root.ui-text.toolchain;",
        "title: root.ui-text.build-defaults;",
        "title: root.ui-text.default-paths;",
        "title: root.ui-text.configuration-health;",
        "scroll-y <=> root.paths-scroll-y;",
        "scroll-y <=> root.health-scroll-y;",
        "if root.settings-status-count == 0: EmptyStateBlock",
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "center-content: true;",
        "collapse-label: root.compact-labels;",
    ] {
        assert!(
            settings.contains(snippet),
            "SettingsPage should keep PanelSlot as the surface shell and inline semantic panel content: {snippet}"
        );
    }

    for forbidden in [
        "ResponsiveSlot {",
        "inherits HubPanel",
        "inherits PanelSlot",
        "component ToolchainPanel",
        "component BuildDefaultsPanel",
        "component DefaultPathsPanel",
        "component ConfigurationHealthPanel",
        "MaterialStyleMetrics.padding_16",
    ] {
        assert!(
            !settings.contains(forbidden),
            "SettingsPage should not keep the old ResponsiveSlot/HubPanel body wrapper after PanelSlot migration: {forbidden}"
        );
    }
}
