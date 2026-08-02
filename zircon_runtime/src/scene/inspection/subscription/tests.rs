use std::collections::BTreeMap;

use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::world_sync::{
    AssetReloadFrameApplyReportDto, WatchKey, WatchRegistration, WorldFact,
};

use crate::scene::World;
use crate::scene::components::NodeKind;

use super::{SubscriptionTable, SubscriptionTableLimits, ancestor_chain_contains};

#[test]
fn watch_allocates_distinct_tokens_and_unwatch_revokes_pending_dirty() {
    let mut table = SubscriptionTable::default();
    let first = table.watch(WatchRegistration::new(WatchKey::ComponentType {
        type_name: "tests.Health".to_string(),
    }));
    let second = table.watch(WatchRegistration::new(WatchKey::ComponentType {
        type_name: "tests.Health".to_string(),
    }));

    assert_ne!(first, second);
    table.invalidate_component_type("tests.Health");
    assert!(table.unwatch(first));
    assert!(!table.unwatch(first));
    assert_eq!(table.len(), 1);
    assert_eq!(table.flush(4).unwrap().dirty, vec![second]);
}

#[test]
fn typed_indexes_route_without_scanning_unrelated_watch_variants() {
    let scene = ResourceId::from_stable_label("tests.scene");
    let other_scene = ResourceId::from_stable_label("tests.other-scene");
    let mut table = SubscriptionTable::default();
    table.watch(WatchRegistration::new(WatchKey::WorldStructure));
    table.watch(WatchRegistration::new(WatchKey::Subtree { root: 91 }));
    let component = table.watch(WatchRegistration::new(WatchKey::ComponentType {
        type_name: "tests.Health".to_string(),
    }));
    let asset = table.watch(WatchRegistration::new(WatchKey::Asset {
        resource_id: scene,
    }));
    table.watch(WatchRegistration::new(WatchKey::Asset {
        resource_id: other_scene,
    }));

    table.invalidate_component_type("tests.Health");
    table.invalidate_asset(scene);

    assert_eq!(table.flush(9).unwrap().dirty, vec![component, asset]);
    let diagnostics = table.diagnostics();
    assert_eq!(diagnostics.direct_key_probes(), 2);
    assert_eq!(diagnostics.matched_tokens(), 2);
}

#[test]
fn subtree_invalidation_walks_ancestry_once_for_many_watches() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(root)).unwrap();

    let mut table = SubscriptionTable::default();
    let root_token = table.watch(WatchRegistration::new(WatchKey::Subtree { root }));
    for unrelated in 10_000..11_000 {
        table.watch(WatchRegistration::new(WatchKey::Subtree {
            root: unrelated,
        }));
    }

    table.invalidate_subtree(&world, child);

    assert_eq!(
        table.flush(world.world_generation()).unwrap().dirty,
        vec![root_token]
    );
    let diagnostics = table.diagnostics();
    assert_eq!(diagnostics.ancestor_walks(), 1);
    assert_eq!(diagnostics.ancestor_visited_allocations(), 1);
    assert_eq!(diagnostics.ancestor_nodes(), 2);
    assert_eq!(diagnostics.direct_key_probes(), 2);
}

#[test]
fn fact_queue_coalesces_by_semantic_identity_and_stays_bounded() {
    let world = World::empty();
    let scene = ResourceId::from_stable_label("tests.scene");
    let limits = SubscriptionTableLimits::new(2, usize::MAX, 4);
    let mut table = SubscriptionTable::with_limits(limits);
    table.record_fact(&world, WorldFact::SceneLoaded { scene });
    table.record_fact(&world, WorldFact::SceneUnloaded { scene });
    for _ in 0..100 {
        table.record_fact(
            &world,
            WorldFact::AssetReloadApplied(AssetReloadFrameApplyReportDto {
                applied: 1,
                ..AssetReloadFrameApplyReportDto::default()
            }),
        );
    }

    assert_eq!(table.pending_fact_count(), 2);
    assert!(table.pending_estimated_bytes() <= limits.max_pending_estimated_bytes());
    let batch = table.flush(world.world_generation()).unwrap();
    assert_eq!(batch.facts.len(), 2);
    assert_eq!(batch.facts[0], WorldFact::SceneUnloaded { scene });
    assert_eq!(table.diagnostics().coalesced_facts(), 100);
    assert_eq!(table.diagnostics().pending_peak_count(), 2);
}

#[test]
fn overflow_marks_world_dirty_and_records_diagnostics() {
    let mut world = World::empty();
    let first = world.spawn_node(NodeKind::Empty);
    let second = world.spawn_node(NodeKind::Empty);
    let mut table = SubscriptionTable::with_limits(SubscriptionTableLimits::new(1, usize::MAX, 1));
    let world_token = table.watch(WatchRegistration::new(WatchKey::WorldStructure));

    table.record_fact(&world, WorldFact::Spawned(first));
    table.record_fact(&world, WorldFact::Spawned(second));
    let batch = table
        .flush(world.world_generation().saturating_add(2))
        .unwrap();

    assert_eq!(batch.facts, vec![WorldFact::Spawned(first)]);
    assert_eq!(batch.dirty, vec![world_token]);
    let diagnostics = table.diagnostics();
    assert_eq!(diagnostics.overflowed_facts(), 1);
    assert_eq!(diagnostics.age_budget_exceeded(), 1);
    assert!(diagnostics.oldest_pending_age_generations() > 1);
    assert!(diagnostics.overflowed());
}

#[test]
fn subtree_walk_stops_at_malformed_parent_cycles() {
    let parents = BTreeMap::from([(3, 4), (4, 3)]);
    assert!(!ancestor_chain_contains(3, 9, |entity| {
        parents.get(&entity).copied()
    }));
    assert!(ancestor_chain_contains(3, 4, |entity| {
        parents.get(&entity).copied()
    }));
}
