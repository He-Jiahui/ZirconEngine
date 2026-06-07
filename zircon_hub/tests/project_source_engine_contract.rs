//! Static contracts for Hub Source Engine selection and registration workflow.

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
            "{source_name} should contain Source Engine snippet {snippet:?}"
        );
    }
}

fn assert_not_contains_any(source_name: &str, source: &str, snippets: &[&str]) {
    for snippet in snippets {
        assert!(
            !source.contains(snippet),
            "{source_name} should not contain obsolete Source Engine snippet {snippet:?}"
        );
    }
}

#[test]
fn new_project_default_engine_follows_active_source_context_in_tauri_session() {
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");
    let dashboard = read_crate_file("web/src/pages/ProjectsDashboard.tsx");

    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "fn sync_new_project_engine_after_active_engine_change(",
            "previous_active_engine_id: Option<&str>,",
            ".filter(|id| self.config.engines.iter().any(|engine| engine.id == *id));",
            "let current_is_valid = current",
            "let followed_previous_active =",
            "if current.is_none() || !current_is_valid || followed_previous_active {",
            "self.new_project_engine_id = active_engine_id;",
            "fn select_engine_by_id(&mut self, engine_id: &str) -> Result<(), HubError>",
            "let active_engine_before = self.config.active_engine_id.clone();",
            "self.config.active_engine_id = Some(engine.id.clone());",
            "self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());",
            "fn activate_project_engine_for_path(&mut self, path: &Path)",
            "self.sync_settings_from_active_engine();",
            "fn register_source_engine_from_settings(&mut self)",
            "let engine_id = source_engine_id(&source_dir);",
            "same_source_engine_path(&engine.source_dir, &source_dir)",
            "self.migrate_project_engine_metadata(&existing.id, &engine_id);",
            "upsert_source_engine(&mut self.config.engines, engine);",
            "ensure_active_source_engine(&self.config.engines, &mut self.config.active_engine_id);",
            "fn prune_stale_project_engine_bindings(&mut self) -> usize",
        ],
    );
    assert_contains_all(
        "ProjectsDashboard.tsx",
        &dashboard,
        &[
            "const [engineId, setEngineId] = useState(state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? \"\");",
            "state.sourceEngines.some((engine) => engine.id === currentEngineId)",
            "return state.activeSourceEngineId ?? state.sourceEngines[0]?.id ?? \"\";",
            "placeholder={text.sourceEngine}",
            "options={state.sourceEngines.map((engine) => ({",
            "detail: engine.sourcePath",
            "onChange={setEngineId}",
            "engineId: engineId || null",
        ],
    );
    assert_not_contains_any(
        "ProjectsDashboard.tsx",
        &dashboard,
        &["engineId: state.activeSourceEngineId,"],
    );
}

#[test]
fn source_engine_registration_uses_shared_filesystem_path_key() {
    let engines_mod = read_crate_file("src/engines/mod.rs");
    let paths = read_crate_file("src/engines/source_engine_paths.rs");
    let runtime_state = read_crate_file("src/tauri_app/runtime_state.rs");

    assert_contains_all(
        "engines/mod.rs",
        &engines_mod,
        &[
            "mod source_engine_paths;",
            "same_source_engine_path",
            "source_engine_display_name",
            "source_engine_id",
        ],
    );
    assert_contains_all(
        "source_engine_paths.rs",
        &paths,
        &[
            "use crate::projects::project_filesystem_path_key;",
            "pub fn source_engine_id(source_dir: &Path) -> String",
            "let key = source_engine_path_key(source_dir);",
            "for byte in key.bytes()",
            "pub fn same_source_engine_path(left: &Path, right: &Path) -> bool",
            "source_engine_path_key(left) == source_engine_path_key(right)",
            "fn source_engine_path_key(path: &Path) -> String",
            "project_filesystem_path_key(path)",
            "source_engine_paths_share_project_filesystem_key_normalization",
        ],
    );
    assert_not_contains_any(
        "source_engine_paths.rs",
        &paths,
        &[
            "replace('\\\\', \"/\")",
            "trim_end_matches('/')",
            "to_ascii_lowercase()",
        ],
    );
    assert_contains_all(
        "runtime_state.rs",
        &runtime_state,
        &[
            "same_source_engine_path",
            "source_engine_display_name",
            "source_engine_id",
            "register_source_engine_from_settings",
            "migrate_project_engine_metadata",
        ],
    );
}

#[test]
fn source_engine_registry_validation_history_and_dtos_are_current_owners() {
    let registry = read_crate_file("src/engines/registry.rs");
    let validation = read_crate_file("src/engines/validation.rs");
    let install = read_crate_file("src/engines/source_engine_install.rs");
    let view_model = read_crate_file("src/tauri_app/view_model.rs");
    let source_engine_dtos = read_crate_file("src/tauri_app/view_model/source_engines.rs");
    let types = read_crate_file("web/src/types/hub.ts");
    let source_engine_list = read_crate_file("web/src/components/data/SourceEngineList.tsx");

    assert_contains_all(
        "registry.rs",
        &registry,
        &[
            "pub fn active_source_engine",
            "pub fn active_source_engine_mut",
            "pub fn ensure_active_source_engine",
            "pub fn upsert_source_engine",
            "pub fn prune_project_engine_bindings",
            "pub fn remove_source_engine",
            "prune_project_engine_bindings_removes_stale_engine_ids",
        ],
    );
    assert_contains_all(
        "validation.rs",
        &validation,
        &[
            "pub enum SourceEngineValidation",
            "MissingRoot",
            "MissingWorkspaceManifest",
            "MissingBuildTool",
            "pub fn validate_source_engine(path: impl AsRef<Path>) -> SourceEngineValidation",
            "path.join(\"Cargo.toml\").is_file()",
            "path.join(\"tools\").join(\"zircon_build.py\").is_file()",
            "source_engine_validation_requires_manifest_and_build_tool",
        ],
    );
    assert_contains_all(
        "source_engine_install.rs",
        &install,
        &[
            "pub struct SourceBuildRecord",
            "pub struct SourceEngineInstall",
            "pub fn staged_engine_dir(&self) -> PathBuf",
            "pub fn record_build(&mut self, record: SourceBuildRecord)",
            "self.build_history.truncate(BUILD_HISTORY_LIMIT);",
            "record_build_keeps_newest_history_and_last_success",
        ],
    );
    assert_contains_all(
        "view_model.rs",
        &view_model,
        &[
            "pub source_engines: Vec<HubSourceEngineSummary>",
            "pub(crate) struct HubSourceBuildHistoryItem",
            "pub(crate) struct HubSourceEngineSummary",
            "pub secondary_detail: String",
            "pub source_path: String",
            "pub output_path: String",
            "pub build_history: Vec<HubSourceBuildHistoryItem>",
            "fn source_engine_rows(snapshot: &HubSnapshot) -> Vec<HubSourceEngineSummary>",
            "let text = HubTextBundle::new(snapshot.settings.language);",
            "text.pair(\"Active\", \"当前\")",
            "text.pair(\"Registered\", \"已注册\")",
            "source_path: path_text(&engine.source_dir, snapshot.settings.language)",
            "output_path: path_text(&engine.output_dir, snapshot.settings.language)",
        ],
    );
    assert_contains_all(
        "source_engines.rs",
        &source_engine_dtos,
        &[
            "pub(crate) fn source_build_history_rows(",
            "detail: text.status_detail(&record.detail)",
            "log_excerpt: text.status_detail(&record.log_excerpt)",
            "secondary_detail: source_build_history_secondary_detail(",
            "finished: relative_time(now_ms, record.finished_unix_ms, language)",
            "output_dir: path_text_en(&record.output_dir)",
            "status_tone(&record.status)",
        ],
    );
    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubSourceBuildHistoryItem",
            "export interface HubSourceEngineSummary",
            "sourcePath: string;",
            "outputPath: string;",
            "active: boolean;",
            "buildHistory: HubSourceBuildHistoryItem[];",
            "secondaryDetail: string;",
            "commandLine: string[];",
            "outputDir: string;",
        ],
    );
    assert_contains_all(
        "SourceEngineList.tsx",
        &source_engine_list,
        &[
            "engines: HubSourceEngineSummary[];",
            "emptyLabel: string;",
            "onSelect?: (engine: HubSourceEngineSummary) => void;",
            "{engine.sourcePath}",
            "{engine.outputPath}",
            "const hasSelectHandler = Boolean(onSelect);",
            "disabled={!hasSelectHandler}",
            "cursor: hasSelectHandler ? \"pointer\" : \"default\"",
            "onClick={() => onSelect?.(engine)}",
        ],
    );
}

#[test]
fn editor_page_renders_source_engine_build_history_from_backend_dtos() {
    let editor_page = read_crate_file("web/src/pages/EditorPage.tsx");
    let types = read_crate_file("web/src/types/hub.ts");
    let fallback = read_crate_file("web/src/data/hubData.ts");
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "types/hub.ts",
        &types,
        &[
            "export interface HubSourceBuildHistoryItem",
            "buildHistory: HubSourceBuildHistoryItem[];",
        ],
    );
    assert_contains_all(
        "EditorPage.tsx",
        &editor_page,
        &[
            "const sourceBuildHistory = useMemo(",
            "activeSourceEngine?.buildHistory ?? []",
            "void onAction(HUB_ACTION.openOutputFolder, undefined, { outputDir: activeSourceEngine?.outputPath })",
            "HubPanel title={text.sourceBuildHistory}",
            "sourceBuildHistory.map((record)",
            "title: record.detail",
            "detail: record.outputDir",
            "secondaryDetail: record.secondaryDetail",
            "meta: record.finished",
            "disabled: !record.outputDir",
            "const record = sourceBuildHistory.find((history) => history.id === item.id);",
            "void onAction(HUB_ACTION.openOutputFolder, item.id, { outputDir: record?.outputDir });",
        ],
    );
    assert_not_contains_any(
        "EditorPage.tsx",
        &editor_page,
        &[
            "void onAction(HUB_ACTION.openOutputFolder, undefined, { path: activeSourceEngine?.outputPath })",
            "onSelect={(item) => void onAction(HUB_ACTION.openOutputFolder, item.id, { path: item.detail })}",
        ],
    );
    assert_contains_all(
        "hubData.ts",
        &fallback,
        &[
            "buildHistory: [",
            "detail: \"已暂存编辑器/运行时载荷\"",
            "secondaryDetail: \"命令：python tools/zircon_build.py --targets editor,runtime；日志：编辑器/运行时目标已完成暂存。\"",
            "logExcerpt: \"编辑器/运行时目标已完成暂存。\"",
            "outputDir: \"E:\\\\Git\\\\ZirconEngine\\\\target\\\\zircon-hub\"",
        ],
    );
    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "Source Engine build-history DTOs",
            "backend `secondaryDetail` string",
            "command/log separator",
            "EditorPage",
            "open-output-folder",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "Source Engine build-history DTOs",
            "secondaryDetail",
            "backend/fallback DTO data",
            "`project_source_engine_contract.rs`",
        ],
    );
}

#[test]
fn source_engine_documentation_records_tauri_react_cutover() {
    let shell_doc = read_repo_file("docs/zircon_hub/ui/tauri-react-shell.md");
    let responsive_doc = read_repo_file("docs/zircon_hub/ui/responsive-component-system.md");

    assert_contains_all(
        "tauri-react-shell.md",
        &shell_doc,
        &[
            "zircon_hub/tests/project_source_engine_contract.rs",
            "cargo test --manifest-path zircon_hub/Cargo.toml --test project_source_engine_contract",
            "## Project Source Engine Contract Cutover",
            "React/MUI Source Engine registration and selection",
            "src/engines/source_engine_paths.rs",
            "src/engines/registry.rs",
            "src/tauri_app/runtime_state.rs",
            "web/src/components/data/SourceEngineList.tsx",
        ],
    );
    assert_contains_all(
        "responsive-component-system.md",
        &responsive_doc,
        &[
            "`project_source_engine_contract.rs`",
            "React/MUI Source Engine registration and selection",
            "SourceEngineList",
        ],
    );
}

#[test]
fn source_engine_contract_is_cut_over_to_current_tauri_react_sources() {
    let contract = read_crate_file("tests/project_source_engine_contract.rs");
    let obsolete_ui_extension = format!("{}{}", ".s", "lint");
    let obsolete_reader = format!("read_{}_file", "ui");
    let obsolete_directory_helper = format!("fn {}_dir", "ui");
    let old_app_path = ["src", "app"].join("/");
    let old_material_text = format!("Material{}", "Text");
    let old_taffy_name = format!("{}{}", "Taf", "fy");

    assert_contains_all(
        "project_source_engine_contract.rs",
        &contract,
        &[
            "src/engines/source_engine_paths.rs",
            "src/engines/registry.rs",
            "src/tauri_app/runtime_state.rs",
            "web/src/components/data/SourceEngineList.tsx",
        ],
    );
    assert_not_contains_any(
        "project_source_engine_contract.rs",
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
