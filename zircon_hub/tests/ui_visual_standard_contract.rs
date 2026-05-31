//! Static contracts for the Hub visual standard and reference design artifacts.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

struct HubVisualArtifact {
    design_png: &'static str,
    runtime_evidence: &'static str,
    responsive_evidence: &'static str,
}

const HUB_VISUAL_ARTIFACTS: &[HubVisualArtifact] = &[
    HubVisualArtifact {
        design_png: "hub-editor.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-editor.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-editor.png",
    },
    HubVisualArtifact {
        design_png: "hub-builds.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-builds.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-builds.png",
    },
    HubVisualArtifact {
        design_png: "hub-assets.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-assets.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-assets.png",
    },
    HubVisualArtifact {
        design_png: "hub-plugins.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-plugins.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-plugins.png",
    },
    HubVisualArtifact {
        design_png: "hub-cloud.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-cloud.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-cloud.png",
    },
    HubVisualArtifact {
        design_png: "hub-team.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-team.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-team.png",
    },
    HubVisualArtifact {
        design_png: "hub-learn.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-learn.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-learn.png",
    },
    HubVisualArtifact {
        design_png: "hub-settings.png",
        runtime_evidence: "target/hub-visual-check-final/main-pages/hub-settings.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-settings.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-new.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-new-project.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-new-project.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-browser.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-browser.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-detail.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-detail.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-detail.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-browser-filter-menu.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-browser-filter-menu.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser-filter-menu.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-browser-sort-menu.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-browser-sort-menu.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-browser-sort-menu.png",
    },
    HubVisualArtifact {
        design_png: "hub-projects-detail-delete-confirm.png",
        runtime_evidence: "target/hub-visual-check-final/hub-projects-detail-delete-confirm.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-detail-delete-confirm.png",
    },
    HubVisualArtifact {
        design_png: "hub-source-engine-popup.png",
        runtime_evidence: "target/hub-visual-check-final/popups/hub-source-engine-popup.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-source-engine-popup.png",
    },
    HubVisualArtifact {
        design_png: "hub-user-menu.png",
        runtime_evidence: "target/hub-visual-check-final/popups/hub-user-menu.png",
        responsive_evidence:
            "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/main-pages/hub-user-menu.png",
    },
    HubVisualArtifact {
        design_png: "hub-state-empty.png",
        runtime_evidence: "target/hub-visual-check-final/states/hub-state-empty.png",
        responsive_evidence:
            "Covered by shared `EmptyStateBlock`/`EmptyStatePanel` contracts and page-specific responsive captures",
    },
    HubVisualArtifact {
        design_png: "hub-state-loading.png",
        runtime_evidence: "target/hub-visual-check-final/states/hub-state-loading.png",
        responsive_evidence:
            "Covered by `TaskStatus::running_operation`, status/task contracts, and responsive Builds captures",
    },
    HubVisualArtifact {
        design_png: "hub-state-error.png",
        runtime_evidence: "target/hub-visual-check-final/states/hub-state-error.png",
        responsive_evidence:
            "Covered by `TaskStatus::error`, `StatusBanner`, and responsive guarded/error-flow captures",
    },
];

const HUB_SUPPLEMENTAL_DESIGN_ARTIFACTS: &[&str] = &[
    "hub-design-structure-layout.png",
    "hub-design-structure-supplement.png",
    "hub-design-functional-details.png",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub must live under the repository root")
        .to_path_buf()
}

fn ui_dir() -> PathBuf {
    crate_dir().join("ui")
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

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path)).unwrap_or_else(|error| {
            panic!("failed to read repository file {path}: {error}");
        }),
    )
}

#[test]
fn hub_visual_spec_is_exported_from_shared_tokens() {
    let tokens = read_ui_file("tokens.slint");
    let components = read_ui_file("components.slint");
    let theme = read_ui_file("theme.slint");

    assert!(
        components.contains("export { HubTokens, HubVisualSpec } from \"tokens.slint\";"),
        "components.slint must expose HubVisualSpec beside HubTokens"
    );

    let visual_spec = tokens
        .split("export global HubVisualSpec")
        .nth(1)
        .expect("tokens.slint must define HubVisualSpec");

    for snippet in [
        "reference-width: MaterialStyleMetrics.size_640 * 2 + MaterialStyleMetrics.size_256 + MaterialStyleMetrics.size_32;",
        "reference-height: MaterialStyleMetrics.size_640 + MaterialStyleMetrics.size_344 + MaterialStyleMetrics.spacing_16 + MaterialStyleMetrics.size_2 + MaterialStyleMetrics.size_1;",
        "panel-radius: MaterialStyleMetrics.border_radius_12;",
        "compact-radius: MaterialStyleMetrics.border_radius_8;",
        "badge-radius: MaterialStyleMetrics.border_radius_4;",
        "panel-padding: HubTokens.space-4;",
        "section-gap: HubTokens.panel-gap;",
        "toolbar-density-height: HubTokens.control-md;",
        "card-cover-height: HubTokens.list-row-lg + MaterialStyleMetrics.size_24 + MaterialStyleMetrics.size_2;",
        "visual-card-height: HubTokens.list-row-lg * 3 - MaterialStyleMetrics.spacing_8;",
        "visual-table-row-height: HubTokens.table-row;",
        "page-background: rgb(21, 21, 21);",
        "chrome-background: rgb(19, 19, 19);",
        "chrome-outline-muted: rgb(33, 33, 33);",
        "panel-background: rgb(32, 32, 32);",
        "panel-raised-background: rgb(34, 34, 34);",
        "panel-hover-background: rgb(40, 40, 40);",
        "outline-muted: rgb(56, 56, 56);",
        "outline-strong: rgb(135, 135, 135);",
        "accent-fill: MaterialPalette.primary_container;",
        "accent-stroke: MaterialPalette.primary;",
        "nav-active-fill: rgb(20, 55, 54);",
        "badge-accent-fill: rgb(28, 56, 56);",
        "badge-accent-stroke: rgb(42, 118, 121);",
        "success-stroke: rgb(112, 226, 128);",
        "warning-fill: MaterialPalette.tertiary_container;",
        "warning-stroke: MaterialPalette.tertiary;",
        "error-fill: MaterialPalette.error_container;",
        "error-stroke: MaterialPalette.error;",
        "button-state-primary-default-background: rgb(8, 119, 121);",
        "button-state-secondary-default-background: rgb(45, 47, 48);",
        "button-state-icon-primary-background: rgb(7, 122, 124);",
        "button-state-strip-background: rgb(22, 22, 22);",
    ] {
        assert!(
            visual_spec.contains(snippet),
            "HubVisualSpec must lock the reference visual token: {snippet}"
        );
    }

    for snippet in [
        "primary: rgb(52, 213, 208),",
        "tertiary: rgb(255, 202, 83),",
        "error: rgb(255, 180, 171),",
        "background: rgb(13, 17, 20),",
        "surfaceContainerLowest: rgb(8, 12, 15),",
        "surfaceContainerLow: rgb(22, 27, 30),",
        "surfaceContainer: rgb(27, 32, 35),",
        "surfaceContainerHigh: rgb(36, 42, 45),",
    ] {
        assert!(
            theme.contains(snippet),
            "ZirconMaterialTheme dark scheme must preserve the Hub reference palette: {snippet}"
        );
    }
}

#[test]
fn hub_shared_components_consume_visual_spec_tokens() {
    for (file_name, snippets) in [
        (
            "surfaces.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "border-radius: HubVisualSpec.panel-radius;",
                "border-radius: HubVisualSpec.badge-radius;",
                "HubVisualSpec.chrome-background",
                "HubVisualSpec.warning-fill",
                "HubVisualSpec.error-fill",
            ][..],
        ),
        (
            "inputs.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "box-height: HubVisualSpec.toolbar-density-height;",
                "select-height: HubVisualSpec.toolbar-density-height;",
            ],
        ),
        (
            "data_display.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "border-radius: HubVisualSpec.panel-radius;",
                "border-radius: HubVisualSpec.compact-radius;",
                "background: !root.enabled ? HubVisualSpec.panel-background",
            ],
        ),
        (
            "table_view_components.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "row-height: HubVisualSpec.visual-table-row-height;",
                "border-radius: HubVisualSpec.compact-radius;",
                "background: root.project.selected ? HubVisualSpec.accent-fill : transparent;",
            ],
        ),
        (
            "layout.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "background: HubVisualSpec.page-background;",
            ],
        ),
        (
            "shell_page_components.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "background: HubVisualSpec.page-background;",
                "background: HubVisualSpec.panel-background;",
            ],
        ),
        (
            "shell_sidebar_components.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "background: HubVisualSpec.chrome-background;",
                "border-radius: HubVisualSpec.panel-radius;",
            ],
        ),
        (
            "project_dashboard_components.slint",
            &[
                "HubVisualSpec,",
                "cover-height: HubVisualSpec.card-cover-height;",
                "card-height: HubVisualSpec.visual-card-height;",
                "border_radius: HubVisualSpec.panel-radius;",
                "HubVisualSpec.button-state-primary-default-background",
                "background: HubVisualSpec.button-state-strip-background;",
            ],
        ),
        (
            "project_browser_components.slint",
            &[
                "HubVisualSpec,",
                "select-height: HubVisualSpec.toolbar-density-height;",
                "border-radius: HubVisualSpec.panel-radius;",
                "background: root.project.selected ? MaterialPalette.secondary_container : HubVisualSpec.panel-background;",
            ],
        ),
        (
            "editor_page_components.slint",
            &[
                "HubVisualSpec,",
                "border-radius: HubVisualSpec.panel-radius;",
                "border-color: root.engine.active ? HubVisualSpec.accent-stroke : HubVisualSpec.outline-muted;",
            ],
        ),
        (
            "shell_header_components.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "pill.state == \"ok\" ? HubVisualSpec.success-stroke",
                "pill.state == \"warn\" ? HubVisualSpec.warning-stroke",
                "pill.state == \"error\" ? HubVisualSpec.error-stroke",
                "HubVisualSpec.outline-strong",
            ],
        ),
        (
            "shell_header_popup_components.slint",
            &[
                "import { HubTokens, HubVisualSpec } from \"tokens.slint\";",
                "border-radius: HubVisualSpec.panel-radius;",
                "border-color: HubVisualSpec.accent-stroke;",
            ],
        ),
        (
            "shared.slint",
            &[
                "HubVisualSpec",
                "border-radius: HubVisualSpec.compact-radius;",
                "border-radius: HubVisualSpec.panel-radius;",
                "background: HubVisualSpec.panel-background;",
            ],
        ),
    ] {
        let source = read_ui_file(file_name);
        for snippet in snippets {
            assert!(
                source.contains(snippet),
                "{file_name} must consume HubVisualSpec through shared visual components; missing {snippet}"
            );
        }
    }
}

#[test]
fn hub_pages_do_not_define_private_visual_literals() {
    let mut violations = Vec::new();

    for entry in fs::read_dir(ui_dir()).expect("failed to read Hub UI directory") {
        let path = entry
            .unwrap_or_else(|error| panic!("failed to read Hub UI directory entry: {error}"))
            .path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("slint") {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("Hub UI file name must be valid UTF-8");
        let color_literals_allowed = matches!(file_name, "theme.slint" | "tokens.slint");
        let source = normalize_newlines(
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display())),
        );

        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            let location = format!("{file_name}:{}", line_index + 1);

            if !color_literals_allowed
                && (trimmed.contains("rgb(")
                    || trimmed.contains("rgba(")
                    || contains_hex_color_literal(trimmed))
            {
                violations.push(format!(
                    "{location} defines a private color literal instead of HubVisualSpec or MaterialPalette: {trimmed}"
                ));
            }

            if let Some(value) = numeric_px_literal_after(trimmed, "border-radius:") {
                violations.push(format!(
                    "{location} defines private border-radius {value}; use HubVisualSpec radius tokens"
                ));
            }

            if let Some(value) = numeric_px_literal_after(trimmed, "row-height:") {
                violations.push(format!(
                    "{location} defines private row-height {value}; use HubVisualSpec.visual-table-row-height or HubTokens row tokens"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Hub pages must not bypass the shared visual standard:\n{}",
        violations.join("\n")
    );
}

#[test]
fn hub_root_pages_remain_shared_component_compositions() {
    for (file_name, root_export, required_snippets) in [
        (
            "project_dashboard.slint",
            "export component ProjectDashboardPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "DashboardToolbar {",
                "DashboardProjectCardsSection {",
                "DashboardRecentProjectsPanel {",
                "DashboardQuickActionsPanel {",
            ][..],
        ),
        (
            "project_new_page.slint",
            "export component ProjectNewPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "ProjectCreateSettingsPanel {",
                "ProjectCreateCompactSummaryPanel {",
                "ProjectTemplateRailPanel {",
            ][..],
        ),
        (
            "project_browser_page.slint",
            "export component ProjectBrowserPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "SearchBox {",
                "ProjectFilterSelect {",
                "ProjectSortSelect {",
                "ProjectBrowserResultsPanel {",
            ][..],
        ),
        (
            "project_detail_page.slint",
            "export component ProjectDetailPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "ProjectDetailStatusStrip {",
                "ProjectDetailInfoSection {",
                "ProjectDetailActionButton {",
                "ProjectDetailPinToggleRow {",
                "ProjectDetailEngineSection {",
            ][..],
        ),
        (
            "projects.slint",
            "export component ProjectsPage inherits Fill",
            &[
                "ProjectDashboardPage {",
                "ProjectNewPage {",
                "ProjectBrowserPage {",
                "ProjectDetailPage {",
            ][..],
        ),
        (
            "editor.slint",
            "export component EditorPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "EditorSourceSummaryPanel {",
                "EditorActionsPanel {",
                "EditorSourceSettingsPanel {",
                "EditorSourceEngineListPanel {",
                "EditorBuildHistoryPanel {",
            ][..],
        ),
        (
            "builds.slint",
            "export component BuildsPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "BuildSourceSummaryPanel {",
                "BuildControlsPanel {",
                "BuildPipelinePanel {",
                "BuildTaskHistoryPanel {",
                "OperationTimelinePanel {",
            ][..],
        ),
        (
            "assets.slint",
            "export component AssetsPage inherits CatalogPage",
            &["CatalogPage", "AssetRow {"][..],
        ),
        (
            "plugins.slint",
            "export component PluginsPage inherits CatalogPage",
            &["CatalogPage", "PluginRow {"][..],
        ),
        (
            "cloud.slint",
            "export component CloudPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "OverviewPanel {",
                "CloudMetricSlot {",
                "CloudPackageActionsPanel {",
                "CloudServicesPanel {",
            ][..],
        ),
        (
            "team.slint",
            "export component TeamPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "OverviewPanel {",
                "TeamSummarySlot {",
                "TeamMembersPanel {",
            ][..],
        ),
        (
            "learn.slint",
            "export component LearnPage inherits CatalogPage",
            &["CatalogPage", "LearnRow {"][..],
        ),
        (
            "settings.slint",
            "export component SettingsPage inherits PageScrollSurface",
            &[
                "background: HubVisualSpec.page-background;",
                "SettingsToolchainPanel {",
                "SettingsBuildDefaultsPanel {",
                "SettingsDefaultPathsPanel {",
                "SettingsConfigurationHealthPanel {",
                "SettingsSaveActionRow {",
                "OperationTimelinePanel {",
            ][..],
        ),
    ] {
        let source = read_ui_file(file_name);
        let exported_components = source
            .lines()
            .filter(|line| line.trim_start().starts_with("export component "))
            .count();
        assert_eq!(
            exported_components, 1,
            "{file_name} must expose one root page component and route page pieces through shared wrappers"
        );
        assert!(
            source.contains(root_export),
            "{file_name} must keep the expected root export: {root_export}"
        );

        for (line_index, line) in source.lines().enumerate() {
            let trimmed = line.trim_start();
            assert!(
                !trimmed.starts_with("component "),
                "{}:{} must not define local visual helper components; move page-specific visuals into shared component modules",
                file_name,
                line_index + 1
            );
        }

        for snippet in required_snippets {
            assert!(
                source.contains(snippet),
                "{file_name} must compose the shared Hub visual/page component {snippet}"
            );
        }
    }
}

#[test]
fn hub_design_png_artifacts_are_complete_and_reference_sized() {
    let dashboard_reference = repo_dir().join("docs/ui-and-layout/hub.png");
    assert!(
        dashboard_reference.exists(),
        "missing Projects Dashboard pixel reference {}",
        dashboard_reference.display()
    );
    let (width, height) = png_dimensions(&dashboard_reference);
    assert_eq!(
        (width, height),
        (1568, 1003),
        "{} must remain the Hub reference canvas",
        dashboard_reference.display()
    );
    let metadata = fs::metadata(&dashboard_reference).unwrap_or_else(|error| {
        panic!(
            "failed to stat Dashboard reference {}: {error}",
            dashboard_reference.display()
        )
    });
    assert!(
        metadata.len() > 100_000,
        "{} should remain the real Projects Dashboard visual reference, not a placeholder",
        dashboard_reference.display()
    );

    for artifact in HUB_VISUAL_ARTIFACTS {
        let path = repo_dir()
            .join("docs/ui-and-layout")
            .join(artifact.design_png);
        assert!(path.exists(), "missing Hub design PNG {}", path.display());
        let (width, height) = png_dimensions(&path);
        assert_eq!(
            (width, height),
            (1568, 1003),
            "{} must match the hub.png reference canvas",
            path.display()
        );
        let metadata = fs::metadata(&path)
            .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()));
        assert!(
            metadata.len() > 16_384,
            "{} should be a rendered design artifact, not an empty placeholder",
            path.display()
        );
    }

    let manifest = read_repo_file("docs/ui-and-layout/hub-ai-reference-manifest.json");
    for snippet in [
        "\"$schema\": \"docs/ui-and-layout/hub-ai-reference-manifest.schema.json\"",
        "\"source_reference\": \"docs/ui-and-layout/hub.png\"",
        "\"prompt_family\"",
        "\"draft_kind\": \"overall-interaction-structure-layout\"",
        "\"draft_usage\": \"Overall interaction structure layout drafts for review; local functional-content callouts are secondary; not acceptance evidence.\"",
        "\"ai_draft_root\": \"docs/ui-and-layout/hub-ai-drafts\"",
        "\"partial_images\": 2",
        "\"design_board_workflow\"",
        "\"source\": \"docs/ui-and-layout/hub-design-board/index.html\"",
        "\"export_command\": \"node docs/ui-and-layout/hub-design-board/export-design-board.mjs\"",
        "\"validate_command\": \"node docs/ui-and-layout/hub-design-board/validate-design-board.mjs\"",
        "\"final_source\": \"docs/ui-and-layout/hub-web-reference/index.html\"",
        "\"export_command\": \"node docs/ui-and-layout/hub-web-reference/export-pages.mjs\"",
        "\"acceptance_evidence\"",
        "\"ledger\": \"docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md\"",
        "\"final_png_inventory\": \"docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md#generated-final-png-inventory\"",
        "\"spot_checks\": \"docs/ui-and-layout/hub-web-reference/SPOT_CHECKS.md\"",
        "\"web_exports\": \"docs/ui-and-layout/hub-web-reference/EXPORTS.md\"",
        "\"visual_validation\": \"node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs\"",
        "\"interaction_validation\": \"node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs\"",
        "\"design_board_validation\": \"node docs/ui-and-layout/hub-design-board/validate-design-board.mjs\"",
        "\"rust_contract\": \"cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1 --test ui_visual_standard_contract\"",
        "\"known_limit\": \"AI drafts and design-board screenshots are not final acceptance evidence; optional cargo check may time out under concurrent Hub library compilation.\"",
        "\"width\": 1568",
        "\"height\": 1003",
        "\"selected_draft\": \"html-final-reference\"",
    ] {
        assert!(
            manifest.contains(snippet),
            "Hub AI reference manifest must describe the AI-directed HTML/CSS-finalized workflow; missing {snippet}"
        );
    }

    let manifest_schema =
        read_repo_file("docs/ui-and-layout/hub-ai-reference-manifest.schema.json");
    for snippet in [
        "\"$id\": \"docs/ui-and-layout/hub-ai-reference-manifest.schema.json\"",
        "\"additionalProperties\": false",
        "\"source_reference\"",
        "\"streaming_generation\"",
        "\"supplemental_design_mode_artifacts\"",
        "\"design_board_workflow\"",
        "\"acceptance_evidence\"",
        "\"references\"",
        "\"minItems\": 19",
        "\"maxItems\": 19",
        "\"minItems\": 3",
        "\"maxItems\": 3",
        "\"selected_draft\"",
        "\"const\": \"html-final-reference\"",
        "\"const\": \"docs/ui-and-layout/hub.png\"",
        "\"const\": 1568",
        "\"const\": 1003",
    ] {
        assert!(
            manifest_schema.contains(snippet),
            "Hub AI reference manifest schema must lock manifest structure; missing {snippet}"
        );
    }

    let exporter = read_repo_file("docs/ui-and-layout/hub-web-reference/export-pages.mjs");
    let visual_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-visuals.mjs");
    let page_registry = read_repo_file("docs/ui-and-layout/hub-web-reference/page-registry.mjs");
    let spot_checks = read_repo_file("docs/ui-and-layout/hub-web-reference/SPOT_CHECKS.md");
    let acceptance_evidence =
        read_repo_file("docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md");
    let design_board_registry =
        read_repo_file("docs/ui-and-layout/hub-design-board/board-registry.mjs");
    let design_board_manifest = read_repo_file("docs/ui-and-layout/hub-design-board/manifest.json");
    let design_board_exporter =
        read_repo_file("docs/ui-and-layout/hub-design-board/export-design-board.mjs");
    let design_board_validator =
        read_repo_file("docs/ui-and-layout/hub-design-board/validate-design-board.mjs");
    let design_board_review =
        read_repo_file("docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW.md");
    let design_board_coverage =
        read_repo_file("docs/ui-and-layout/hub-design-board/STRUCTURE_COVERAGE_MATRIX.md");
    for snippet in [
        "export const CANVAS_WIDTH = 1568;",
        "export const CANVAS_HEIGHT = 1003;",
        "hub-web-reference-1568x1003.png",
        "export const EXPORTS_LIST = [",
    ] {
        assert!(
            page_registry.contains(snippet),
            "Hub web reference page registry must own final canvas and export list; missing {snippet}"
        );
    }
    for snippet in [
        "CANVAS_WIDTH",
        "CANVAS_HEIGHT",
        "EXPORTS_LIST",
        "playwright",
        "screenshot",
        "--channel",
        "msedge",
        ".hub-shell",
        "ZIRCON_HUB_WEB_REFERENCE_PORT",
        "EADDRINUSE",
        "listenOn",
        "server.address",
        "isInsideRepo",
        "activePort",
    ] {
        assert!(
            exporter.contains(snippet),
            "Hub web reference exporter must own final PNG capture; missing {snippet}"
        );
    }
    for snippet in [
        "validateSpotChecks",
        "validateAcceptanceEvidence",
        "spotCheckArtifacts",
        "SPOT_CHECKS.md",
        "ACCEPTANCE_EVIDENCE.md",
        "AI-directed, HTML/CSS-finalized Hub reference PNGs",
        "19-page export validation",
        "against `docs/ui-and-layout/hub.png`",
        "Latest review: 2026-05-30",
        "no clipped text",
        "no overlap",
        "matching density",
        "row.result !== \"Pass\"",
        "must cite replay or validation evidence",
        "Web reference visual validation | Pass",
        "optional `cargo check` timeout",
        "validateJsonSchemaSubset",
        "additionalProperties === false",
        "schema.required",
        "schema.pattern",
        "schema.minItems",
        "schema.maxItems",
        "validateJsonSchemaSubsetSelfTest",
        "expectSchemaFailure",
        "rejects unknown top-level properties",
        "rejects missing reference output",
        "rejects short reference inventory",
        "rejects malformed AI draft path",
        "rejects wrong canvas width",
    ] {
        assert!(
            visual_validator.contains(snippet),
            "Hub web reference visual validator must lock representative spot-check coverage; missing {snippet}"
        );
    }
    for snippet in [
        "Hub Web Reference Spot Checks",
        "AI-directed, HTML/CSS-finalized Hub reference PNGs",
        "Latest review: 2026-05-30",
        "| Artifact | Page id | Inspect for | Result | Evidence |",
        "| --- | --- | --- | --- | --- |",
        "`hub.png` | `projects-dashboard`",
        "`hub-editor.png` | `hub-editor`",
        "`hub-assets.png` | `hub-assets`",
        "`hub-projects-detail-delete-confirm.png` | `hub-projects-detail-delete-confirm`",
        "`hub-source-engine-popup.png` | `hub-source-engine-popup`",
        "`hub-state-empty.png` | `hub-state-empty`",
        "`hub-state-error.png` | `hub-state-error`",
        "| Pass |",
        "index.html?page=hub-editor",
        "index.html?page=hub-assets",
        "index.html?page=hub-source-engine-popup",
        "validate-visuals.mjs",
    ] {
        assert!(
            spot_checks.contains(snippet),
            "Hub web-reference spot-check list must cover representative manual review rows; missing {snippet}"
        );
    }
    for snippet in [
        "Hub Web Reference Acceptance Evidence",
        "Date: 2026-05-30",
        "Canvas: `1568x1003`",
        "node docs/ui-and-layout/hub-web-reference/export-pages.mjs",
        "Default port `5198` falls back to a free local port",
        "explicit `ZIRCON_HUB_WEB_REFERENCE_PORT` remains strict",
        "node docs/ui-and-layout/hub-web-reference/validate-visuals.mjs",
        "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
        "docs/ui-and-layout/hub-ai-reference-manifest.schema.json",
        "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
        "cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1",
        "--test ui_visual_standard_contract",
        "Generated Final PNG Inventory",
        "Web reference export | Pass",
        "default port fallback and strict explicit-port failure were both exercised",
        "Web reference visual validation | Pass",
        "AI manifest schema, schema subset validation, negative schema self-tests",
        "Web reference interaction validation | Pass",
        "left no `zircon-hub-cdp-*` temp profile",
        "Design-board validation | Pass",
        "Focused Rust visual contract | Pass",
        "8 passed, 0 failed, 0 ignored",
        "AI drafts are direction records only",
        "not acceptance evidence for or against this visual-reference slice",
        "Accept only the final web-reference PNGs and their static validation package",
    ] {
        assert!(
            acceptance_evidence.contains(snippet),
            "Hub web-reference acceptance evidence must record the closeout gate; missing {snippet}"
        );
    }
    for artifact in HUB_VISUAL_ARTIFACTS {
        let snippet = format!("- `{}`", artifact.design_png);
        assert!(
            acceptance_evidence.contains(&snippet),
            "Hub web-reference acceptance evidence must list generated PNG {}; missing {snippet}",
            artifact.design_png
        );
    }
    for snippet in [
        "DESIGN_BOARD_SOURCE",
        "DESIGN_BOARD_LIST",
        "STRUCTURE_REVIEW_REQUIRED_TEXT",
        "STRUCTURE_COVERAGE_EXPECTED_ROWS",
        "hub-design-structure-layout.png",
        "hub-design-structure-supplement.png",
        "hub-design-functional-details.png",
    ] {
        assert!(
            design_board_registry.contains(snippet),
            "Hub design-board registry must own supplemental design-mode artifacts; missing {snippet}"
        );
    }
    for snippet in [
        "\"review_priority\": \"overall-interaction-structure-layout\"",
        "\"category\": \"primary-structure\"",
        "\"category\": \"secondary-functional-detail\"",
        "\"structure_review_checklist\": \"docs/ui-and-layout/hub-design-board/STRUCTURE_REVIEW.md\"",
    ] {
        assert!(
            design_board_manifest.contains(snippet),
            "Hub design-board manifest must describe structure-first review; missing {snippet}"
        );
    }
    for snippet in [
        "Microsoft/Edge/Application/msedge.exe",
        "remote-debugging-port",
        "Page.captureScreenshot",
        ".design-shell",
        "DESIGN_BOARD_LIST",
    ] {
        assert!(
            design_board_exporter.contains(snippet),
            "Hub design-board exporter must capture supplemental boards; missing {snippet}"
        );
    }
    for snippet in [
        "validateManifest",
        "validateStructureReview",
        "validateStructureCoverageMatrix",
        "validateBrowserBoards",
        "fixed canvas without scroll overflow",
    ] {
        assert!(
            design_board_validator.contains(snippet),
            "Hub design-board validator must enforce structure review contracts; missing {snippet}"
        );
    }
    for snippet in [
        "Overall Structure Checklist",
        "Functional Detail Checklist",
        "Acceptance Order",
        "Shell frame",
        "Overlay layer",
        "Responsive structure",
    ] {
        assert!(
            design_board_review.contains(snippet),
            "Hub design-board structure checklist must cover {snippet}"
        );
    }
    for snippet in [
        "Review item",
        "Primary artifact",
        "Secondary artifact",
        "Functional detail",
        "primary artifact",
    ] {
        assert!(
            design_board_coverage.contains(snippet),
            "Hub design-board coverage matrix must cover {snippet}"
        );
    }

    assert!(
        !repo_dir().join("tools/generate-hub-design-assets.py").exists(),
        "tools/generate-hub-design-assets.py must not remain the authoritative Hub design PNG generator"
    );

    let expected_artifacts = HUB_VISUAL_ARTIFACTS
        .iter()
        .map(|artifact| artifact.design_png)
        .chain(std::iter::once("hub.png"))
        .collect::<BTreeSet<_>>();
    let actual_artifacts = fs::read_dir(repo_dir().join("docs/ui-and-layout"))
        .expect("failed to read docs/ui-and-layout")
        .map(|entry| {
            entry
                .unwrap_or_else(|error| panic!("failed to read docs/ui-and-layout entry: {error}"))
                .path()
        })
        .filter_map(|path| {
            let file_name = path.file_name()?.to_str()?;
            (file_name == "hub.png"
                || (file_name.starts_with("hub-")
                    && !file_name.starts_with("hub-design-")
                    && file_name.ends_with(".png")))
            .then_some(file_name.to_owned())
        })
        .collect::<BTreeSet<_>>();
    let expected_artifacts = expected_artifacts
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual_artifacts, expected_artifacts,
        "docs/ui-and-layout must not contain orphaned Hub design PNGs; update HUB_VISUAL_ARTIFACTS, the generator, and the artifact matrix together"
    );

    let supplemental = HUB_SUPPLEMENTAL_DESIGN_ARTIFACTS
        .iter()
        .map(|artifact| artifact.to_owned())
        .collect::<BTreeSet<_>>();
    for artifact in supplemental {
        let path = repo_dir().join("docs/ui-and-layout").join(artifact);
        assert!(
            path.exists(),
            "missing supplemental Hub design-mode PNG {}",
            path.display()
        );
        let (width, height) = png_dimensions(&path);
        assert_eq!(
            (width, height),
            (1568, 1003),
            "{} must match the Hub structure review canvas",
            path.display()
        );
        let metadata = fs::metadata(&path)
            .unwrap_or_else(|error| panic!("failed to stat {}: {error}", path.display()));
        assert!(
            metadata.len() > 16_384,
            "{} should be a rendered supplemental design-mode artifact",
            path.display()
        );
    }
}

#[test]
fn hub_ai_reference_manifest_and_web_exporter_cover_artifacts() {
    let manifest = read_repo_file("docs/ui-and-layout/hub-ai-reference-manifest.json");
    let exporter = read_repo_file("docs/ui-and-layout/hub-web-reference/export-pages.mjs");
    let interaction_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-interactions.mjs");
    let page_registry = read_repo_file("docs/ui-and-layout/hub-web-reference/page-registry.mjs");
    let web_app = read_repo_file("docs/ui-and-layout/hub-web-reference/app.js");
    let web_styles = read_repo_file("docs/ui-and-layout/hub-web-reference/styles.css");
    let responsive_validator =
        read_repo_file("docs/ui-and-layout/hub-web-reference/validate-responsive.mjs");

    for artifact in HUB_VISUAL_ARTIFACTS {
        let page_id = artifact.design_png.trim_end_matches(".png");
        for snippet in [
            format!("\"page_id\": \"{page_id}\""),
            format!("\"output\": \"{}\"", artifact.design_png),
            format!("\"ai_draft\": \"docs/ui-and-layout/hub-ai-drafts/{page_id}.png\""),
        ] {
            assert!(
                manifest.contains(&snippet),
                "Hub AI manifest must cover {} with {snippet}",
                artifact.design_png
            );
        }
        for snippet in [
            format!("\"{page_id}\""),
            format!("\"{}\"", artifact.design_png),
        ] {
            assert!(
                page_registry.contains(&snippet),
                "Hub web page registry must capture {} with {snippet}",
                artifact.design_png
            );
        }
        assert!(
            web_app.contains(&format!("\"{page_id}\"")),
            "Hub web app must route the page id {page_id} for {}",
            artifact.design_png
        );
    }

    for snippet in [
        "EXPORTS_LIST",
        "selectedExports",
        "captureAll",
        "capture(pageId, target)",
    ] {
        assert!(
            exporter.contains(snippet),
            "Hub web reference exporter must consume the shared page registry; missing {snippet}"
        );
    }

    for snippet in [
        "knownPageIds",
        "[data-route]",
        "a[href*='?page=']",
        "readExportReplayRows",
        "replayPathToFileUrl",
        "output filename replay",
        "EXPORTS.md replay",
        "representative Hub web-reference click routes",
        "assertSafeTemporaryProfile",
        "taskkill",
        "zircon-hub-cdp-",
        "button.engine-select",
        ".quick-row:first-child",
    ] {
        assert!(
            interaction_validator.contains(snippet),
            "Hub web reference interaction validator must lock route/replay coverage; missing {snippet}"
        );
    }

    for snippet in [
        "projects-dashboard",
        "renderProjectsDashboard",
        "renderProjectBrowser",
        "renderProjectDetail",
        "renderSourceEnginePopup",
        "renderUserMenu",
        "renderState",
    ] {
        assert!(
            web_app.contains(snippet),
            "Hub web reference app must render the full reference workflow; missing {snippet}"
        );
    }

    for snippet in [
        ".browser-table {\n  --browser-table-columns: 42px minmax(0, 1fr) 86px 88px 90px 96px;",
        ".browser-head,\n.browser-row {\n  width: 100%;\n  min-width: 0;\n  display: grid;",
        "grid-template-columns: var(--browser-table-columns);",
        ".browser-head > span",
        "text-overflow: ellipsis;",
    ] {
        assert!(
            web_styles.contains(snippet),
            "Hub web reference styles must keep Project Browser header and rows on the same grid column model; missing {snippet}"
        );
    }

    for snippet in [
        "[\"layout-review-crop\", 1915, 508]",
        "const headCells = [...browserHead.children]",
        "for (const [rowIndex, browserRow] of browserRows.entries())",
        "browser row \" + rowIndex + \" column \" + index + \" is not aligned with the header",
        "browser row \" + rowIndex + \" column \" + index + \" width does not match the header",
    ] {
        assert!(
            responsive_validator.contains(snippet),
            "Hub web reference responsive validation must lock Browser table column alignment; missing {snippet}"
        );
    }
}

#[test]
fn hub_ai_structure_drafts_are_recorded_for_layout_review() {
    let manifest = read_repo_file("docs/ui-and-layout/hub-ai-reference-manifest.json");

    for snippet in [
        "structure-layout direction images only",
        "Overall interaction structure layout drafts for review",
        "local functional-content callouts are secondary",
        "not acceptance evidence",
        "Save the last streamed partial image",
        "supplemental_design_mode_artifacts",
        "design_board_workflow",
        "docs/ui-and-layout/hub-design-board/index.html",
        "docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
        "hub-design-structure-layout.png",
        "hub-design-structure-supplement.png",
        "hub-design-functional-details.png",
    ] {
        assert!(
            manifest.contains(snippet),
            "Hub AI manifest must keep the structure-first draft policy; missing {snippet}"
        );
    }

    for artifact in HUB_VISUAL_ARTIFACTS {
        let page_id = artifact.design_png.trim_end_matches(".png");
        let draft_path = repo_dir()
            .join("docs/ui-and-layout/hub-ai-drafts")
            .join(format!("{page_id}.png"));
        assert!(
            draft_path.exists(),
            "missing Hub AI structure-layout draft {}",
            draft_path.display()
        );
        let (width, height) = png_dimensions(&draft_path);
        assert_eq!(
            (width, height),
            (1024, 1024),
            "{} must be a lightweight structure-layout draft canvas",
            draft_path.display()
        );
        let metadata = fs::metadata(&draft_path)
            .unwrap_or_else(|error| panic!("failed to stat {}: {error}", draft_path.display()));
        assert!(
            metadata.len() > 16_384,
            "{} should be a real AI structure-layout draft, not an empty placeholder",
            draft_path.display()
        );
    }
}

#[test]
fn hub_visual_documentation_lists_reference_design_artifacts() {
    let ui_index = read_repo_file("docs/ui-and-layout/index.md");
    let hub_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    for snippet in [
        "Hub Visual Design Artifacts",
        "Hub Visual Artifact Matrix",
        "`hub.png` remains the Projects Dashboard pixel reference",
        "`docs/ui-and-layout/hub-ai-reference-manifest.json`",
        "`docs/ui-and-layout/hub-ai-reference-manifest.schema.json`",
        "schema subset validation",
        "negative schema self-tests",
        "`docs/ui-and-layout/hub-web-reference/export-pages.mjs`",
        "`docs/ui-and-layout/hub-web-reference/validate-interactions.mjs`",
        "falls back to a free local port",
        "`ZIRCON_HUB_WEB_REFERENCE_PORT`",
        "docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md",
        "generated 19-file final PNG inventory",
        "export port fallback evidence",
        "interaction temp-profile cleanup evidence",
        "known optional `cargo check` timeout",
        "AI-directed, HTML/CSS-finalized",
        "structure-layout direction drafts",
        "overall interaction structure",
        "local functional-content callouts",
        "hub-design-structure-layout.png",
        "hub-design-structure-supplement.png",
        "hub-design-functional-details.png",
        "docs/ui-and-layout/hub-design-board/index.html",
        "node docs/ui-and-layout/hub-design-board/export-design-board.mjs",
        "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
        "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
        "`1568x1003`",
        "`hub.png`",
        "Visual acceptance for Hub pages",
        "matching design PNG and runtime capture",
        "same component density",
        "no overlapping UI",
        "no clipped text or controls",
        "consistent button and badge states",
        "stable panel hierarchy",
        "shared empty/loading/error styling",
        "target/hub-visual-check-final/hub-projects-dashboard.png",
        "target/hub-visual-check-responsive-0529/{1600x1024,1280x900,1024x720}/hub-projects-dashboard.png",
    ] {
        assert!(
            ui_index.contains(snippet),
            "docs/ui-and-layout/index.md must list Hub visual design artifacts; missing {snippet}"
        );
    }

    for artifact in HUB_VISUAL_ARTIFACTS {
        let design_snippet = format!("`{}`", artifact.design_png);
        for snippet in [
            design_snippet.as_str(),
            artifact.runtime_evidence,
            artifact.responsive_evidence,
        ] {
            assert!(
                ui_index.contains(snippet),
                "docs/ui-and-layout/index.md must list {} with its runtime evidence; missing {snippet}",
                artifact.design_png
            );
        }
    }

    let matrix_rows = hub_visual_artifact_matrix_design_pngs(&ui_index);
    let matrix_row_set = matrix_rows.iter().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        matrix_rows.len(),
        matrix_row_set.len(),
        "docs/ui-and-layout/index.md Hub Visual Artifact Matrix must not contain duplicate design rows"
    );
    assert_eq!(
        matrix_row_set,
        hub_visual_documented_artifact_names(),
        "docs/ui-and-layout/index.md Hub Visual Artifact Matrix must exactly list hub.png plus HUB_VISUAL_ARTIFACTS"
    );

    for snippet in [
        "Hub Visual Consistency Contract",
        "HubVisualSpec",
        "docs/ui-and-layout/hub.png",
        "docs/ui-and-layout/hub-ai-reference-manifest.json",
        "docs/ui-and-layout/hub-ai-reference-manifest.schema.json",
        "docs/ui-and-layout/hub-web-reference/export-pages.mjs",
        "docs/ui-and-layout/hub-web-reference/validate-visuals.mjs",
        "docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
        "docs/ui-and-layout/hub-web-reference/ACCEPTANCE_EVIDENCE.md",
        "19 generated final `hub-*.png` files",
        "optional `cargo check` attempt timed out",
        "ui_visual_standard_contract",
        "real Hub captures remain runtime evidence",
        "AI-directed, HTML/CSS-finalized",
        "manifest schema contract",
        "schema subset validation",
        "negative schema self-tests",
        "falls back to a free local port",
        "ZIRCON_HUB_WEB_REFERENCE_PORT",
        "Export port policy",
        "interaction temporary profile cleanup",
        "no `zircon-hub-cdp-*` temp profile",
        "structure-layout direction drafts",
        "overall interaction structure",
        "local functional-content callouts",
        "hub-design-structure-layout.png",
        "hub-design-structure-supplement.png",
        "hub-design-functional-details.png",
        "docs/ui-and-layout/hub-design-board/index.html",
        "node docs/ui-and-layout/hub-design-board/export-design-board.mjs",
        "node docs/ui-and-layout/hub-design-board/validate-design-board.mjs",
        "node docs/ui-and-layout/hub-web-reference/validate-interactions.mjs",
        "not acceptance evidence",
        "artifact matrix",
        "target/hub-visual-check-responsive-0529/",
        "target/hub-visual-check-final/states/",
        "global-state design PNGs",
        "matching `1568x1003` runtime captures",
        "private `rgb()` branches",
        "root page composition",
        "layout/data/callback forwarding",
        "DashboardToolbar",
        "ProjectBrowserResultsPanel",
        "EditorActionsPanel",
        "BuildTaskHistoryPanel",
        "CloudServicesPanel",
        "TeamMembersPanel",
        "SettingsToolchainPanel",
        "OperationTimelinePanel",
        "HubVisualSpec.success-stroke",
        "TaskStatus::running_operation",
        "TaskStatus::error",
        "Manual visual acceptance remains stricter than file existence",
        "component density against the matching design PNG",
        "no overlapping UI",
        "no clipped text or controls",
        "consistent button and badge states",
        "stable panel hierarchy",
        "shared empty/loading/error styling",
    ] {
        assert!(
            hub_doc.contains(snippet),
            "responsive-component-system.md must document the visual contract; missing {snippet}"
        );
    }
}

fn hub_visual_documented_artifact_names() -> BTreeSet<String> {
    hub_visual_artifact_names()
        .into_iter()
        .chain(std::iter::once("hub.png".to_owned()))
        .collect()
}

fn hub_visual_artifact_matrix_design_pngs(source: &str) -> Vec<String> {
    let section = source
        .split("### Hub Visual Artifact Matrix")
        .nth(1)
        .expect("docs/ui-and-layout/index.md must contain the Hub Visual Artifact Matrix section");
    let mut rows = Vec::new();
    let mut in_table = false;

    for line in section.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("| ") {
            in_table = true;
            if let Some(value) = trimmed
                .split('|')
                .nth(1)
                .map(str::trim)
                .filter(|value| value.starts_with("`hub") && value.ends_with("`"))
            {
                rows.push(value.trim_matches('`').to_owned());
            }
        } else if in_table {
            break;
        }
    }

    rows
}

fn hub_visual_artifact_names() -> BTreeSet<String> {
    HUB_VISUAL_ARTIFACTS
        .iter()
        .map(|artifact| artifact.design_png.to_owned())
        .collect()
}

fn contains_hex_color_literal(line: &str) -> bool {
    let bytes = line.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] != b'#' {
            continue;
        }

        let mut digit_count = 0;
        while index + 1 + digit_count < bytes.len()
            && bytes[index + 1 + digit_count].is_ascii_hexdigit()
        {
            digit_count += 1;
        }

        if matches!(digit_count, 6 | 8)
            && bytes
                .get(index + 1 + digit_count)
                .is_none_or(|byte| !byte.is_ascii_hexdigit())
        {
            return true;
        }
    }

    false
}

fn numeric_px_literal_after<'a>(line: &'a str, property: &str) -> Option<&'a str> {
    let value = line.strip_prefix(property)?.trim();
    let token = value
        .trim_end_matches(';')
        .split_whitespace()
        .next()
        .unwrap_or_default();
    if token.len() > 2 && token.ends_with("px") && token[..token.len() - 2].parse::<f32>().is_ok() {
        Some(token)
    } else {
        None
    }
}

fn png_dimensions(path: &Path) -> (u32, u32) {
    let bytes = fs::read(path)
        .unwrap_or_else(|error| panic!("failed to read PNG {}: {error}", path.display()));
    assert!(
        bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "{} is not a PNG file",
        path.display()
    );
    let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    (width, height)
}
