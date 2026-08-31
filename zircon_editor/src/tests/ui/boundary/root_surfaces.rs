#[test]
fn editor_crate_root_stops_flattening_asset_editor_and_workbench_specialists() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let source = std::fs::read_to_string(crate_root).expect("editor crate root");

    for forbidden in [
        "pub struct EditorModule",
        "impl EngineModule for EditorModule",
        "pub use ui::asset_editor::{",
        "pub use ui::host::{",
        "pub use ui::workbench::autolayout::{",
        "pub use ui::workbench::event::{",
        "pub use ui::workbench::fixture::{",
        "pub use ui::workbench::layout::{",
        "pub use ui::workbench::model::{",
        "pub use ui::workbench::project::{",
        "pub use ui::workbench::reflection::{",
        "pub use ui::workbench::snapshot::{",
        "pub use ui::workbench::startup::{",
        "pub use ui::workbench::state::EditorState;",
        "pub use ui::workbench::view::{",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor crate root to stop flattening specialist surface `{forbidden}`"
        );
    }
}

#[test]
fn editor_ui_root_stops_flattening_binding_asset_editor_control_and_template_specialists() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui")
        .join("mod.rs");
    let source = std::fs::read_to_string(ui_root).expect("editor ui root");

    for forbidden in [
        "pub use asset_editor::{",
        "pub use binding::{",
        "pub use control::{",
        "pub use template::{",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected zircon_editor ui root to stop flattening specialist surface `{forbidden}`"
        );
    }
}

#[test]
fn ui_runtime_access_is_limited_to_composition_and_typed_service_leaves() {
    let ui_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("ui");
    let mut sources = Vec::new();
    collect_rust_sources(&ui_root, &ui_root, &mut sources);

    let allowed_core_handle_owners = [
        "host/editor_asset_manager/handle.rs",
        "host/editor_manager.rs",
        "host/runtime_services.rs",
        "retained_host/app.rs",
        "retained_host/app/asset_runtime_access.rs",
        "retained_host/app/automation.rs",
        "retained_host/app/host_lifecycle/startup/constructors.rs",
        "retained_host/app/runtime_lease.rs",
        "retained_host/viewport/render_framework_access.rs",
    ];
    let mut violations = Vec::new();

    for (relative, source) in sources {
        if relative.contains("/tests/")
            || relative.ends_with("/tests.rs")
            || relative.ends_with("_tests.rs")
        {
            continue;
        }
        let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
        if contains_token(production, "CoreHandle")
            && !allowed_core_handle_owners.contains(&relative.as_str())
        {
            violations.push(format!("{relative}: unexpected CoreHandle owner"));
        }
        if contains_token(production, "ManagerResolver") {
            violations.push(format!("{relative}: UI must not retain a ManagerResolver"));
        }
        for forbidden in ["LevelSystem", "World"] {
            if imports_runtime_scene_token(production, forbidden) {
                violations.push(format!(
                    "{relative}: production UI bypasses gateway with {forbidden}"
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "UI runtime-access boundary violations:\n{}",
        violations.join("\n")
    );
}

fn collect_rust_sources(
    root: &std::path::Path,
    current: &std::path::Path,
    sources: &mut Vec<(String, String)>,
) {
    for entry in std::fs::read_dir(current).expect("read UI source directory") {
        let entry = entry.expect("read UI source entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(root, &path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let relative = path
                .strip_prefix(root)
                .expect("UI source stays below the UI root")
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(&path).expect("read UI Rust source");
            sources.push((relative, source));
        }
    }
}

fn contains_token(source: &str, token: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|candidate| candidate == token)
}

fn imports_runtime_scene_token(source: &str, token: &str) -> bool {
    let mut in_grouped_import = false;

    for line in source.lines() {
        if line.contains("use zircon_runtime::scene::{") {
            in_grouped_import = true;
        }
        if in_grouped_import && contains_token(line, token) {
            return true;
        }
        if in_grouped_import && line.contains("};") {
            in_grouped_import = false;
        }
    }

    source.lines().any(|line| {
        (line.contains("use zircon_runtime::scene::") || line.contains("zircon_runtime::scene::"))
            && contains_token(line, token)
    })
}
