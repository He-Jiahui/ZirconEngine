use std::fs;

use zircon_runtime::scene::{NodeKind, World};

use super::{PlaySceneSource, PlaySnapshotStore};

fn test_root(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("zircon-editor-play-{name}-{}", std::process::id()))
}

#[test]
fn snapshot_materialization_roundtrips_versioned_scene_and_cleans_owned_root() {
    let root = test_root("snapshot");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let mut world = World::empty();
    world.spawn_node(NodeKind::Cube);
    let source = PlaySceneSource::from_world(&world).unwrap();
    let store = PlaySnapshotStore::default();
    let mut scene = store.materialize(&root, &source).unwrap();

    let text = fs::read_to_string(scene.path()).unwrap();
    let decoded = zircon_runtime::scene::DynamicScene::from_versioned_json(&text).unwrap();
    assert_eq!(decoded.entities.len(), 1);
    let owned_root = scene.path().parent().unwrap().to_path_buf();
    scene.cleanup().unwrap();
    assert!(!owned_root.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn persisted_scene_is_borrowed_and_never_deleted() {
    let root = test_root("persisted");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("scenes")).unwrap();
    let path = root.join("scenes/main.scene.toml");
    fs::write(&path, "scene = true").unwrap();
    let store = PlaySnapshotStore::default();
    let mut scene = store
        .materialize(&root, &PlaySceneSource::persisted("scenes/main.scene.toml"))
        .unwrap();

    assert_eq!(scene.path(), path);
    scene.cleanup().unwrap();
    assert!(path.exists());
    let _ = fs::remove_dir_all(root);
}
