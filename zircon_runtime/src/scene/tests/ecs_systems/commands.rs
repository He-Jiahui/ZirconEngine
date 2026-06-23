use super::*;

#[test]
fn commands_are_deferred_until_apply_deferred() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Queued".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.entity(entity).insert((Health(7),));
        let spawned = commands
            .spawn((Name("Spawned".to_string()), Health(3)))
            .id();
        commands.insert_resource(Score(9));
        assert_eq!(spawned, 2);
    }

    assert!(world.get::<Health>(entity).is_none());
    assert!(world.get_resource::<Score>().is_none());
    assert_eq!(world.query::<&Health>().iter(&world).count(), 0);

    world.apply_deferred();

    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get_resource::<Score>(), Some(&Score(9)));
    assert_eq!(world.query::<&Health>().iter(&world).count(), 2);
}

#[test]
fn entity_commands_spawn_empty_and_entity_or_spawn_apply_in_queue_order() {
    let mut world = World::empty();
    let reserved = {
        let mut commands = world.commands();
        let reserved = commands.spawn_empty().id();
        commands
            .entity(reserved)
            .insert((Name("Reserved".to_string()), Health(1)));
        commands
            .entity_or_spawn(42)
            .insert((Name("Explicit".to_string()), Health(2)));
        reserved
    };

    assert!(!world.contains_entity(reserved));
    assert!(!world.contains_entity(42));

    world.apply_deferred();

    assert_eq!(world.get::<Name>(reserved).unwrap().0, "Reserved");
    assert_eq!(world.get::<Health>(reserved), Some(&Health(1)));
    assert_eq!(world.get::<Name>(42).unwrap().0, "Explicit");
    assert_eq!(world.get::<Health>(42), Some(&Health(2)));
}
