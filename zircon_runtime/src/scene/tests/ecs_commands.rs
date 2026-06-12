use crate::scene::components::Name;
use crate::scene::ecs::{Component, DeferredCommandOperation};
use crate::scene::World;

#[derive(Debug, PartialEq, Eq)]
struct Health(u32);

impl Component for Health {}

#[derive(Debug, PartialEq, Eq)]
struct Marker;

impl Component for Marker {}

#[test]
fn deferred_command_success_report_counts_applied_commands_without_errors() {
    let mut world = World::empty();
    let entity = world.spawn((Name("Target".to_string()),)).unwrap();

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(7));
        commands.entity(entity).insert((Marker,));
    }

    let report = world.apply_deferred();

    assert_eq!(report.applied_count(), 2);
    assert_eq!(report.error_count(), 0);
    assert!(report.is_success());
    assert_eq!(world.get::<Health>(entity), Some(&Health(7)));
    assert_eq!(world.get::<Marker>(entity), Some(&Marker));
}

#[test]
fn command_queue_on_despawned_entity_target_is_reported_not_silently_dropped() {
    let mut world = World::empty();
    let entity = world
        .spawn((Name("Removed".to_string()), Health(1)))
        .unwrap();
    assert!(world.remove_entity(entity));

    {
        let mut commands = world.commands();
        commands.insert(entity, Health(2));
        commands.remove::<Health>(entity);
        commands.despawn(entity);
    }

    let report = world.apply_deferred();
    let errors = report.errors();

    assert_eq!(report.applied_count(), 3);
    assert_eq!(report.error_count(), 3);
    assert!(!report.is_success());
    assert_eq!(errors[0].operation(), DeferredCommandOperation::Insert);
    assert_eq!(errors[0].entity(), entity);
    assert!(errors[0].message().contains("missing entity"));
    assert_eq!(errors[1].operation(), DeferredCommandOperation::Remove);
    assert_eq!(errors[1].entity(), entity);
    assert!(errors[1].message().contains("missing entity"));
    assert_eq!(errors[2].operation(), DeferredCommandOperation::Despawn);
    assert_eq!(errors[2].entity(), entity);
    assert!(errors[2].message().contains("missing entity"));
    assert!(!world.has_deferred_commands());
    assert!(world.get::<Health>(entity).is_none());
}
