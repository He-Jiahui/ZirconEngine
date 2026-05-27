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
    let layout = read_ui_file("layout.slint");
    let inputs = read_ui_file("inputs.slint");
    let surfaces = read_ui_file("surfaces.slint");
    let shared = read_ui_file("shared.slint");
    let material_bridge = read_ui_file("material_bridge.slint");
    let dashboard = read_ui_file("project_dashboard.slint");
    let project_pages = read_ui_file("project_pages.slint");
    let editor = read_ui_file("editor.slint");
    let builds = read_ui_file("builds.slint");
    let settings = read_ui_file("settings.slint");
    let cloud = read_ui_file("cloud.slint");
    let team = read_ui_file("team.slint");
    let assets = read_ui_file("assets.slint");
    let plugins = read_ui_file("plugins.slint");
    let learn = read_ui_file("learn.slint");

    for (name, source) in [
        ("components.slint", &components),
        ("data_display.slint", &data_display),
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
        "menu := PopupMenu",
        "export component HubTextField",
        "material-field := TextField",
        "export component SearchBox",
        "search-field := SearchBar",
    ] {
        assert!(
            inputs.contains(snippet),
            "inputs.slint must keep the Hub wrapper backed by the Material primitive: {snippet}"
        );
    }

    for snippet in [
        "if root.variant != \"elevated\": OutlinedCard",
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
        "if root.active: FilledIconButton",
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
        "EmptyStateBlock",
        "ListTile",
        "ScrollView",
    ] {
        assert!(
            data_display.contains(snippet),
            "data_display.slint must keep real list/table surfaces backed by Material wrappers: {snippet}"
        );
    }

    for (page, source, snippets) in [
        (
            "project_dashboard.slint",
            &dashboard,
            &[
                "Flow",
                "PanelGrid",
                "PanelSlot",
                "ResponsiveSlot",
                "SearchBox",
                "ProjectFilterSelect",
                "ProjectSortSelect",
                "ActionRow",
                "EmptyStateBlock",
                "EmptyStatePanel",
            ][..],
        ),
        (
            "project_pages.slint",
            &project_pages,
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
            "editor.slint",
            &editor,
            &[
                "WorkspacePanelSection",
                "PanelSlot",
                "ResponsiveSlot",
                "HubTextField",
                "InfoRow",
                "ActionRow",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "builds.slint",
            &builds,
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
            &settings,
            &[
                "WorkspacePanelSection",
                "PanelSlot",
                "EmptyStateBlock",
                "HubTextField",
                "SegmentButton",
                "PanelListViewport",
                "PathSettingRow",
                "SettingStatusRow",
            ][..],
        ),
        (
            "cloud.slint",
            &cloud,
            &[
                "WorkspacePanelSection",
                "OverviewPanel",
                "PanelSlot",
                "ResponsiveSlot",
                "MetricCard",
                "PanelListViewport",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "team.slint",
            &team,
            &[
                "WorkspacePanelSection",
                "OverviewPanel",
                "PanelSlot",
                "ResponsiveSlot",
                "MetricCard",
                "PanelListViewport",
                "EmptyStateBlock",
            ][..],
        ),
        (
            "assets.slint",
            &assets,
            &[
                "CatalogPage",
                "InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: root.content-width < HubTokens.breakpoint-medium;",
            ][..],
        ),
        (
            "plugins.slint",
            &plugins,
            &[
                "CatalogPage",
                "InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: root.content-width < HubTokens.breakpoint-medium;",
            ][..],
        ),
        (
            "learn.slint",
            &learn,
            &[
                "CatalogPage",
                "InfoRow",
                "row-height: HubTokens.list-row-lg + HubTokens.space-6;",
                "collapse-label: root.content-width < HubTokens.breakpoint-medium;",
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
