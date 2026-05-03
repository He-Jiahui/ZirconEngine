use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue, SyncInterestDescriptor,
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
