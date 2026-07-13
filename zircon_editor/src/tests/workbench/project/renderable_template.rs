use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};

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

    let loaded = EditorProjectDocument::load_from_path(&created_root).unwrap();
    assert_eq!(loaded.manifest.name, "WelcomeProject");
    assert_eq!(
        loaded.manifest.default_scene.to_string(),
        "res://scenes/main.scene.toml"
    );
    assert!(!loaded.world.nodes().is_empty());

    let _ = fs::remove_dir_all(&location);
}
