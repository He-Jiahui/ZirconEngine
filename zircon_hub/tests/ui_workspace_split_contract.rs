//! Static contracts for React/MUI workspace main/sidebar split geometry.

use std::{fs, path::PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repo_dir() -> PathBuf {
    crate_dir()
        .parent()
        .expect("zircon_hub crate should live under the repository root")
        .to_path_buf()
}

fn normalize_newlines(source: String) -> String {
    source.replace("\r\n", "\n")
}

fn read_crate_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(crate_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read Hub crate file {path}: {error}")),
    )
}

fn read_repo_file(path: &str) -> String {
    normalize_newlines(
        fs::read_to_string(repo_dir().join(path))
            .unwrap_or_else(|error| panic!("failed to read repository file {path}: {error}")),
    )
}

fn assert_contains_all(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            source.contains(snippet),
            "{source_name} should contain workspace-split snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete workspace-split snippet {snippet:?}"
        );
    }
}

#[test]
fn workspace_pages_share_main_sidebar_split_and_collapse_rule() {
    for (page, split_grid) in [
        (
            "ProjectsDashboard.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
        ),
        (
            "ProjectBrowserPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
        ),
        (
            "ProjectDetailPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.4fr)\"",
        ),
        (
            "EditorPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
        ),
        (
            "BuildsPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
        ),
        (
            "CatalogPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
        ),
        (
            "CloudPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
        ),
        (
            "TeamPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
        ),
        (
            "SettingsPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.42fr)\"",
        ),
        (
            "WorkspacePage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(
            page,
            &source,
            &[
                "display: \"grid\"",
                split_grid,
                "gap: 1.4",
                "@media (max-width: 1180px)",
                "gridTemplateColumns: \"1fr\"",
                "HubPanel",
            ],
        );
        assert_not_contains_any(
            page,
            &source,
            &[
                "HubWorkspaceSplitState",
                "workspace-split",
                "main-basis",
                "side-basis",
                "side-panel-min-width",
                "overview-min-width",
            ],
        );
    }
}

#[test]
fn split_pages_keep_main_work_and_sidebar_support_panels_separate() {
    for (page, main_panel, support_panels) in [
        (
            "ProjectsDashboard.tsx",
            "title={text.recentProjects}",
            vec!["HubPanel title={text.quickActions}"],
        ),
        (
            "ProjectBrowserPage.tsx",
            "HubPanel title={text.allProjects}",
            vec![
                "HubPanel title={text.quickActions}",
                "HubPanel title={text.sourceEngines}",
            ],
        ),
        (
            "ProjectDetailPage.tsx",
            "HubPanel title={text.projectOverview}",
            vec![
                "HubPanel title={text.quickActions}",
                "HubPanel title={text.sourceEngines}",
                "HubPanel title={text.package}",
            ],
        ),
        (
            "EditorPage.tsx",
            "HubPanel title={text.launchTarget}",
            vec![
                "HubPanel title={common.sourceEngines}",
                "HubPanel title={common.quickActions}",
                "HubPanel title={text.workspaceTree}",
            ],
        ),
        (
            "BuildsPage.tsx",
            "HubPanel title={text.buildWorkflow}",
            vec![
                "HubPanel title={common.selectedProject}",
                "HubPanel title={common.sourceEngines}",
                "HubPanel title={text.outputTree}",
            ],
        ),
        (
            "CatalogPage.tsx",
            "HubPanel title={catalogPanelTitle(mode, text)}",
            vec![
                "HubPanel title={text.selectedEntry}",
                "HubPanel title={text.catalogTree}",
                "HubPanel title={common.sourceEngines}",
            ],
        ),
        (
            "CloudPage.tsx",
            "HubPanel title={text.packageOutputs}",
            vec![
                "HubPanel title={text.packageTarget}",
                "HubPanel title={text.installReadiness}",
                "HubPanel title={text.currentStatus}",
            ],
        ),
        (
            "TeamPage.tsx",
            "HubPanel title={text.teamMembers}",
            vec![
                "HubPanel title={text.repositoryIdentity}",
                "HubPanel title={text.teamTree}",
                "HubPanel title={text.latestAction}",
            ],
        ),
        (
            "SettingsPage.tsx",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            vec![
                "HubPanel title={settingsText.configurationHealthPanel}",
                "HubPanel title={settingsText.activeSourceEnginePanel}",
            ],
        ),
        (
            "WorkspacePage.tsx",
            "HubPanel title={common.sourceEngines}",
            vec![
                "HubPanel title={settingsText.heading}",
                "HubPanel title={settingsText.advancedConfigurationPanel}",
                "HubPanel title={state.ui.editor.workspaceTree}",
            ],
        ),
    ] {
        let source = read_crate_file(&format!("web/src/pages/{page}"));
        assert_contains_all(page, &source, &[main_panel]);
        assert_contains_all(
            page,
            &source,
            &support_panels.into_iter().collect::<Vec<_>>(),
        );
    }
}

#[test]
fn settings_page_keeps_explicit_left_and_right_split_groups() {
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");

    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "<Box sx={{ minWidth: 0, display: \"grid\", gap: 1.4 }}>",
            "<Box sx={{ minWidth: 0, display: \"grid\", gap: 1.4, alignContent: \"start\" }}>",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            "HubPanel title={settingsText.configurationPathsPanel}",
            "HubPanel title={settingsText.pathDefaultsPanel}",
            "HubPanel title={settingsText.advancedConfigurationPanel}",
            "HubPanel title={settingsText.configurationHealthPanel}",
            "HubPanel title={settingsText.activeSourceEnginePanel}",
        ],
    );
}

#[test]
fn workspace_split_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_workspace_split_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_workspace_split_contract",
            "## Workspace Split Contract Cutover",
            "React/MUI workspace main/sidebar split geometry",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/pages/WorkspacePage.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_workspace_split_contract.rs`",
            "React/MUI workspace main/sidebar split geometry",
            "shared main/sidebar split grids, responsive collapse rule, and support-panel grouping",
        ],
    );
}

#[test]
fn workspace_split_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_workspace_split_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_workspace_split_contract.rs",
        &contract,
        &[
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/pages/WorkspacePage.tsx",
        ],
    );
    assert_not_contains_any(
        "ui_workspace_split_contract.rs",
        &contract,
        &[
            obsolete_ui_extension.as_str(),
            obsolete_reader.as_str(),
            obsolete_directory_helper.as_str(),
            old_app_path.as_str(),
            old_material_text.as_str(),
            old_taffy_name.as_str(),
        ],
    );
}
