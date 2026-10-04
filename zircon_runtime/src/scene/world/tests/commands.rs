use crate::scene::World;

#[test]
fn ecs_commands_deferred_metrics_observe_only_entered_apply_boundaries() {
    let mut world = World::empty();

    assert_eq!(world.apply_deferred().applied_count(), 0);
    assert_eq!(world.deferred_command_metrics().world_apply_count(), 0);

    world.commands().queue_fn(|_: &mut World| {});
    assert_eq!(world.apply_deferred().applied_count(), 1);
    assert_eq!(world.deferred_command_metrics().world_apply_count(), 1);
}
