use std::fs;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::asset::project::{ProjectManager, ProjectManifest};
use crate::asset::AssetUri;
use crate::core::framework::scene::{SceneArtifactTerminal, SceneArtifactWaitResult};
use crate::core::resource::ResourceLocator;
use crate::core::runtime::{EngineTaskGraph, EngineTaskGraphOptions};
use crate::scene::{world::SceneProjectError, DefaultLevelManager, LevelMetadata, World};

const DEFAULT_LEVEL_MANAGER_SOURCE: &str = include_str!("../default_level_manager.rs");

#[test]
fn standalone_level_manager_rejects_artifact_io_without_an_implicit_process_owner() {
    let manager = DefaultLevelManager::default();
    let level = manager.create_level(World::empty(), LevelMetadata::default());

    let error = manager
        .save_world(
            level.handle(),
            "standalone-manager-must-not-write.scene.json",
        )
        .expect_err("standalone scene managers must not acquire a process task owner");

    assert!(matches!(error, SceneProjectError::RuntimeUnavailable));
    assert!(!DEFAULT_LEVEL_MANAGER_SOURCE.contains("TaskPools::process_default()"));
    assert!(DEFAULT_LEVEL_MANAGER_SOURCE.contains("scene_io_pool: Option<TaskPool>"));
}

#[test]
fn scene_save_returns_a_ticket_and_persists_on_the_bounded_io_lane() {
    let task_graph =
        EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1)).unwrap();
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "zircon_scene_artifact_ticket_{}_{}",
        std::process::id(),
        unique
    ));
    fs::create_dir_all(root.join("assets/scenes")).unwrap();
    ProjectManifest::new(
        "Scene Artifact Ticket Fixture",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(root.join("zircon-project.toml"))
    .unwrap();
    let project = ProjectManager::open(&root).unwrap();
    let manager = DefaultLevelManager::with_scene_io_pool(task_graph.worker_pool().clone());
    let level = manager.create_level(World::empty(), LevelMetadata::default());
    let uri = ResourceLocator::parse("res://scenes/main.scene.toml").unwrap();

    let ticket = manager.save_level(level.handle(), &project, &uri).unwrap();

    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(10)),
        SceneArtifactWaitResult::Terminal(SceneArtifactTerminal::Succeeded)
    );
    assert!(root.join("assets/scenes/main.scene.toml").is_file());

    drop(manager);
    task_graph
        .shutdown(Duration::from_secs(2))
        .expect("scene artifact worker should join after its lane closes");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn world_project_document_rejects_bytes_beyond_the_lane_quote() {
    let error = World::empty().project_document_bytes(1).unwrap_err();

    assert!(error
        .to_string()
        .contains("scene artifact exceeds 1 byte limit"));
}
