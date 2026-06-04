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
    let list_container = read_ui_file("list_container_components.slint");
    let table_view = read_ui_file("table_view_components.slint");
    let tree_view = read_ui_file("tree_view_components.slint");
    let catalog_components = read_ui_file("catalog_page_components.slint");
    let data_surface = format!(
        "{data_display}\n{list_container}\n{table_view}\n{tree_view}\n{catalog_components}"
    );
    let layout = read_ui_file("layout.slint");
    let inputs = read_ui_file("inputs.slint");
    let text_inputs = read_ui_file("text_input_components.slint");
    let input_state_components = read_ui_file("input_state_components.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let button_components = read_ui_file("button_components.slint");
    let icon_button_components = read_ui_file("icon_button_components.slint");
    let material_bridge = read_ui_file("material_bridge.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let dashboard_components = read_ui_file("project_dashboard_components.slint");
    let project_card_flow_components = read_ui_file("project_card_flow_components.slint");
    let dashboard_surface =
        format!("{dashboard}\n{dashboard_components}\n{project_card_flow_components}");
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
    let catalog_detail_components = read_ui_file("catalog_detail_components.slint");
    let row_slot_components = read_ui_file("row_slot_components.slint");
    let assets = read_ui_file("assets.slint");
    let assets_surface = format!("{assets}\n{catalog_components}");
    let plugins = read_ui_file("plugins.slint");
    let plugins_surface = format!("{plugins}\n{catalog_components}");
    let learn = read_ui_file("learn.slint");
    let learn_surface = format!("{learn}\n{catalog_components}");

    for (name, source) in [
        ("components.slint", &components),
        ("data_display.slint", &data_display),
        ("list_container_components.slint", &list_container),
        ("table_view_components.slint", &table_view),
        ("tree_view_components.slint", &tree_view),
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
        "export component FlowScrollSurface",
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
        "export component HubSelectTrigger",
        "trigger := OutlineButton",
        "export component ToolbarSelect",
        "HubSelectTrigger {",
        "menu := HubSelectMenu",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep the Hub select/button wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
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
            text_inputs.contains(snippet),
            "text_input_components.slint must keep the Hub text-input wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
        "export component HubCheckBox",
        "material-check := MaterialCheckBox",
        "export component HubCheckBoxRow",
        "material-row := MaterialCheckBoxTile",
        "export component HubSwitch",
        "material-switch := MaterialSwitch",
        "export component HubToggleRow",
        "HubSwitch {",
        "export component HubComboBox",
        "material-combo := HubSelectDropDownSurface",
    ] {
        assert!(
            input_state_components.contains(snippet),
            "input_state_components.slint must keep the Hub state wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
        "root.variant == \"selected\" ? HubVisualSpec.panel-hover-background : HubVisualSpec.panel-background",
        "if root.show-action: HubPanelHeaderActionButton",
        "export component OverviewPanel inherits HubPanel",
        "export component EmptyStateBlock inherits Rectangle",
        "export component EmptyStatePanel inherits HubPanel",
        "MaterialText {",
    ] {
        assert!(
            surfaces.contains(snippet),
            "surfaces.slint must keep cards/text on Material primitives and actions on shared Hub button wrappers: {snippet}"
        );
    }

    for snippet in [
        "FilledButton,",
        "OutlineButton,",
        "TonalButton,",
        "if root.primary &&",
        "export component HubFormActionRow",
        "export component HubDisclosureButton",
        "export component HubHeaderCommandGroup",
        "export component HubPanelNavigationCommand",
        "export component HubPanelHeaderActionButton",
        "export component HubUserMenuTriggerButton",
        "export component HubSidebarCollapseButton",
        "StateLayerArea {",
    ] {
        assert!(
            button_components.contains(snippet),
            "button_components.slint must keep public Hub button APIs wired to Material buttons: {snippet}"
        );
    }
    for snippet in [
        "FilledIconButton,",
        "OutlineIconButton,",
        "export component IconButton",
        "export component HubTopbarIconButton",
        "export component HubBackButton",
        "export component HubFlowNextButton",
        "export component HubRowActionButton",
        "export component HubFloatingIconButton",
        "export component HubMoreMenuButton",
        "StateLayerArea {",
    ] {
        assert!(
            icon_button_components.contains(snippet),
            "icon_button_components.slint must keep public Hub icon-button APIs wired to Material icon buttons: {snippet}"
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
        "HubMetricSlot",
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
            "project workflow pages/components",
            &project_surface,
            &[
                "PanelSlot",
                "ResponsiveSlot",
                "SearchBox",
                "ProjectFilterSelect",
                "ProjectSortSelect",
                "HubPathFieldRow",
            ][..],
        ),
        (
            "project new page/components",
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
                "HubRowSelectionSlot",
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
                "ProjectDetailMainPanel",
                "ProjectDetailStatusStrip",
                "ProjectDetailInfoSection",
                "ProjectDetailActionsSection",
                "HubActionCommandButton",
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
                "HubPathFieldRow",
                "InfoRow",
                "ActionRow",
                "export component EditorActionsPanel inherits HubListPanelSlot",
                "export component EditorSourceSummaryPanel inherits HubContentPanelSlot",
                "export component EditorSourceSettingsPanel inherits HubFormPanelSlot",
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
                "HubMetricSlot",
                "HubListPanelSlot",
                "export component CloudMetricSlot inherits HubMetricSlot",
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
                "PanelSlot",
                "HubMetricSlot",
                "HubListPanelSlot",
                "export component TeamSummarySlot inherits HubMetricSlot",
                "export component TeamActionRow inherits ActionRow",
                "export component TeamIdentityRow inherits InfoRow",
                "export component TeamMemberRow inherits InfoRow",
                "HubTabbedListPanelSlot",
                "export component TeamMembersPanel inherits HubTabbedListPanelSlot",
                "export component TeamActionsPanel inherits HubListPanelSlot",
                "collapse-label: label-collapse.collapsed;",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "assets.slint",
            &assets_surface,
            &[
                "CatalogPage",
                "CatalogColumnRow",
                "export component AssetRow inherits CatalogColumnRow",
                "row-height: HubTokens.list-row-md;",
                "collapse-label: label-collapse.collapsed;",
                "HubRowLeadingIconSlot",
                "HubRowMetaSlot",
                "HubRowTrailingSlot",
            ][..],
        ),
        (
            "plugins.slint",
            &plugins_surface,
            &[
                "CatalogPage",
                "CatalogColumnRow",
                "export component PluginRow inherits CatalogColumnRow",
                "row-height: HubTokens.list-row-md;",
                "collapse-label: label-collapse.collapsed;",
                "HubRowLeadingIconSlot",
                "HubRowMetaSlot",
                "HubRowTrailingSlot",
            ][..],
        ),
        (
            "learn.slint",
            &learn_surface,
            &[
                "CatalogPage",
                "CatalogColumnRow",
                "export component LearnRow inherits CatalogColumnRow",
                "row-height: HubTokens.list-row-md;",
                "collapse-label: label-collapse.collapsed;",
                "HubRowLeadingIconSlot",
                "HubRowMetaSlot",
                "HubRowTrailingSlot",
            ][..],
        ),
    ] {
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{page} must consume the real Material/Taffy wrapper instead of relying on a sample surface: {snippet}"
            );
        }

        for snippet in [
            "export component CatalogDetailPanel inherits HubContentPanelSlot",
            "body-padding: MaterialStyleMetrics.padding_16;",
            "body-spacing: HubTokens.toolbar-gap;",
            "content-spacing: HubTokens.toolbar-gap;",
            "component CatalogDetailPreviewBand inherits Rectangle",
            "component CatalogDetailStatGrid inherits Rectangle",
            "component CatalogDetailCheckList inherits Rectangle",
            "component CatalogDetailCheckRow inherits Rectangle",
            "CatalogDetailPreviewBand {",
            "CatalogDetailStatGrid {",
            "CatalogDetailCheckList {",
            "HubRowLeadingIconSlot",
            "HubRowMainSlot",
            "HubRowTrailingSlot",
        ] {
            assert!(
                catalog_detail_components.contains(snippet),
                "CatalogDetailPanel must stay decomposed into panel, preview, stat, check-list, and row-slot components: {snippet}"
            );
        }

        for forbidden in [
            "export component CatalogDetailPanel inherits ResponsiveSlot",
            "HubPanel {\n        width: parent.width;\n        height: parent.height;",
        ] {
            assert!(
                !catalog_detail_components.contains(forbidden),
                "CatalogDetailPanel should not reintroduce its old page-local panel shell after adopting PanelSlot: {forbidden}"
            );
        }

        let check_row = catalog_detail_components
            .split("component CatalogDetailCheckRow")
            .nth(1)
            .and_then(|source| source.split("component CatalogDetailPreviewBand").next())
            .expect(
                "catalog_detail_components.slint must declare CatalogDetailCheckRow before CatalogDetailPreviewBand",
            );
        for snippet in [
            "HubRowLeadingIconSlot {",
            "HubRowMainSlot {",
            "title: root.title;",
            "detail: root.detail;",
            "title-foreground: HubTokens.text-primary;",
            "HubRowTrailingSlot {",
        ] {
            assert!(
                check_row.contains(snippet),
                "CatalogDetailCheckRow must compose check-row content through the shared row slot family: {snippet}"
            );
        }
        for forbidden in ["MaterialText {", "MutedText {", "VerticalLayout {"] {
            assert!(
                !check_row.contains(forbidden),
                "CatalogDetailCheckRow should not return to local text-stack layout after adopting HubRowMainSlot: {forbidden}"
            );
        }

        for snippet in [
            "export component HubRowLeadingIconSlot inherits Rectangle",
            "shell-border: HubVisualSpec.neutral-icon-stroke;",
            "shell-background: HubVisualSpec.neutral-icon-background;",
            "icon-foreground: HubVisualSpec.neutral-icon-foreground;",
            "export component HubRowMainSlot inherits Rectangle",
            "style: MaterialTypography.label_large;",
            "style: MaterialTypography.body_small;",
            "export component HubRowSelectionSlot inherits Rectangle",
            "HubCheckBox",
            "check-state: root.check-state;",
            "export component HubRowTrailingSlot inherits Rectangle",
            "StatusBadge",
            "HubRowActionButton",
            "in property <bool> show-action: false;",
            "private property <bool> action-visible: root.show-action || root.show-chevron;",
            "button-size: root.action-size;",
            "button-radius: root.action-radius;",
            "framed: root.action-framed;",
            "state-layer-color: root.action-state-layer-color;",
            "width: root.slot-width;",
        ] {
            assert!(
                row_slot_components.contains(snippet),
                "row_slot_components.slint must own shared neutral leading and trailing row slots: {snippet}"
            );
        }
        for forbidden in [
            "import { HubIconButton } from \"button_components.slint\";",
            "HubIconButton {",
            "StateLayerArea {",
            "if root.action-visible && !root.action-framed: Rectangle",
        ] {
            assert!(
                !row_slot_components.contains(forbidden),
                "HubRowTrailingSlot should consume HubRowActionButton instead of owning local action-button internals: {forbidden}"
            );
        }
    }

    let app = read_ui_file("app.slint");
    assert!(
        app.contains("has-selected-project: root.project-detail.selected;"),
        "app.slint must pass selected-project state into catalog/workspace pages that surface scoped copy"
    );
}
