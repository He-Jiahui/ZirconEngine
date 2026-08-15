use super::*;

#[test]
fn commands_are_deferred_until_apply_deferred() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Queued".to_string()),)).unwrap();
    let spawned;

    {
        let mut commands = world.commands();
        commands.entity(entity).insert((Health(7),));
        spawned = commands
            .spawn((Name("Spawned".to_string()), Health(3)))
            .into_deferred_entity();
        commands.insert_resource(Score(9));
    }

    assert!(world.get::<Health>(entity).is_none());
    assert!(world.get_resource::<Score>().is_none());
    assert_eq!(world.query::<&Health>().iter(&world).count(), 0);

    let report = world.apply_deferred();
    let spawned = report
        .resolve(&spawned)
        .expect("published deferred spawn must resolve only after the barrier");

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(9)));
    assert_eq!(world.query::<&Health>().iter(&world).count(), 2);
    assert_eq!(
        world.get::<Name>(spawned).map(|name| name.0.as_str()),
        Some("Spawned")
    );
}

#[test]
fn entity_commands_spawn_empty_and_deferred_handle_apply_in_queue_order() {
    let mut world = World::empty();
    let reserved = {
        let mut commands = world.commands();
        let mut reserved = commands.spawn_empty();
        reserved.insert((Name("Reserved".to_string()), Health(1)));
        let reserved = reserved.into_deferred_entity();
        reserved
    };

    assert_eq!(world.query::<&Health>().iter(&world).count(), 0);

    let report = world.apply_deferred();
    let reserved = report
        .resolve(&reserved)
        .expect("successful deferred spawn must resolve after publication");

    assert_eq!(world.get::<Name>(reserved).unwrap().0, "Reserved");
    assert_eq!(world.get::<Health>(reserved), Some(&Health(1)));
}
