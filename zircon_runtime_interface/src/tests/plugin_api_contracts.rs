use crate::{
    ZrByteBufferRef, ZrComponentDescV1, ZrHostApiV3, ZrHostApiV4, ZrHostAssetApiV1,
    ZrHostBridgeApiV1, ZrHostDiagnosticsApiV1, ZrHostEcsApiV1, ZrHostEcsApiV2, ZrHostEventApiV1,
    ZrNativeSystemAccessV1, ZrPluginStateSnapshotApiV1, ZrStatusCode, ZrSystemRegistrationV1,
    ZrSystemRegistrationV2, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1,
    ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1, ZR_PLUGIN_ENTRY_SYMBOL_V3, ZR_PLUGIN_ENTRY_SYMBOL_V4,
};

#[test]
fn abi_v3_layout_is_stable() {
    host_api_v3_table_records_domain_tables_and_size();
    host_api_v3_ecs_registration_dtos_have_stable_layout();
    plugin_snapshot_api_and_buffer_ref_are_abi_plain_data();
    host_api_v3_domain_table_sizes_are_pointer_dense();
}

#[test]
fn host_api_v3_table_records_domain_tables_and_size() {
    let api = ZrHostApiV3::empty();

    assert_eq!(ZR_PLUGIN_ENTRY_SYMBOL_V3, b"zircon_plugin_entry_v3\0");
    assert_eq!(api.abi_version, 3);
    assert_eq!(api.size_bytes, core::mem::size_of::<ZrHostApiV3>());
    assert_eq!(core::mem::size_of::<ZrHostApiV3>(), 88);
    assert_eq!(core::mem::offset_of!(ZrHostApiV3, ecs), 16);
    assert_eq!(
        core::mem::offset_of!(ZrHostApiV3, asset),
        core::mem::offset_of!(ZrHostApiV3, ecs) + core::mem::size_of::<ZrHostEcsApiV1>()
    );
    assert_eq!(
        core::mem::offset_of!(ZrHostApiV3, event),
        core::mem::offset_of!(ZrHostApiV3, asset) + core::mem::size_of::<ZrHostAssetApiV1>()
    );
    assert_eq!(
        core::mem::offset_of!(ZrHostApiV3, bridge),
        core::mem::offset_of!(ZrHostApiV3, event) + core::mem::size_of::<ZrHostEventApiV1>()
    );
    assert_eq!(
        core::mem::offset_of!(ZrHostApiV3, diagnostics),
        core::mem::offset_of!(ZrHostApiV3, bridge) + core::mem::size_of::<ZrHostBridgeApiV1>()
    );
    assert!(api.ecs.register_system.is_none());
    assert!(api.ecs.register_component.is_none());
    assert!(api.ecs.spawn_command.is_none());
    assert!(api.asset.request.is_none());
    assert!(api.event.emit.is_none());
    assert!(api.event.drain.is_none());
    assert!(api.bridge.call.is_none());
    assert!(api.diagnostics.emit.is_none());
    assert!(api.diagnostics.metric.is_none());
}

#[test]
fn host_api_v4_adds_a_versioned_typed_ecs_registration_table() {
    let api = ZrHostApiV4::empty();
    let access = ZrNativeSystemAccessV1::empty();
    let system = ZrSystemRegistrationV2::empty(4);

    assert_eq!(ZR_PLUGIN_ENTRY_SYMBOL_V4, b"zircon_plugin_entry_v4\0");
    assert_eq!(api.abi_version, 4);
    assert_eq!(api.size_bytes, core::mem::size_of::<ZrHostApiV4>());
    assert_eq!(core::mem::size_of::<ZrHostApiV4>(), 88);
    assert_eq!(core::mem::offset_of!(ZrHostApiV4, ecs), 16);
    assert_eq!(core::mem::size_of::<ZrHostEcsApiV2>(), 24);
    assert!(api.ecs.register_system.is_none());
    assert!(api.ecs.register_component.is_none());
    assert!(api.ecs.spawn_command.is_none());

    assert_eq!(access.abi_version, 1);
    assert_eq!(
        access.size_bytes,
        core::mem::size_of::<ZrNativeSystemAccessV1>()
    );
    assert_eq!(core::mem::size_of::<ZrNativeSystemAccessV1>(), 40);
    assert_eq!(core::mem::offset_of!(ZrNativeSystemAccessV1, mode), 16);
    assert_eq!(core::mem::offset_of!(ZrNativeSystemAccessV1, stable_id), 24);
    assert_eq!(access.mode, ZR_NATIVE_SYSTEM_ACCESS_MODE_READ_V1);
    assert_eq!(access.domain, ZR_NATIVE_SYSTEM_ACCESS_DOMAIN_COMPONENT_V1);

    assert_eq!(
        system.size_bytes,
        core::mem::size_of::<ZrSystemRegistrationV2>()
    );
    assert_eq!(core::mem::size_of::<ZrSystemRegistrationV2>(), 128);
    assert_eq!(core::mem::offset_of!(ZrSystemRegistrationV2, accesses), 88);
    assert_eq!(
        core::mem::offset_of!(ZrSystemRegistrationV2, thread_affinity),
        104
    );
    assert_eq!(core::mem::offset_of!(ZrSystemRegistrationV2, invoke), 112);
    assert!(system.accesses.is_null());
    assert_eq!(system.access_count, 0);
}

#[test]
fn host_api_v3_ecs_registration_dtos_have_stable_layout() {
    let system = ZrSystemRegistrationV1::empty(3);
    let component = ZrComponentDescV1::empty(3);

    assert_eq!(
        system.size_bytes,
        core::mem::size_of::<ZrSystemRegistrationV1>()
    );
    assert_eq!(core::mem::size_of::<ZrSystemRegistrationV1>(), 104);
    assert_eq!(core::mem::offset_of!(ZrSystemRegistrationV1, stage), 32);
    assert_eq!(core::mem::offset_of!(ZrSystemRegistrationV1, order), 36);
    assert_eq!(core::mem::offset_of!(ZrSystemRegistrationV1, invoke), 88);
    assert!(system.system_id.is_empty());
    assert!(system.set_names.is_null());
    assert!(system.before.is_null());
    assert!(system.after.is_null());
    assert!(system.invoke.is_none());

    assert_eq!(
        component.size_bytes,
        core::mem::size_of::<ZrComponentDescV1>()
    );
    assert_eq!(core::mem::size_of::<ZrComponentDescV1>(), 72);
    assert_eq!(core::mem::offset_of!(ZrComponentDescV1, schema), 48);
    assert!(component.type_id.is_empty());
    assert!(component.display_name.is_empty());
    assert!(component.schema.is_empty());
}

#[test]
fn plugin_snapshot_api_and_buffer_ref_are_abi_plain_data() {
    let snapshot_api = ZrPluginStateSnapshotApiV1::empty(3);
    let buffer = ZrByteBufferRef::empty();

    assert_eq!(
        snapshot_api.size_bytes,
        core::mem::size_of::<ZrPluginStateSnapshotApiV1>()
    );
    assert_eq!(core::mem::size_of::<ZrPluginStateSnapshotApiV1>(), 32);
    assert!(snapshot_api.save.is_none());
    assert!(snapshot_api.restore.is_none());
    assert!(buffer.data.is_null());
    assert_eq!(buffer.capacity, 0);
    assert!(buffer.written.is_null());
}

#[test]
fn host_api_v3_domain_table_sizes_are_pointer_dense() {
    assert_eq!(core::mem::size_of::<ZrHostEcsApiV1>(), 24);
    assert_eq!(core::mem::size_of::<ZrHostAssetApiV1>(), 8);
    assert_eq!(core::mem::size_of::<ZrHostEventApiV1>(), 16);
    assert_eq!(core::mem::size_of::<ZrHostBridgeApiV1>(), 8);
    assert_eq!(core::mem::size_of::<ZrHostDiagnosticsApiV1>(), 16);
}

#[test]
fn bridge_not_enabled_status_code_is_stable() {
    assert_eq!(ZrStatusCode::BridgeNotEnabled.as_raw(), 7);
    assert_eq!(ZrStatusCode::from_raw(7), ZrStatusCode::BridgeNotEnabled);
}
