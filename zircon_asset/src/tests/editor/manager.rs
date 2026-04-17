use std::fs;
use std::time::Duration;

use crate::tests::project::unique_temp_project_root;
use crate::tests::support::{
    write_checker_png, write_default_material, write_default_scene, write_triangle_obj,
    write_valid_wgsl,
};
use crate::{
    AssetMetaDocument, AssetUri, DefaultEditorAssetManager, EditorAssetChangeKind,
    EditorAssetManager as EditorAssetManagerFacade, PreviewState, ProjectManifest, ProjectPaths,
};
use zircon_resource::ResourceKind;

#[test]
fn editor_asset_manager_builds_catalog_and_direct_reference_graph() {
    let root = unique_temp_project_root("editor_asset_manager_catalog");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = DefaultEditorAssetManager::default();
    manager.open_project(&root).unwrap();

    let material = manager
        .record_by_locator(&AssetUri::parse("res://materials/grid.material.toml").unwrap())
        .expect("material catalog record")
        .clone();
    let scene = manager
        .record_by_locator(&AssetUri::parse("res://scenes/main.scene.toml").unwrap())
        .expect("scene catalog record")
        .clone();

    let material_refs = manager.direct_references(material.asset_uuid);
    assert_eq!(material_refs.len(), 2);
    assert!(material_refs
        .iter()
        .any(|record| record.locator == AssetUri::parse("res://shaders/pbr.wgsl").unwrap()));
    assert!(material_refs
        .iter()
        .any(|record| record.locator == AssetUri::parse("res://textures/checker.png").unwrap()));

    let scene_refs = manager.direct_references(scene.asset_uuid);
    assert!(scene_refs
        .iter()
        .any(|record| record.locator == AssetUri::parse("res://models/triangle.obj").unwrap()));
    assert!(scene_refs
        .iter()
        .any(|record| record.locator
            == AssetUri::parse("res://materials/grid.material.toml").unwrap()));

    let referenced_by_material = manager.referenced_by(material.asset_uuid);
    assert!(referenced_by_material
        .iter()
        .any(|record| record.locator == AssetUri::parse("res://scenes/main.scene.toml").unwrap()));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn editor_asset_manager_marks_preview_dirty_and_refreshes_visible_assets() {
    let root = unique_temp_project_root("editor_asset_manager_preview");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = DefaultEditorAssetManager::default();
    manager.open_project(&root).unwrap();

    let scene = manager
        .record_by_locator(&AssetUri::parse("res://scenes/main.scene.toml").unwrap())
        .expect("scene catalog record")
        .clone();
    let preview_path = manager.preview_artifact_path(scene.asset_uuid);

    manager.mark_preview_dirty(scene.asset_uuid).unwrap();
    assert_eq!(
        manager
            .record_by_uuid(scene.asset_uuid)
            .unwrap()
            .preview_state,
        PreviewState::Dirty
    );
    assert!(!preview_path.exists());

    manager
        .request_preview_refresh(scene.asset_uuid, false)
        .unwrap();
    assert_eq!(
        manager
            .record_by_uuid(scene.asset_uuid)
            .unwrap()
            .preview_state,
        PreviewState::Dirty
    );
    assert!(!preview_path.exists());

    manager
        .request_preview_refresh(scene.asset_uuid, true)
        .unwrap();
    assert_eq!(
        manager
            .record_by_uuid(scene.asset_uuid)
            .unwrap()
            .preview_state,
        PreviewState::Ready
    );
    assert!(preview_path.exists());

    let meta = AssetMetaDocument::load(
        manager
            .record_by_uuid(scene.asset_uuid)
            .unwrap()
            .meta_path
            .clone(),
    )
    .unwrap();
    assert_eq!(meta.preview_state, PreviewState::Ready);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn editor_asset_manager_catalog_snapshot_exposes_folder_tree_and_details() {
    let root = unique_temp_project_root("editor_asset_manager_snapshot");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = DefaultEditorAssetManager::default();
    manager.open_project(&root).unwrap();

    let snapshot = EditorAssetManagerFacade::catalog_snapshot(&manager);
    assert_eq!(snapshot.project_name, "Sandbox");
    assert_eq!(snapshot.default_scene_uri, "res://scenes/main.scene.toml");
    assert_eq!(snapshot.project_root, root.to_string_lossy());
    assert!(snapshot
        .folders
        .iter()
        .any(|folder| folder.folder_id == "res://"));
    assert!(snapshot
        .folders
        .iter()
        .any(|folder| folder.folder_id == "res://materials"));
    assert!(snapshot
        .folders
        .iter()
        .any(|folder| folder.folder_id == "res://scenes"));

    let material = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://materials/grid.material.toml")
        .expect("material snapshot");
    assert_eq!(material.display_name, "grid.material");
    assert_eq!(material.file_name, "grid.material.toml");
    assert_eq!(material.extension, "toml");
    assert_eq!(material.preview_state, PreviewState::Dirty);
    assert_eq!(material.direct_reference_uuids.len(), 2);

    let details = EditorAssetManagerFacade::asset_details(&manager, &material.uuid)
        .expect("material details");
    assert_eq!(details.asset.uuid, material.uuid);
    assert_eq!(details.direct_references.len(), 2);
    assert!(details
        .direct_references
        .iter()
        .any(|reference| reference.locator == "res://shaders/pbr.wgsl"));
    assert!(details
        .referenced_by
        .iter()
        .any(|reference| reference.locator == "res://scenes/main.scene.toml"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn editor_asset_manager_emits_catalog_and_preview_change_events() {
    let root = unique_temp_project_root("editor_asset_manager_events");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = DefaultEditorAssetManager::default();
    let events = EditorAssetManagerFacade::subscribe_editor_asset_changes(&manager);

    manager.open_project(&root).unwrap();

    let catalog_event = events.recv_timeout(Duration::from_millis(250)).unwrap();
    assert_eq!(catalog_event.kind, EditorAssetChangeKind::CatalogChanged);
    assert!(catalog_event.catalog_revision >= 1);

    let snapshot = EditorAssetManagerFacade::catalog_snapshot(&manager);
    let scene = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://scenes/main.scene.toml")
        .expect("scene snapshot");

    EditorAssetManagerFacade::request_preview_refresh(&manager, &scene.uuid, true).unwrap();

    let preview_event = events.recv_timeout(Duration::from_millis(250)).unwrap();
    assert_eq!(preview_event.kind, EditorAssetChangeKind::PreviewChanged);
    assert_eq!(preview_event.uuid.as_deref(), Some(scene.uuid.as_str()));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn editor_asset_manager_keeps_last_good_preview_when_marked_dirty() {
    let root = unique_temp_project_root("editor_asset_manager_last_good_preview");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let manager = DefaultEditorAssetManager::default();
    manager.open_project(&root).unwrap();

    let texture = EditorAssetManagerFacade::catalog_snapshot(&manager)
        .assets
        .into_iter()
        .find(|asset| asset.locator == "res://textures/checker.png")
        .expect("texture snapshot");
    EditorAssetManagerFacade::request_preview_refresh(&manager, &texture.uuid, true).unwrap();

    let preview_path = manager
        .record_by_uuid(texture.uuid.parse().unwrap())
        .unwrap()
        .preview_artifact_path;
    assert!(preview_path.exists());

    EditorAssetManagerFacade::mark_preview_dirty(&manager, &texture.uuid).unwrap();
    let details =
        EditorAssetManagerFacade::asset_details(&manager, &texture.uuid).expect("texture details");
    assert_eq!(details.asset.preview_state, PreviewState::Dirty);
    assert!(details.asset.preview_artifact_path.ends_with(".png"));
    assert!(preview_path.exists());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn editor_asset_manager_catalog_snapshot_includes_ui_asset_kinds_and_references() {
    let root = unique_temp_project_root("editor_asset_manager_ui_assets");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(
        paths
            .assets_root()
            .join("materials")
            .join("grid.material.toml"),
    );
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));
    write_ui_assets(&paths);

    let manager = DefaultEditorAssetManager::default();
    manager.open_project(&root).unwrap();

    let snapshot = EditorAssetManagerFacade::catalog_snapshot(&manager);
    let layout = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://ui/layouts/editor.ui.toml")
        .expect("ui layout asset");
    let widget = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://ui/widgets/button.ui.toml")
        .expect("ui widget asset");
    let style = snapshot
        .assets
        .iter()
        .find(|asset| asset.locator == "res://ui/styles/theme.ui.toml")
        .expect("ui style asset");

    assert_eq!(layout.kind, ResourceKind::UiLayout);
    assert_eq!(widget.kind, ResourceKind::UiWidget);
    assert_eq!(style.kind, ResourceKind::UiStyle);

    let layout_details =
        EditorAssetManagerFacade::asset_details(&manager, &layout.uuid).expect("ui layout details");
    assert!(layout_details
        .direct_references
        .iter()
        .any(|reference| reference.locator == "res://ui/widgets/button.ui.toml"));
    assert!(layout_details
        .direct_references
        .iter()
        .any(|reference| reference.locator == "res://ui/styles/theme.ui.toml"));

    let style_details =
        EditorAssetManagerFacade::asset_details(&manager, &style.uuid).expect("ui style details");
    assert!(style_details.referenced_by.iter().any(|reference| {
        reference.locator == "res://ui/layouts/editor.ui.toml"
            || reference.locator == "res://ui/widgets/button.ui.toml"
    }));

    let _ = fs::remove_dir_all(root);
}

fn write_ui_assets(paths: &ProjectPaths) {
    let widget_path = paths
        .assets_root()
        .join("ui")
        .join("widgets")
        .join("button.ui.toml");
    let style_path = paths
        .assets_root()
        .join("ui")
        .join("styles")
        .join("theme.ui.toml");
    let layout_path = paths
        .assets_root()
        .join("ui")
        .join("layouts")
        .join("editor.ui.toml");
    fs::create_dir_all(widget_path.parent().unwrap()).unwrap();
    fs::create_dir_all(style_path.parent().unwrap()).unwrap();
    fs::create_dir_all(layout_path.parent().unwrap()).unwrap();

    fs::write(
        &widget_path,
        r#"
[asset]
kind = "widget"
id = "ui.widgets.button"
version = 1
display_name = "Button Widget"

[imports]
styles = ["res://ui/styles/theme.ui.toml"]

[root]
node = "root"

[components.ButtonRoot]
root = "root"

[nodes.root]
kind = "native"
type = "Button"
classes = ["primary"]
props = { text = "Press" }
"#,
    )
    .unwrap();

    fs::write(
        &style_path,
        r##"
[asset]
kind = "style"
id = "ui.styles.theme"
version = 1
display_name = "Theme"

[[stylesheets]]
id = "theme"

[[stylesheets.rules]]
selector = ".primary"
set = { self.background = { color = "#224488" } }
"##,
    )
    .unwrap();

    fs::write(
        &layout_path,
        r#"
[asset]
kind = "layout"
id = "ui.layouts.editor"
version = 1
display_name = "Editor Layout"

[imports]
widgets = ["res://ui/widgets/button.ui.toml#ButtonRoot"]
styles = ["res://ui/styles/theme.ui.toml"]

[root]
node = "root"

[nodes.root]
kind = "reference"
component_ref = "res://ui/widgets/button.ui.toml#ButtonRoot"
"#,
    )
    .unwrap();
}

#[test]
fn editor_asset_manager_is_split_by_responsibility_modules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("editor")
        .join("manager");

    for relative in [
        "mod.rs",
        "default_editor_asset_manager.rs",
        "project_sync.rs",
        "preview_refresh.rs",
        "folder_projection.rs",
        "reference_analysis.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected editor asset manager module {relative} under {:?}",
            root
        );
    }
}
