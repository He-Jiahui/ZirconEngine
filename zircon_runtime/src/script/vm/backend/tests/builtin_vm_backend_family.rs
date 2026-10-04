use std::sync::Arc;

use super::{BuiltinVmBackendFamily, VmBackendFamily};

#[test]
fn builtin_backend_resolutions_share_arc_storage() {
    let family = BuiltinVmBackendFamily;
    let mock = family.resolve("builtin:mock").unwrap();
    let mock_alias = family.resolve("mock").unwrap();
    let unavailable = family.resolve("builtin:unavailable").unwrap();
    let unavailable_alias = family.resolve("unavailable").unwrap();

    assert!(Arc::ptr_eq(&mock, &mock_alias));
    assert!(Arc::ptr_eq(&unavailable, &unavailable_alias));
    assert!(!Arc::ptr_eq(&mock, &unavailable));
}
