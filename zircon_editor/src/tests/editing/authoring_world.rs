use zircon_runtime::scene::{DefaultLevelManager, LevelMetadata, Scene};

use crate::core::editing::authoring_world::{AuthoringWorldSeed, EditorAuthoringWorld};
use crate::core::gateway::{EditorRuntimeGatewayHandle, GatewayError};

#[test]
fn authoring_facade_replaces_and_clears_the_stable_gateway() {
    let handle = EditorRuntimeGatewayHandle::detached();
    let initial =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let replacement =
        DefaultLevelManager::default().create_level(Scene::default(), LevelMetadata::default());
    let mut facade = EditorAuthoringWorld::loaded(&handle, AuthoringWorldSeed::from(initial))
        .expect("initial authoring world");

    assert!(facade.is_loaded());
    assert_eq!(facade.try_with_world(|scene| scene.nodes().len()), Some(0));

    facade
        .replace(AuthoringWorldSeed::from(replacement))
        .expect("replacement authoring world");
    assert_eq!(facade.try_with_world(|scene| scene.nodes().len()), Some(0));

    facade.clear().expect("clear authoring world");
    assert!(!facade.is_loaded());
    assert_eq!(facade.try_snapshot(), None);
    assert_eq!(
        handle.with_world(&mut |_| {}),
        Err(GatewayError::RequiresSerializedAccess)
    );
}
