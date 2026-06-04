use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue, SyncReplicationBudget,
};

use crate::net_replication_runtime_manager;

#[test]
fn replication_schedule_respects_update_frequency_and_priority_order() {
    let replication = net_replication_runtime_manager();
    replication.register_component(
        SyncComponentDescriptor::new("SlowState", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("value", "u8"))
            .with_update_hz(2)
            .with_replication_priority(20),
    );
    replication.register_component(
        SyncComponentDescriptor::new("FastState", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("value", "u8"))
            .with_update_hz(10)
            .with_replication_priority(5),
    );

    let slow = NetObjectId::new(21);
    let fast = NetObjectId::new(22);
    replication.publish_snapshot(slow, "SlowState", [SyncFieldValue::new("value", [1])]);
    replication.publish_snapshot(fast, "FastState", [SyncFieldValue::new("value", [2])]);
    let session = NetSessionId::new(60);

    let first = replication.scheduled_snapshots(
        session,
        0,
        SyncReplicationBudget::new().with_max_snapshots(4),
    );
    assert_eq!(first.sent_snapshots.len(), 2);
    assert_eq!(first.sent_snapshots[0].object, slow);
    assert_eq!(first.sent_snapshots[1].object, fast);
    assert_eq!(first.deferred_snapshots, 0);
    assert_eq!(first.skipped_not_due, 0);

    let second = replication.scheduled_snapshots(
        session,
        100,
        SyncReplicationBudget::new().with_max_snapshots(4),
    );
    assert_eq!(second.sent_snapshots.len(), 1);
    assert_eq!(second.sent_snapshots[0].object, fast);
    assert_eq!(second.skipped_not_due, 1);

    let third = replication.scheduled_snapshots(
        session,
        500,
        SyncReplicationBudget::new().with_max_snapshots(4),
    );
    assert_eq!(third.sent_snapshots.len(), 2);
    assert_eq!(third.sent_snapshots[0].object, slow);
    assert_eq!(third.sent_snapshots[1].object, fast);
}
