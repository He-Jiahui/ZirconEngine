use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue, SyncInterestDescriptor, SyncReplicationBudget,
};

use crate::net_replication_runtime_manager;

#[test]
fn replication_schedule_enforces_snapshot_and_byte_budgets_after_interest_culling() {
    let replication = net_replication_runtime_manager();
    replication.register_component(
        SyncComponentDescriptor::new("NearState", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("payload", "bytes"))
            .with_interest_group("near")
            .with_replication_priority(1),
    );
    replication.register_component(
        SyncComponentDescriptor::new("FarState", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("payload", "bytes"))
            .with_interest_group("far")
            .with_replication_priority(100),
    );
    let session = NetSessionId::new(70);
    replication.set_interest(SyncInterestDescriptor::new(session).with_group("near"));

    let first = NetObjectId::new(31);
    let second = NetObjectId::new(32);
    let hidden = NetObjectId::new(33);
    replication.publish_snapshot(first, "NearState", [SyncFieldValue::new("payload", [1, 2])]);
    replication.publish_snapshot(
        second,
        "NearState",
        [SyncFieldValue::new("payload", [3, 4])],
    );
    replication.publish_snapshot(hidden, "FarState", [SyncFieldValue::new("payload", [5])]);

    let snapshot_budget_report = replication.scheduled_snapshots(
        session,
        0,
        SyncReplicationBudget::new().with_max_snapshots(1),
    );

    assert_eq!(snapshot_budget_report.sent_snapshots.len(), 1);
    assert_eq!(snapshot_budget_report.sent_snapshots[0].object, first);
    assert_eq!(snapshot_budget_report.deferred_snapshots, 1);
    assert_eq!(snapshot_budget_report.skipped_by_interest, 1);

    let byte_budget_session = NetSessionId::new(71);
    replication.set_interest(SyncInterestDescriptor::new(byte_budget_session).with_group("near"));
    let byte_budget_report = replication.scheduled_snapshots(
        byte_budget_session,
        0,
        SyncReplicationBudget::new()
            .with_max_snapshots(4)
            .with_max_bytes(2),
    );

    assert_eq!(byte_budget_report.sent_snapshots.len(), 1);
    assert_eq!(byte_budget_report.sent_snapshots[0].object, first);
    assert_eq!(byte_budget_report.used_bytes, 2);
    assert_eq!(byte_budget_report.deferred_snapshots, 1);
    assert_eq!(byte_budget_report.skipped_by_interest, 1);

    let unconstrained_session = NetSessionId::new(72);
    replication.set_interest(SyncInterestDescriptor::new(unconstrained_session).with_group("near"));
    let unconstrained_report =
        replication.scheduled_snapshots(unconstrained_session, 0, SyncReplicationBudget::new());

    assert_eq!(unconstrained_report.sent_snapshots.len(), 2);
    assert_eq!(unconstrained_report.deferred_snapshots, 0);

    let repeat_tick_report = replication.scheduled_snapshots(
        unconstrained_session,
        0,
        SyncReplicationBudget::new()
            .with_max_snapshots(4)
            .with_max_bytes(2),
    );

    assert!(repeat_tick_report.sent_snapshots.is_empty());
    assert_eq!(repeat_tick_report.skipped_not_due, 2);
    assert_eq!(repeat_tick_report.skipped_by_interest, 1);
}
