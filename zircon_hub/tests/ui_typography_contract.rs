//! Static contracts for Zircon Hub Material typography usage.

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
fn shared_typography_wrappers_use_material_text() {
    let shared = read_ui_file("shared.slint");
    for snippet in [
        "MaterialText,",
        "export component FieldLabel inherits MaterialText",
        "style: MaterialTypography.label_large;",
        "export component MutedText inherits MaterialText",
        "style: MaterialTypography.body_small;",
    ] {
        assert!(
            shared.contains(snippet),
            "shared typography wrappers should delegate text metrics to MaterialText; missing {snippet}"
        );
    }

    let field_label = shared
        .split("export component FieldLabel")
        .nth(1)
        .and_then(|source| source.split("export component MutedText").next())
        .expect("shared.slint must declare FieldLabel before MutedText");
    let muted_text = shared
        .split("export component MutedText")
        .nth(1)
        .expect("shared.slint must declare MutedText");
    for forbidden in [
        "inherits Text",
        "font-size:",
        "font-weight:",
        "font_size:",
        "font_weight:",
    ] {
        assert!(
            !field_label.contains(forbidden) && !muted_text.contains(forbidden),
            "shared typography wrappers should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn builds_current_task_status_uses_material_text() {
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");

    for snippet in [
        "HubSection,",
        "HubSection {",
        "section-height: root.current-task-section-height;",
        "section-height: root.history-section-height;",
        "title: root.current-task-title;",
        "title: root.build-history-title;",
        "MaterialText,",
        "MaterialText {",
        "text: root.status-label;",
        "style: MaterialTypography.headline_small;",
    ] {
        assert!(
            builds_components.contains(snippet),
            "BuildTaskHistoryPanel current task status should delegate typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        builds.contains("BuildTaskHistoryPanel {")
            && !builds.contains("MaterialText {")
            && !builds.contains("MutedText {"),
        "BuildsPage should compose BuildTaskHistoryPanel instead of owning current-task text nodes"
    );
    assert!(
        !builds_components
            .lines()
            .any(|line| line.trim() == "Text {"),
        "BuildTaskHistoryPanel should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !builds.contains(forbidden) && !builds_components.contains(forbidden),
            "BuildsPage and BuildTaskHistoryPanel should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn editor_empty_states_use_shared_material_text_block() {
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let source_empty = editor_components
        .split("if root.source-engine-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| {
            source
                .split("if root.source-build-history-count == 0")
                .next()
        })
        .expect(
            "editor_page_components.slint must declare source-engine EmptyStateBlock before build-history block",
        );
    let history_empty = editor_components
        .split("if root.source-build-history-count == 0: EmptyStateBlock")
        .nth(1)
        .expect("editor_page_components.slint must declare source build-history EmptyStateBlock");
    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");

    for snippet in [
        "height: root.empty-height;",
        "title: root.empty-title;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            source_empty.contains(snippet),
            "Editor source-engine empty state should delegate content bindings to EmptyStateBlock; missing {snippet}"
        );
    }
    for snippet in [
        "height: root.empty-height;",
        "title: root.empty-title;",
        "body-padding: MaterialStyleMetrics.padding_16;",
        "center-content: true;",
    ] {
        assert!(
            history_empty.contains(snippet),
            "Editor build-history empty state should delegate content bindings to EmptyStateBlock; missing {snippet}"
        );
    }
    for snippet in ["MaterialText {", "text: root.title;"] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should own Material text for Editor empty states; missing {snippet}"
        );
    }
    assert!(
        !source_empty.lines().any(|line| line.trim() == "Text {")
            && !history_empty.lines().any(|line| line.trim() == "Text {"),
        "Editor empty states should not return to raw Text nodes"
    );
    assert!(
        editor.contains("EditorSourceEngineListPanel {")
            && editor.contains("EditorBuildHistoryPanel {")
            && !editor.contains("if root.source-engine-count == 0: EmptyStateBlock")
            && !editor.contains("if root.source-build-history-count == 0: EmptyStateBlock"),
        "editor.slint should compose typed side-list panels instead of owning empty-state text blocks inline"
    );
}

#[test]
fn cloud_and_team_workspace_typography_uses_material_text() {
    let data_display = read_ui_file("data_display.slint");
    let cloud = read_ui_file("cloud.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");
    let cloud_surface = format!("{cloud}\n{cloud_components}");
    let team = read_ui_file("team.slint");
    let team_components = read_ui_file("team_page_components.slint");
    let team_surface = format!("{team}\n{team_components}");

    let metric_card = data_display
        .split("export component MetricCard")
        .nth(1)
        .and_then(|source| source.split("export component BuildHistoryRow").next())
        .expect("data_display.slint must declare MetricCard before BuildHistoryRow");
    for snippet in [
        "MaterialText {",
        "text: root.primary;",
        "style: MaterialTypography.title_small;",
    ] {
        assert!(
            metric_card.contains(snippet),
            "MetricCard should delegate primary metric typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !metric_card.lines().any(|line| line.trim() == "Text {"),
        "MetricCard should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !metric_card.contains(forbidden),
            "MetricCard should not return to raw Text font bindings: {forbidden}"
        );
    }
    for (page_name, source) in [("CloudPage", &cloud_surface), ("TeamPage", &team_surface)] {
        assert!(
            source.contains("MetricCard {"),
            "{page_name} should reuse the shared data-display MetricCard for summary metrics"
        );
        assert!(
            source.contains("HubListPanelSlot,"),
            "{page_name} should route its lower row list through the shared HubListPanelSlot shell"
        );
    }
    for forbidden in [
        "component CloudMetric inherits",
        "component TeamSummaryItem",
        "component TeamSummaryCard",
    ] {
        assert!(
            !cloud_surface.contains(forbidden) && !team_surface.contains(forbidden),
            "Cloud/Team should not keep page-local metric card components after MetricCard extraction: {forbidden}"
        );
    }
    assert!(
        !cloud.contains("component CloudMetricSlot")
            && !cloud.contains("component CloudServiceRow")
            && !cloud.contains("component CloudServicesPanel")
            && !team.contains("component TeamSummarySlot")
            && !team.contains("component TeamMemberRow")
            && !team.contains("component TeamMembersPanel"),
        "Cloud/Team pages should import page-specific panel/row wrappers from component modules instead of defining them inline"
    );

    let surfaces = read_ui_file("surfaces.slint");
    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");
    let team_empty = team_components
        .split("if root.member-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("for member in root.members").next())
        .expect("team_page_components.slint must declare member empty state before member rows");
    for snippet in [
        "title: root.ui-text.no-team-members-found;",
        "detail: root.members-empty-detail;",
        "extra-detail: root.repository-path;",
        "body-padding: MaterialStyleMetrics.padding_16;",
    ] {
        assert!(
            team_empty.contains(snippet),
            "Team empty state should delegate content bindings to EmptyStateBlock; missing {snippet}"
        );
    }
    assert!(
        team.contains("TeamMembersPanel {")
            && !team.contains("if root.member-count == 0: EmptyStateBlock")
            && !team.contains("for member in root.members: TeamMemberRow {"),
        "team.slint should compose TeamMembersPanel instead of owning member empty-state or row repetition inline"
    );
    for snippet in [
        "MaterialText {",
        "style: root.title-prominent ? MaterialTypography.title_medium : MaterialTypography.label_large;",
    ] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should delegate shared empty-state typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !team_empty.lines().any(|line| line.trim() == "Text {")
            && !empty_block.lines().any(|line| line.trim() == "Text {"),
        "Team empty state should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !team_empty.contains(forbidden) && !empty_block.contains(forbidden),
            "Team empty state should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn dashboard_project_card_and_empty_titles_use_material_text() {
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    let surfaces = read_ui_file("surfaces.slint");
    let project_card = dashboard_components
        .split("export component ProjectCard")
        .nth(1)
        .and_then(|source| source.split("export component ProjectFlow").next())
        .expect("project_dashboard_components.slint must export ProjectCard before ProjectFlow");
    for snippet in [
        "MaterialText,",
        "MaterialText {",
        "text: root.project.title;",
        "style: MaterialTypography.title_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            dashboard_surface.contains(snippet) || project_card.contains(snippet),
            "ProjectCard title should delegate typography to MaterialText; missing {snippet}"
        );
    }
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_card.contains(forbidden),
            "ProjectCard title should not return to raw Text font bindings: {forbidden}"
        );
    }

    let project_flow = dashboard_components
        .split("export component ProjectFlow")
        .nth(1)
        .and_then(|source| {
            source
                .split("export component DashboardQuickActionRow")
                .next()
        })
        .expect(
            "project_dashboard_components.slint must export ProjectFlow before DashboardQuickActionRow",
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
            "ProjectFlow empty state should delegate layout and typography to EmptyStatePanel; missing {snippet}"
        );
    }

    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");
    let empty_panel = surfaces
        .split("export component EmptyStatePanel")
        .nth(1)
        .and_then(|source| source.split("export component StatusBanner").next())
        .expect("surfaces.slint must declare EmptyStatePanel before StatusBanner");
    for snippet in ["inherits HubPanel", "EmptyStateBlock {"] {
        assert!(
            empty_panel.contains(snippet),
            "EmptyStatePanel should wrap EmptyStateBlock in a HubPanel shell; missing {snippet}"
        );
    }
    for snippet in [
        "MaterialText {",
        "text: root.title;",
        "style: root.title-prominent ? MaterialTypography.title_medium : MaterialTypography.label_large;",
        "if root.detail != \"\": MutedText",
    ] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should own MaterialText/MutedText empty-state typography; missing {snippet}"
        );
    }
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_flow.contains(forbidden)
                && !empty_panel.contains(forbidden)
                && !empty_block.contains(forbidden),
            "ProjectFlow empty state title should not return to raw Text font bindings: {forbidden}"
        );
    }
}

#[test]
fn project_workflow_typography_uses_material_text() {
    let components = read_ui_file("project_page_components.slint");
    let browser_components = read_ui_file("project_browser_components.slint");
    let detail_components = read_ui_file("project_detail_components.slint");
    let new_page = read_ui_file("project_new_page.slint");
    let browser_page = read_ui_file("project_browser_page.slint");
    let detail_page = read_ui_file("project_detail_page.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let project_text_surface = format!(
        "{components}\n{browser_components}\n{detail_components}\n{new_page}\n{browser_page}\n{detail_page}"
    );

    for snippet in [
        "MaterialText,",
        "style: MaterialTypography.title_large;",
        "style: MaterialTypography.label_medium;",
        "style: MaterialTypography.label_large;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            project_text_surface.contains(snippet),
            "Project workflow shared components should delegate typography to MaterialText; missing {snippet}"
        );
    }

    for component_name in [
        "PageHeader",
        "ProjectSettingSummaryRow",
        "ProjectCreateSettingsPanel",
        "ProjectCreateCompactSummaryPanel",
        "ProjectDetailStatusStrip",
        "ProjectDetailPinToggleRow",
        "ProjectDetailInfoSection",
        "ProjectDetailEngineSection",
        "ProjectBrowserRow",
    ] {
        let component_source = if component_name == "ProjectBrowserRow" {
            &browser_components
        } else if component_name.starts_with("ProjectDetail") {
            &detail_components
        } else {
            &components
        };
        let component = component_source
            .split(&format!("export component {component_name}"))
            .nth(1)
            .and_then(|source| {
                if component_name == "ProjectDetailStatusStrip" {
                    source
                        .split("export component ProjectDetailPinToggleRow")
                        .next()
                } else if component_name == "ProjectDetailPinToggleRow" {
                    source
                        .split("export component ProjectDetailInfoSection")
                        .next()
                } else if component_name == "ProjectDetailInfoSection" {
                    source
                        .split("export component ProjectDetailEngineSection")
                        .next()
                } else if component_name == "ProjectDetailEngineSection" {
                    source.split("export component ").next()
                } else {
                    source.split("export component ").next()
                }
            })
            .unwrap_or_else(|| panic!("Project workflow UI must declare {component_name}"));
        if component_name == "ProjectCreateCompactSummaryPanel" {
            assert!(
                component.contains("ProjectCreateSummary {"),
                "{component_name} should delegate visible text to MaterialText-backed project summary components"
            );
        } else if component_name == "ProjectCreateSettingsPanel" {
            assert!(
                component.contains("PanelHeader {")
                    && (component.contains("ProjectSettingSummaryRow {")
                        || component.contains("ProjectEngineChoiceList {")),
                "{component_name} should delegate visible text to MaterialText-backed project workflow components"
            );
        } else if component_name == "ProjectDetailInfoSection"
            || component_name == "ProjectDetailEngineSection"
        {
            assert!(
                component.contains("inherits HubSection")
                    && (component.contains("ProjectSettingSummaryRow {")
                        || component.contains("ProjectEngineChoiceList {")),
                "{component_name} should delegate section text through the MaterialText-backed HubSection primitive"
            );
        } else if component_name == "ProjectDetailPinToggleRow" {
            assert!(
                component.contains("inherits HubToggleRow")
                    && component.contains("label: root.detail.pinned")
                    && component.contains("supporting-text: root.detail.pinned"),
                "{component_name} should delegate visible text to the MaterialText-backed HubToggleRow primitive"
            );
        } else {
            assert!(
                component.contains("MaterialText {"),
                "{component_name} should use MaterialText for visible text"
            );
        }
        assert!(
            !component.lines().any(|line| line.trim() == "Text {"),
            "{component_name} should not return to raw Text nodes"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !component.contains(forbidden),
                "{component_name} should not return to raw Text font bindings: {forbidden}"
            );
        }
    }

    for snippet in [
        "MaterialText,",
        "text: root.ui-text.source-engine;",
        "text: root.copy.modified-prefix + root.detail.modified;",
        "style: MaterialTypography.body_small;",
        "vertical_alignment: center;",
    ] {
        assert!(
            components.contains(snippet)
                || detail_components.contains(snippet)
                || new_page.contains(snippet)
                || detail_page.contains(snippet)
                || project_pages.contains(snippet)
                || browser_page.contains(snippet),
            "Project workflow pages should use MaterialText for section/status labels; missing {snippet}"
        );
    }
    let browser_results_panel = browser_components
        .split("export component ProjectBrowserResultsPanel")
        .nth(1)
        .expect("project_browser_components.slint must declare ProjectBrowserResultsPanel");
    let browser_empty = browser_results_panel
        .split("if root.row-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("}").next())
        .expect(
            "ProjectBrowserResultsPanel must declare an EmptyStateBlock for empty browser results",
        );
    let engine_choice_list = components
        .split("export component ProjectEngineChoiceList")
        .nth(1)
        .expect("project_page_components.slint must declare ProjectEngineChoiceList");
    let engine_choice_empty = engine_choice_list
        .split("if root.engine-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("for engine in root.engines").next())
        .expect(
            "ProjectEngineChoiceList must declare an EmptyStateBlock for missing source engines",
        );
    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");
    for snippet in [
        "height: HubTokens.list-row-lg;",
        "title: root.ui-text.no-projects-match;",
        "body-padding: HubTokens.space-4;",
        "center-content: true;",
    ] {
        assert!(
            browser_empty.contains(snippet),
            "Project Browser empty state should delegate content bindings to EmptyStateBlock; missing {snippet}"
        );
    }
    for snippet in [
        "height: root.list-height;",
        "title: root.empty-title;",
        "body-padding: HubTokens.space-4;",
        "center-content: true;",
    ] {
        assert!(
            engine_choice_empty.contains(snippet),
            "Project source-engine empty state should delegate content bindings to EmptyStateBlock through ProjectEngineChoiceList; missing {snippet}"
        );
    }
    assert!(
        components.contains("empty-title: root.ui-text.register-source-engine-before-create;")
            && new_page.contains("ui-text: root.ui-text;")
            && detail_components.contains("empty-title: root.copy.no-source-engine-available;")
            && detail_page.contains("copy: root.ui-text;"),
        "ProjectNewPage and ProjectDetailEngineSection should provide their own Source Engine empty-state copy to ProjectEngineChoiceList"
    );
    for snippet in ["MaterialText {", "text: root.title;"] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should own MaterialText typography for Project workflow empty states; missing {snippet}"
        );
    }
    assert!(
        !browser_empty.contains("MutedText {")
            && !engine_choice_empty.contains("MutedText {")
            && !browser_empty.lines().any(|line| line.trim() == "Text {"),
        "Project workflow empty states should not return to local MutedText/raw Text nodes"
    );
    assert!(
        browser_page.contains("ProjectBrowserResultsPanel {")
            && !browser_page.contains("EmptyStateBlock {")
            && !browser_page.contains("PanelHeader {"),
        "ProjectBrowserPage should compose the typed results panel instead of owning browser result text"
    );
    assert!(
        !project_pages.lines().any(|line| line.trim() == "Text {")
            && !new_page.lines().any(|line| line.trim() == "Text {")
            && !browser_page.lines().any(|line| line.trim() == "Text {")
            && !detail_page.lines().any(|line| line.trim() == "Text {"),
        "Project workflow page modules should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !project_pages.contains(forbidden)
                && !new_page.contains(forbidden)
                && !browser_page.contains(forbidden)
                && !detail_page.contains(forbidden),
            "Project workflow page modules should not return to raw Text font bindings: {forbidden}"
        );
    }
    assert!(
        project_pages.contains("export { ProjectNewPage } from \"project_new_page.slint\";")
            && project_pages
                .contains("export { ProjectBrowserPage } from \"project_browser_page.slint\";")
            && project_pages
                .contains("export { ProjectDetailPage } from \"project_detail_page.slint\";")
            && !project_pages.contains("export component ProjectDetailPage inherits"),
        "project_pages.slint should keep New, Browser, and Detail pages available through dedicated page modules"
    );
}

#[test]
fn settings_health_empty_state_uses_material_text() {
    let settings_components = read_ui_file("settings_page_components.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let health_empty = settings_components
        .split("if root.settings-status-count == 0: EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("for status in root.settings-statuses").next())
        .expect(
            "settings_page_components.slint must declare configuration health EmptyStateBlock before status rows",
        );
    let empty_block = surfaces
        .split("export component EmptyStateBlock")
        .nth(1)
        .and_then(|source| source.split("export component EmptyStatePanel").next())
        .expect("surfaces.slint must declare EmptyStateBlock before EmptyStatePanel");

    for snippet in [
        "title: root.ui-text.no-configuration-checks;",
        "detail: root.ui-text.configuration-health-empty-detail;",
        "body-padding: HubTokens.space-4;",
        "center-content: true;",
    ] {
        assert!(
            health_empty.contains(snippet),
            "Settings health empty state should delegate content bindings to EmptyStateBlock; missing {snippet}"
        );
    }
    for snippet in [
        "MaterialText {",
        "style: root.title-prominent ? MaterialTypography.title_medium : MaterialTypography.label_large;",
    ] {
        assert!(
            empty_block.contains(snippet),
            "EmptyStateBlock should delegate shared empty-state typography to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !health_empty.lines().any(|line| line.trim() == "Text {")
            && !empty_block.lines().any(|line| line.trim() == "Text {"),
        "Settings health empty state should not return to raw Text nodes"
    );
    for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
        assert!(
            !health_empty.contains(forbidden) && !empty_block.contains(forbidden),
            "Settings health empty state should not return to raw Text font bindings: {forbidden}"
        );
    }
}
