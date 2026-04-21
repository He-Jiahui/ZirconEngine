use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use zircon_runtime::asset::project::ProjectPaths;

use crate::ui::workbench::project::EditorProjectDocument;
use crate::ui::workbench::startup::{NewProjectDraft, NewProjectTemplate};

#[test]
fn create_renderable_template_scaffolds_directory_project_defaults() {
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

    let created_root = EditorProjectDocument::create_renderable_template(&draft).unwrap();
    let paths = ProjectPaths::from_root(&created_root).unwrap();

    assert!(paths.root().exists());
    assert!(paths.manifest_path().exists());
    assert!(paths
        .assets_root()
        .join("scenes")
        .join("main.scene.toml")
        .exists());
    assert!(paths
        .assets_root()
        .join("materials")
        .join("default.material.toml")
        .exists());
    assert!(paths
        .assets_root()
        .join("shaders")
        .join("pbr.wgsl")
        .exists());
    assert!(paths.library_root().exists());

    let loaded = EditorProjectDocument::load_from_path(&created_root).unwrap();
    assert_eq!(loaded.manifest.name, "WelcomeProject");
    assert_eq!(
        loaded.manifest.default_scene.to_string(),
        "res://scenes/main.scene.toml"
    );
    assert!(!loaded.world.nodes().is_empty());

    let _ = fs::remove_dir_all(&location);
}
