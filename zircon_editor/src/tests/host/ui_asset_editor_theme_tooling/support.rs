use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectManager;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    FOUNDATION_MODULE_NAME, module_descriptor as foundation_module_descriptor,
};
use zircon_runtime::scene::DefaultLevelManager;

use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::host::module::{self, module_descriptor};
use crate::ui::workbench::project::EditorProjectDocument;

pub(crate) const DUPLICATE_THEME_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_dedupe"
version = 1
display_name = "Theme Dedupe"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#223344"
panel = "$accent"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Button"
control_id = "SaveButton"
props = { text = "Save" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) const DUPLICATE_IMPORTED_THEME_ASSET: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "shared_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) const THEME_SUMMARY_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.test.theme_summary"
version = 1
display_name = "Theme Summary"

[imports]
styles = ["res://ui/theme/shared_theme.zui"]

[tokens]
accent = "#4488ff"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
control_id = "RootLabel"
props = { text = "Theme Summary" }

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "#RootLabel"
set = { self = { text = "Theme Summary Local" } }
"##;

pub(crate) const IMPORTED_THEME_COLLISION_ASSET: &str = r##"
[asset]
kind = "style"
id = "ui.theme.shared_theme"
version = 1
display_name = "Shared Theme"

[tokens]
accent = "#223344"
panel = "$accent"

[[stylesheets]]
id = "local_theme"

[[stylesheets.rules]]
selector = "Button"
set = { self = { text = "$panel" } }
"##;

pub(crate) fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

pub(crate) fn write_ui_asset(path: impl AsRef<Path>, source: &str) {
    crate::tests::support::write_test_ui_asset(path, source).unwrap();
}

pub(crate) fn editor_runtime_with_config_path(path: &Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
    runtime.store_config_value(
        crate::ui::host::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
        serde_json::json!([
            crate::ui::host::EDITOR_SUBSYSTEM_ANIMATION_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_UI_ASSET_AUTHORING,
            crate::ui::host::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS,
            crate::ui::host::EDITOR_SUBSYSTEM_NATIVE_WINDOW_HOSTING,
        ]),
    );
    runtime
        .register_module(foundation_module_descriptor())
        .unwrap();
    runtime
        .register_module(zircon_runtime::asset::module_descriptor())
        .unwrap();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(FOUNDATION_MODULE_NAME).unwrap();
    runtime
        .activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
        .unwrap();
    runtime.activate_module(module::EDITOR_MODULE_NAME).unwrap();
    crate::tests::support::configure_editor_test_runtime_build_set(&runtime);
    runtime
}

pub(crate) fn setup_theme_project(
    config_prefix: &str,
    project_prefix: &str,
    layout_source: &str,
    imported_source: &str,
) -> (PathBuf, PathBuf, CoreRuntime) {
    let config_path = unique_temp_path(config_prefix);
    let project_root = unique_temp_dir(project_prefix);
    let runtime = editor_runtime_with_config_path(&config_path);
    let project_name = project_root
        .file_name()
        .and_then(|name| name.to_str())
        .expect("theme project root must have a UTF-8 final component");
    let location = project_root.parent().expect("theme project root parent");
    ProjectAuthority::default()
        .create_project(&NewProjectDraft {
            project_name: project_name.to_string(),
            location: location.to_string_lossy().into_owned(),
            template: NewProjectTemplate::RenderableEmpty,
        })
        .expect("theme fixture project should be created");
    let mut project =
        ProjectManager::open(&project_root).expect("theme fixture project should open");
    project
        .scan_and_import()
        .expect("theme fixture project should scan");
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_scene_to_project(
        &project,
        &project.manifest().default_scene,
        &world,
        None,
    )
    .unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.zui");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.zui");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    write_ui_asset(&layout_path, layout_source);
    write_ui_asset(&imported_theme_path, imported_source);

    (config_path, project_root, runtime)
}

pub(crate) fn cleanup_theme_project(config_path: &Path, project_root: &Path) {
    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = fs::remove_file(config_path);
    let _ = fs::remove_dir_all(project_root);
}
