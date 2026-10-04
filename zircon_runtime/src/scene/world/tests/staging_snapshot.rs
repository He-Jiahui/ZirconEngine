use crate::scene::{DynamicSceneError, Resource, World};

#[derive(Debug, PartialEq, Eq)]
struct LiveOnlyResource(u32);

impl Resource for LiveOnlyResource {}

#[test]
fn dynamic_scene_staging_snapshot_restores_live_only_state_on_success_and_failure() {
    let mut world = World::empty();
    world.insert_resource(LiveOnlyResource(47));

    let (snapshot, _) = world
        .clone_for_dynamic_scene_staging(1024 * 1024)
        .expect("bounded persistent world should clone");

    assert_eq!(
        world.get_resource::<LiveOnlyResource>(),
        Some(&LiveOnlyResource(47))
    );
    assert_eq!(snapshot.get_resource::<LiveOnlyResource>(), None);

    let error = world
        .clone_for_dynamic_scene_staging(0)
        .expect_err("zero bytes must reject the target snapshot");
    assert!(matches!(
        error,
        DynamicSceneError::TargetSnapshotTooLarge { .. }
    ));
    assert_eq!(
        world.get_resource::<LiveOnlyResource>(),
        Some(&LiveOnlyResource(47))
    );
}
