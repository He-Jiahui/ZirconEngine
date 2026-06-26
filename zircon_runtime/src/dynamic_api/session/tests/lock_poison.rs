use std::panic::{self, AssertUnwindSafe};

use zircon_runtime_interface::{ZrStatus, ZrStatusCode};

use super::super::{
    destroy_session, insert_session, lock_registry, lock_session, with_session,
    RuntimeDynamicSession, RuntimeDynamicSessionProfile,
};

#[test]
fn dynamic_api_session_registry_accessors_recover_poisoned_locks() {
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _registry = lock_registry();
        panic!("poison dynamic API session registry lock");
    }));

    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session(session);

    let stored_session = {
        let registry = lock_registry();
        registry
            .sessions
            .get(&handle.raw())
            .cloned()
            .expect("inserted dynamic API session should exist")
    };
    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _session = lock_session(stored_session.as_ref());
        panic!("poison dynamic API session lock");
    }));

    let status = with_session(handle, |_| ZrStatus::ok());
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");

    let destroy = unsafe { destroy_session(handle) };
    assert_eq!(destroy.status_code(), ZrStatusCode::Ok, "{destroy:?}");
}
