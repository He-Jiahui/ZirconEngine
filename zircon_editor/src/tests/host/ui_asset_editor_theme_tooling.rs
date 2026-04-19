use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use toml::Value;
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime::ui::template::UiAssetLoader;

use crate::module::module_descriptor;
use crate::project::EditorProjectDocument;
use crate::{module, EditorManager, EDITOR_MANAGER_NAME};

fn unique_temp_path(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}"))
}

fn editor_runtime_with_config_path(path: &std::path::Path) -> CoreRuntime {
    std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", path);
    let runtime = CoreRuntime::new();
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
    runtime
}

const DUPLICATE_THEME_UI_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.tests.asset.theme_dedupe"
version = 1
display_name = "Theme Dedupe"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

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

const DUPLICATE_IMPORTED_THEME_ASSET: &str = r##"
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

const THEME_SUMMARY_LAYOUT_ASSET: &str = r##"
[asset]
kind = "layout"
id = "editor.test.theme_summary"
version = 1
display_name = "Theme Summary"

[imports]
styles = ["res://ui/theme/shared_theme.ui.toml"]

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

const IMPORTED_THEME_COLLISION_ASSET: &str = r##"
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

#[test]
fn editor_manager_prunes_duplicate_local_theme_overrides() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_prune_duplicates");
    let project_root = unique_temp_dir("zircon_editor_theme_prune_duplicates_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, DUPLICATE_THEME_UI_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, DUPLICATE_IMPORTED_THEME_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");

    let before = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("pane before theme prune");
    assert!(before
        .theme_refactor_items
        .iter()
        .any(|item| item.contains("duplicate local token • accent")));
    assert!(before
        .theme_refactor_items
        .iter()
        .any(|item| item.contains("duplicate local rule • local_theme • Button")));

    assert!(manager
        .prune_ui_asset_editor_duplicate_local_theme_overrides(&instance_id)
        .expect("prune duplicate local theme overrides"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save pruned ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved pruned ui asset");
    assert!(!document.tokens.contains_key("accent"));
    assert!(!document.tokens.contains_key("panel"));
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_theme_refactor_items_individually() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_apply_refactor");
    let project_root = unique_temp_dir("zircon_editor_theme_apply_refactor_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, DUPLICATE_THEME_UI_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, DUPLICATE_IMPORTED_THEME_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");

    assert!(manager
        .apply_ui_asset_editor_theme_refactor_item(&instance_id, 0)
        .expect("apply duplicate token refactor"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save refactored ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved refactored ui asset");
    assert!(!document.tokens.contains_key("accent"));
    assert!(document.tokens.contains_key("panel"));
    assert_eq!(document.stylesheets[0].rules.len(), 1);

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_theme_rule_helper_items_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_apply_helper");
    let project_root = unique_temp_dir("zircon_editor_theme_apply_helper_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, DUPLICATE_THEME_UI_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, DUPLICATE_IMPORTED_THEME_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, 0)
        .expect("apply detach helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save helper-applied ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved helper ui asset");
    assert!(document.imports.styles.is_empty());
    assert!(document.tokens.contains_key("shared_theme_accent"));

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_compare_diff_theme_helper_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_compare_diff_helper");
    let project_root = unique_temp_dir("zircon_editor_theme_compare_diff_helper_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, THEME_SUMMARY_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt compare diffs from selected theme (3)")
        .expect("compare diff theme helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, helper_index)
        .expect("apply compare diff theme helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save compare diff helper-applied ui asset editor");
    let document =
        UiAssetLoader::load_toml_str(&saved).expect("saved compare diff helper ui asset");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    assert_eq!(
        document.tokens.get("panel"),
        Some(&Value::String("$accent".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("adopted compare diff imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_theme_rule_body_helper_items_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_rule_body_helper");
    let project_root = unique_temp_dir("zircon_editor_theme_rule_body_helper_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, THEME_SUMMARY_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let token_helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported token • accent = \"#223344\"")
        .expect("imported token helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, token_helper_index)
        .expect("apply imported token helper"));

    let rule_helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt imported rule • local_theme • Button")
        .expect("imported rule helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, rule_helper_index)
        .expect("apply imported rule helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save helper-applied ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved helper ui asset");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_theme_batch_adopt_helper_items_for_imported_sources() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_batch_adopt_helper");
    let project_root = unique_temp_dir("zircon_editor_theme_batch_adopt_helper_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, THEME_SUMMARY_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, IMPORTED_THEME_COLLISION_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Adopt all imported changes (3)")
        .expect("batch imported theme helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, helper_index)
        .expect("apply batch imported theme helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save batch helper-applied ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved helper ui asset");
    assert_eq!(
        document.tokens.get("accent"),
        Some(&Value::String("#223344".to_string()))
    );
    assert_eq!(
        document.tokens.get("panel"),
        Some(&Value::String("$accent".to_string()))
    );
    let button_rule = document
        .stylesheets
        .iter()
        .find(|sheet| sheet.id == "local_theme")
        .and_then(|sheet| sheet.rules.iter().find(|rule| rule.selector == "Button"))
        .expect("adopted imported rule");
    assert_eq!(
        button_rule.set.self_values.get("text"),
        Some(&Value::String("$panel".to_string()))
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_prunes_selected_theme_compare_duplicates() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_compare_prune_helper");
    let project_root = unique_temp_dir("zircon_editor_theme_compare_prune_helper_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, DUPLICATE_THEME_UI_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, DUPLICATE_IMPORTED_THEME_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    assert!(manager
        .select_ui_asset_editor_theme_source(&instance_id, 1)
        .expect("select imported theme source"));

    let helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item == "Prune compare duplicates shared with selected theme (3)")
        .expect("compare prune theme helper");
    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, helper_index)
        .expect("apply compare prune theme helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save compare prune helper-applied ui asset editor");
    let document =
        UiAssetLoader::load_toml_str(&saved).expect("saved compare prune helper ui asset");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert_eq!(
        document.imports.styles,
        vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn editor_manager_applies_theme_batch_refactor_helper() {
    let _guard = crate::tests::support::env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_theme_batch_refactor");
    let project_root = unique_temp_dir("zircon_editor_theme_batch_refactor_project");
    let runtime = editor_runtime_with_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let world = DefaultLevelManager::default()
        .create_default_level()
        .snapshot();
    EditorProjectDocument::save_to_path(&project_root, &world, None).unwrap();

    let layout_path = project_root
        .join("assets")
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    let imported_theme_path = project_root
        .join("assets")
        .join("ui")
        .join("theme")
        .join("shared_theme.ui.toml");
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    fs::create_dir_all(imported_theme_path.parent().unwrap()).unwrap();
    fs::write(&layout_path, DUPLICATE_THEME_UI_LAYOUT_ASSET).unwrap();
    fs::write(&imported_theme_path, DUPLICATE_IMPORTED_THEME_ASSET).unwrap();

    manager.open_project(&project_root).unwrap();

    let instance_id = manager
        .open_ui_asset_editor_by_id("res://ui/layouts/editor.ui.toml", None)
        .expect("ui asset editor should open from project asset id");
    let helper_index = manager
        .ui_asset_editor_pane_presentation(&instance_id)
        .expect("theme helper presentation")
        .theme_rule_helper_items
        .iter()
        .position(|item| item.starts_with("Apply all theme refactors ("))
        .expect("batch theme refactor helper");

    assert!(manager
        .apply_ui_asset_editor_theme_rule_helper_item(&instance_id, helper_index)
        .expect("apply batch theme refactor helper"));

    let saved = manager
        .save_ui_asset_editor(&instance_id)
        .expect("save batch refactored ui asset editor");
    let document = UiAssetLoader::load_toml_str(&saved).expect("saved batch refactored ui asset");
    assert!(document.tokens.is_empty());
    assert!(document.stylesheets.is_empty());
    assert!(
        document.imports.styles.is_empty()
            || document.imports.styles == vec!["res://ui/theme/shared_theme.ui.toml".to_string()]
    );

    std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");
    let _ = fs::remove_file(path);
    let _ = fs::remove_dir_all(project_root);
}
