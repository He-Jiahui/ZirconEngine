use zircon_runtime::core::framework::net::{
    NetObjectId, SyncAuthority, SyncComponentDescriptor, SyncDelta, SyncFieldDescriptor,
    SyncFieldValue,
};

use crate::net_replication_runtime_manager;

fn f32_field(name: &str, value: f32) -> SyncFieldValue {
    SyncFieldValue::new(name, value.to_le_bytes())
}

#[test]
fn interpolation_window_smooths_updates() {
    let replica = net_replication_runtime_manager();
    replica.register_component(
        SyncComponentDescriptor::new("Transform", SyncAuthority::Server)
            .with_field(SyncFieldDescriptor::new("x", "f32")),
    );

    let object = NetObjectId::new(101);
    replica
        .apply_delta_at(
            SyncDelta::new(object, "Transform", 1, [f32_field("x", 0.0)]),
            0,
        )
        .unwrap();
    replica
        .apply_delta_at(
            SyncDelta::new(object, "Transform", 2, [f32_field("x", 10.0)]),
            100,
        )
        .unwrap();

    let midpoint = replica
        .interpolated_f32_field(object, "Transform", "x", 150)
        .unwrap();
    assert!((midpoint - 5.0).abs() <= f32::EPSILON);

    let latest = replica
        .interpolated_f32_field(object, "Transform", "x", 250)
        .unwrap();
    assert!((latest - 10.0).abs() <= f32::EPSILON);
}
