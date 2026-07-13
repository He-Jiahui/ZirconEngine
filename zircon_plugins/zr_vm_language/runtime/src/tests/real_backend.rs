use crate::{register_zr_vm_backend, ZR_VM_PROJECT_BACKEND_SELECTOR};
use zircon_runtime::script::{VmBackend, ZrVmBackend};

use super::support::{
    build_real_backend_host, fixture_state_blob, DocumentedZrVmExampleFixture, ZrVmProjectFixture,
};

#[test]
fn real_backend_loads_native_host_modules_and_roundtrips_lifecycle() {
    let fixture = ZrVmProjectFixture::new_with_extension_channels("native_host_roundtrip", "0.1.0");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    register_zr_vm_backend(&manager);

    let packages = manager
        .discover_packages(&fixture.root)
        .expect("discover zr_vm package");
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);

    let slot = manager
        .load_discovered_package(&packages[0])
        .expect("load and activate zr_vm package");
    let record = manager.slot(slot).expect("loaded slot record");
    assert_eq!(record.backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);
    assert_eq!(
        record.source.zr_vm_project_path,
        Some(fixture.project_path.clone())
    );
    assert_eq!(
        manager
            .registered_systems(zircon_runtime::script::VmSystemStage::Update)
            .len(),
        1
    );
    assert_eq!(manager.registered_behavior_nodes().len(), 1);
    assert_eq!(manager.registered_rpc_handlers().len(), 1);
    assert_eq!(manager.registered_editor_operations().len(), 1);

    manager
        .hot_reload_discovered_slot(slot, &packages[0])
        .expect("save, reload, restore, and reactivate zr_vm package");
    assert_eq!(manager.slot(slot).expect("reloaded slot").generation, 2);
    assert_eq!(
        manager.registered_systems(zircon_runtime::script::VmSystemStage::Update)[0]
            .callback
            .generation,
        2
    );
    assert_eq!(
        manager
            .run_registered_systems(zircon_runtime::script::VmSystemStage::Update, 1.0 / 60.0)
            .expect("invoke reloaded real ZrVM system callback"),
        1
    );
    manager
        .unload_slot(slot)
        .expect("deactivate and unload slot");
    assert!(manager.list_slots().is_empty());
    assert!(manager.registered_behavior_nodes().is_empty());
    assert!(manager.registered_rpc_handlers().is_empty());
    assert!(manager.registered_editor_operations().is_empty());
}

#[test]
fn real_backend_session_preserves_lifecycle_state() {
    let fixture = ZrVmProjectFixture::new("native_host_session_state", "0.1.0");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager
        .discover_packages(&fixture.root)
        .expect("discover zr_vm package");
    let host = build_real_backend_host(&manager, &packages[0]);
    let mut instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .expect("load zr_vm package instance");

    instance
        .activate(&host)
        .expect("activate persistent session");
    let activated = instance.save_state().expect("save state after activate");
    assert_eq!(activated, fixture_state_blob("activated"));
    assert_eq!(
        instance
            .state_schema()
            .expect("query reflected state schema")
            .expect("fixture publishes reflected state schema")
            .schema_version,
        zircon_runtime::script::VM_STATE_SCHEMA_VERSION_V2,
    );

    instance
        .restore_state(&fixture_state_blob("hot"))
        .expect("restore state in persistent session");
    let restored = instance.save_state().expect("save state after restore");
    assert_eq!(restored, fixture_state_blob("hot"));
}

#[test]
fn real_backend_loads_documented_minimal_example() {
    let fixture = DocumentedZrVmExampleFixture::copy_from_docs();
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager
        .discover_packages(&fixture.root)
        .expect("discover documented zr_vm example package");
    assert_eq!(packages.len(), 1);
    assert_eq!(packages[0].backend_name, ZR_VM_PROJECT_BACKEND_SELECTOR);

    let host = build_real_backend_host(&manager, &packages[0]);
    let mut instance = ZrVmBackend::default()
        .load_package(&packages[0].package, &host)
        .expect("load documented zr_vm example");

    instance
        .activate(&host)
        .expect("activate documented example");
    let activated = instance
        .save_state()
        .expect("save documented example state")
        .payload;
    assert_eq!(String::from_utf8(activated).unwrap(), "activated");

    instance
        .restore_state(&zircon_runtime::script::VmStateBlob::from_payload(
            b"docs".to_vec(),
        ))
        .expect("restore documented example state");
    let restored = instance
        .save_state()
        .expect("save restored documented example state")
        .payload;
    assert_eq!(String::from_utf8(restored).unwrap(), "docs");
    instance
        .deactivate()
        .expect("deactivate documented example");
}
