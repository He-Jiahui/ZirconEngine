mod asset_scene;
mod component_structure;
mod physics_animation_components;
mod property_paths;
mod semantics;
mod support;
mod world_basics;
mod world_driver;

use crate::scene::{DefaultLevelManager, RuntimeObject, RuntimeSystem};

#[test]
fn level_manager_produces_level_systems() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_default_level();
    assert!(manager.level(level.handle()).is_some());
}

#[test]
fn runtime_semantics_keep_ecs_roles_explicit() {
    let level = DefaultLevelManager::default().create_default_level();

    assert_eq!(level.object_kind(), "system");
    assert_eq!(level.system_name(), "LevelSystem");
}
