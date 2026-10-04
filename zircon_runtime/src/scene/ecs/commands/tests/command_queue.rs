use super::{CommandQueue, World};

#[test]
fn ecs_commands_apply_records_one_world_owned_boundary() {
    let mut queue = CommandQueue::default();
    queue.push(|_: &mut World| {});

    queue.apply(&mut World::empty());

    assert_eq!(queue.metrics().world_apply_count(), 1);
}
