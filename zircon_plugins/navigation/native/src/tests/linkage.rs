use crate::{native_backend_version, native_runtime_modules_available};

#[test]
fn native_recast_detour_modules_are_linked() {
    assert_eq!(native_backend_version(), 1);
    assert!(native_runtime_modules_available());
}
