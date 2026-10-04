use super::RenderSceneRegistry;
use crate::core::framework::render::RenderWorldSnapshotHandle;

#[test]
fn registry_keys_persistent_scene_ownership_by_world_lineage_not_snapshot_generation() {
    let mut registry = RenderSceneRegistry::default();
    let world = RenderWorldSnapshotHandle::new(41);

    registry.projector_for_world(world.with_generation(3));
    registry.projector_for_world(world.with_generation(4));

    assert_eq!(registry.projectors.len(), 1);
    assert_eq!(
        registry
            .projectors
            .get(&world.raw())
            .expect("world projector")
            .read()
            .world(),
        world.with_generation(0)
    );
}

#[test]
fn failed_world_release_staging_preserves_the_persistent_scene() {
    let mut registry = RenderSceneRegistry::default();
    let world = RenderWorldSnapshotHandle::new(42);
    registry.projector_for_world(world.with_generation(7));

    let result = registry.release_world_with_staging(world, |_| Err::<(), _>("backpressure"));

    assert_eq!(result, Err("backpressure"));
    assert!(registry.projectors.contains_key(&world.raw()));
}

#[test]
fn accepted_world_release_removes_exactly_one_world_lineage() {
    let mut registry = RenderSceneRegistry::default();
    let released = RenderWorldSnapshotHandle::new(43);
    let retained = RenderWorldSnapshotHandle::new(44);
    registry.projector_for_world(released.with_generation(2));
    registry.projector_for_world(retained.with_generation(9));

    let released_delta_count = registry
        .release_world_with_staging(released, |deltas| Ok::<_, ()>(deltas.len()))
        .expect("release staging")
        .expect("registered world");

    assert_eq!(released_delta_count, 0);
    assert!(!registry.projectors.contains_key(&released.raw()));
    assert!(registry.projectors.contains_key(&retained.raw()));
    assert_eq!(
        registry
            .release_world_with_staging(released, |_| Ok::<_, ()>(()))
            .expect("unknown release"),
        None
    );
}
