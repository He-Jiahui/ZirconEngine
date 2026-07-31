use std::panic::{self, AssertUnwindSafe};

use zircon_runtime_interface::{ZrStatus, ZrStatusCode};

use super::super::ffi::destroy_session;
use super::super::profile::RuntimeDynamicSessionProfile;
use super::super::registry::{insert_session, poison_registry_lock_for_test, with_session};
use super::super::state::RuntimeDynamicSession;

#[test]
fn dynamic_api_session_registry_accessors_recover_poisoned_locks() {
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        poison_registry_lock_for_test();
    }));

    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session(session);

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        with_session(handle, |_| panic!("poison dynamic API session lock"));
    }));

    let status = with_session(handle, |_| ZrStatus::ok());
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let destroy = unsafe { destroy_session(handle) };
    assert_eq!(destroy.status_code(), ZrStatusCode::Ok, "{destroy:?}");
}
