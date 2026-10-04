//! Static contracts for React/MUI workspace main/sidebar split geometry.
//! 约束主工作区与辅助侧栏的响应式分栏关系及支持面板归属。

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

// 源码片段跨检出平台比较时统一换行；这里不会执行被检查的前端代码。
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

// 主内容和辅助侧栏在各工作区页面共享折叠断点。
#[test]
fn workspace_pages_share_main_sidebar_split_and_collapse_rule() {
    for (page, split_grid, content_snippet) in [
        (
            "ProjectsDashboard.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
            "HubPanel",
        ),
        (
            "ProjectBrowserPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(320px, 0.42fr)\"",
            "HubPanel",
        ),
        (
            "ProjectDetailPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.4fr)\"",
            "HubPanel",
        ),
        (
            "EditorPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            "HubPanel",
        ),
        (
            "BuildsPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            "HubPanel",
        ),
        (
            "CatalogPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            "HubPanel",
        ),
        (
            "CloudPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            "HubPanel",
        ),
        (
            "TeamPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.55fr)\"",
            "HubPanel",
        ),
        (
            "SettingsPage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.42fr)\"",
            "SettingsSection",
        ),
        (
            "WorkspacePage.tsx",
            "gridTemplateColumns: \"minmax(0, 1fr) minmax(330px, 0.58fr)\"",
            "HubPanel",
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
                content_snippet,
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

// 主任务与辅助面板可在页面或子组件内分开持有。
// 页面主区和支持面板可位于不同组件；这里按真实页面与侧栏消费文件分别审查。
#[test]
fn split_pages_keep_main_work_and_sidebar_support_panels_separate() {
    for (page, main_source_path, support_source_path, main_panel, support_panels) in [
        (
            "ProjectsDashboard.tsx",
            "web/src/pages/ProjectsDashboard.tsx",
            "web/src/pages/ProjectsDashboard.tsx",
            "title={text.recentProjects}",
            vec!["HubPanel title={text.quickActions}"],
        ),
        (
            "ProjectBrowserPage.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "web/src/pages/ProjectBrowserPage.tsx",
            "HubPanel title={text.allProjects}",
            vec![
                "HubPanel title={text.quickActions}",
                "HubPanel title={text.sourceEngines}",
            ],
        ),
        (
            "ProjectDetailPage.tsx",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectDetailSidebar.tsx",
            "HubPanel title={text.projectOverview}",
            vec![
                "HubPanel title={text.quickActions}",
                "HubPanel title={text.sourceEngines}",
                "HubPanel title={text.package}",
            ],
        ),
        (
            "EditorPage.tsx",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/EditorPage.tsx",
            "HubPanel title={text.launchTarget}",
            vec![
                "HubPanel title={common.sourceEngines}",
                "HubPanel title={common.quickActions}",
                "HubPanel title={text.workspaceTree}",
            ],
        ),
        (
            "BuildsPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "HubPanel title={text.buildWorkflow}",
            vec![
                "HubPanel title={common.selectedProject}",
                "HubPanel title={common.sourceEngines}",
                "HubPanel title={text.outputTree}",
            ],
        ),
        (
            "CatalogPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "HubPanel title={catalogPanelTitle(mode, text)}",
            vec![
                "HubPanel title={text.selectedEntry}",
                "HubPanel title={text.catalogTree}",
                "HubPanel title={common.sourceEngines}",
            ],
        ),
        (
            "CloudPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "HubPanel title={text.packageOutputs}",
            vec![
                "HubPanel title={text.packageTarget}",
                "HubPanel title={text.installReadiness}",
                "HubPanel title={text.currentStatus}",
            ],
        ),
        (
            "TeamPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "HubPanel title={text.teamMembers}",
            vec![
                "HubPanel title={text.repositoryIdentity}",
                "HubPanel title={text.teamTree}",
                "HubPanel title={text.latestAction}",
            ],
        ),
        (
            "SettingsPage.tsx",
            "web/src/components/data/SettingsSection.tsx",
            "web/src/components/data/SettingsSection.tsx",
            "HubPanel title={settingsText.buildDefaultsPanel}",
            vec![
                "HubPanel title={settingsText.configurationHealthPanel}",
                "HubPanel title={settingsText.activeSourceEnginePanel}",
            ],
        ),
        (
            "WorkspacePage.tsx",
            "web/src/pages/WorkspacePage.tsx",
            "web/src/pages/WorkspacePage.tsx",
            "HubPanel title={common.sourceEngines}",
            vec![
                "HubPanel title={settingsText.heading}",
                "HubPanel title={settingsText.advancedConfigurationPanel}",
                "HubPanel title={state.ui.editor.workspaceTree}",
            ],
        ),
    ] {
        let main_source = read_crate_file(main_source_path);
        let support_source = read_crate_file(support_source_path);
        assert_contains_all(page, &main_source, &[main_panel]);
        assert_contains_all(
            page,
            &support_source,
            &support_panels.into_iter().collect::<Vec<_>>(),
        );
    }
}

// 设置页两侧分组应维持明确的配置与状态信息层级。
#[test]
fn settings_section_keeps_explicit_left_and_right_split_groups() {
    let settings = read_crate_file("web/src/components/data/SettingsSection.tsx");

    assert_contains_all(
        "SettingsSection.tsx",
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

// 文档记录分栏规则和页面覆盖范围。
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
            "web/src/components/data/ProjectDetailSidebar.tsx",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/components/data/SettingsSection.tsx",
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

// 自检分栏契约仍观察现行工作区页面。
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
            "web/src/components/data/ProjectDetailSidebar.tsx",
            "web/src/pages/EditorPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/SettingsPage.tsx",
            "web/src/components/data/SettingsSection.tsx",
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
