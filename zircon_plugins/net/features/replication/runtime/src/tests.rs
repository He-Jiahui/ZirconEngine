use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue, SyncInterestDescriptor, SyncReplicationBudget,
};

use super::{
    net_replication_runtime_manager, plugin_feature_registration,
    NET_REPLICATION_FEATURE_CAPABILITY, NET_REPLICATION_FEATURE_ID,
    NET_REPLICATION_FEATURE_MANAGER_NAME, NET_REPLICATION_FEATURE_MODULE_NAME,
};

#[test]
fn replication_feature_registration_contributes_runtime_module_and_manager() {
    let report = plugin_feature_registration();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.manifest.id, NET_REPLICATION_FEATURE_ID);
    assert!(report
        .manifest
        .capabilities
        .iter()
        .any(|capability| capability == NET_REPLICATION_FEATURE_CAPABILITY));
    let module = report
        .extensions
        .modules()
        .iter()
        .find(|module| module.name == NET_REPLICATION_FEATURE_MODULE_NAME)
        .expect("replication feature module should be registered");
    assert_eq!(
        module.managers[0].name.to_string(),
        NET_REPLICATION_FEATURE_MANAGER_NAME
    );
}

#[test]
fn replication_manager_emits_dirty_field_delta_and_filters_interest_groups() {
    let replication = net_replication_runtime_manager();
    replication.register_component(
        SyncComponentDescriptor::new("Transform", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("x", "f32"))
            .with_field(SyncFieldDescriptor::new("y", "f32"))
            .with_interest_group("nearby"),
    );

    let object = NetObjectId::new(7);
    let first = replication
        .publish_snapshot(
            object,
            "Transform",
            [SyncFieldValue::new("x", [1]), SyncFieldValue::new("y", [2])],
        )
        .unwrap();
    let second = replication
        .publish_snapshot(
            object,
            "Transform",
            [SyncFieldValue::new("x", [1]), SyncFieldValue::new("y", [3])],
        )
        .unwrap();

    assert_eq!(first.changed_fields.len(), 2);
    assert_eq!(second.changed_fields.len(), 1);
    assert_eq!(second.changed_fields[0].name, "y");

    let session = NetSessionId::new(9);
    replication.set_interest(SyncInterestDescriptor::new(session).with_group("nearby"));
    assert_eq!(replication.visible_snapshots(session).len(), 1);
    let far_session = NetSessionId::new(10);
    replication.set_interest(SyncInterestDescriptor::new(far_session).with_group("far"));
    assert!(replication.visible_snapshots(far_session).is_empty());
}

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
