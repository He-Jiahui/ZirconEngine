use zircon_runtime::core::framework::net::{
    NetObjectId, NetworkIdentity, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncReplicationStrategy,
};

use crate::net_replication_runtime_manager;

#[test]
fn replication_table_compiles_from_descriptors() {
    let replication = net_replication_runtime_manager();
    replication.register_component(
        SyncComponentDescriptor::new("Transform", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("translation", "Vec3"))
            .with_field(SyncFieldDescriptor::new("rotation", "Quat"))
            .with_replication_strategy(SyncReplicationStrategy::Interval)
            .with_update_hz(30)
            .with_replication_priority(10)
            .with_interest_group("nearby"),
    );
    replication.register_component(
        SyncComponentDescriptor::new("Inventory", SyncAuthority::ClientOwned)
            .with_field(SyncFieldDescriptor::new("items", "Vec<ItemStack>"))
            .with_replication_strategy(SyncReplicationStrategy::Once)
            .with_replication_priority(2),
    );

    let identity = NetworkIdentity::new(NetObjectId::new(42), SyncAuthority::Server);
    assert_eq!(identity.object, NetObjectId::new(42));
    assert_eq!(identity.authority, SyncAuthority::Server);

    let table = replication.compile_replication_table();
    assert_eq!(table.len(), 2);
    assert!(!table.is_empty());

    let inventory = table.entry_for_component("Inventory").unwrap();
    assert_eq!(inventory.dense_index, 0);
    assert_eq!(inventory.authority, SyncAuthority::ClientOwned);
    assert_eq!(
        inventory.replication_strategy,
        SyncReplicationStrategy::Once
    );
    assert_eq!(inventory.fields.len(), 1);

    let transform = table.entry_for_component("Transform").unwrap();
    assert_eq!(transform.dense_index, 1);
    assert_eq!(transform.authority, SyncAuthority::Server);
    assert_eq!(
        transform.replication_strategy,
        SyncReplicationStrategy::Interval
    );
    assert_eq!(transform.update_hz, 30);
    assert_eq!(transform.replication_priority, 10);
    assert_eq!(transform.interest_group.as_deref(), Some("nearby"));
    assert_eq!(
        table
            .entries()
            .iter()
            .map(|entry| entry.component_type.as_str())
            .collect::<Vec<_>>(),
        vec!["Inventory", "Transform"]
    );
}
