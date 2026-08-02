use super::*;

#[test]
fn ui_v2_file_cache_reuses_compiled_store_and_resolves_transitive_styles() {
    let temp_dir = v2_cache_temp_dir("res_alias_imports");
    let assets_root = temp_dir.join("assets");
    let layout_path = assets_root.join("ui/editor/layout.zui");
    let base_style_path = assets_root.join("ui/theme/base.zui");
    let material_style_path = assets_root.join("ui/theme/material.zui");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(base_style_path.parent().unwrap()).unwrap();
    std::fs::write(
        &layout_path,
        r##"
[asset]
kind = "view"
id = "ui.editor.layout"
version = 2

[imports]
styles = ["res://ui/theme/base.zui"]

[root]
node = "root"

[nodes.root]
component = "Label"
control_id = "CacheRoot"
classes = ["cache-root"]
props = { text = "Cache" }
"##,
    )
    .unwrap();
    std::fs::write(
        &base_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.base"
version = 2

[imports]
styles = ["res://ui/theme/material.zui"]

[tokens]
base_color = "$material_color"

[[stylesheets]]
id = "base"

[[stylesheets.rules]]
selector = ".cache-root"
set = { self = { foreground_color = "$base_color" } }
"##,
    )
    .unwrap();
    std::fs::write(
        &material_style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.material"
version = 2

[tokens]
material_color = "#abcdef"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let first = cache.load_store(vec![layout_path.clone()]).unwrap();
    let second = cache.load_store(vec![layout_path]).unwrap();

    assert!(!first.cache_hit);
    assert!(second.cache_hit);
    assert_eq!(second.root_asset_id, "ui.editor.layout");
    assert!(second.store.get("res://ui/theme/base.zui").is_some());
    assert!(second.store.get("res://ui/theme/material.zui").is_some());
    assert!(Arc::ptr_eq(&first.compiled, &second.compiled));
    assert_eq!(cache.len(), 1);

    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.file_cache"),
        second.root_document.as_ref(),
        second.compiled.as_ref(),
    )
    .unwrap();
    let root = surface.tree.nodes.values().next().unwrap();
    let metadata = root.template_metadata.as_ref().unwrap();
    assert_eq!(
        metadata
            .attributes
            .get("foreground_color")
            .and_then(Value::as_str),
        Some("#abcdef")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_uses_persistent_cache_across_cache_instances() {
    let temp_dir = v2_cache_temp_dir("persistent_cache_roundtrip");
    let assets_root = temp_dir.join("assets");
    let cache_root = temp_dir.join("cache");
    let layout_path = assets_root.join("ui/editor/layout.zui");
    let style_path = assets_root.join("ui/theme/persistent.zui");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(style_path.parent().unwrap()).unwrap();
    std::fs::write(
        &layout_path,
        r##"
[asset]
kind = "view"
id = "ui.editor.persistent_layout"
version = 2

[imports]
styles = ["res://ui/theme/persistent.zui"]

[root]
node = "root"

[nodes.root]
component = "Label"
control_id = "PersistentRoot"
classes = ["persistent-root"]
props = { text = "Persistent" }
"##,
    )
    .unwrap();
    std::fs::write(
        &style_path,
        r##"
[asset]
kind = "style"
id = "ui.theme.persistent"
version = 2

[[stylesheets]]
id = "persistent"

[[stylesheets.rules]]
selector = ".persistent-root"
set = { self = { foreground_color = "#123456" } }
"##,
    )
    .unwrap();

    let mut first_cache = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root.clone());
    let first = first_cache.load_store(vec![layout_path.clone()]).unwrap();
    let mut second_cache = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root);
    let second = second_cache.load_store(vec![layout_path]).unwrap();

    assert!(!first.cache_hit);
    assert!(!first.persistent_cache_hit);
    assert!(second.cache_hit);
    assert!(second.persistent_cache_hit);
    assert_eq!(second.root_asset_id, "ui.editor.persistent_layout");
    assert!(second.store.get("res://ui/theme/persistent.zui").is_some());
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.persistent_cache"),
        second.root_document.as_ref(),
        second.compiled.as_ref(),
    )
    .unwrap();
    let root = surface.tree.nodes.values().next().unwrap();
    assert_eq!(
        root.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("foreground_color"))
            .and_then(Value::as_str),
        Some("#123456")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_rebuilds_when_persistent_cache_dependency_changes() {
    let temp_dir = v2_cache_temp_dir("persistent_cache_invalidates");
    let assets_root = temp_dir.join("assets");
    let cache_root = temp_dir.join("cache");
    let layout_path = assets_root.join("ui/editor/layout.zui");
    let style_path = assets_root.join("ui/theme/persistent.zui");
    std::fs::create_dir_all(layout_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(style_path.parent().unwrap()).unwrap();
    std::fs::write(
        &layout_path,
        r##"
[asset]
kind = "view"
id = "ui.editor.persistent_layout"
version = 2

[imports]
styles = ["res://ui/theme/persistent.zui"]

[root]
node = "root"

[nodes.root]
component = "Label"
control_id = "PersistentRoot"
classes = ["persistent-root"]
props = { text = "Persistent" }
"##,
    )
    .unwrap();
    std::fs::write(&style_path, persistent_cache_style("#123456")).unwrap();

    let mut first_cache = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root.clone());
    first_cache.load_store(vec![layout_path.clone()]).unwrap();
    std::fs::write(&style_path, persistent_cache_style("#654321")).unwrap();
    let mut second_cache = UiV2PrototypeStoreFileCache::with_persistent_cache(cache_root);
    let second = second_cache.load_store(vec![layout_path]).unwrap();

    assert!(!second.cache_hit);
    assert!(!second.persistent_cache_hit);
    let surface = UiV2SurfaceBuilder::build_surface_from_compiled_document(
        UiTreeId::new("runtime.ui.v2.persistent_cache_invalidates"),
        second.root_document.as_ref(),
        second.compiled.as_ref(),
    )
    .unwrap();
    let root = surface.tree.nodes.values().next().unwrap();
    assert_eq!(
        root.template_metadata
            .as_ref()
            .and_then(|metadata| metadata.attributes.get("foreground_color"))
            .and_then(Value::as_str),
        Some("#654321")
    );

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_resolves_builtin_asset_id_widget_imports() {
    let temp_dir = v2_cache_temp_dir("asset_id_widget_imports");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/windows/workbench_window.zui");
    let component_path =
        assets_root.join("ui/editor/components/workbench/shell/activity_drawer_window.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(component_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.workbench"
version = 2

[imports]
widgets = ["editor.workbench.shell.activity_drawer_window#ActivityDrawerWindow"]

[root]
node = "root"

[nodes.root]
component = "ActivityDrawerWindow"
control_id = "WorkbenchWindow"
"##,
    )
    .unwrap();
    std::fs::write(
        &component_path,
        r##"
[asset]
kind = "component"
id = "editor.workbench.shell.activity_drawer_window"
version = 2

[components.ActivityDrawerWindow]
root = "root"

[nodes.root]
component = "VerticalGroup"
control_id = "ActivityDrawerWindowRoot"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![window_path]).unwrap();

    assert!(outcome
        .store
        .get("editor.workbench.shell.activity_drawer_window")
        .is_some());
    assert!(outcome
        .store
        .get("res://ui/editor/components/workbench/shell/activity_drawer_window.zui")
        .is_some());
    let root = outcome
        .compiled
        .arena
        .root
        .and_then(|handle| outcome.compiled.arena.node(handle))
        .expect("expanded root");
    assert_eq!(root.component, "VerticalGroup");
    assert_eq!(root.control_id.as_deref(), Some("WorkbenchWindow"));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_rejects_widget_imports_with_multiple_component_fragments() {
    let temp_dir = v2_cache_temp_dir("ambiguous_widget_import");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/windows/workbench_window.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.ambiguous_import"
version = 2

[imports]
widgets = ["res://ui/editor/components/button.zui#Button#Unexpected"]

[root]
node = "root"

[nodes.root]
component = "Label"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let error = cache
        .load_store(vec![window_path])
        .expect_err("ambiguous widget imports must fail during file-source discovery");
    assert!(matches!(
        error,
        UiV2AssetError::InvalidDocument { asset_id, detail }
            if asset_id == "editor.window.ambiguous_import"
                && detail.contains("exactly one non-empty #Component suffix")
    ));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_resolves_res_imports_from_package_root_when_source_has_assets_folder() {
    let temp_dir = v2_cache_temp_dir("nested_assets_resource_imports");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/windows/workbench_window.zui");
    let workspace_path =
        assets_root.join("ui/editor/components/workbench/modules/core/assets/workspace.zui");
    let primitive_path =
        assets_root.join("ui/editor/components/workbench/primitives/inputs/button.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(workspace_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(primitive_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.workbench"
version = 2

[imports]
widgets = ["res://ui/editor/components/workbench/modules/core/assets/workspace.zui#Workspace"]

[root]
node = "root"

[nodes.root]
component = "Workspace"
"##,
    )
    .unwrap();
    std::fs::write(
        &workspace_path,
        r##"
[asset]
kind = "component"
id = "editor.workbench.assets_workspace"
version = 2

[imports]
widgets = ["res://ui/editor/components/workbench/primitives/inputs/button.zui#WorkbenchButton"]

[components.Workspace]
root = "root"

[nodes.root]
component = "WorkbenchButton"
control_id = "WorkspaceRoot"
"##,
    )
    .unwrap();
    std::fs::write(
        &primitive_path,
        r##"
[asset]
kind = "component"
id = "editor.workbench.button"
version = 2

[components.WorkbenchButton]
root = "root"

[nodes.root]
component = "Button"
control_id = "ButtonRoot"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![window_path]).unwrap();

    assert!(outcome
        .store
        .get("res://ui/editor/components/workbench/modules/core/assets/workspace.zui")
        .is_some());
    assert!(outcome
        .store
        .get("res://ui/editor/components/workbench/primitives/inputs/button.zui")
        .is_some());

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_applies_zui_profile_for_uppercase_extension() {
    let temp_dir = v2_cache_temp_dir("uppercase_zui_profile");
    let assets_root = temp_dir.join("assets");
    let view_path = assets_root.join("ui/editor/Workbench.ZUI");
    std::fs::create_dir_all(view_path.parent().unwrap()).unwrap();
    std::fs::write(
        &view_path,
        r##"
[asset]
kind = "view"
id = "editor.workbench.uppercase_zui"
version = 2

[root]
node = "root"

[nodes.root]
component = "Container"
control_id = "UppercaseZuiRoot"
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![view_path]).unwrap();

    assert_eq!(outcome.root_asset_id, "editor.workbench.uppercase_zui");
    assert!(outcome.store.get("res://ui/editor/Workbench.ZUI").is_some());
    let root = outcome
        .compiled
        .arena
        .root
        .and_then(|handle| outcome.compiled.arena.node(handle))
        .expect("compiled .zui root");
    assert_eq!(root.control_id.as_deref(), Some("UppercaseZuiRoot"));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn ui_v2_file_cache_prefers_zui_asset_id_over_legacy_v2_document() {
    let temp_dir = v2_cache_temp_dir("asset_id_prefers_zui");
    let assets_root = temp_dir.join("assets");
    let window_path = assets_root.join("ui/editor/window.zui");
    let legacy_component_path = assets_root.join("ui/legacy/shared_component.zui");
    let zui_component_path = assets_root.join("ui/components/shared_component.zui");
    std::fs::create_dir_all(window_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(legacy_component_path.parent().unwrap()).unwrap();
    std::fs::create_dir_all(zui_component_path.parent().unwrap()).unwrap();
    std::fs::write(
        &window_path,
        r##"
[asset]
kind = "view"
id = "editor.window.prefers_zui"
version = 2

[imports]
widgets = ["editor.shared.component#SharedComponent"]

[root]
node = "root"

[nodes.root]
component = "SharedComponent"
control_id = "WindowRoot"
"##,
    )
    .unwrap();
    std::fs::write(
        &legacy_component_path,
        r##"
[asset]
kind = "component"
id = "editor.shared.component"
version = 2

[components.SharedComponent]
root = "legacy_root"

[nodes.legacy_root]
component = "Label"
control_id = "LegacyComponentRoot"
props = { text = "Legacy" }
"##,
    )
    .unwrap();
    std::fs::write(
        &zui_component_path,
        r##"
[asset]
kind = "component"
id = "editor.shared.component"
version = 2

[components.SharedComponent]
root = "zui_root"

[nodes.zui_root]
component = "Button"
control_id = "ZuiComponentRoot"
props = { text = "Zui" }
"##,
    )
    .unwrap();
    let mut cache = UiV2PrototypeStoreFileCache::new();

    let outcome = cache.load_store(vec![window_path]).unwrap();

    assert!(outcome
        .store
        .get("res://ui/components/shared_component.zui")
        .is_some());
    assert!(outcome
        .store
        .get("res://ui/legacy/shared_component.zui")
        .is_none());
    let root = outcome
        .compiled
        .arena
        .root
        .and_then(|handle| outcome.compiled.arena.node(handle))
        .expect("expanded root");
    assert_eq!(root.component, "Button");
    assert_eq!(root.control_id.as_deref(), Some("WindowRoot"));
    assert_eq!(root.props.get("text").and_then(Value::as_str), Some("Zui"));

    let _ = std::fs::remove_dir_all(temp_dir);
}
