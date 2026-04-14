//! ECS worlds, level systems, persistence, and render extraction.

pub type EntityId = u64;
pub type NodeId = EntityId;

mod components;
mod level_system;
mod module;
mod serializer;
mod world;

pub use components::{
    Active, CameraComponent, DirectionalLight, Hierarchy, LocalTransform, MeshRenderer, Name,
    NodeKind, NodeRecord, RenderCameraSnapshot, RenderDirectionalLightSnapshot,
    RenderExtractPacket, RenderGizmoSnapshot, RenderMeshSnapshot, RenderSceneSnapshot, SceneNode,
    Schedule, SystemStage, WorldTransform,
};
pub use level_system::{LevelLifecycleState, LevelMetadata, LevelSystem};
pub use module::{
    create_default_level, load_level_asset, module_descriptor, DefaultLevelManager,
    SCENE_MODULE_NAME, WORLD_DRIVER_NAME, LEVEL_MANAGER_NAME, DEFAULT_LEVEL_MANAGER_NAME,
    WorldDriver,
};
pub use serializer::SceneAssetSerializer;
pub use world::{SceneProjectError, World};

pub type Scene = World;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use image::{ImageBuffer, ImageFormat, Rgba};
    use zircon_asset::{
        AlphaMode, AssetReference, AssetUri, ImportedAsset, MaterialAsset, ProjectManager,
        ProjectManifest, ProjectPaths, SceneAsset, SceneCameraAsset, SceneEntityAsset,
        SceneMeshInstanceAsset, TransformAsset,
    };
    use zircon_math::{Transform, Vec3};
    use zircon_manager::LevelManager;
    use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

    use super::{DefaultLevelManager, NodeKind, SystemStage, World};

    fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }

    fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
        ResourceHandle::new(ResourceId::from_stable_label(label))
    }

    fn asset_reference(uri: &str) -> AssetReference {
        AssetReference::from_locator(AssetUri::parse(uri).unwrap())
    }

    fn project_model_handle(project: &ProjectManager, uri: &str) -> ResourceHandle<ModelMarker> {
        let uri = AssetUri::parse(uri).unwrap();
        ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
    }

    fn project_material_handle(
        project: &ProjectManager,
        uri: &str,
    ) -> ResourceHandle<MaterialMarker> {
        let uri = AssetUri::parse(uri).unwrap();
        ResourceHandle::new(project.asset_id_for_uri(&uri).unwrap())
    }

    #[test]
    fn world_bootstraps_with_renderable_defaults() {
        let world = World::new();
        let snapshot = world.to_render_snapshot();

        assert!(!snapshot.meshes.is_empty());
        assert!(snapshot.show_grid);
        assert_eq!(
            world.schedule().stages.last(),
            Some(&SystemStage::RenderExtract)
        );
    }

    #[test]
    fn spawned_entities_have_unique_ids() {
        let mut world = World::new();
        let first = world.spawn_node(NodeKind::Cube);
        let second = world.spawn_node(NodeKind::Cube);
        assert_ne!(first, second);
    }

    #[test]
    fn hierarchy_updates_world_transform() {
        let mut world = World::new();
        let parent = world.spawn_node(NodeKind::Cube);
        let child = world.spawn_node(NodeKind::Mesh);
        world.update_transform(
            parent,
            Transform::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        );
        world.update_transform(child, Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)));
        world.set_parent(child, Some(parent));

        assert_eq!(
            world.world_transform(child).unwrap().translation,
            Vec3::new(7.0, 0.0, 0.0)
        );
    }

    #[test]
    fn updated_transform_is_reflected_in_render_extract() {
        let mut world = World::new();
        let cube = world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id;
        world.update_transform(cube, Transform::from_translation(Vec3::new(2.0, 3.0, 4.0)));

        let snapshot = world.to_render_extract();
        let mesh_snapshot = snapshot
            .meshes
            .iter()
            .find(|mesh_snapshot| mesh_snapshot.node_id == cube)
            .unwrap();
        assert_eq!(
            mesh_snapshot.transform.translation,
            Vec3::new(2.0, 3.0, 4.0)
        );
    }

    #[test]
    fn project_roundtrip_preserves_imported_meshes() {
        let mut world = World::new();
        let imported = world.spawn_mesh_node(
            model_handle("res://models/robot.obj"),
            material_handle("res://materials/robot.material.toml"),
        );
        world.set_selected(Some(imported));

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("zircon_scene_roundtrip_{unique}.json"));
        world.save_project_to_path(&path).unwrap();
        let loaded = World::load_project_from_path(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded.selected_node(), Some(imported));
        let imported_node = loaded.find_node(imported).unwrap();
        assert!(matches!(imported_node.kind, NodeKind::Mesh));
        assert_eq!(
            imported_node.mesh.as_ref().unwrap().model,
            model_handle("res://models/robot.obj")
        );
    }

    #[test]
    fn level_manager_produces_level_systems() {
        let manager = DefaultLevelManager::default();
        let level = manager.create_default_level();
        assert!(manager.level(level.handle()).is_some());
    }

    #[test]
    fn node_record_roundtrip_restores_same_entity() {
        let mut world = World::new();
        let cube = world.spawn_node(NodeKind::Cube);
        let record = world.node_record(cube).unwrap();

        assert!(world.remove_entity(cube));
        assert!(!world.contains_entity(cube));

        world.insert_node_record(record.clone()).unwrap();

        let restored = world.node_record(cube).unwrap();
        assert_eq!(restored, record);
    }

    #[test]
    fn recursive_remove_returns_parent_and_children_records() {
        let mut world = World::new();
        let parent = world.spawn_node(NodeKind::Cube);
        let child = world.spawn_node(NodeKind::Mesh);
        world.set_parent_checked(child, Some(parent)).unwrap();

        let removed = world.remove_entity_recursive(parent);
        assert_eq!(removed.len(), 2);
        assert!(!world.contains_entity(parent));
        assert!(!world.contains_entity(child));
    }

    #[test]
    fn set_parent_checked_rejects_hierarchy_cycles() {
        let mut world = World::new();
        let parent = world.spawn_node(NodeKind::Cube);
        let child = world.spawn_node(NodeKind::Mesh);
        world.set_parent_checked(child, Some(parent)).unwrap();

        let error = world.set_parent_checked(parent, Some(child)).unwrap_err();

        assert!(error.contains("cycle"));
        assert_eq!(world.find_node(parent).unwrap().parent, None);
        assert_eq!(world.find_node(child).unwrap().parent, Some(parent));
    }

    #[test]
    fn scene_assets_instantiate_world_with_asset_bound_meshes() {
        let root = unique_temp_project_root("scene_asset");
        let project = create_test_project(&root);
        let world = World::load_scene_from_uri(
            &project,
            &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        )
        .unwrap();

        let mesh_node = world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Mesh))
            .unwrap();
        let mesh = mesh_node.mesh.as_ref().unwrap();
        assert_eq!(
            mesh.model,
            project_model_handle(&project, "res://models/triangle.obj")
        );
        assert_eq!(
            mesh.material,
            project_material_handle(&project, "res://materials/grid.material.toml")
        );

        let saved = world.to_scene_asset(&project).unwrap();
        let saved_mesh = saved
            .entities
            .iter()
            .find_map(|entity| entity.mesh.as_ref())
            .unwrap();
        assert_eq!(saved_mesh.model.to_string(), "res://models/triangle.obj");
        assert_eq!(
            saved_mesh.material.to_string(),
            "res://materials/grid.material.toml"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn render_extract_keeps_gizmo_overlay_for_asset_bound_meshes() {
        let root = unique_temp_project_root("scene_gizmo");
        let project = create_test_project(&root);
        let mut world = World::load_scene_from_uri(
            &project,
            &AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        )
        .unwrap();
        let mesh_node = world
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Mesh))
            .unwrap()
            .id;
        world.set_selected(Some(mesh_node));

        let extract = world.to_render_extract();
        let mesh = extract
            .meshes
            .iter()
            .find(|mesh| mesh.node_id == mesh_node)
            .unwrap();
        assert_eq!(
            mesh.model,
            project_model_handle(&project, "res://models/triangle.obj")
        );
        assert_eq!(
            mesh.material,
            project_material_handle(&project, "res://materials/grid.material.toml")
        );
        assert_eq!(extract.gizmo.unwrap().target_node, mesh_node);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn level_manager_instantiates_and_saves_scene_assets() {
        let root = unique_temp_project_root("scene_manager");
        let project = create_test_project(&root);
        let manager = DefaultLevelManager::default();
        let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();

        let level = manager.load_level(&project, &scene_uri).unwrap();
        let summary = manager.level_summary(level.handle()).unwrap();
        assert!(summary.entity_count >= 2);

        let saved_uri = AssetUri::parse("res://scenes/saved.scene.toml").unwrap();
        manager
            .save_level(level.handle(), &project, &saved_uri)
            .unwrap();

        let mut reloaded_project = ProjectManager::open(&root).unwrap();
        reloaded_project.scan_and_import().unwrap();
        let ImportedAsset::Scene(scene) = reloaded_project.load_artifact(&saved_uri).unwrap() else {
            panic!("saved scene did not reimport as scene asset");
        };
        assert_eq!(scene.entities.len(), 3);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn level_manager_facade_loads_and_saves_directory_project_scenes() {
        let root = unique_temp_project_root("level_manager_facade");
        let manager = DefaultLevelManager::default();
        let scene_uri = AssetUri::parse("res://scenes/main.scene.toml").unwrap();
        let project = create_test_project(&root);
        drop(project);

        let handle = LevelManager::load_level_asset(
            &manager,
            root.to_string_lossy().as_ref(),
            &scene_uri.to_string(),
        )
        .unwrap();
        let summary = manager.level_summary(handle).unwrap();
        assert!(summary.entity_count >= 2);

        let saved_uri = AssetUri::parse("res://scenes/facade.scene.toml").unwrap();
        LevelManager::save_level_asset(
            &manager,
            handle,
            root.to_string_lossy().as_ref(),
            &saved_uri.to_string(),
        )
        .unwrap();

        let mut reloaded = ProjectManager::open(&root).unwrap();
        reloaded.scan_and_import().unwrap();
        let ImportedAsset::Scene(scene) = reloaded.load_artifact(&saved_uri).unwrap() else {
            panic!("saved scene did not reimport as scene asset");
        };
        assert!(scene.entities.len() >= 2);

        let _ = fs::remove_dir_all(root);
    }

    fn unique_temp_project_root(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zircon_scene_{label}_{unique}"))
    }

    fn create_test_project(root: &PathBuf) -> ProjectManager {
        let paths = ProjectPaths::from_root(root).unwrap();
        paths.ensure_layout().unwrap();
        ProjectManifest::new(
            "SceneSandbox",
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

        let mut project = ProjectManager::open(root).unwrap();
        project.scan_and_import().unwrap();
        project
    }

    fn write_valid_wgsl(path: PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            path,
            r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    let x = f32(i32(vertex_index) - 1);
    return vec4f(x, 0.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f {
    return vec4f(1.0, 0.4, 0.2, 1.0);
}
"#,
        )
        .unwrap();
    }

    fn write_checker_png(path: PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        ImageBuffer::<Rgba<u8>, _>::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            }
        })
        .save_with_format(path, ImageFormat::Png)
        .unwrap();
    }

    fn write_triangle_obj(path: PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(
            path,
            "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
        )
        .unwrap();
    }

    fn write_default_material(path: PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let material = MaterialAsset {
            name: Some("Grid".to_string()),
            shader: asset_reference("res://shaders/pbr.wgsl"),
            base_color: [0.8, 0.8, 0.8, 1.0],
            base_color_texture: Some(asset_reference("res://textures/checker.png")),
            normal_texture: None,
            metallic: 0.1,
            roughness: 0.8,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            emissive: [0.0, 0.0, 0.0],
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        };
        fs::write(path, material.to_toml_string().unwrap()).unwrap();
    }

    fn write_default_scene(path: PathBuf) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let scene = SceneAsset {
            entities: vec![
                SceneEntityAsset {
                    entity: 1,
                    name: "Camera".to_string(),
                    parent: None,
                    transform: TransformAsset {
                        translation: [0.0, 2.0, 5.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                    active: true,
                    camera: Some(SceneCameraAsset {
                        fov_y_radians: 1.0471976,
                        z_near: 0.1,
                        z_far: 200.0,
                    }),
                    mesh: None,
                    directional_light: None,
                },
                SceneEntityAsset {
                    entity: 2,
                    name: "Triangle".to_string(),
                    parent: None,
                    transform: TransformAsset {
                        translation: [0.0, 0.0, 0.0],
                        rotation: [0.0, 0.0, 0.0, 1.0],
                        scale: [1.0, 1.0, 1.0],
                    },
                active: true,
                camera: None,
                mesh: Some(SceneMeshInstanceAsset {
                    model: asset_reference("res://models/triangle.obj"),
                    material: asset_reference("res://materials/grid.material.toml"),
                }),
                directional_light: None,
            },
            ],
        };
        fs::write(path, scene.to_toml_string().unwrap()).unwrap();
    }
}
