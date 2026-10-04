use std::sync::Arc;

use super::{VmBackendFamily, ZrVmBackendFamily};

#[test]
fn zr_vm_backend_resolutions_share_arc_storage() {
    let family = ZrVmBackendFamily;
    let canonical = family.resolve("zr_vm:project").unwrap();
    let alias = family.resolve("project").unwrap();

    assert!(Arc::ptr_eq(&canonical, &alias));
}
