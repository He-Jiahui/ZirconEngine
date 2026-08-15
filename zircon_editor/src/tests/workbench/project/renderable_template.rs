use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::resource::{AssetUuid, ResourceId};

use crate::core::project::{NewProjectDraft, NewProjectTemplate, ProjectAuthority};
use crate::ui::workbench::project::EditorProjectDocument;

#[test]
fn project_authority_scaffolds_directory_project_defaults() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let location = std::env::temp_dir().join(format!("zircon_editor_welcome_{unique}"));
    fs::create_dir_all(&location).unwrap();
    let draft = NewProjectDraft {
        project_name: "WelcomeProject".to_string(),
        location: location.to_string_lossy().into_owned(),
        template: NewProjectTemplate::RenderableEmpty,
    };

    let created_root = ProjectAuthority::default()
        .create_project(&draft)
        .unwrap()
        .root;
    let paths = ProjectPaths::from_root(&created_root).unwrap();

    assert!(paths.root().exists());
    assert!(paths.manifest_path().exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("scenes")
        .join("main.scene.toml")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("default.zmaterial")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("materials")
        .join("default.zmaterial.zmeta")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("models")
        .join("cube.obj")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("models")
        .join("cube.obj.zmeta")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("pbr_shader.zmeta")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("pbr_shader")
        .join("pbr.zshader")
        .exists());
    assert!(paths
        .asset_root(&zircon_runtime_interface::project::RelPath::project_assets())
        .join("shaders")
        .join("pbr_shader")
        .join("pbr.wgsl")
        .exists());
    assert!(paths.cache_root().exists());
    assert!(paths.registry_root().exists());

    let mut project = ProjectManager::open(&created_root).unwrap();
    project.scan_and_import().unwrap();

    let loaded = EditorProjectDocument::load_from_project_for_tests(&project).unwrap();
    assert_eq!(loaded.manifest.name, "WelcomeProject");
    assert_eq!(
        loaded.manifest.default_scene.to_string(),
        "res://scenes/main.scene.toml"
    );
    let camera = loaded.world.node_record(1).unwrap();
    assert_eq!(camera.name, "Camera");
    assert!(camera.active);
    assert!(camera.camera.is_some());

    let sun = loaded.world.node_record(2).unwrap();
    assert_eq!(sun.name, "Sun");
    assert!(sun.active);
    assert_eq!(sun.mobility, Mobility::Static);
    assert!(sun.directional_light.is_some());

    let cube = loaded.world.node_record(3).unwrap();
    assert_eq!(cube.name, "Cube");
    assert!(cube.active);
    assert_eq!(cube.mobility, Mobility::Static);
    let mesh = cube
        .mesh
        .expect("canonical cube must retain its mesh renderer");
    assert_eq!(
        mesh.model.id(),
        project_resource_id("00000000-0000-0000-0000-000000000002"),
        "the canonical cube model must resolve through its persisted project asset reference"
    );
    assert_eq!(
        mesh.material.id(),
        project_resource_id("00000000-0000-0000-0000-000000000003"),
        "the canonical cube material must resolve through its persisted project asset reference"
    );

    let _ = fs::remove_dir_all(&location);
}

fn project_resource_id(asset_uuid: &str) -> ResourceId {
    ResourceId::from_asset_uuid(asset_uuid.parse::<AssetUuid>().unwrap())
}
