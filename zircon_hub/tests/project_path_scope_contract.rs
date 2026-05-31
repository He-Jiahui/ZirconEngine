//! Static contracts for Hub selected-project path scope and refresh wiring.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .to_string_lossy()
            .into_owned()
    }))
}

fn read_crate_file(path: &str) -> String {
    fs::read_to_string(crate_dir().join(path))
        .map(|source| source.replace("\r\n", "\n"))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn selected_project_workflow_reuses_shared_path_keys() {
    let metadata = read_crate_file("src/projects/metadata.rs");
    for snippet in [
        "pub fn project_filesystem_path_key(path: impl AsRef<Path>) -> String",
        ".canonicalize()",
        "project_metadata_key(resolved)",
        "looks_like_windows_drive_path(&text)",
    ] {
        assert!(
            metadata.contains(snippet),
            "project metadata must expose one shared path-key rule for persisted and filesystem paths; missing {snippet}"
        );
    }

    let projects_mod = read_crate_file("src/projects/mod.rs");
    assert!(
        projects_mod.contains("project_filesystem_path_key"),
        "projects/mod.rs must re-export project_filesystem_path_key for Hub scoped catalogs"
    );

    for (label, file) in [
        ("Assets catalog", "src/assets/catalog.rs"),
        ("Plugins catalog", "src/plugins/catalog.rs"),
        ("Learn catalog", "src/learn/catalog.rs"),
        ("Team Git discovery", "src/team/local_git.rs"),
    ] {
        let source = read_crate_file(file);
        assert!(
            source.contains("use crate::projects::project_filesystem_path_key;"),
            "{label} must use the shared filesystem path key instead of a local slash/case helper"
        );
        assert!(
            source.contains("project_filesystem_path_key("),
            "{label} must apply the shared filesystem path key before deduplicating scanned roots"
        );
        assert!(
            !source.contains("fn normalized_path_key"),
            "{label} must not keep a private normalized_path_key helper that can drift from selected-project matching"
        );
    }
}

#[test]
fn team_projection_uses_project_metadata_key_for_repository_scope() {
    let team = read_crate_file("src/app/view_model/team.rs");

    assert!(
        team.contains("use crate::projects::project_metadata_key;"),
        "Team projection must use the same selected-project metadata key as runtime/project views"
    );
    for snippet in [
        "let project_key = project_metadata_key(project_path);",
        "let repository_key = project_metadata_key(repository_path);",
        "project_key.starts_with(&format!(\"{repository_key}/\"))",
        "repository_key.starts_with(&format!(\"{project_key}/\"))",
        "Selected project repository unavailable; showing Source Engine repository",
        "\"Source Engine repository\"",
        "fn team_summary_labels_source_engine_fallback_for_missing_selected_project_repository()",
    ] {
        assert!(
            team.contains(snippet),
            "Team selected-project repository matching must use shared metadata keys; missing {snippet}"
        );
    }
    for forbidden in [
        "fn normalized_path_key",
        "fn looks_like_windows_path",
        "\"Source engine repository\"",
    ] {
        assert!(
            !team.contains(forbidden),
            "Team projection should not keep local path-normalization helpers after shared key convergence: {forbidden}"
        );
    }
}

#[test]
fn runtime_scoped_roots_share_project_filesystem_path_key() {
    let root_paths = read_crate_file("src/app/runtime/root_paths.rs");
    for snippet in [
        "use crate::projects::project_filesystem_path_key;",
        "pub(super) fn push_unique_root",
        "pub(super) fn push_development_roots",
        "let candidate_key = project_filesystem_path_key(&path);",
        "project_filesystem_path_key(root) == candidate_key",
        "fn compiled_repo_root() -> Option<PathBuf>",
    ] {
        assert!(
            root_paths.contains(snippet),
            "runtime root collection must share canonical project filesystem path keys; missing {snippet}"
        );
    }

    let runtime = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime.contains("mod root_paths;"),
        "runtime.rs must register the focused root_paths module instead of keeping root-list helpers in page refresh modules"
    );

    let scoped_views = read_crate_file("src/app/runtime/source_scoped_views.rs");
    for snippet in [
        "use super::{root_paths::push_development_roots, HubRuntime};",
        "pub(super) fn selected_project_catalog_root(&self) -> Option<std::path::PathBuf>",
        "pub(super) fn source_engine_catalog_roots(&self) -> Vec<std::path::PathBuf>",
        "push_development_roots(&mut roots, engine.source_dir.clone());",
    ] {
        assert!(
            scoped_views.contains(snippet),
            "Scoped view roots should collect source/current/compiled roots once through shared runtime root logic; missing {snippet}"
        );
    }

    for (label, file) in [
        ("Asset refresh", "src/app/runtime/asset_catalog.rs"),
        ("Learn refresh", "src/app/runtime/learn_catalog.rs"),
        ("Plugin refresh", "src/app/runtime/plugin_catalog.rs"),
    ] {
        let source = read_crate_file(file);
        assert!(
            source.contains("self.source_engine_catalog_roots()"),
            "{label} should consume scope-derived engine roots instead of rebuilding development root fallback locally"
        );
        assert!(
            !source.contains("fn push_non_empty")
                && !source.contains("fn compiled_repo_root")
                && !source.contains("push_development_roots(")
                && !source.contains("roots.iter().any(|root| root == &path)"),
            "{label} must not keep direct PathBuf equality root de-duplication after root_paths convergence"
        );
    }

    let team = read_crate_file("src/app/runtime/team_overview.rs");
    for snippet in [
        "push_unique_root(&mut roots, project_root);",
        "for source_root in self.source_engine_catalog_roots()",
        "push_unique_root(&mut roots, source_root);",
    ] {
        assert!(
            team.contains(snippet),
            "Team refresh should keep selected project first while sharing runtime root key logic; missing {snippet}"
        );
    }
    assert!(
        !team.contains("fn push_non_empty")
            && !team.contains("fn compiled_repo_root")
            && !team.contains("roots.iter().any(|root| root == &path)"),
        "Team refresh must not keep direct PathBuf equality root de-duplication after root_paths convergence"
    );
}

#[test]
fn source_engine_refreshes_use_one_scoped_view_helper() {
    let runtime = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime.contains("mod source_scoped_views;"),
        "runtime.rs must register a focused helper for Source Engine scoped page refreshes"
    );

    let scoped_views = read_crate_file("src/app/runtime/source_scoped_views.rs");
    for snippet in [
        "pub(super) fn refresh_source_scoped_views(&mut self) -> Result<(), HubError>",
        "self.refresh_asset_catalog()?;",
        "self.refresh_learn_catalog()?;",
        "self.refresh_plugin_catalog()?;",
        "self.refresh_team_overview()",
    ] {
        assert!(
            scoped_views.contains(snippet),
            "Source Engine scoped refreshes should stay centralized; missing {snippet}"
        );
    }

    let repeated_refresh_chain = concat!(
        "self.refresh_asset_catalog()?;\n",
        "        self.refresh_learn_catalog()?;\n",
        "        self.refresh_plugin_catalog()?;\n",
        "        self.refresh_team_overview()?"
    );
    assert!(
        !runtime.contains(repeated_refresh_chain),
        "runtime.rs should call refresh_source_scoped_views instead of repeating the full refresh chain"
    );
    for snippet in ["self.refresh_source_scoped_views()?;"] {
        assert!(
            runtime.contains(snippet),
            "runtime.rs must use the shared Source Engine scoped refresh helper; missing {snippet}"
        );
    }

    let folder_picker = read_crate_file("src/app/runtime/folder_picker.rs");
    let source_browse = folder_picker
        .split("\"source\" => {")
        .nth(1)
        .and_then(|source| source.split("\"output\" => {").next())
        .expect("folder_picker.rs must declare source folder branch before output branch");
    let output_browse = folder_picker
        .split("\"output\" => {")
        .nth(1)
        .and_then(|source| source.split("\"device-install\" => {").next())
        .expect("folder_picker.rs must declare output folder branch before device-install branch");
    for (label, branch) in [
        ("Source folder selection", source_browse),
        ("Output folder selection", output_browse),
    ] {
        for snippet in [
            "self.register_source_engine_from_settings();",
            "self.refresh_source_scoped_views()?;",
        ] {
            assert!(
                branch.contains(snippet),
                "{label} should register the Source Engine and refresh scoped views through the shared helper; missing {snippet}"
            );
        }
    }
    assert!(
        !folder_picker.contains(repeated_refresh_chain),
        "folder_picker.rs should not repeat the full Source Engine scoped refresh chain"
    );

    assert!(
        runtime.contains(
            "self.register_source_engine_from_settings();\n        self.refresh_source_scoped_views()?;"
        ) && runtime.contains(
            "self.validate_active_source_engine_for_build(command_line.clone())?;"
        ),
        "Build should refresh Source Engine scoped views after registering settings and before validating/projecting the building snapshot"
    );

    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    for snippet in [
        "fn refresh_project_context_views(",
        "selected_project_changed: bool,",
        "active_engine_changed: bool,",
        "if active_engine_changed {\n            self.refresh_source_scoped_views()",
        "} else if selected_project_changed {\n            self.refresh_selected_project_scoped_views()",
        "self.refresh_project_context_views(\n            selected_project_changed,\n            self.config.active_engine_id != active_engine_before,\n        )?;",
    ] {
        assert!(
            workspace.contains(snippet),
            "Project actions should refresh Learn plus project-scoped views when their bound Source Engine changes; missing {snippet}"
        );
    }
}

#[test]
fn selected_project_refresh_detection_uses_normalized_paths() {
    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    for snippet in [
        "fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool",
        "(Some(before), Some(after)) => !project_paths_match(before, after)",
        "selected_project_path_changed(\n            selected_before.as_deref(),\n            self.selected_project_path.as_deref(),\n        )",
    ] {
        assert!(
            workspace.contains(snippet),
            "selected-project scoped refresh detection must use normalized project path matching; missing {snippet}"
        );
    }
    assert!(
        !workspace.contains("self.selected_project_path != selected_before"),
        "selected-project scoped refresh detection must not use direct Option<PathBuf> equality after path-key convergence"
    );
}
