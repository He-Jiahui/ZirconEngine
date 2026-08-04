use zircon_runtime_interface::{
    ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1, ZrRuntimeViewportHandle,
    ZrStatusCode,
};

use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;

use super::super::ffi;
use super::super::registry::{destroy_session_slot, insert_session, with_session};

#[test]
fn session_abi_retains_canonical_latest_value_per_viewport() {
    let handle = insert_session(
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
            .expect("headless session"),
    );
    let viewport_a = ZrRuntimeViewportHandle::new(3);
    let viewport_b = ZrRuntimeViewportHandle::new(4);
    let attributes = ZrRuntimeHighlightRenderAttributesV1::outlined([0.2, 0.5, 0.8, 1.0]);

    let newest = unsafe {
        ffi::submit_highlight_set(
            handle,
            ZrRuntimeHighlightSetV1::new(viewport_a, 9, &[8, 2, 8], attributes),
        )
    };
    assert_eq!(newest.status_code(), ZrStatusCode::Ok, "{newest:?}");
    let other = unsafe {
        ffi::submit_highlight_set(
            handle,
            ZrRuntimeHighlightSetV1::new(viewport_b, 1, &[11], attributes),
        )
    };
    assert_eq!(other.status_code(), ZrStatusCode::Ok, "{other:?}");
    let stale = unsafe {
        ffi::submit_highlight_set(
            handle,
            ZrRuntimeHighlightSetV1::new(viewport_a, 8, &[99], attributes),
        )
    };
    assert_eq!(stale.status_code(), ZrStatusCode::Ok, "{stale:?}");

    let status = with_session(handle, |session| {
        let first = session
            .level
            .viewport_highlight_set(viewport_a.raw())
            .unwrap();
        let second = session
            .level
            .viewport_highlight_set(viewport_b.raw())
            .unwrap();
        assert_eq!(first.generation(), 9);
        assert_eq!(first.set().entities(), &[2, 8]);
        assert_eq!(second.generation(), 1);
        assert_eq!(second.set().entities(), &[11]);
        zircon_runtime_interface::ZrStatus::ok()
    });
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}
