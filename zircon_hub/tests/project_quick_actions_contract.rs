//! Static contracts for Hub selected-project quick action targeting.

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
fn quick_actions_fallback_only_when_no_project_is_selected() {
    let quick_actions = read_crate_file("src/app/view_model/quick_actions.rs");
    for snippet in [
        "return QuickActionProjectTarget::StaleSelection;",
        "fn quick_actions_do_not_fallback_when_selected_project_is_stale()",
        "Selected project is no longer available to build",
        "Selected project unavailable; launch editor without a project",
    ] {
        assert!(
            quick_actions.contains(snippet),
            "Quick Actions should not silently use latest recent project when selected_project_path is set but stale; missing {snippet}"
        );
    }

    let workspace = read_crate_file("src/app/runtime/project_workspace.rs");
    for snippet in [
        "let had_selected_project = self.selected_project_path.is_some();",
        "if had_selected_project {\n            return None;\n        }",
        "fn selected_or_latest_recent_project_for_named_action(",
        "Selected project is no longer available to package",
        "Selected project is no longer available to install",
        "Selected project is no longer available to build",
    ] {
        assert!(
            workspace.contains(snippet),
            "Runtime quick-action helpers should fallback to latest recent only when no selected project existed; missing {snippet}"
        );
    }

    let runtime = read_crate_file("src/app/runtime.rs");
    assert!(
        runtime.contains("fn quick_action_target_does_not_fallback_when_selected_project_is_stale()"),
        "Runtime unit coverage should lock stale selected-project behavior before quick actions can fallback"
    );
    assert!(
        runtime.contains("fn quick_action_build_reports_stale_selected_project()"),
        "Runtime unit coverage should keep stale selected-project quick-action errors explicit"
    );
}
