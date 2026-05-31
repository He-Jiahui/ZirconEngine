//! Static contracts that real Hub pages use Material/Taffy wrappers instead of sample surfaces.

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
fn material_and_taffy_coverage_uses_real_hub_surfaces() {
    assert!(
        !ui_dir().join("placeholder.slint").exists(),
        "Hub user-facing routes should use real page implementations, not a retained PlaceholderPage file"
    );

    let components = read_ui_file("components.slint");
    let data_display = read_ui_file("data_display.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let data_surface = format!("{data_display}\n{table_view}");
    let layout = read_ui_file("layout.slint");
    let inputs = read_ui_file("inputs.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let shared = read_ui_file("shared.slint");
    let material_bridge = read_ui_file("material_bridge.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let dashboard_surface = format!("{dashboard}\n{dashboard_components}");
    let project_pages = read_ui_file("project_pages.slint");
    let project_new_page = read_ui_file("project_new_page.slint");
    let project_browser_page = read_ui_file("project_browser_page.slint");
    let project_detail_page = read_ui_file("project_detail_page.slint");
    let project_components = read_ui_file("project_page_components.slint");
    let project_browser_components = read_ui_file("project_browser_components.slint");
    let project_detail_components = read_ui_file("project_detail_components.slint");
    let project_surface = format!(
        "{project_pages}\n{project_new_page}\n{project_browser_page}\n{project_detail_page}\n{project_components}\n{project_browser_components}\n{project_detail_components}"
    );
    let editor = read_ui_file("editor.slint");
    let editor_components = read_ui_file("editor_page_components.slint");
    let editor_surface = format!("{editor}\n{editor_components}");
    let builds = read_ui_file("builds.slint");
    let builds_components = read_ui_file("builds_page_components.slint");
    let builds_surface = format!("{builds}\n{builds_components}");
    let settings = read_ui_file("settings.slint");
    let settings_components = read_ui_file("settings_page_components.slint");
    let settings_surface = format!("{settings}\n{settings_components}");
    let cloud = read_ui_file("cloud.slint");
    let cloud_components = read_ui_file("cloud_page_components.slint");
    let cloud_surface = format!("{cloud}\n{cloud_components}");
    let team = read_ui_file("team.slint");
    let team_components = read_ui_file("team_page_components.slint");
    let team_surface = format!("{team}\n{team_components}");
    let catalog_components = read_ui_file("catalog_page_components.slint");
    let assets = read_ui_file("assets.slint");
    let assets_surface = format!("{assets}\n{catalog_components}");
    let plugins = read_ui_file("plugins.slint");
    let plugins_surface = format!("{plugins}\n{catalog_components}");
    let learn = read_ui_file("learn.slint");
    let learn_surface = format!("{learn}\n{catalog_components}");

    for (name, source) in [
        ("components.slint", &components),
        ("data_display.slint", &data_display),
        ("table_view_components.slint", &table_view),
    ] {
        for removed_sample in ["ButtonStates", "Button States", "ComponentSamples"] {
            assert!(
                !source.contains(removed_sample),
                "{name} should not reintroduce the removed development sample surface: {removed_sample}"
            );
        }
    }

    for (name, source) in [
        ("project_dashboard.slint", &dashboard),
        ("project_pages.slint", &project_pages),
        ("project_new_page.slint", &project_new_page),
        ("project_browser_page.slint", &project_browser_page),
        ("project_detail_page.slint", &project_detail_page),
    ] {
        assert!(
            !source.contains("ComponentSamples"),
            "{name} must not expose the internal ComponentSamples surface in user-facing Hub pages"
        );
    }

    for snippet in [
        "export component Flow",
        "export component PanelGrid",
        "export component WorkspacePanelSection",
        "export component ResponsiveSlot",
        "export component ResponsiveCollapse",
    ] {
        assert!(
            layout.contains(snippet),
            "layout.slint must expose the Taffy primitive used by real Hub pages: {snippet}"
        );
    }

    for snippet in [
        "export component SegmentButton",
        "material-segment := SegmentedButton",
        "export component ToolbarSelect",
        "trigger := OutlineButton",
        "menu := HubPopupMenu",
        "export component HubTextField",
        "material-field := TextField",
        "export component SearchBox",
        "search-field := TextInput",
        "border-radius: HubVisualSpec.compact-radius;",
        "out property <bool> focused: search-field.has-focus;",
        "private property <color> state-border:",
        "border-color: root.state-border;",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep the Hub wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
        "root.variant == \"selected\" ? HubVisualSpec.panel-hover-background : HubVisualSpec.panel-background",
        "if root.show-action: OutlineButton",
        "export component OverviewPanel inherits HubPanel",
        "export component EmptyStateBlock inherits Rectangle",
        "export component EmptyStatePanel inherits HubPanel",
        "MaterialText {",
    ] {
        assert!(
            surfaces.contains(snippet),
            "surfaces.slint must keep cards/actions/text on Material primitives: {snippet}"
        );
    }

    for snippet in [
        "FilledButton,",
        "FilledIconButton,",
        "OutlineButton,",
        "OutlineIconButton,",
        "if root.primary &&",
        "export component IconButton",
        "StateLayerArea {",
    ] {
        assert!(
            shared.contains(snippet),
            "shared.slint must keep public Hub button APIs wired to Material buttons: {snippet}"
        );
    }

    for snippet in [
        "OutlinedCard",
        "TextField",
        "FilledButton",
        "OutlineButton",
        "FilledIconButton",
        "OutlineIconButton",
        "Vertical",
    ] {
        assert!(
            material_bridge.contains(snippet) && components.contains(snippet),
            "material_bridge.slint and components.slint must re-export Material primitive {snippet}"
        );
    }

    for snippet in [
        "CatalogPage",
        "PanelListViewport",
        "InfoRow",
        "ActionRow",
        "MetricCard",
        "BuildHistoryRow",
        "HubTableView",
        "HubTableBody",
        "EmptyStateBlock",
        "ListTile",
        "ScrollView",
    ] {
        assert!(
            data_surface.contains(snippet),
            "data-display and table-view modules must keep real list/table surfaces backed by Material wrappers: {snippet}"
        );
    }

    for (page, source, snippets) in [
        (
            "project_dashboard.slint",
            &dashboard_surface,
            &[
                "Flow",
                "PanelGrid",
                "HubTableView",
                "ResponsiveSlot",
                "SearchBox",
                "ProjectFilterSelect",
                "ProjectSortSelect",
                "HubListPanelSlot",
                "ActionRow",
                "EmptyStateBlock",
                "EmptyStatePanel",
            ][..],
        ),
        (
            "project_pages.slint",
            &project_surface,
            &[
                "PanelSlot",
                "ResponsiveSlot",
                "SearchBox",
                "ProjectFilterSelect",
                "ProjectSortSelect",
                "HubTextField",
            ][..],
        ),
        (
            "project_new_page.slint",
            &project_surface,
            &[
                "PageScrollSurface",
                "PanelSlot",
                "ProjectCreateSettingsPanel",
                "ProjectCreateCompactSummaryPanel",
                "ProjectCreateField",
                "ProjectCreateActionRow",
                "ProjectCreateSummary",
                "ProjectEngineChoiceList",
                "ProjectTemplateRailPanel",
                "TemplateChoiceRow",
                "HubCheckBox",
                "HubListPanelSlot",
                "PanelListViewport",
            ][..],
        ),
        (
            "project_browser_page.slint",
            &project_surface,
            &[
                "PageScrollSurface",
                "ResponsiveSlot",
                "SearchBox",
                "ProjectFilterSelect",
                "ProjectSortSelect",
                "ProjectBrowserResultsPanel",
                "ProjectBrowserTableHeader",
                "ProjectBrowserRow",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "project_detail_page.slint",
            &project_surface,
            &[
                "PageScrollSurface",
                "PanelSlot",
                "ProjectDetailStatusStrip",
                "ProjectDetailInfoSection",
                "ProjectDetailActionButton",
                "ProjectDetailPinToggleRow",
                "ProjectDetailEngineSection",
                "StatusBanner",
            ][..],
        ),
        (
            "editor.slint",
            &editor_surface,
            &[
                "WorkspacePanelSection",
                "PanelSlot",
                "ResponsiveSlot",
                "HubTextField",
                "InfoRow",
                "ActionRow",
                "export component EditorActionsPanel inherits HubListPanelSlot",
                "export component EditorSourceSummaryPanel inherits PanelSlot",
                "export component EditorSourceSettingsPanel inherits PanelSlot",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "builds.slint",
            &builds_surface,
            &[
                "WorkspacePanelSection",
                "PanelSlot",
                "InfoRow",
                "ActionRow",
                "BuildHistoryRow",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "settings.slint",
            &settings_surface,
            &[
                "WorkspacePanelSection",
                "PanelSlot",
                "EmptyStateBlock",
                "HubTextField",
                "HubComboBox",
                "HubListPanelSlot",
                "PathSettingRow",
                "SettingStatusRow",
                "SettingsComboChoice",
                "SettingsSaveActionRow",
            ][..],
        ),
        (
            "cloud.slint",
            &cloud_surface,
            &[
                "WorkspacePanelSection",
                "OverviewPanel",
                "PanelSlot",
                "ResponsiveSlot",
                "MetricCard",
                "HubListPanelSlot",
                "export component CloudMetricSlot inherits ResponsiveSlot",
                "export component CloudPackageActionRow inherits ActionRow",
                "export component CloudPackageActionsPanel inherits HubListPanelSlot",
                "export component CloudServiceRow inherits InfoRow",
                "export component CloudServicesPanel inherits HubListPanelSlot",
                "collapse-label: label-collapse.collapsed;",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "team.slint",
            &team_surface,
            &[
                "WorkspacePanelSection",
                "OverviewPanel",
                "PanelSlot",
                "ResponsiveSlot",
                "MetricCard",
                "HubListPanelSlot",
                "export component TeamSummarySlot inherits ResponsiveSlot",
                "export component TeamMemberRow inherits InfoRow",
                "export component TeamMembersPanel inherits HubListPanelSlot",
                "collapse-label: label-collapse.collapsed;",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "assets.slint",
            &assets_surface,
            &[
                "CatalogPage",
                "InfoRow",
                "export component AssetRow inherits InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: label-collapse.collapsed;",
            ][..],
        ),
        (
            "plugins.slint",
            &plugins_surface,
            &[
                "CatalogPage",
                "InfoRow",
                "export component PluginRow inherits InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: label-collapse.collapsed;",
            ][..],
        ),
        (
            "learn.slint",
            &learn_surface,
            &[
                "CatalogPage",
                "InfoRow",
                "export component LearnRow inherits InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: label-collapse.collapsed;",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{page} must consume the real Material/Taffy wrapper instead of relying on a sample surface: {snippet}"
            );
        }
    }

    let app = read_ui_file("app.slint");
    assert!(
        app.contains("has-selected-project: root.project-detail.selected;"),
        "app.slint must pass selected-project state into catalog/workspace pages that surface scoped copy"
    );
}
