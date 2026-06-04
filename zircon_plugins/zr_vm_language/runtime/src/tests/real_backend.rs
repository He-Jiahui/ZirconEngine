use crate::{register_zr_vm_backend, ZR_VM_PROJECT_BACKEND_SELECTOR};

use super::support::{build_real_backend_host, DocumentedZrVmExampleFixture, ZrVmProjectFixture};

#[test]
fn real_backend_loads_native_host_modules_and_roundtrips_lifecycle() {
    let fixture = ZrVmProjectFixture::new("native_host_roundtrip", "0.1.0");
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

    manager
        .hot_reload_discovered_slot(slot, &packages[0])
        .expect("save, reload, restore, and reactivate zr_vm package");
    manager
        .unload_slot(slot)
        .expect("deactivate and unload slot");
    assert!(manager.list_slots().is_empty());
}

#[test]
fn real_backend_session_preserves_lifecycle_state() {
    let fixture = ZrVmProjectFixture::new("native_host_session_state", "0.1.0");
    let manager = zircon_runtime::script::VmPluginManager::mock();
    let packages = manager
        .discover_packages(&fixture.root)
        .expect("discover zr_vm package");
    let host = build_real_backend_host(&manager, &packages[0]);
    let mut instance = crate::real_backend::load_project_package(&packages[0].package, &host)
        .expect("load zr_vm package instance");

    instance
        .activate(&host)
        .expect("activate persistent session");
    let activated = instance
        .save_state()
        .expect("save state after activate")
        .bytes;
    assert_eq!(String::from_utf8(activated).unwrap(), "activated");

    instance
        .restore_state(&zircon_runtime::script::VmStateBlob {
            bytes: b"hot".to_vec(),
        })
        .expect("restore state in persistent session");
    let restored = instance
        .save_state()
        .expect("save state after restore")
        .bytes;
    assert_eq!(String::from_utf8(restored).unwrap(), "hot:restored");
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
    let mut instance = crate::real_backend::load_project_package(&packages[0].package, &host)
        .expect("load documented zr_vm example");

    instance
        .activate(&host)
        .expect("activate documented example");
    let activated = instance
        .save_state()
        .expect("save documented example state")
        .bytes;
    assert_eq!(String::from_utf8(activated).unwrap(), "activated");

    instance
        .restore_state(&zircon_runtime::script::VmStateBlob {
            bytes: b"docs".to_vec(),
        })
        .expect("restore documented example state");
    let restored = instance
        .save_state()
        .expect("save restored documented example state")
        .bytes;
    assert_eq!(String::from_utf8(restored).unwrap(), "docs:restored");
    instance
        .deactivate()
        .expect("deactivate documented example");
}
