use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::inspection::{SubscriptionTable, SubscriptionTableLimits};
use zircon_runtime::scene::World;
use zircon_runtime_interface::resource::ResourceId;
use zircon_runtime_interface::world_sync::{
    AssetReloadFrameApplyReportDto, WatchKey, WatchRegistration, WorldFact,
};

#[test]
fn public_subscription_table_lifecycle_and_flush_contract() {
    let mut table = SubscriptionTable::default();
    let first = table.watch(WatchRegistration::new(WatchKey::ComponentType {
        type_name: "tests.Health".to_string(),
    }));
    let second = table.watch(WatchRegistration::new(WatchKey::ComponentType {
        type_name: "tests.Health".to_string(),
    }));

    table.invalidate_component_type("tests.Health");
    assert!(table.unwatch(first));
    assert_eq!(table.flush(7).unwrap().dirty, vec![second]);
    assert!(table.flush(8).is_none());
}

#[test]
fn public_subscription_table_matches_subtree_and_asset_throats() {
    let mut world = World::empty();
    let root = world.spawn_node(NodeKind::Empty);
    let child = world.spawn_node(NodeKind::Empty);
    world.set_parent_checked(child, Some(root)).unwrap();

    let scene = ResourceId::from_stable_label("tests.scene");
    let mut table = SubscriptionTable::default();
    let subtree = table.watch(WatchRegistration::new(WatchKey::Subtree { root }));
    let asset = table.watch(WatchRegistration::new(WatchKey::Asset {
        resource_id: scene,
    }));

    table.record_fact(
        &world,
        WorldFact::Reparented {
            entity: child,
            new_parent: Some(root),
        },
    );
    table.invalidate_asset(scene);

    let batch = table.flush(world.world_generation()).unwrap();
    assert_eq!(batch.dirty, vec![subtree, asset]);
    assert_eq!(batch.facts.len(), 1);
}

#[test]
#[ignore = "100k scale evidence"]
fn public_subscription_table_100k_watch_routing_is_direct_and_bounded() {
    let mut table = SubscriptionTable::with_limits(SubscriptionTableLimits::new(4, usize::MAX, 4));
    let mut target = None;
    for index in 0..100_000 {
        let token = table.watch(WatchRegistration::new(WatchKey::ComponentType {
            type_name: format!("tests.Component{index}"),
        }));
        if index == 50_000 {
            target = Some(token);
        }
    }

    table.invalidate_component_type("tests.Component50000");
    assert_eq!(table.flush(1).unwrap().dirty, vec![target.unwrap()]);
    assert_eq!(table.diagnostics().direct_key_probes(), 1);
    assert_eq!(table.diagnostics().matched_tokens(), 1);

    let world = World::empty();
    for _ in 0..100_000 {
        table.record_fact(
            &world,
            WorldFact::AssetReloadApplied(AssetReloadFrameApplyReportDto {
                applied: 1,
                ..AssetReloadFrameApplyReportDto::default()
            }),
        );
    }
    let batch = table.flush(world.world_generation()).unwrap();
    assert_eq!(batch.facts.len(), 1);
    assert_eq!(table.diagnostics().coalesced_facts(), 99_999);
    assert_eq!(table.diagnostics().pending_peak_count(), 1);
    assert!(!table.diagnostics().overflowed());
}
