//! Static contracts for Hub Source Engine selection and registration workflow.

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
fn new_project_default_engine_follows_active_source_context() {
    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    for snippet in [
        "pub(super) fn sync_new_project_engine_after_active_engine_change(",
        "previous_active_engine_id: Option<&str>,",
        "let active_engine_id = self\n            .config\n            .active_engine_id\n            .clone()\n            .filter(|id| self.config.engines.iter().any(|engine| engine.id == *id));",
        "let current_is_valid = current\n            .as_deref()\n            .is_some_and(|id| self.config.engines.iter().any(|engine| engine.id == id));",
        "let followed_previous_active =\n            current.as_deref().is_some() && current.as_deref() == previous_active_engine_id;",
        "if current.is_none() || !current_is_valid || followed_previous_active {\n            self.new_project_engine_id = active_engine_id;",
    ] {
        assert!(
            workspace.contains(snippet),
            "New Project's default Source Engine should follow active engine changes unless the user selected another valid engine; missing {snippet}"
        );
    }

    let activate_project_engine = workspace
        .split("pub(super) fn activate_project_engine_for_path")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn remember_project_metadata_for_path").next())
        .expect("project_workspace.rs must declare activate_project_engine_for_path before remember_project_metadata_for_path");
    for snippet in [
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.config.active_engine_id = Some(engine_id);",
        "self.sync_settings_from_active_engine();",
        "self.sync_new_project_engine_after_active_engine_change(\n                active_engine_before.as_deref(),\n            );",
    ] {
        assert!(
            activate_project_engine.contains(snippet),
            "Project-bound Source Engine activation should update the New Project default when it follows active engine context; missing {snippet}"
        );
    }

    let detail_engine = workspace
        .split("pub(super) fn select_project_detail_engine_by_id")
        .nth(1)
        .and_then(|source| source.split("pub(super) fn toggle_selected_project_pin").next())
        .expect("project_workspace.rs must declare select_project_detail_engine_by_id before toggle_selected_project_pin");
    assert!(
        detail_engine.contains(
            "self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());"
        ),
        "Changing a selected project's bound Source Engine should keep New Project defaults aligned with active Source context"
    );

    let runtime = read_crate_file("src/app/runtime.rs");
    let select_engine = runtime
        .split("fn select_engine_by_id")
        .nth(1)
        .and_then(|source| source.split("fn rename_active_engine").next())
        .expect("runtime.rs must declare select_engine_by_id before rename_active_engine");
    for snippet in [
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());",
    ] {
        assert!(
            select_engine.contains(snippet),
            "Manual Source Engine selection should update New Project defaults when they follow active Source context; missing {snippet}"
        );
    }

    let remove_engine = runtime
        .split("fn remove_engine_by_id")
        .nth(1)
        .and_then(|source| source.split("fn cycle_active_engine").next())
        .expect("runtime.rs must declare remove_engine_by_id before cycle_active_engine");
    assert!(
        remove_engine.contains(
            "self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());"
        ),
        "Removing a Source Engine should repair New Project defaults when the selected default becomes invalid"
    );

    let register_engine = runtime
        .split("fn register_source_engine_from_settings")
        .nth(1)
        .and_then(|source| source.split("fn migrate_project_engine_metadata").next())
        .expect(
            "runtime.rs must declare register_source_engine_from_settings before migrate_project_engine_metadata",
        );
    for snippet in [
        "let active_engine_before = self.config.active_engine_id.clone();",
        "self.config.active_engine_id = Some(engine_id);",
        "self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());",
    ] {
        assert!(
            register_engine.contains(snippet),
            "Settings-based Source Engine registration should update New Project defaults when active Source context changes; missing {snippet}"
        );
    }
}

#[test]
fn source_engine_registration_uses_shared_filesystem_path_key() {
    let runtime = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime.contains("mod source_engine_paths;"),
        "runtime.rs must register the focused Source Engine path module instead of keeping ID/path-key helpers inline"
    );
    assert!(
        runtime.contains("use self::source_engine_paths::{"),
        "runtime.rs must consume Source Engine ID/path helpers from source_engine_paths.rs"
    );
    for forbidden in [
        "fn source_engine_path_key",
        "fn same_source_engine_path",
        "fn source_engine_id",
        "to_string_lossy().to_ascii_lowercase().bytes()",
    ] {
        assert!(
            !runtime.contains(forbidden),
            "runtime.rs should not keep private Source Engine path normalization helpers after source_engine_paths convergence: {forbidden}"
        );
    }

    let paths = read_crate_file("src/app/runtime/source_engine_paths.rs");
    for snippet in [
        "use crate::projects::project_filesystem_path_key;",
        "pub(super) fn source_engine_id(source_dir: &Path) -> String",
        "let key = source_engine_path_key(source_dir);",
        "for byte in key.bytes()",
        "pub(super) fn same_source_engine_path(left: &Path, right: &Path) -> bool",
        "project_filesystem_path_key(path)",
        "pub(super) fn source_engine_display_name(source_dir: &Path) -> String",
    ] {
        assert!(
            paths.contains(snippet),
            "Source Engine IDs and de-duplication must share the canonical project filesystem path key; missing {snippet}"
        );
    }
    assert!(
        !paths.contains("replace('\\\\', \"/\")")
            && !paths.contains("trim_end_matches('/')")
            && !paths.contains("to_ascii_lowercase()"),
        "source_engine_paths.rs must not reimplement slash/case normalization outside project_filesystem_path_key"
    );
}
