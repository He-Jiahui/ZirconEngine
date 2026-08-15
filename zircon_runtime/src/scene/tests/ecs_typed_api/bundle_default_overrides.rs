use std::sync::{Arc, Mutex};

use crate::core::math::Transform;
use crate::scene::components::{
    ActiveSelf, Hierarchy, LocalTransform, MeshRenderer, Mobility, Name, RenderLayerMask,
};
use crate::scene::ecs::LifecycleEventKind;
use crate::scene::World;

use super::{Health, Mana};

#[test]
fn empty_bundle_spawn_publishes_each_node_record_default_once() {
    let mut world = World::empty();
    let lifecycle_counts = Arc::new(Mutex::new([0usize; 21]));

    macro_rules! observe_default_component {
        ($component:ty, $add_index:expr, $replace_index:expr, $insert_index:expr) => {{
            let adds = Arc::clone(&lifecycle_counts);
            let replacements = Arc::clone(&lifecycle_counts);
            let inserts = Arc::clone(&lifecycle_counts);
            world.observe_component_lifecycle::<$component>(
                LifecycleEventKind::Add,
                move |_world, _event| {
                    adds.lock().expect("lifecycle counters")[$add_index] += 1;
                },
            );
            world.observe_component_lifecycle::<$component>(
                LifecycleEventKind::Replace,
                move |_world, _event| {
                    replacements.lock().expect("lifecycle counters")[$replace_index] += 1;
                },
            );
            world.observe_component_lifecycle::<$component>(
                LifecycleEventKind::Insert,
                move |_world, _event| {
                    inserts.lock().expect("lifecycle counters")[$insert_index] += 1;
                },
            );
        }};
    }

    observe_default_component!(Name, 0, 1, 2);
    observe_default_component!(Hierarchy, 3, 4, 5);
    observe_default_component!(LocalTransform, 6, 7, 8);
    observe_default_component!(ActiveSelf, 9, 10, 11);
    observe_default_component!(RenderLayerMask, 12, 13, 14);
    observe_default_component!(Mobility, 15, 16, 17);
    observe_default_component!(MeshRenderer, 18, 19, 20);

    let entity = world
        .spawn(())
        .expect("empty bundle spawn must commit its staged defaults exactly once");

    for component_id in [
        world.component_id::<Name>(),
        world.component_id::<Hierarchy>(),
        world.component_id::<LocalTransform>(),
        world.component_id::<ActiveSelf>(),
        world.component_id::<RenderLayerMask>(),
        world.component_id::<Mobility>(),
        world.component_id::<MeshRenderer>(),
    ] {
        assert_eq!(world.component_count_for_id(component_id), 1);
    }
    assert!(world.contains_component::<Name>(entity));
    assert!(world.contains_component::<Hierarchy>(entity));
    assert!(world.contains_component::<LocalTransform>(entity));
    assert!(world.contains_component::<ActiveSelf>(entity));
    assert!(world.contains_component::<RenderLayerMask>(entity));
    assert!(world.contains_component::<Mobility>(entity));
    assert!(world.contains_component::<MeshRenderer>(entity));
    assert_eq!(
        *lifecycle_counts.lock().expect("lifecycle counters"),
        [1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1,]
    );
}

#[test]
fn explicit_bundle_values_replace_node_record_defaults_without_intermediate_lifecycle() {
    let mut world = World::empty();
    let parent = world
        .spawn(())
        .expect("parent default node must publish before its child");
    let mut explicit_transform = Transform::default();
    explicit_transform.translation.x = 42.0;
    let explicit_name = "Explicit bundle child".to_string();
    let component_ids = [
        world.component_id::<Name>(),
        world.component_id::<Hierarchy>(),
        world.component_id::<LocalTransform>(),
    ];
    let storage_rows_before =
        component_ids.map(|component_id| world.component_count_for_id(component_id));
    let lifecycle_counts = Arc::new(Mutex::new([0usize; 6]));
    let name_adds_from_observer = Arc::clone(&lifecycle_counts);
    let name_replacements_from_observer = Arc::clone(&lifecycle_counts);
    let hierarchy_adds_from_observer = Arc::clone(&lifecycle_counts);
    let hierarchy_replacements_from_observer = Arc::clone(&lifecycle_counts);
    let transform_adds_from_observer = Arc::clone(&lifecycle_counts);
    let transform_replacements_from_observer = Arc::clone(&lifecycle_counts);
    let observed_name = explicit_name.clone();

    world.observe_component_lifecycle::<Name>(LifecycleEventKind::Add, move |world, event| {
        name_adds_from_observer.lock().expect("lifecycle counters")[0] += 1;
        assert_eq!(
            world
                .get::<Name>(event.entity())
                .map(|name| name.0.as_str()),
            Some(observed_name.as_str())
        );
        assert_eq!(
            world
                .get::<Hierarchy>(event.entity())
                .map(|hierarchy| hierarchy.parent),
            Some(Some(parent))
        );
        assert_eq!(
            world
                .get::<LocalTransform>(event.entity())
                .map(|transform| transform.transform),
            Some(explicit_transform)
        );
    });
    world.observe_component_lifecycle::<Name>(
        LifecycleEventKind::Replace,
        move |_world, _event| {
            name_replacements_from_observer
                .lock()
                .expect("lifecycle counters")[1] += 1;
        },
    );
    world.observe_component_lifecycle::<Hierarchy>(
        LifecycleEventKind::Add,
        move |_world, _event| {
            hierarchy_adds_from_observer
                .lock()
                .expect("lifecycle counters")[2] += 1;
        },
    );
    world.observe_component_lifecycle::<Hierarchy>(
        LifecycleEventKind::Replace,
        move |_world, _event| {
            hierarchy_replacements_from_observer
                .lock()
                .expect("lifecycle counters")[3] += 1;
        },
    );
    world.observe_component_lifecycle::<LocalTransform>(
        LifecycleEventKind::Add,
        move |_world, _event| {
            transform_adds_from_observer
                .lock()
                .expect("lifecycle counters")[4] += 1;
        },
    );
    world.observe_component_lifecycle::<LocalTransform>(
        LifecycleEventKind::Replace,
        move |_world, _event| {
            transform_replacements_from_observer
                .lock()
                .expect("lifecycle counters")[5] += 1;
        },
    );
    world.reset_ecs_frame_performance_diagnostics();

    let entity = world
        .spawn((
            Name(explicit_name),
            Hierarchy {
                parent: Some(parent),
            },
            LocalTransform {
                transform: explicit_transform,
            },
        ))
        .expect("explicit fixed values must replace their node-record defaults in one commit");

    assert_eq!(
        world.get::<Name>(entity).map(|name| name.0.as_str()),
        Some("Explicit bundle child")
    );
    assert_eq!(
        world
            .get::<Hierarchy>(entity)
            .map(|hierarchy| hierarchy.parent),
        Some(Some(parent))
    );
    assert_eq!(
        world
            .get::<LocalTransform>(entity)
            .map(|transform| transform.transform),
        Some(explicit_transform)
    );
    let record = world
        .node_record(entity)
        .expect("published child must retain its fixed component projection");
    assert_eq!(record.name, "Explicit bundle child");
    assert_eq!(record.parent, Some(parent));
    assert_eq!(record.transform, explicit_transform);
    for (component_id, rows_before) in component_ids.into_iter().zip(storage_rows_before) {
        assert_eq!(world.component_count_for_id(component_id), rows_before + 1);
    }
    assert_eq!(
        *lifecycle_counts.lock().expect("lifecycle counters"),
        [1, 0, 1, 0, 1, 0]
    );
    let bundle = world
        .ecs_frame_performance_diagnostics()
        .bundle_transactions;
    assert_eq!(bundle.committed_transactions, 1);
    assert_eq!(bundle.final_archetype_transitions, 1);
    assert_eq!(bundle.intermediate_signatures, 0);
    assert_eq!(
        bundle.component_storage_moves, 7,
        "three explicit fixed values replace their defaults before publication"
    );
    assert_eq!(bundle.lifecycle_events, 14);
    assert_eq!(
        bundle.staged_value_allocations, 10,
        "staging includes defaults and explicit inputs, unlike canonical publications"
    );
}

#[test]
fn tuple_bundle_lifecycle_events_follow_staged_order_after_final_publish() {
    let mut world = World::empty();
    let observed_events = Arc::new(Mutex::new(Vec::new()));
    let observed_events_from_health_add = Arc::clone(&observed_events);
    let observed_events_from_health_insert = Arc::clone(&observed_events);
    let observed_events_from_mana_add = Arc::clone(&observed_events);
    let observed_events_from_mana_insert = Arc::clone(&observed_events);

    world.observe_component_lifecycle::<Health>(LifecycleEventKind::Add, move |world, event| {
        assert_eq!(world.get::<Health>(event.entity()), Some(&Health(7)));
        assert_eq!(world.get::<Mana>(event.entity()), Some(&Mana(9)));
        observed_events_from_health_add
            .lock()
            .expect("lifecycle event order")
            .push("health:add");
    });
    world.observe_component_lifecycle::<Health>(
        LifecycleEventKind::Insert,
        move |_world, _event| {
            observed_events_from_health_insert
                .lock()
                .expect("lifecycle event order")
                .push("health:insert");
        },
    );
    world.observe_component_lifecycle::<Mana>(LifecycleEventKind::Add, move |_world, _event| {
        observed_events_from_mana_add
            .lock()
            .expect("lifecycle event order")
            .push("mana:add");
    });
    world.observe_component_lifecycle::<Mana>(LifecycleEventKind::Insert, move |_world, _event| {
        observed_events_from_mana_insert
            .lock()
            .expect("lifecycle event order")
            .push("mana:insert");
    });

    world
        .spawn((Health(7), Mana(9)))
        .expect("validated tuple bundle must publish atomically");

    assert_eq!(
        observed_events
            .lock()
            .expect("lifecycle event order")
            .as_slice(),
        ["health:add", "health:insert", "mana:add", "mana:insert"]
    );
}
