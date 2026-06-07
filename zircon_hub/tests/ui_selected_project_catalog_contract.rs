//! Static contracts for React/MUI selected-project catalog scope.

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
            "{source_name} should contain selected-project catalog snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete selected-project catalog snippet {snippet:?}"
        );
    }
}

#[test]
fn tauri_runtime_refreshes_catalogs_from_selected_project_and_source_engine_roots() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let scoped_views = read_crate_file("src/tauri_app/runtime_state/scoped_views.rs");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "mod scoped_views;",
            "refresh_source_scoped_views",
            "refresh_selected_project_scoped_views",
            "refresh_project_context_views",
            "self.refresh_source_scoped_views()",
            "self.refresh_selected_project_scoped_views()",
        ],
    );
    assert_contains_all(
        "runtime_state/scoped_views.rs",
        &scoped_views,
        &[
            "discover_asset_catalog_for_scope",
            "discover_learn_catalog_for_scope",
            "discover_plugin_catalog_with_project_roots",
            "fn refresh_asset_catalog(&mut self) -> Result<(), HubError>",
            "self.asset_catalog = discover_asset_catalog_for_scope(",
            "self.selected_project_catalog_root()",
            "self.config",
            "recent_projects",
            "fn refresh_learn_catalog(&mut self) -> Result<(), HubError>",
            "self.learn_catalog = discover_learn_catalog_for_scope(",
            "fn refresh_plugin_catalog(&mut self) -> Result<(), HubError>",
            "self.plugin_catalog = discover_plugin_catalog_with_project_roots(",
            "self.selected_project_catalog_root().into_iter()",
            "fn refresh_team_overview(&mut self) -> Result<(), HubError>",
            "if let Some(project_root) = self.selected_project_catalog_root()",
            "self.source_engine_catalog_roots()",
            "fn selected_project_catalog_root(&self) -> Option<PathBuf>",
            ".selected_project()",
            "fn source_engine_catalog_roots(&self) -> Vec<PathBuf>",
            "scope.source_engine.engine_id()",
            "push_development_roots(&mut roots, engine.source_dir.clone());",
        ],
    );
}

#[test]
fn discovery_modules_prioritize_selected_project_scope_before_engine_scope() {
    let assets = read_crate_file("src/assets/catalog.rs");
    let plugins = read_crate_file("src/plugins/catalog.rs");
    let learn = read_crate_file("src/learn/catalog.rs");

    assert_contains_all(
        "assets/catalog.rs",
        &assets,
        &[
            "pub const SELECTED_PROJECT_ASSET_SOURCE: &str = \"Selected Project\";",
            "pub const PROJECT_ASSET_SOURCE: &str = \"Project\";",
            "(\"Editor\", &[\"zircon_editor\", \"assets\"])",
            "(\"Runtime\", &[\"zircon_runtime\", \"assets\"])",
            "pub fn discover_asset_catalog_for_scope",
            "collect_project_asset_roots(",
            "SELECTED_PROJECT_ASSET_SOURCE",
            "struct RankedAssetCatalogEntry",
            "root_rank",
            "source_priority(&left.entry.source)",
            ".then_with(|| left.root_rank.cmp(&right.root_rank))",
            "SELECTED_PROJECT_ASSET_SOURCE => 0",
            "PROJECT_ASSET_SOURCE => 1",
            "project_filesystem_path_key(root)",
            "discover_asset_catalog_keeps_first_source_engine_root_before_fallback_limit",
        ],
    );
    assert_contains_all(
        "plugins/catalog.rs",
        &plugins,
        &[
            "pub const PROJECT_PLUGIN_SCOPE: &str = \"Project\";",
            "pub const ENGINE_PLUGIN_SCOPE: &str = \"Engine\";",
            "pub fn discover_plugin_catalog_with_project_roots",
            "collect_project_plugin_manifests",
            "scope_rank(&left.scope)",
            "PROJECT_PLUGIN_SCOPE => 0",
            "ENGINE_PLUGIN_SCOPE => 1",
            "project_filesystem_path_key(&manifest_path)",
        ],
    );
    assert_contains_all(
        "learn/catalog.rs",
        &learn,
        &[
            "pub const SELECTED_PROJECT_LEARN_SOURCE: &str = \"Selected Project\";",
            "pub const SOURCE_ENGINE_LEARN_SOURCE: &str = \"Source Engine\";",
            "pub fn discover_learn_catalog_for_scope",
            "collect_docs_root(",
            "SELECTED_PROJECT_LEARN_SOURCE",
            "struct RankedLearnCatalogEntry",
            "root_rank",
            "source_priority(&left.entry.source)",
            ".then_with(|| left.root_rank.cmp(&right.root_rank))",
            "SELECTED_PROJECT_LEARN_SOURCE => 0",
            "SOURCE_ENGINE_LEARN_SOURCE => 1",
            "project_filesystem_path_key(&docs_root)",
            "discover_learn_catalog_keeps_first_source_engine_root_before_fallback_limit",
        ],
    );
}

#[test]
fn tauri_view_model_exposes_catalog_scope_dtos_to_react() {
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let catalog_dto = read_crate_file("src/tauri_app/view_model/catalog.rs");
    let learn_actions = read_crate_file("src/tauri_app/runtime_state/learn_actions.rs");
    let types = read_crate_file("web/src/types/hub.ts");

    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub assets: Vec<HubAssetItem>",
            "pub plugins: Vec<HubPluginItem>",
            "pub learn_resources: Vec<HubLearnItem>",
            "pub team: HubTeamSummary",
            "assets: asset_rows(snapshot)",
            "plugins: plugin_rows(snapshot)",
            "learn_resources: learn_rows(snapshot)",
            "team: team_summary(&snapshot.team, snapshot.settings.language)",
            "fn team_summary(team: &TeamOverview, language: HubLanguage) -> HubTeamSummary",
            "pub detail: String",
            "pub source: String",
            "pub source_key: String",
            "pub category_key: String",
            "pub scope: String",
            "pub scope_key: String",
            "pub editor_scoped: bool",
        ],
    );
    assert_contains_all(
        "view_model/catalog.rs",
        &catalog_dto,
        &[
            "pub(super) fn asset_rows(snapshot: &HubSnapshot) -> Vec<HubAssetItem>",
            "detail: asset_detail(&asset.kind, &path, language)",
            "source: localized_catalog_scope(&asset.source, language)",
            "source_key: catalog_scope_key(&asset.source).to_string()",
            "pub(super) fn plugin_rows(snapshot: &HubSnapshot) -> Vec<HubPluginItem>",
            "maturity_tone: plugin_maturity_tone(&plugin.maturity).to_string()",
            "scope: localized_catalog_scope(&plugin.scope, language)",
            "scope_key: catalog_scope_key(&plugin.scope).to_string()",
            "editor_scoped: plugin.editor_scoped",
            "pub(super) fn learn_rows(snapshot: &HubSnapshot) -> Vec<HubLearnItem>",
            "source: localized_catalog_scope(&resource.source, language)",
            "category_key: catalog_category_key(&resource.category).to_string()",
            "source_key: catalog_scope_key(&resource.source).to_string()",
            "path_text_en(&asset.path)",
            "path_text_en(&plugin.manifest_path)",
            "path_text_en(&resource.path)",
            "fn asset_detail(kind: &str, path: &str, language: HubLanguage) -> String",
            "fn localized_catalog_scope(scope: &str, language: HubLanguage) -> String",
            "fn catalog_scope_key(scope: &str) -> &'static str",
            "scope == \"Editor\"",
            "scope == \"Runtime\"",
            "fn catalog_category_key(category: &str) -> &'static str",
        ],
    );
    assert_contains_all(
        "runtime_state/learn_actions.rs",
        &learn_actions,
        &[
            "fn open_resource_targets(",
            "push_unique_resource_target(&mut targets",
            "targets",
            ".iter()",
            "path == target.as_str() || resource.title == target.as_str()",
            "Resource is not present in the current Learn catalog",
            "open_resource_payload_path_can_identify_catalog_entry_when_resource_id_is_stale",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubAssetItem",
            "detail: string;",
            "source: string;",
            "sourceKey: string;",
            "path: string;",
            "export interface HubPluginItem",
            "maturityTone: StatusTone;",
            "scope: string;",
            "scopeKey: string;",
            "editorScoped: boolean;",
            "manifestPath: string;",
            "packageRoot: string;",
            "export interface HubLearnItem",
            "source: string;",
            "sourceKey: string;",
            "categoryKey: string;",
            "export interface HubTeamSummary",
            "repositoryPath: string;",
            "repositoryAvailable: boolean;",
        ],
    );
}

#[test]
fn editor_plugin_page_filters_by_stable_scope_flags_not_localized_copy() {
    let plugin_catalog = read_crate_file("src/plugins/catalog.rs");
    let catalog_dto = read_crate_file("src/tauri_app/view_model/catalog.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let data = read_crate_file("web/src/data/hubData.ts");
    let editor = read_crate_file("web/src/pages/EditorPage.tsx");
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "plugins/catalog.rs",
        &plugin_catalog,
        &[
            "pub editor_scoped: bool",
            "supported_targets: Vec<String>",
            "kind: Option<String>",
            "target_modes: Vec<String>",
            "capabilities: Vec<String>",
            "let editor_scoped = plugin_manifest_is_editor_scoped(&manifest);",
            "editor_scoped,",
            "fn plugin_manifest_is_editor_scoped(manifest: &PluginManifest) -> bool",
            "editor_host",
            "editor_scoped_manifest_does_not_depend_on_description_copy",
        ],
    );
    assert_contains_all(
        "view_model/catalog.rs",
        &catalog_dto,
        &["editor_scoped: plugin.editor_scoped"],
    );
    assert_contains_all("types/hub.ts", &types, &["editorScoped: boolean;"]);
    assert_contains_all(
        "hubData.ts",
        &data,
        &["editorScoped: true", "editorScoped: false"],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor,
        &[
            "state.plugins.filter((plugin) => plugin.editorScoped)",
            "editorPlugins.map((plugin)",
        ],
    );
    assert_not_contains_any(
        "EditorPage.tsx",
        &editor,
        &[
            "plugin.defaultPackaging.some((entry) => entry.toLowerCase().includes(\"editor\"))",
            "plugin.description.toLowerCase().includes(\"editor\")",
            "plugin.scope.toLowerCase().includes(\"editor\")",
        ],
    );
    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &["Editor plugin scope is a stable `editorScoped` DTO flag"],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &["Editor plugin scope is a stable `editorScoped` DTO flag"],
    );
}

#[test]
fn catalog_page_unifies_assets_plugins_learn_scope_copy_and_filters() {
    let catalog = read_crate_file("web/src/pages/CatalogPage.tsx");

    assert_contains_all(
        "CatalogPage.tsx",
        &catalog,
        &[
            "state.activePage === \"plugins\" || state.activePage === \"learn\" ? state.activePage : \"assets\"",
            "const [selectedRowId, setSelectedRowId] = useState<string | null>(null);",
            "const rows = useMemo(() => catalogRows(state, mode, text), [mode, state, text]);",
            "const visibleRows = useMemo(() => filterRows(rows, mode, tab, query), [mode, query, rows, tab]);",
            "const selectedRow = useMemo(() =>",
            "const selectedCandidates = visibleRows.length > 0 ? visibleRows : rows;",
            "return selectedCandidates.find((row) => row.id === selectedRowId) ?? selectedCandidates[0];",
            "const categoryCount = new Set(rows.map((row) => row.category)).size;",
            "const scopeCount = new Set(rows.map((row) => row.scope)).size;",
            "HubSearchField",
            "MetricCard label={text.scopes}",
            "HubTabs value={tab}",
            "HubPanel title={catalogPanelTitle(mode, text)}",
            "HubPanel title={text.selectedEntry}",
            "HubPanel title={text.catalogTree}",
            "HubPanel title={common.quickActions}",
            "HubPanel title={common.sourceEngines}",
            "quickActionProjectTargetPayload",
            "const project = state.selectedProject;",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "HubButton",
            "state.ui.actions.openResource",
            "HUB_ACTION.openResource",
            "onAction(HUB_ACTION.openResource, undefined, { resourceId: row.id, path: row.path })",
            "selected: selectedRow?.id === row.id",
            "onSelect={(item) => setSelectedRowId(item.id)}",
            "state.plugins.map((plugin) => ({",
            "scope: plugin.scope",
            "scopeKey: plugin.scopeKey",
            "path: plugin.manifestPath || plugin.packageRoot",
            "tone: plugin.maturityTone",
            "state.learnResources.map((resource) => ({",
            "scope: resource.source",
            "categoryKey: resource.categoryKey",
            "scopeKey: resource.sourceKey",
            "state.assets.map((asset) => ({",
            "detail: asset.detail",
            "scope: asset.source",
            "scopeKey: asset.sourceKey",
            "mode === \"learn\"",
            "row.categoryKey === tab",
            "row.scopeKey === \"project\"",
            "row.scopeKey === \"engine\"",
            "tone: asset.sourceKey === \"project\" ? \"success\" : \"neutral\"",
        ],
    );
    assert_not_contains_any(
        "CatalogPage.tsx",
        &catalog,
        &[
            "row.scope.toLowerCase().includes(\"project\")",
            "row.scope.toLowerCase().includes(\"engine\") || row.scope.toLowerCase().includes(\"source\")",
            "row.category.toLowerCase().includes(tab)",
            "asset.source.toLowerCase().includes(\"project\")",
            "detail: `${asset.kind} - ${asset.path}`",
            "detail: `${asset.kind} — ${asset.path}`",
            "plugin.maturity.toLowerCase().includes(\"stable\")",
        ],
    );
}

#[test]
fn team_cloud_and_build_pages_consume_selected_project_scope_data() {
    let team = read_crate_file("web/src/pages/TeamPage.tsx");
    let cloud = read_crate_file("web/src/pages/CloudPage.tsx");
    let builds = read_crate_file("web/src/pages/BuildsPage.tsx");

    assert_contains_all(
        "TeamPage.tsx",
        &team,
        &[
            "state.team.members.map((member)",
            "state.actionHistory.map((action)",
            "detail: state.team.repositoryPath",
            "children: state.team.members.map((member)",
            "HubPanel title={text.teamMembers}",
            "HubPanel title={text.repositoryIdentity}",
            "HubPanel title={text.teamTree}",
            "HubPanel title={common.sourceEngines}",
            "SourceEngineList engines={state.sourceEngines} emptyLabel={state.ui.shell.noSourceEngineRegistered} onSelect={(engine) => void onAction(HUB_ACTION.selectEngine, engine.id)}",
        ],
    );
    assert_contains_all(
        "CloudPage.tsx",
        &cloud,
        &[
            "const project = state.selectedProject;",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}",
            "{ id: \"project\", title: common.project, detail: workflowProject?.name ?? common.noProjectSelected }",
            "{ id: \"project-path\", title: common.path, detail: workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected }",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
            "HubSwitch checked={Boolean(workflowProject && (!(\"exists\" in workflowProject) || workflowProject.exists))} label={state.ui.editor.projectAvailable}",
            "detail={workflowProject ? workflowProjectPath(workflowProject) : common.noProjectSelected}",
            "HubCheckbox checked={state.settings.defaultDeviceInstallDir !== common.notConfigured}",
        ],
    );
    assert_not_contains_any(
        "CloudPage.tsx",
        &cloud,
        &[
            "const projectTarget = projectTargetPayload(project);",
            "undefined, projectTarget",
            "project?.path ?? common.noProjectSelected",
        ],
    );
    assert_contains_all(
        "BuildsPage.tsx",
        &builds,
        &[
            "const project = state.selectedProject;",
            "const workflowProjectTarget = workflowProjectTargetPayload(state);",
            "const workflowProject = workflowTargetProject(state);",
            "const quickActionProjectTarget = quickActionProjectTargetPayload(project);",
            "onClick={() => void onAction(HUB_ACTION.buildProject, undefined, workflowProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.packageProject, undefined, workflowProjectTarget)}",
            "onClick={() => void onAction(HUB_ACTION.installDevice, undefined, workflowProjectTarget)}",
            "meta: workflowProject?.name ?? common.noProjectSelected",
            "detail: workflowProject?.name ?? common.noSelectedProject",
            "void onAction(actionId, undefined, workflowProjectTarget);",
            "workflowProject ? (",
            "{ id: \"project\", title: workflowProject.name, detail: workflowProjectPath(workflowProject)",
            "EmptyStateBlock title={common.noProjectSelected} detail={text.noProjectSelectedDetail}",
            "QuickActions actions={state.quickActions} onAction={(action) => void onAction(action.id, undefined, quickActionProjectTarget)}",
        ],
    );
    assert_not_contains_any(
        "BuildsPage.tsx",
        &builds,
        &[
            "const projectTarget = projectTargetPayload(project);",
            "undefined, projectTarget",
            "meta: project?.name ?? common.noProjectSelected",
            "detail: project?.name ?? common.noSelectedProject",
        ],
    );
}

#[test]
fn selected_project_catalog_documentation_records_react_mui_contract_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/ui_selected_project_catalog_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test ui_selected_project_catalog_contract",
            "## Selected Project Catalog Contract Cutover",
            "React/MUI selected-project catalog scope",
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/runtime_state/learn_actions.rs",
            "src/tauri_app/view_model.rs",
            "src/assets/catalog.rs",
            "src/plugins/catalog.rs",
            "src/learn/catalog.rs",
            "Source Engine asset roots such as `Editor` and `Runtime`",
            "`sourceKey` to `engine`",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`ui_selected_project_catalog_contract.rs`",
            "React/MUI selected-project catalog scope",
            "selected project and Source Engine catalog discovery",
            "CatalogPage mode projection and scope filtering",
            "stale row id can fall back to the supplied catalog path",
            "asset row detail is projected by Rust/fallback DTOs instead of page-local punctuation",
            "Team, Cloud, and Builds selected-project-aware surfaces",
            "Builds and Cloud workflow buttons and target panels use `workflowProjectTargetPayload`",
            "QuickActions panels keep the selected-project-only payload path",
            "Catalog QuickActions forward the selected-project target payload",
            "Source Engine asset roots such as `Editor` and `Runtime`",
            "`sourceKey` is normalized to `engine`",
        ],
    );
}

#[test]
fn selected_project_catalog_contract_is_cut_over_to_react_sources() {
    let contract = read_crate_file("tests/ui_selected_project_catalog_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "ui_selected_project_catalog_contract.rs",
        &contract,
        &[
            "web/src/pages/CatalogPage.tsx",
            "web/src/pages/TeamPage.tsx",
            "web/src/pages/CloudPage.tsx",
            "web/src/pages/BuildsPage.tsx",
            "web/src/types/hub.ts",
            "src/tauri_app/runtime_state.rs",
            "src/tauri_app/runtime_state/learn_actions.rs",
            "src/tauri_app/view_model.rs",
            "src/assets/catalog.rs",
            "src/plugins/catalog.rs",
            "src/learn/catalog.rs",
        ],
    );
    assert_not_contains_any(
        "ui_selected_project_catalog_contract.rs",
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
