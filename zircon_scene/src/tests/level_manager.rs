use std::fs;

use zircon_asset::{AssetUri, ImportedAsset, ProjectManager};
use zircon_manager::LevelManager;

use crate::DefaultLevelManager;

use super::support::{create_test_project, unique_temp_project_root};

#[test]
fn level_manager_produces_level_systems() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    assert!(manager.level(level.handle()).is_some());
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
