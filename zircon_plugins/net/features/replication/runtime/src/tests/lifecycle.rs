use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue,
};

use crate::net_replication_runtime_manager;

#[test]
fn replication_manager_supports_late_join_snapshot_and_despawn_lifecycle() {
    let replication = net_replication_runtime_manager();
    replication.register_component(
        SyncComponentDescriptor::new("Health", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("hp", "u16")),
    );
    let object = NetObjectId::new(11);
    replication.publish_snapshot(
        object,
        "Health",
        [SyncFieldValue::new("hp", 100_u16.to_le_bytes())],
    );

    let late_join = replication.late_join_snapshots(NetSessionId::new(50));
    assert_eq!(late_join.len(), 1);
    assert_eq!(late_join[0].object, object);

    let despawned = replication.despawn_object(object);
    assert_eq!(despawned.len(), 1);
    assert!(replication
        .late_join_snapshots(NetSessionId::new(51))
        .is_empty());
}
