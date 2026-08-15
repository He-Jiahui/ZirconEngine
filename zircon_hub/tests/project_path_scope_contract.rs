//! Static contracts for React/MUI Hub project path scope and path display.

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
            "{source_name} should contain project-path-scope snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete project-path-scope snippet {snippet:?}"
        );
    }
}

#[test]
fn project_modules_expose_one_shared_path_key_and_validation_surface() {
    let metadata = read_crate_file("src/projects/metadata.rs");
    let projects_mod = read_crate_file("src/projects/mod.rs");
    let validation = read_crate_file("src/projects/validation.rs");
    let create_request = read_crate_file("src/projects/create_project_request.rs");
    let editor_recent_sync = read_crate_file("src/projects/editor_recent_sync.rs");

    assert_contains_all(
        "metadata.rs",
        &metadata,
        &[
            "pub fn project_metadata_key(path: impl AsRef<Path>) -> String",
            "let mut text = path.as_ref().to_string_lossy().replace('\\\\', \"/\");",
            "looks_like_windows_drive_path(&text)",
            "pub fn project_filesystem_path_key(path: impl AsRef<Path>) -> String",
            ".canonicalize()",
            "project_metadata_key(resolved)",
            "pub fn project_paths_match(left: impl AsRef<Path>, right: impl AsRef<Path>) -> bool",
            "metadata_key_normalizes_separators_and_trailing_slashes",
            "project_paths_match_uses_metadata_key_normalization",
            "filesystem_path_key_canonicalizes_when_possible",
        ],
    );
    assert_contains_all(
        "projects/mod.rs",
        &projects_mod,
        &[
            "project_filesystem_path_key",
            "project_metadata_key",
            "project_paths_match",
            "validate_project_root",
            "ProjectValidation",
        ],
    );
    assert_contains_all(
        "validation.rs",
        &validation,
        &[
            "pub enum ProjectValidation",
            "Valid",
            "MissingRoot",
            "MissingManifest",
            "InvalidManifest",
            "pub fn validate_project_root(path: impl AsRef<Path>) -> ProjectValidation",
            "path.join(\"zircon-project.toml\").is_file()",
        ],
    );
    assert_contains_all(
        "create_project_request.rs",
        &create_request,
        &[
            "pub location: PathBuf",
            "pub fn validate_launch_fields(&self) -> Result<(), CreateProjectRequestError>",
            "project location is required",
            "pub fn target_root(&self) -> PathBuf",
            "self.location.join(&self.project_name)",
            "create_request_preserves_name_and_validates_launch_fields",
        ],
    );
    assert_contains_all(
        "editor_recent_sync.rs",
        &editor_recent_sync,
        &[
            "use super::metadata::{project_metadata_key, project_paths_match};",
            "pub last_project_path: Option<PathBuf>",
            ".any(|project| project_paths_match(&project.path, Path::new(path)))",
            "let key = project_metadata_key(&entry.path);",
        ],
    );
}

#[test]
fn catalogs_team_and_runtime_roots_share_filesystem_path_keys() {
    for (label, file) in [
        ("Assets catalog", "src/assets/catalog.rs"),
        ("Plugins catalog", "src/plugins/catalog.rs"),
        ("Learn catalog", "src/learn/catalog.rs"),
        ("Team Git discovery", "src/team/local_git.rs"),
    ] {
        let source = read_crate_file(file);
        assert_contains_all(
            label,
            &source,
            &[
                "use crate::projects::project_filesystem_path_key;",
                "project_filesystem_path_key(",
            ],
        );
        assert_not_contains_any(
            label,
            &source,
            &["fn normalized_path_key", "fn looks_like_windows_path"],
        );
    }

    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let scoped_views = read_crate_file("src/tauri_app/runtime_state/scoped_views.rs");
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "use crate::projects::{",
            "project_metadata_key",
            "project_paths_match",
            "mod scoped_views;",
            "fn refresh_project_context_views(",
            "self.refresh_source_scoped_views()",
            "self.refresh_selected_project_scoped_views()",
        ],
    );
    assert_contains_all(
        "runtime_state/scoped_views.rs",
        &scoped_views,
        &[
            "use crate::projects::project_filesystem_path_key;",
            "discover_asset_catalog_for_scope",
            "discover_learn_catalog_for_scope",
            "discover_plugin_catalog_with_project_roots",
            "discover_team_overview",
            "fn selected_project_catalog_root(&self) -> Option<PathBuf>",
            ".selected_project()",
            "fn source_engine_catalog_roots(&self) -> Vec<PathBuf>",
            "push_development_roots(&mut roots, engine.source_dir.clone());",
            "fn push_unique_root(roots: &mut Vec<PathBuf>, path: PathBuf)",
            "let candidate_key = project_filesystem_path_key(&path);",
            "project_filesystem_path_key(root) == candidate_key",
            "fn push_development_roots(roots: &mut Vec<PathBuf>, source_dir: PathBuf)",
            "fn compiled_repo_root() -> Option<PathBuf>",
        ],
    );
}

#[test]
fn tauri_runtime_selected_project_paths_use_shared_matching() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let quick_actions = read_crate_file("src/tauri_app/runtime_state/quick_actions.rs");
    let editor_launch = read_crate_file("src/tauri_app/runtime_state/editor_launch_actions.rs");
    let delivery_actions =
        read_crate_file("src/tauri_app/runtime_state/project_delivery_actions.rs");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "fn selected_recent_project(&mut self) -> Option<RecentProject>",
            "let selected_path = self.selected_project_path.clone()?;",
            ".find(|project| project_paths_match(&project.path, &selected_path))",
            "self.selected_project_path = Some(project.path.clone());",
            "self.selected_project_path = None;",
            "fn find_recent_project(&self, target: &str) -> Option<RecentProject>",
            "let target_key = project_metadata_key(target);",
            "project_paths_match(&project.path, target)",
            "project_metadata_key(&project.path) == target_key",
            "fn startup_selected_project_path(",
            ".find(|project| project_paths_match(&project.path, path))",
            ".find(|project| project_paths_match(&project.path, last_project_path))",
        ],
    );
    assert_contains_all(
        "runtime_state/quick_actions.rs",
        &quick_actions,
        &[
            "action_target_for_project_failure",
            "\"Project\".to_string()",
        ],
    );
    assert_not_contains_any(
        "runtime_state/quick_actions.rs",
        &quick_actions,
        &["self.selected_project_path != selected_before"],
    );
    assert_contains_all(
        "runtime_state/editor_launch_actions.rs",
        &editor_launch,
        &[
            "self.activate_project_engine_for_path(&project.path);",
            "selected_project_path_changed(",
            "fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool",
            "(Some(before), Some(after)) => !project_paths_match(before, after)",
        ],
    );
    assert_contains_all(
        "runtime_state/project_delivery_actions.rs",
        &delivery_actions,
        &[
            "selected_or_latest_recent_project_for_named_action",
            "validate_project_root(&project.path)",
            "HubMessageId::Project(ProjectMessageId::RootInvalid)",
        ],
    );
}

#[test]
fn view_model_and_types_project_paths_as_display_dtos() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let display = read_crate_file("src/tauri_app/view_model/display.rs");
    let action_history = read_crate_file("src/tauri_app/view_model/action_history.rs");
    let settings_dto = read_crate_file("src/tauri_app/view_model/settings_dto.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub path: String",
            "pub location: String",
            "pub source_path: String",
            "pub output_path: String",
            "path: path_text(&project.path, language)",
            "location: summary.path",
            "path: path_text(path, snapshot.settings.language)",
            "source_path: path_text(&engine.source_dir, snapshot.settings.language)",
            "output_path: path_text(&engine.output_dir, snapshot.settings.language)",
            "repository_path: path_text(&team.repository_path, language)",
            "repository_available: !team.repository_path.as_os_str().is_empty()",
        ],
    );
    assert_contains_all(
        "view_model/display.rs",
        &display,
        &[
            "pub(crate) fn path_text(path: &Path, language: HubLanguage) -> String",
            ".pair(\"Not configured\", \"未配置\")",
            "pub(crate) fn path_text_en(path: &Path) -> String",
        ],
    );
    assert_contains_all(
        "action_history.rs",
        &action_history,
        &[
            "let output_dir = record.output_dir.as_deref().map(path_text_en);",
            "output_dir,",
        ],
    );
    assert_contains_all(
        "settings_dto.rs",
        &settings_dto,
        &[
            "pub default_project_dir: String",
            "pub default_build_output_dir: String",
            "pub default_device_install_dir: String",
            "default_project_dir: path_text(&settings.default_project_dir, settings.language)",
            "default_build_output_dir: path_text(",
            "default_device_install_dir: path_text(",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "path: string;",
            "location: string;",
            "sourcePath: string;",
            "outputPath: string;",
            "repositoryPath: string;",
            "repositoryAvailable: boolean;",
            "outputDir: string | null;",
            "defaultProjectDir: string;",
            "defaultBuildOutputDir: string;",
            "defaultDeviceInstallDir: string;",
        ],
    );
}

#[test]
fn react_components_display_paths_from_dtos_without_path_normalization_helpers() {
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");
    let create_dialog = read_crate_file("web/src/components/overlays/CreateProjectDialog.tsx");
    let browser = read_crate_file("web/src/pages/ProjectBrowserPage.tsx");
    let detail = read_crate_file("web/src/pages/ProjectDetailPage.tsx");
    let metrics = read_crate_file("web/src/components/data/ProjectMetricsGrid.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let settings = read_crate_file("web/src/pages/SettingsPage.tsx");
    let settings_section = read_crate_file("web/src/components/data/SettingsSection.tsx");
    let project_card = read_crate_file("web/src/components/data/ProjectCard.tsx");
    let project_table = read_crate_file("web/src/components/data/ProjectTable.tsx");
    let source_engine_list = read_crate_file("web/src/components/data/SourceEngineList.tsx");

    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const tableProjects = state.browserProjects.length > 0 ? state.browserProjects : state.recentProjects;",
            "return tableProjects.filter((project) => `${project.name} ${project.location}`.toLowerCase().includes(query));",
            "projects={visibleRows}",
            "<CreateProjectDialog",
        ],
    );
    assert_contains_all(
        "CreateProjectDialog.tsx",
        &create_dialog,
        &[
            "const [projectLocation, setProjectLocation] = useState(defaultProjectDir);",
            "setProjectLocation(defaultProjectDir);",
            "HubTextField label={text.location} value={projectLocation}",
        ],
    );
    assert_contains_all(
        "ProjectBrowserPage.tsx",
        &browser,
        &["`${project.name} ${project.location}`.toLowerCase().includes(query)"],
    );
    assert_contains_all(
        "ProjectDetailPage.tsx",
        &detail,
        &[
            "title: text.location, detail: project.path",
            "{ id: \"project-id\", title: text.projectId, detail: project.id }",
            "detail: project.path",
            "{project?.path ?? state.pageSubtitle}",
            "<ProjectMetricsGrid",
        ],
    );
    assert_contains_all(
        "ProjectMetricsGrid.tsx",
        &metrics,
        &["detail={project.exists ? text.ready : text.pathUnavailable}"],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "meta: state.settings.defaultBuildOutputDir",
            "meta: state.settings.defaultDeviceInstallDir",
            "{ id: \"project\", title: workflowProject.name, detail: workflowProjectPath(workflowProject), meta: \"status\" in workflowProject ? workflowProject.status : workflowProject.modified }",
            "{ id: \"output\", title: common.output, detail: state.settings.defaultBuildOutputDir }",
            "<HubList items={action.detailRows} />",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "secondaryDetail: action.outputDir ?? common.noOutputDirectory,",
            "{ id: \"package-root\", label: text.packageOutput, detail: state.settings.defaultBuildOutputDir }",
            "{ id: \"device-root\", label: text.deviceInstall, detail: state.settings.defaultDeviceInstallDir }",
            "{ id: \"project-path\", title: common.path, detail: workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected }",
            "detail={workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected}",
            "detail={state.settings.defaultDeviceInstallDir}",
        ],
    );
    assert_contains_all(
        "SettingsPage.tsx",
        &settings,
        &[
            "{ id: \"projects\", label: labels.defaultProjectDir, detail: draft.defaultProjectDir }",
            "<SettingsSection",
        ],
    );
    assert_contains_all(
        "SettingsSection.tsx",
        &settings_section,
        &[
            "label={labels.defaultProjectDir}",
            "label={labels.defaultBuildOutputDir}",
            "label={labels.defaultDeviceInstallDir}",
        ],
    );
    assert_contains_all("ProjectCard.tsx", &project_card, &["{project.path}"]);
    assert_contains_all(
        "ProjectTable.tsx",
        &project_table,
        &[
            "<HeaderCell>{labels.location}</HeaderCell>",
            "<BodyCell>{project.location}</BodyCell>",
        ],
    );
    assert_contains_all(
        "SourceEngineList.tsx",
        &source_engine_list,
        &["{engine.sourcePath}", "{engine.outputPath}"],
    );

    for (name, source) in [
        ("ProjectsDashboard.tsx", &dashboard),
        ("ProjectBrowserPage.tsx", &browser),
        ("ProjectDetailPage.tsx", &detail),
        ("BuildsPage.tsx", &builds),
        ("CloudPage.tsx", &cloud),
        ("SettingsPage.tsx", &settings),
    ] {
        assert_not_contains_any(
            name,
            source,
            &[
                "project_metadata_key",
                "project_paths_match",
                "project_filesystem_path_key",
            ],
        );
    }
}

#[test]
fn project_path_scope_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/project_path_scope_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test project_path_scope_contract",
            "## Project Path Scope Contract Cutover",
            "React/MUI project path scope and path display",
            "src/projects/metadata.rs",
            "src/projects/validation.rs",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/view_model.rs",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectTable.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_path_scope_contract.rs`",
            "React/MUI project path scope and path display",
            "Rust path keys and validation own normalization while React displays path DTO strings",
        ],
    );
}

#[test]
fn project_path_scope_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/project_path_scope_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "project_path_scope_contract.rs",
        &contract,
        &[
            "src/projects/metadata.rs",
            "src/projects/validation.rs",
            "src/projects/create_project_request.rs",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/view_model.rs",
            "web/src/pages/ProjectDetailPage.tsx",
            "web/src/components/data/ProjectTable.tsx",
        ],
    );
    assert_not_contains_any(
        "project_path_scope_contract.rs",
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
