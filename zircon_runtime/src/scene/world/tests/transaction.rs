use crate::core::framework::scene::{ComponentPropertyPath, EntityPath};
use crate::scene::NodeKind;

use super::World;

#[path = "transaction/retained_schedule.rs"]
mod retained_schedule;

#[test]
fn staged_world_commit_stales_compiled_binding_when_entity_ids_are_reused() {
    let mut current = World::empty();
    let root = current
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let hero = current
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    current.rename_node(root, "Root").unwrap();
    current.rename_node(hero, "Hero").unwrap();
    current.set_parent_checked(hero, Some(root)).unwrap();
    let writer = current
        .compile_scene_property_writer(
            &EntityPath::parse("Root/Hero").unwrap(),
            &ComponentPropertyPath::parse("Transform.translation").unwrap(),
        )
        .unwrap()
        .unwrap();

    let mut staged = World::empty();
    let staged_root = staged
        .spawn_node(NodeKind::Empty)
        .expect("test scene spawn should succeed");
    let staged_hero = staged
        .spawn_node(NodeKind::Mesh)
        .expect("test scene spawn should succeed");
    assert_eq!(root, staged_root);
    assert_eq!(hero, staged_hero);
    staged.rename_node(staged_root, "Root").unwrap();
    staged.rename_node(staged_hero, "Hero").unwrap();
    staged
        .set_parent_checked(staged_hero, Some(staged_root))
        .unwrap();

    let _retired = current.commit_staged_scene_state(staged).unwrap();

    assert!(!writer.is_current_for(&current));
}

#[test]
fn compact_dynamic_scene_publication_preserves_prior_outer_staging_and_event_prefix() {
    use crate::scene::components::Name;
    use crate::scene::ecs::LifecycleEventKind;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let mut source = World::empty();
    source.spawn_node(NodeKind::Empty).unwrap();
    source.spawn_node(NodeKind::Empty).unwrap();
    let scene = crate::scene::DynamicScene::from_world(&source).unwrap();
    let mut target = World::empty();
    let prefix_entity = target.spawn_node(NodeKind::Empty).unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    target.observe_component_lifecycle::<Name>(
        LifecycleEventKind::Add,
        // SAFETY: Only an external atomic counter changes; retained World payload and dynamic
        // JSON are untouched.
        unsafe {
            use crate::core::framework::retained_memory::RetainedLifecycleObserverV1;
            RetainedLifecycleObserverV1::preserves_history_snapshot_v1(move |_, _| {
                observed.fetch_add(1, Ordering::SeqCst);
            })
        },
    );
    target.record_staged_lifecycle_events = true;
    let name = target.registered_component_id::<Name>().unwrap();
    target.trigger_component_lifecycle(LifecycleEventKind::Add, prefix_entity, name);
    let prefix = target.staged_lifecycle_events.clone();
    assert_eq!(prefix.len(), 1);

    let remap = scene.spawn_into(&mut target).unwrap();
    assert_eq!(remap.len(), 2);
    assert!(target.record_staged_lifecycle_events);
    assert_eq!(
        &target.staged_lifecycle_events[..prefix.len()],
        prefix.as_slice()
    );
    assert_eq!(target.staged_lifecycle_events.len(), prefix.len() + 24);
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "compact publication must not dispatch the enclosing owner's events"
    );

    target.record_staged_lifecycle_events = false;
    let events = std::mem::take(&mut target.staged_lifecycle_events);
    let execution = target.begin_schedule_execution();
    for event in events {
        assert!(execution.is_bound_to(&target));
        target.dispatch_component_lifecycle(event);
    }
    assert!(execution.is_bound_to(&target));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        3,
        "outer owner publishes prefix and both new Name callbacks exactly once"
    );
}
