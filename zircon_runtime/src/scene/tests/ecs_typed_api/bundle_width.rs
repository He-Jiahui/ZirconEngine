use crate::scene::ecs::Component;
use crate::scene::World;

use super::{Health, Mana};

// A `World::spawn` creates a Mesh node with these seven fixed components:
// Name, Hierarchy, LocalTransform, ActiveSelf, RenderLayerMask, MeshRenderer, and Mobility.
const DEFAULT_MESH_NODE_COMPONENTS: u64 = 7;
const LIFECYCLE_EVENTS_PER_COMPONENT_PUBLICATION: u64 = 2;

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotOne;
impl Component for BundleSlotOne {}

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotTwo;
impl Component for BundleSlotTwo {}

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotThree;
impl Component for BundleSlotThree {}

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotFour;
impl Component for BundleSlotFour {}

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotFive;
impl Component for BundleSlotFive {}

#[derive(Debug, PartialEq, Eq)]
struct BundleSlotSix;
impl Component for BundleSlotSix {}

#[test]
fn bundle_widths_zero_through_eight_publish_only_final_archetypes() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();

    world.spawn(()).expect("zero-width bundle must publish");
    world
        .spawn((Health(1),))
        .expect("one-component bundle must publish");
    world
        .spawn((Health(2), Mana(2)))
        .expect("two-component bundle must publish");
    world
        .spawn((Health(3), Mana(3), BundleSlotOne))
        .expect("three-component bundle must publish");
    world
        .spawn((Health(4), Mana(4), BundleSlotOne, BundleSlotTwo))
        .expect("four-component bundle must publish");
    world
        .spawn((
            Health(5),
            Mana(5),
            BundleSlotOne,
            BundleSlotTwo,
            BundleSlotThree,
        ))
        .expect("five-component bundle must publish");
    world
        .spawn((
            Health(6),
            Mana(6),
            BundleSlotOne,
            BundleSlotTwo,
            BundleSlotThree,
            BundleSlotFour,
        ))
        .expect("six-component bundle must publish");
    world
        .spawn((
            Health(7),
            Mana(7),
            BundleSlotOne,
            BundleSlotTwo,
            BundleSlotThree,
            BundleSlotFour,
            BundleSlotFive,
        ))
        .expect("seven-component bundle must publish");
    world
        .spawn((
            Health(8),
            Mana(8),
            BundleSlotOne,
            BundleSlotTwo,
            BundleSlotThree,
            BundleSlotFour,
            BundleSlotFive,
            BundleSlotSix,
        ))
        .expect("eight-component bundle must publish");

    let expected_publications = (0..=8)
        .map(|tuple_width| expected_non_overriding_spawn_publications(1, tuple_width))
        .sum();
    assert_non_overriding_bundle_metrics(&world, 9, expected_publications);
}

#[test]
fn eight_component_bundle_publishes_one_final_signature() {
    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();

    let entity = world
        .spawn((
            Health(1),
            Mana(2),
            BundleSlotOne,
            BundleSlotTwo,
            BundleSlotThree,
            BundleSlotFour,
            BundleSlotFive,
            BundleSlotSix,
        ))
        .expect("the maximum supported tuple bundle must publish");

    assert!(world.contains_component::<Health>(entity));
    assert!(world.contains_component::<Mana>(entity));
    assert!(world.contains_component::<BundleSlotOne>(entity));
    assert!(world.contains_component::<BundleSlotTwo>(entity));
    assert!(world.contains_component::<BundleSlotThree>(entity));
    assert!(world.contains_component::<BundleSlotFour>(entity));
    assert!(world.contains_component::<BundleSlotFive>(entity));
    assert!(world.contains_component::<BundleSlotSix>(entity));

    assert_non_overriding_bundle_metrics(
        &world,
        1,
        expected_non_overriding_spawn_publications(1, 8),
    );
}

fn expected_non_overriding_spawn_publications(spawn_count: u64, tuple_width: u64) -> u64 {
    spawn_count * (DEFAULT_MESH_NODE_COMPONENTS + tuple_width)
}

fn assert_non_overriding_bundle_metrics(
    world: &World,
    expected_transactions: u64,
    expected_publications: u64,
) {
    let bundle = world
        .ecs_frame_performance_diagnostics()
        .bundle_transactions;
    assert_eq!(bundle.committed_transactions, expected_transactions);
    assert_eq!(
        bundle.final_archetype_transitions, expected_transactions,
        "each spawned entity may publish only its final archetype location"
    );
    assert_eq!(
        bundle.intermediate_signatures, 0,
        "bundle publication must not expose per-component signatures"
    );
    assert_eq!(
        bundle.component_storage_moves, expected_publications,
        "each staged component must publish through canonical storage exactly once"
    );
    assert_eq!(
        bundle.lifecycle_events,
        expected_publications * LIFECYCLE_EVENTS_PER_COMPONENT_PUBLICATION,
        "each canonical publication must emit one Add and one Insert event after commit"
    );
    assert_eq!(
        bundle.staged_value_allocations, expected_publications,
        "without default overrides, every staged owned value must become one canonical publication"
    );
}

#[test]
fn one_thousand_bundle_spawns_publish_only_final_archetypes() {
    const SPAWN_COUNT: usize = 1_000;

    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();
    for index in 0..SPAWN_COUNT {
        world
            .spawn((Health(index as u32),))
            .expect("preflighted one-component bundle spawn must publish");
    }

    let spawn_count = SPAWN_COUNT as u64;
    assert_non_overriding_bundle_metrics(
        &world,
        spawn_count,
        expected_non_overriding_spawn_publications(spawn_count, 1),
    );
}

#[test]
#[ignore = "explicit 100k Bundle transaction performance acceptance"]
fn one_hundred_thousand_bundle_spawns_publish_only_final_archetypes() {
    const SPAWN_COUNT: usize = 100_000;

    let mut world = World::empty();
    world.reset_ecs_frame_performance_diagnostics();
    for index in 0..SPAWN_COUNT {
        world
            .spawn((Health(index as u32),))
            .expect("preflighted one-component bundle spawn must publish");
    }

    let spawn_count = SPAWN_COUNT as u64;
    assert_non_overriding_bundle_metrics(
        &world,
        spawn_count,
        expected_non_overriding_spawn_publications(spawn_count, 1),
    );
}
