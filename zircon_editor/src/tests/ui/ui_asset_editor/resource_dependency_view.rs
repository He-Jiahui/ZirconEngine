use crate::ui::asset_editor::UiAssetEditorCommand;
use std::path::Path;
use zircon_runtime::ui::template::UiResourcePathResolver;
use zircon_runtime_interface::ui::template::{UiResourceDependencySource, UiResourceKind};

use super::support::open_design_session;

const RESOURCE_DEPENDENCY_LAYOUT: &str = r##"
[asset]
kind = "layout"
id = "editor.resource_dependency_view"
version = 3
display_name = "Resource Dependency View"

[imports]
resources = [
  { kind = "font", uri = "res://fonts/editor.font.toml", fallback = { mode = "placeholder", uri = "res://fonts/system.ttf" } },
]

[root]
node_id = "root"
kind = "native"
type = "Label"
control_id = "ResourceLabel"
props = { icon = "asset://images/resource-a.png" }

[[stylesheets]]
id = "local"

[[stylesheets.rules]]
selector = "Label"
set = { self = { background_image = "asset://images/background.png" } }
"##;

#[test]
fn ui_asset_editor_session_exposes_resource_dependencies_after_compile() {
    let session = open_design_session(
        "asset://ui/tests/resource_dependency_view.ui.toml",
        RESOURCE_DEPENDENCY_LAYOUT,
    );

    let dependencies = session.resource_dependencies();

    assert!(dependencies.iter().any(|dependency| {
        dependency.source == UiResourceDependencySource::DocumentImport
            && dependency.path == "imports.resources[0]"
            && dependency.reference.kind == UiResourceKind::Font
            && dependency.reference.uri == "res://fonts/editor.font.toml"
    }));
    assert!(dependencies.iter().any(|dependency| {
        dependency.source == UiResourceDependencySource::NodeProp
            && dependency.path == "root.props.icon"
            && dependency.reference.kind == UiResourceKind::Image
            && dependency.reference.uri == "asset://images/resource-a.png"
    }));

    let pane = session.pane_presentation();
    assert!(pane
        .resource_dependency_items
        .iter()
        .any(|item| item.contains("DocumentImport") && item.contains("editor.font.toml")));
}

#[test]
fn ui_asset_editor_session_exposes_resource_diagnostics_after_compile() {
    let session = open_design_session(
        "asset://ui/tests/resource_dependency_view.ui.toml",
        RESOURCE_DEPENDENCY_LAYOUT,
    );

    assert!(session.resource_diagnostics().is_empty());
    assert!(session
        .pane_presentation()
        .resource_diagnostic_items
        .is_empty());
}

#[test]
fn ui_asset_editor_resource_dependencies_refresh_after_source_edit() {
    let mut session = open_design_session(
        "asset://ui/tests/resource_dependency_view.ui.toml",
        RESOURCE_DEPENDENCY_LAYOUT,
    );
    assert!(session
        .resource_dependencies()
        .iter()
        .any(|dependency| dependency.reference.uri == "asset://images/resource-a.png"));

    let edited = RESOURCE_DEPENDENCY_LAYOUT.replace(
        "asset://images/resource-a.png",
        "asset://images/resource-b.png",
    );
    session
        .apply_command(UiAssetEditorCommand::edit_source(edited))
        .expect("edit resource source");

    assert!(session
        .resource_dependencies()
        .iter()
        .any(|dependency| dependency.reference.uri == "asset://images/resource-b.png"));
    assert!(!session
        .resource_dependencies()
        .iter()
        .any(|dependency| dependency.reference.uri == "asset://images/resource-a.png"));
}

#[test]
fn ui_asset_editor_resource_view_clears_when_resource_compile_fails() {
    let mut session = open_design_session(
        "asset://ui/tests/resource_dependency_view.ui.toml",
        RESOURCE_DEPENDENCY_LAYOUT,
    );
    assert!(!session.resource_dependencies().is_empty());

    let invalid = RESOURCE_DEPENDENCY_LAYOUT.replace(
        "icon = \"asset://images/resource-a.png\"",
        "icon = { kind = \"image\", uri = \"http://cdn.example/resource-a.png\" }",
    );
    session
        .apply_command(UiAssetEditorCommand::edit_source(invalid))
        .expect("invalid resource edit is reported through diagnostics");

    assert!(!session.diagnostics().is_empty());
    assert!(session.resource_dependencies().is_empty());
    assert!(session.resource_diagnostics().is_empty());
}

#[test]
fn ui_asset_editor_resource_view_reports_resolver_missing_files() {
    let mut session = open_design_session(
        "asset://ui/tests/resource_dependency_view.ui.toml",
        RESOURCE_DEPENDENCY_LAYOUT,
    );
    let temp_root = std::env::temp_dir().join(format!(
        "zircon_editor_resource_view_{}",
        std::process::id()
    ));
    let res_root = temp_root.join("res");
    let asset_root = temp_root.join("asset");
    write_fixture(&res_root.join("fonts/editor.font.toml"));
    write_fixture(&res_root.join("fonts/system.ttf"));
    write_fixture(&asset_root.join("images/background.png"));

    let resolver = UiResourcePathResolver::default()
        .with_res_root(&res_root)
        .with_asset_root(&asset_root);
    assert!(session.set_resource_path_resolver(resolver.clone()));

    assert!(session.resource_diagnostics().iter().any(|diagnostic| {
        diagnostic.code == "missing_resource_file"
            && diagnostic.path == "root.props.icon"
            && diagnostic.message.contains("asset://images/resource-a.png")
    }));
    assert!(session
        .pane_presentation()
        .resource_diagnostic_items
        .iter()
        .any(|item| item.contains("missing_resource_file")
            && item.contains("asset://images/resource-a.png")));

    write_fixture(&asset_root.join("images/resource-a.png"));
    assert!(!session.set_resource_path_resolver(resolver.clone()));
    session.refresh_resource_path_resolver();
    let _ = std::fs::remove_dir_all(&temp_root);

    assert!(session.resource_diagnostics().is_empty());
    assert!(session
        .pane_presentation()
        .resource_diagnostic_items
        .is_empty());
}

fn write_fixture(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, b"fixture").unwrap();
}
