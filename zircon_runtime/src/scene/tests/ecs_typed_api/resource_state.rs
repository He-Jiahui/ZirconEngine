use super::*;

#[test]
fn world_resources_are_registered_and_replaced_by_type() {
    let mut world = World::empty();

    assert!(!world.contains_resource::<FrameCounter>());
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    let resource_id = world.resource_id::<FrameCounter>();
    assert_eq!(resource_id.index(), 0);
    assert_eq!(world.insert_resource(FrameCounter(1)), None);
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(1));
    assert!(world.contains_resource::<FrameCounter>());
    assert!(world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    world.clear_trackers();
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    world.resource_mut::<FrameCounter>().0 += 1;
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    assert_eq!(
        world.insert_resource(FrameCounter(9)),
        Some(FrameCounter(2))
    );
    assert_eq!(world.resource::<FrameCounter>(), &FrameCounter(9));
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());

    assert_eq!(
        world.remove_resource::<FrameCounter>(),
        Some(FrameCounter(9))
    );
    assert!(!world.contains_resource::<FrameCounter>());
    assert!(!world.is_resource_added::<FrameCounter>());
    assert!(!world.is_resource_changed::<FrameCounter>());

    world.clear_trackers();
    assert_eq!(world.insert_resource(FrameCounter(4)), None);
    assert!(world.is_resource_added::<FrameCounter>());
    assert!(world.is_resource_changed::<FrameCounter>());
    assert_eq!(
        world.registered_resource_id::<FrameCounter>(),
        Some(resource_id)
    );
}
