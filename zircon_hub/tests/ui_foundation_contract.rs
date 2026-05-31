//! Static contracts for Zircon Hub Material foundation and layout primitives.

use std::{fs, path::PathBuf};

fn ui_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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

fn read_crate_file(name: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(name)).unwrap_or_else(|error| {
            panic!("failed to read Hub crate file {name}: {error}");
        }),
    )
}

fn slint_files() -> Vec<PathBuf> {
    let mut files = fs::read_dir(ui_dir())
        .expect("failed to read Hub UI directory")
        .map(|entry| entry.expect("failed to read Hub UI entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "slint")
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn count_component_declarations(source: &str) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("export component ") || trimmed.starts_with("component ")
        })
        .count()
}

#[test]
fn components_entrypoint_stays_thin_and_reexports_new_modules() {
    let components = read_ui_file("components.slint");
    let line_count = components.lines().count();
    assert!(
        line_count <= 160,
        "components.slint should remain a re-export entrypoint; found {line_count} lines"
    );

    for module in [
        "tokens.slint",
        "layout.slint",
        "surfaces.slint",
        "inputs.slint",
        "shell_header_components.slint",
        "shell_sidebar_components.slint",
        "shell_page_components.slint",
        "button_components.slint",
        "navigation.slint",
        "data_display.slint",
        "table_view_components.slint",
        "operation_timeline_components.slint",
        "overlays.slint",
        "material_bridge.slint",
    ] {
        assert!(
            components.contains(module),
            "components.slint must re-export {module}"
        );
    }
}

#[test]
fn foundation_visual_tokens_include_focus_disabled_and_status_roles() {
    let tokens = read_ui_file("tokens.slint");

    for snippet in [
        "in property <length> focus-ring-width: MaterialStyleMetrics.size_2;",
        "in property <length> focus-ring-offset: MaterialStyleMetrics.size_2;",
        "in property <color> focus-ring-color: MaterialPalette.primary;",
        "in property <float> disabled-opacity: MaterialPalette.disable_opacity;",
        "in property <color> status-neutral-fill: MaterialPalette.surface_container_high;",
        "in property <color> status-neutral-stroke: MaterialPalette.outline_variant;",
        "in property <color> status-neutral-foreground: MaterialPalette.on_surface_variant;",
        "in property <color> status-info-fill: MaterialPalette.secondary_container;",
        "in property <color> status-info-stroke: MaterialPalette.secondary;",
        "in property <color> status-info-foreground: MaterialPalette.on_secondary_container;",
        "in property <color> status-success-fill: MaterialPalette.primary_container;",
        "in property <color> status-success-stroke: root.success-stroke;",
        "in property <color> status-success-foreground: MaterialPalette.on_primary_container;",
        "in property <color> nav-active-fill: rgb(20, 55, 54);",
        "in property <color> badge-accent-fill: rgb(28, 56, 56);",
        "in property <color> badge-accent-stroke: rgb(42, 118, 121);",
        "in property <color> badge-accent-foreground: MaterialPalette.on_primary_container;",
        "in property <color> status-warning-fill: root.warning-fill;",
        "in property <color> status-warning-stroke: root.warning-stroke;",
        "in property <color> status-warning-foreground: MaterialPalette.on_tertiary_container;",
        "in property <color> status-error-fill: root.error-fill;",
        "in property <color> status-error-stroke: root.error-stroke;",
        "in property <color> status-error-foreground: MaterialPalette.on_error_container;",
    ] {
        assert!(
            tokens.contains(snippet),
            "HubVisualSpec must expose shared focus, disabled, and semantic status design tokens; missing {snippet}"
        );
    }
}

#[test]
fn app_and_page_entrypoints_stay_structural() {
    for (file, root_component) in [
        ("app.slint", "HubWindow"),
        ("projects.slint", "ProjectsPage"),
        ("project_dashboard.slint", "ProjectDashboardPage"),
        ("project_new_page.slint", "ProjectNewPage"),
        ("project_browser_page.slint", "ProjectBrowserPage"),
        ("project_detail_page.slint", "ProjectDetailPage"),
        ("editor.slint", "EditorPage"),
        ("builds.slint", "BuildsPage"),
        ("settings.slint", "SettingsPage"),
        ("cloud.slint", "CloudPage"),
        ("team.slint", "TeamPage"),
        ("assets.slint", "AssetsPage"),
        ("plugins.slint", "PluginsPage"),
        ("learn.slint", "LearnPage"),
    ] {
        let source = read_ui_file(file);
        assert_eq!(
            count_component_declarations(&source),
            1,
            "{file} should expose only its root {root_component}; helper/component bodies belong in focused owner modules"
        );
        assert!(
            source.contains(&format!("export component {root_component} ")),
            "{file} must keep {root_component} as its single root component"
        );
    }

    let aggregate = read_ui_file("project_pages.slint");
    let expected_aggregate = [
        "export { ProjectNewPage } from \"project_new_page.slint\";",
        "export { ProjectBrowserPage } from \"project_browser_page.slint\";",
        "export { ProjectDetailPage } from \"project_detail_page.slint\";",
    ];
    assert_eq!(
        aggregate.lines().collect::<Vec<_>>(),
        expected_aggregate,
        "project_pages.slint should remain a pure secondary-page aggregate with no implementation body"
    );
    assert!(
        !ui_dir().join("shell.slint").exists(),
        "shell.slint was a migration-only compatibility note and must stay deleted; shell ownership belongs to focused shell_*_components.slint files"
    );
}

#[test]
fn focused_component_modules_own_page_helpers() {
    for (file, required_components) in [
        (
            "project_dashboard_components.slint",
            &[
                ComponentExport::Public("ProjectCover"),
                ComponentExport::Public("ProjectCard"),
                ComponentExport::Public("ProjectFlow"),
                ComponentExport::Public("DashboardProjectCardsSection"),
                ComponentExport::Public("DashboardToolbar"),
                ComponentExport::Public("DashboardRecentProjectsPanel"),
                ComponentExport::Public("DashboardQuickActionsPanel"),
            ][..],
        ),
        (
            "project_page_components.slint",
            &[
                ComponentExport::Public("PageHeader"),
                ComponentExport::Public("ProjectCreateSettingsPanel"),
                ComponentExport::Public("ProjectCreateField"),
                ComponentExport::Public("ProjectCreateActionRow"),
                ComponentExport::Public("ProjectEngineChoiceList"),
                ComponentExport::Public("ProjectTemplateRailPanel"),
            ][..],
        ),
        (
            "project_browser_components.slint",
            &[
                ComponentExport::Public("ProjectFilterSelect"),
                ComponentExport::Public("ProjectSortSelect"),
                ComponentExport::Public("ProjectBrowserTableHeader"),
                ComponentExport::Public("ProjectBrowserRow"),
                ComponentExport::Public("ProjectBrowserResultsPanel"),
            ][..],
        ),
        (
            "project_detail_components.slint",
            &[
                ComponentExport::Public("ProjectDetailActionButton"),
                ComponentExport::Public("ProjectDetailPinToggleRow"),
                ComponentExport::Public("ProjectDetailStatusStrip"),
                ComponentExport::Public("ProjectDetailInfoSection"),
                ComponentExport::Public("ProjectDetailEngineSection"),
            ][..],
        ),
        (
            "editor_page_components.slint",
            &[
                ComponentExport::Public("SourceEngineRow"),
                ComponentExport::Public("EditorSideListPanel"),
                ComponentExport::Public("EditorSourceEngineListPanel"),
                ComponentExport::Public("EditorBuildHistoryPanel"),
                ComponentExport::Public("EditorActionsPanel"),
                ComponentExport::Public("EditorSourceSummaryPanel"),
                ComponentExport::Public("EditorSourceSettingsPanel"),
            ][..],
        ),
        (
            "builds_page_components.slint",
            &[
                ComponentExport::Public("BuildSourceSummaryPanel"),
                ComponentExport::Public("BuildControlsPanel"),
                ComponentExport::Public("BuildPipelinePanel"),
                ComponentExport::Public("BuildTaskHistoryPanel"),
            ][..],
        ),
        (
            "settings_page_components.slint",
            &[
                ComponentExport::Public("SettingsToolchainPanel"),
                ComponentExport::Public("SettingsBuildDefaultsPanel"),
                ComponentExport::Public("SettingsDefaultPathsPanel"),
                ComponentExport::Public("SettingsConfigurationHealthPanel"),
            ][..],
        ),
        (
            "cloud_page_components.slint",
            &[
                ComponentExport::Public("CloudMetricSlot"),
                ComponentExport::Public("CloudPackageActionsPanel"),
                ComponentExport::Public("CloudServicesPanel"),
            ][..],
        ),
        (
            "team_page_components.slint",
            &[
                ComponentExport::Public("TeamSummarySlot"),
                ComponentExport::Public("TeamMembersPanel"),
            ][..],
        ),
        (
            "catalog_page_components.slint",
            &[
                ComponentExport::Public("AssetRow"),
                ComponentExport::Public("PluginRow"),
                ComponentExport::Public("LearnRow"),
            ][..],
        ),
        (
            "table_view_components.slint",
            &[
                ComponentExport::Public("TableColumnHeader"),
                ComponentExport::Public("ProjectTableRow"),
                ComponentExport::Public("HubTableBody"),
                ComponentExport::Public("DataTable"),
                ComponentExport::Public("HubTableView"),
            ][..],
        ),
        (
            "operation_timeline_components.slint",
            &[
                ComponentExport::Public("OperationTimelineRow"),
                ComponentExport::Public("OperationTimelinePanel"),
            ][..],
        ),
        (
            "shell_header_components.slint",
            &[
                ComponentExport::Public("HubTopHeader"),
                ComponentExport::Private("WindowDragRegion"),
                ComponentExport::Private("HeaderControlSlot"),
            ][..],
        ),
        (
            "shell_header_popup_components.slint",
            &[
                ComponentExport::Public("HeaderEngineSelector"),
                ComponentExport::Private("HeaderEngineOption"),
            ][..],
        ),
        (
            "shell_sidebar_components.slint",
            &[
                ComponentExport::Public("HubNavSidebar"),
                ComponentExport::Public("NavStatusPanel"),
            ][..],
        ),
        (
            "shell_page_components.slint",
            &[
                ComponentExport::Public("HubPageHeader"),
                ComponentExport::Public("HubStatusBar"),
            ][..],
        ),
    ] {
        let source = read_ui_file(file);
        for component in required_components {
            let declaration = match component {
                ComponentExport::Public(name) => format!("export component {name} "),
                ComponentExport::Private(name) => format!("component {name} "),
            };
            assert!(
                source.contains(&declaration),
                "{file} must own focused page/chrome helper {component:?}"
            );
        }
    }
}

#[derive(Debug)]
enum ComponentExport {
    Public(&'static str),
    Private(&'static str),
}

#[test]
fn hub_directly_registers_and_reexports_material_template() {
    let material_template = crate_dir()
        .parent()
        .expect("zircon_hub lives below the repository root")
        .join("dev/material-rust-template/material-1.0/material.slint");
    assert!(
        material_template.is_file(),
        "Hub must import the local Slint Material template directly from {}",
        material_template.display()
    );

    let build = read_crate_file("build.rs");
    for snippet in [
        "config.library_paths.insert(",
        "\"material\".to_string()",
        "dev/material-rust-template/material-1.0/material.slint",
    ] {
        assert!(
            build.contains(snippet),
            "build.rs must register the local Material template with Slint as @material; missing {snippet}"
        );
    }

    let material_bridge = read_ui_file("material_bridge.slint");
    assert!(
        material_bridge.contains("} from \"@material\";"),
        "material_bridge.slint must directly re-export the local @material template library"
    );
    assert!(
        material_bridge.lines().count() <= 120,
        "material_bridge.slint should stay a direct export bridge, not an implementation file"
    );

    let components = read_ui_file("components.slint");
    assert!(
        components.contains("material_bridge.slint"),
        "components.slint must expose Material template components through material_bridge.slint"
    );
    for component in [
        "AppBar",
        "CheckBox",
        "ActionChip",
        "Dialog",
        "Drawer",
        "DropDownMenu",
        "HorizontalDivider",
        "Vertical",
        "VerticalDivider",
        "Horizontal",
        "Grid",
        "ScrollView",
        "ElevatedCard",
        "FilledCard",
        "OutlinedCard",
        "FilledButton",
        "IconButton",
        "ListView",
        "MaterialWindow",
        "MaterialWindowAdapter",
        "NavigationRail",
        "CircularProgressIndicator",
        "SearchBar",
        "StateLayerArea",
        "SegmentedButton",
        "Switch",
        "TabBar",
        "TextButton",
        "TextField",
        "ToolTip",
        "MaterialStyleMetrics",
        "MaterialPalette",
        "MaterialTypography",
    ] {
        assert!(
            material_bridge.contains(component) && components.contains(component),
            "Hub must expose Material template component {component}"
        );
    }
    assert!(
        !material_bridge.contains("\n    Badge,"),
        "material_bridge.slint omits Material Badge so components.slint can keep Hub Badge as the public Badge name"
    );
    for path in slint_files() {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        if file_name == "material_bridge.slint" {
            continue;
        }
        assert!(
            !normalize_newlines(source).contains("from \"@material\""),
            "{file_name} must import Slint Material through material_bridge.slint, leaving @material as a single compiler bridge"
        );
    }

    let layout = read_ui_file("layout.slint");
    assert!(
        layout.contains(
            "import { HorizontalDivider, ScrollView, VerticalDivider } from \"material_bridge.slint\";"
        ) && layout.contains("page-scroll := ScrollView"),
        "Page must use the ScrollView component through the Hub Material bridge"
    );
    assert!(
        layout.contains("if root.vertical: VerticalDivider {")
            && layout.contains("if !root.vertical: HorizontalDivider {")
            && !layout.contains("background: root.tone;"),
        "Divider must use the direct Material HorizontalDivider/VerticalDivider primitives instead of a hand-drawn line"
    );
}

#[test]
fn hub_applies_zircon_material_theme_to_material_palette() {
    let app = read_ui_file("app.slint");
    let overlays = read_ui_file("overlays.slint");
    for snippet in [
        "import { ZirconMaterialTheme } from \"theme.slint\";",
        "HubWindowView,",
        "export component HubWindow inherits HubWindowView",
        "ZirconMaterialTheme { }",
    ] {
        assert!(
            app.contains(snippet),
            "HubWindow must inherit the shared window wrapper and install the Zircon Material theme before rendering controls; missing {snippet}"
        );
    }
    for snippet in [
        "import {",
        "MaterialWindow,",
        "} from \"material_bridge.slint\";",
        "export component HubWindowView inherits MaterialWindow",
    ] {
        assert!(
            overlays.contains(snippet),
            "HubWindowView must own the imported Slint Material window after window-shell extraction; missing {snippet}"
        );
    }

    let theme = read_ui_file("theme.slint");
    for snippet in [
        "import { MaterialPalette } from \"material_bridge.slint\";",
        "export component ZirconMaterialTheme",
        "MaterialPalette.schemes = {",
        "primary: rgb(52, 213, 208),",
        "primaryContainer: rgb(0, 81, 84),",
        "tertiary: rgb(255, 202, 83),",
        "background: rgb(13, 17, 20),",
        "surfaceContainerLow: rgb(22, 27, 30),",
    ] {
        assert!(
            theme.contains(snippet),
            "Zircon Material theme must keep the Hub palette teal/dark instead of the template default blue; missing {snippet}"
        );
    }
}

#[test]
fn hub_stays_standalone_slint_launcher_boundary() {
    let cargo_toml = read_crate_file("Cargo.toml");
    let build = read_crate_file("build.rs");
    let lib = read_crate_file("src/lib.rs");
    let main = read_crate_file("src/main.rs");
    let runtime = read_crate_file("src/app/runtime.rs");
    let editor_launch = read_crate_file("src/process/editor_launch.rs");

    for snippet in [
        "name = \"zircon_hub\"",
        "Standalone desktop launcher and install hub for ZirconEngine.",
        "slint = \"1.16.1\"",
        "autobins = false",
    ] {
        assert!(
            cargo_toml.contains(snippet),
            "zircon_hub Cargo manifest must keep the standalone Slint desktop launcher identity; missing {snippet}"
        );
    }

    let dependencies = cargo_toml
        .split("[dependencies]")
        .nth(1)
        .and_then(|source| source.split("[build-dependencies]").next())
        .expect("zircon_hub Cargo.toml must contain dependencies before build-dependencies");
    for forbidden in [
        "zircon_runtime",
        "zircon_editor",
        "zircon_app",
        "libloading",
        "webview",
        "wry",
        "tao",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "zircon_hub runtime dependencies must not pull editor/runtime lifecycle or WebView/browser stacks into the Hub: {forbidden}"
        );
    }

    for snippet in [
        "let input = manifest_dir.join(\"ui/app.slint\");",
        "config.enable_experimental = true;",
        "dev/material-rust-template/material-1.0/material.slint",
    ] {
        assert!(
            build.contains(snippet),
            "build.rs must compile Hub-owned Slint UI directly with the local Material template; missing {snippet}"
        );
    }
    for forbidden in [
        "zircon_editor/ui",
        "zircon_runtime/ui",
        "webview",
        "html",
        "css",
    ] {
        assert!(
            !build.to_ascii_lowercase().contains(forbidden),
            "build.rs must not route Hub UI through editor/runtime/WebView/HTML/CSS paths: {forbidden}"
        );
    }

    assert!(
        main.contains("zircon_hub::app::run()") && !main.contains("zircon_editor"),
        "main.rs should remain a thin Hub launcher entrypoint without editor-owned UI startup"
    );
    for forbidden in ["zircon_runtime", "libloading"] {
        assert!(
            !lib.contains(forbidden) && !runtime.contains(forbidden),
            "Hub library/runtime must not register or dynamically load engine/editor lifecycle surfaces: {forbidden}"
        );
    }
    for forbidden in [
        "use zircon_editor",
        "zircon_editor::",
        "extern crate zircon_editor",
    ] {
        assert!(
            !lib.contains(forbidden) && !runtime.contains(forbidden),
            "Hub library/runtime must not link editor lifecycle APIs directly; editor executable names belong only to child-process launch context: {forbidden}"
        );
    }

    for snippet in [
        "pub enum EditorLaunchRequest",
        "OpenProject { project_path: PathBuf }",
        "CreateProject(CreateProjectRequest)",
        "args.push(\"--project\".to_string());",
        "args.push(\"--create-project\".to_string());",
        "pub fn launch_editor(command: &EditorLaunchCommand) -> Result<Child, HubError>",
        "Command::new(&command.executable)",
    ] {
        assert!(
            editor_launch.contains(snippet),
            "Hub must launch the editor as a child process with stable CLI context instead of owning editor UI or runtime lifecycle; missing {snippet}"
        );
    }
}

#[test]
fn layout_primitives_and_tokens_are_declared() {
    let layout = read_ui_file("layout.slint");
    for primitive in [
        "Stack",
        "Row",
        "Column",
        "Flow",
        "Page",
        "PanelGrid",
        "WorkspacePanelSection",
        "ResponsiveSlot",
        "ResponsiveState",
        "Fill",
        "Divider",
    ] {
        assert!(
            layout.contains(&format!("component {primitive}")),
            "layout.slint must declare {primitive}"
        );
    }
    let fill = layout
        .split("export component Fill")
        .nth(1)
        .and_then(|source| source.split("export component Divider").next())
        .expect("layout.slint must declare Fill before Divider");
    for snippet in [
        "in property <length> fill-spacing: HubTokens.space-0;",
        "horizontal-stretch: 1;",
        "vertical-stretch: 1;",
        "VerticalLayout {",
        "spacing: root.fill-spacing;",
        "@children",
    ] {
        assert!(
            fill.contains(snippet),
            "Fill must own child fill layout so page hosts do not repeat parent geometry bindings; missing {snippet}"
        );
    }

    let responsive_slot = layout
        .split("export component ResponsiveSlot")
        .nth(1)
        .and_then(|source| source.split("export component ResponsiveState").next())
        .expect("layout.slint must declare ResponsiveSlot before ResponsiveState");
    for snippet in [
        "in property <length> basis: HubTokens.panel-min-md;",
        "in property <float> grow: 1;",
        "in property <float> shrink: 1;",
        "in property <int> order: 0;",
    ] {
        assert!(
            responsive_slot.contains(snippet),
            "ResponsiveSlot must keep shared Taffy sizing and ordering inputs; missing {snippet}"
        );
    }
    for forbidden in [
        "flex-basis: root.basis;",
        "flex-grow: root.grow;",
        "flex-shrink: root.shrink;",
        "flex-order: root.order;",
    ] {
        assert!(
            !responsive_slot.contains(forbidden),
            "ResponsiveSlot must not set Slint flex properties inside the component definition; use sites mirror them on direct Flexbox children: {forbidden}"
        );
    }

    let tokens = read_ui_file("tokens.slint");
    for token in [
        "import { MaterialPalette, MaterialStyleMetrics } from \"material_bridge.slint\";",
        "space-0",
        "space-1: MaterialStyleMetrics.spacing_4",
        "space-2: MaterialStyleMetrics.spacing_8",
        "space-3: MaterialStyleMetrics.spacing_12",
        "space-4: MaterialStyleMetrics.spacing_16",
        "breakpoint-compact: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_200 * 2",
        "breakpoint-medium: MaterialStyleMetrics.size_640 * 2",
        "breakpoint-wide: MaterialStyleMetrics.size_640 * 2 + MaterialStyleMetrics.size_256",
        "page-padding: MaterialStyleMetrics.padding_28 + MaterialStyleMetrics.size_2",
        "page-padding-compact: MaterialStyleMetrics.padding_16",
        "panel-gap: MaterialStyleMetrics.spacing_16 - MaterialStyleMetrics.size_1 * 2",
        "toolbar-gap: MaterialStyleMetrics.spacing_8 + MaterialStyleMetrics.size_1 * 2",
        "icon-xs: MaterialStyleMetrics.spacing_16",
        "icon-sm: MaterialStyleMetrics.icon_size_18",
        "icon-md: MaterialStyleMetrics.size_20",
        "icon-lg: MaterialStyleMetrics.padding_28",
        "icon-xl: MaterialStyleMetrics.size_32 + MaterialStyleMetrics.size_1 * 2",
        "control-sm: MaterialStyleMetrics.size_32 + MaterialStyleMetrics.size_1 * 2",
        "control-md: MaterialStyleMetrics.size_40 - MaterialStyleMetrics.size_1 * 2",
        "control-lg: MaterialStyleMetrics.size_40 + MaterialStyleMetrics.size_1 * 2",
        "input-field: MaterialStyleMetrics.size_56",
        "table-row: MaterialStyleMetrics.padding_28",
        "list-row-sm: MaterialStyleMetrics.size_56",
        "list-row-md: MaterialStyleMetrics.size_72",
        "list-row-lg: MaterialStyleMetrics.size_80 + MaterialStyleMetrics.size_6",
        "project-dashboard-toolbar-search-ratio: 0.24",
        "project-dashboard-toolbar-select-ratio: 0.142",
        "project-dashboard-card-ratio: 0.23125",
        "project-dashboard-lower-compact-ratio: 0.60",
        "project-dashboard-lower-regular-ratio: 0.393",
        "project-dashboard-table-ratio: 0.58",
        "project-new-main-stack-ratio: 0.60",
        "project-new-side-stack-ratio: 0.40",
        "project-browser-toolbar-search-ratio: 0.40",
        "project-browser-toolbar-select-ratio: 0.125",
        "project-detail-cover-ratio: 0.25",
        "project-detail-info-row-ratio: 0.05",
        "project-detail-version-badge-ratio: 0.10",
        "project-detail-pin-badge-ratio: 0.0714286",
        "project-detail-main-panel-ratio: 0.50",
        "project-detail-action-panel-ratio: 0.25",
        "project-detail-main-stack-ratio: 0.60",
        "project-detail-action-stack-ratio: 0.40",
        "workspace-row-editor-summary: root.control-md + root.list-row-sm * 3 + root.toolbar-gap * 3 + root.space-4 * 2",
        "workspace-row-editor-config: MaterialStyleMetrics.size_200 * 2 + MaterialStyleMetrics.size_72 + MaterialStyleMetrics.size_4",
        "workspace-row-build-summary: root.control-md + root.list-row-md * 5 + root.toolbar-gap * 5 + root.space-4 * 2",
        "workspace-row-build-detail: root.control-md + root.list-row-md * 4 + root.toolbar-gap * 4 + root.space-4 * 2",
        "workspace-row-settings-controls: MaterialStyleMetrics.size_200 + MaterialStyleMetrics.size_80 + MaterialStyleMetrics.size_6",
        "workspace-row-settings-detail: MaterialStyleMetrics.size_200 * 2 - MaterialStyleMetrics.size_8",
        "workspace-row-cloud-metrics: root.list-row-lg + root.space-6",
        "workspace-row-team-summary: root.list-row-lg + root.space-7",
        "breakpoint-short: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_256 + MaterialStyleMetrics.size_64",
        "shell-row-min: MaterialStyleMetrics.size_52",
        "shell-row-max: MaterialStyleMetrics.size_56",
        "nav-width-collapsed-min: MaterialStyleMetrics.size_80",
        "nav-width-collapsed-max: MaterialStyleMetrics.size_80",
        "nav-width-expanded-min: MaterialStyleMetrics.size_200",
        "window-resize-border: MaterialStyleMetrics.size_6",
        "window-min-width: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_344",
        "window-min-height: MaterialStyleMetrics.size_640",
        "window-preferred-width: root.breakpoint-wide + root.nav-width-collapsed-min",
        "window-preferred-height: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_360 + MaterialStyleMetrics.size_24",
        "user-menu-min-width: root.nav-width-collapsed-max + root.control-lg + root.space-1",
        "user-menu-max-width: root.nav-width-collapsed-max * 2 + root.space-1",
        "surface-page: MaterialPalette.background",
        "surface-panel: MaterialPalette.surface_container_low",
        "accent: MaterialPalette.primary",
    ] {
        assert!(tokens.contains(token), "tokens.slint is missing {token}");
    }

    let panel_grid = layout
        .split("export component PanelGrid")
        .nth(1)
        .and_then(|source| source.split("export component Page").next())
        .expect("layout.slint must declare PanelGrid before Page");
    for snippet in [
        "in property <length> grid-padding: HubTokens.space-0;",
        "in property <length> grid-padding-left: root.grid-padding;",
        "in property <length> grid-padding-right: root.grid-padding;",
        "in property <length> grid-padding-top: root.grid-padding;",
        "in property <length> grid-padding-bottom: root.grid-padding;",
        "padding-left: root.grid-padding-left;",
        "padding-right: root.grid-padding-right;",
        "padding-top: root.grid-padding-top;",
        "padding-bottom: root.grid-padding-bottom;",
    ] {
        assert!(
            panel_grid.contains(snippet),
            "PanelGrid must expose token-backed padding slots without page-local wrapper geometry; missing {snippet}"
        );
    }
    for forbidden in [
        "background: HubVisualSpec",
        "HubPanel {",
        "PanelSlot {",
        "MaterialText {",
        "ActionRow {",
        "InfoRow {",
    ] {
        assert!(
            !panel_grid.contains(forbidden),
            "PanelGrid must remain geometry-only and not absorb surface/display ownership: {forbidden}"
        );
    }

    let page = layout
        .split("export component Page")
        .nth(1)
        .and_then(|source| source.split("export component PageScrollSurface").next())
        .expect("layout.slint must declare Page before PageScrollSurface");
    for snippet in [
        "in property <length> page-padding: root.width < HubTokens.breakpoint-compact ? HubTokens.page-padding-compact : HubTokens.page-padding;",
        "in property <length> page-padding-x: root.page-padding;",
        "in property <length> page-padding-y: root.page-padding;",
        "padding-left: root.page-padding-x;",
        "padding-right: root.page-padding-x;",
        "padding-top: root.page-padding-y;",
    ] {
        assert!(
            page.contains(snippet),
            "PageScrollSurface must own compact-aware token padding and derived content geometry; missing {snippet}"
        );
    }
    for forbidden in [
        "PanelHeader {",
        "HubPanel {",
        "ActionRow {",
        "InfoRow {",
        "StatusBanner {",
    ] {
        assert!(
            !page.contains(forbidden),
            "PageScrollSurface must stay a scroll/layout primitive and not absorb chrome or row content: {forbidden}"
        );
    }
}

#[test]
fn hub_surfaces_are_backed_by_material_cards() {
    let surfaces = read_ui_file("surfaces.slint");
    for snippet in [
        "MaterialPalette",
        "MaterialStyleMetrics",
        "MaterialText",
        "MaterialTypography",
        "OutlineButton",
        "if root.variant == \"elevated\": ElevatedCard",
        "if root.variant != \"elevated\": OutlinedCard",
        "border-radius: HubVisualSpec.panel-radius",
        "border-color: root.variant == \"danger\" ? HubVisualSpec.error-stroke",
        "@children",
    ] {
        assert!(
            surfaces.contains(snippet),
            "HubPanel must preserve its Hub API while using Material Card surfaces; missing {snippet}"
        );
    }

    let badge = surfaces
        .split("export component Badge")
        .nth(1)
        .and_then(|source| source.split("export component StatusBadge").next())
        .expect("surfaces.slint must declare Badge before StatusBadge");
    for snippet in [
        "MaterialText {",
        "text: root.text;",
        "style: MaterialTypography.label_medium;",
        "horizontal_alignment: center;",
        "vertical_alignment: center;",
    ] {
        assert!(
            badge.contains(snippet),
            "Badge should keep its Hub tone shell while delegating text styling to MaterialText; missing {snippet}"
        );
    }
    assert!(
        !badge.lines().any(|line| line.trim() == "Text {"),
        "Badge should not return to a raw Text element for its label"
    );
    for forbidden in [
        "font-size: MaterialTypography.label_medium.font_size;",
        "font-weight: MaterialTypography.label_medium.font_weight;",
    ] {
        assert!(
            !badge.contains(forbidden),
            "Badge should not return to raw Text/font bindings for its label: {forbidden}"
        );
    }

    let hub_panel_end = surfaces
        .find("export component HubCard")
        .expect("surfaces.slint must declare HubCard after HubPanel");
    let hub_panel = &surfaces[..hub_panel_end];
    let children_index = hub_panel
        .find("@children")
        .expect("HubPanel must expose @children");
    assert!(
        !hub_panel[children_index..].contains("TouchArea"),
        "HubPanel must not place a panel-level TouchArea above @children; it blocks nested controls"
    );

    let panel_header = surfaces
        .split("export component PanelHeader")
        .nth(1)
        .and_then(|source| source.split("export component StatusBanner").next())
        .expect("surfaces.slint must declare PanelHeader before StatusBanner");
    for snippet in [
        "if root.show-action: OutlineButton",
        "height: HubVisualSpec.toolbar-density-height;",
        "text: root.action-text;",
        "icon: @image-url(\"../assets/icons/ui/chevron-right.svg\");",
        "root.action-clicked();",
        "MaterialText {",
        "style: MaterialTypography.title_medium;",
    ] {
        assert!(
            panel_header.contains(snippet),
            "PanelHeader action should be a Material OutlineButton; missing {snippet}"
        );
    }
    for forbidden in ["panel-action-area", "TouchArea", "CenteredIcon"] {
        assert!(
            !panel_header.contains(forbidden),
            "PanelHeader action should not return to a custom painted click layer: {forbidden}"
        );
    }

    let status_banner = surfaces
        .split("export component StatusBanner")
        .nth(1)
        .and_then(|source| source.split("export component HubSection").next())
        .expect("surfaces.slint must declare StatusBanner");
    for snippet in [
        "MaterialText {",
        "style: MaterialTypography.title_small;",
        "height: HubTokens.list-row-sm;",
        "vertical-stretch: 0;",
    ] {
        assert!(
            status_banner.contains(snippet),
            "StatusBanner title should use MaterialText typography; missing {snippet}"
        );
    }
    let hub_section = surfaces
        .split("export component HubSection")
        .nth(1)
        .expect("surfaces.slint must declare HubSection");
    assert!(
        surfaces.contains("export component HubSection inherits Rectangle"),
        "surfaces.slint must expose HubSection as a lightweight Rectangle section primitive"
    );
    for snippet in [
        "section-height: HubTokens.list-row-lg;",
        "section-spacing: HubTokens.toolbar-gap;",
        "in property <bool> stretch: false;",
        "vertical-stretch: root.stretch ? 1 : 0;",
        "PanelHeader {",
        "title: root.title;",
        "subtitle: root.subtitle;",
        "@children",
    ] {
        assert!(
            hub_section.contains(snippet),
            "HubSection should centralize in-panel section headers without adding nested panel chrome; missing {snippet}"
        );
    }
    for forbidden in ["HubPanel {", "PanelSlot {"] {
        assert!(
            !hub_section.contains(forbidden),
            "HubSection must stay a lightweight in-panel section and avoid nested panel chrome: {forbidden}"
        );
    }
    for (name, source) in [
        ("PanelHeader", panel_header),
        ("StatusBanner", status_banner),
        ("HubSection", hub_section),
    ] {
        assert!(
            !source.lines().any(|line| line.trim() == "Text {"),
            "{name} title typography should not return to a raw Text element"
        );
        for forbidden in ["font-size:", "font-weight:", "font_size:", "font_weight:"] {
            assert!(
                !source.contains(forbidden),
                "{name} title typography should not return to raw Text font bindings: {forbidden}"
            );
        }
    }
}
