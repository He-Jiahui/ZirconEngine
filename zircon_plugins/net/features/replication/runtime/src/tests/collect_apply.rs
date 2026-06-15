use zircon_runtime::core::framework::net::{
    NetObjectId, NetSessionId, SyncAuthority, SyncComponentDescriptor, SyncFieldDescriptor,
    SyncFieldValue,
};

use crate::net_replication_runtime_manager;

fn field_bytes<'a>(fields: &'a [SyncFieldValue], name: &str) -> Option<&'a [u8]> {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.bytes.as_slice())
}

#[test]
fn dual_world_replicates_spawn_update_despawn() {
    let source = net_replication_runtime_manager();
    let replica = net_replication_runtime_manager();
    let descriptor = SyncComponentDescriptor::new("Transform", SyncAuthority::Server)
        .with_field(SyncFieldDescriptor::new("x", "f32"))
        .with_field(SyncFieldDescriptor::new("y", "f32"));
    source.register_component(descriptor.clone());
    replica.register_component(descriptor);

    let object = NetObjectId::new(91);
    let spawn = source
        .collect_snapshot_delta(
            object,
            "Transform",
            [SyncFieldValue::new("x", [1]), SyncFieldValue::new("y", [2])],
        )
        .unwrap();
    assert_eq!(spawn.sequence, 1);
    assert_eq!(spawn.changed_fields.len(), 2);
    assert!(!spawn.is_despawn());

    let spawned = replica.apply_delta(spawn.clone()).unwrap();
    assert_eq!(spawned.object, object);
    assert_eq!(field_bytes(&spawned.fields, "x"), Some(&[1][..]));
    assert_eq!(field_bytes(&spawned.fields, "y"), Some(&[2][..]));

    let update = source
        .collect_snapshot_delta(
            object,
            "Transform",
            [SyncFieldValue::new("x", [5]), SyncFieldValue::new("y", [2])],
        )
        .unwrap();
    assert_eq!(update.sequence, 2);
    assert_eq!(update.changed_fields.len(), 1);
    assert_eq!(update.changed_fields[0].name, "x");

    let updated = replica.apply_delta(update.clone()).unwrap();
    assert_eq!(field_bytes(&updated.fields, "x"), Some(&[5][..]));
    assert_eq!(field_bytes(&updated.fields, "y"), Some(&[2][..]));

    let stale = replica.apply_delta(spawn).unwrap();
    assert_eq!(field_bytes(&stale.fields, "x"), Some(&[5][..]));

    let despawn = source.collect_despawn_deltas(object);
    assert_eq!(despawn.len(), 1);
    assert!(despawn[0].is_despawn());
    assert!(despawn[0].sequence > update.sequence);
    assert!(despawn[0].changed_fields.is_empty());
    assert!(source.late_join_snapshots(NetSessionId::new(92)).is_empty());

    assert!(replica.apply_delta(despawn[0].clone()).is_none());
    assert!(replica
        .late_join_snapshots(NetSessionId::new(93))
        .is_empty());
}
