use zircon_runtime_interface::{
    ZrRuntimeViewportHandle, ZR_RUNTIME_EVENT_KIND_IME_V1, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
    ZR_RUNTIME_IME_STATE_DISABLED_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
};

use super::dispatch_focus_events;
use crate::entry::runtime_entry_app::ime_input::{
    focus_changed_events, focus_changed_events_with_native_source, RuntimeImeInputAdmission,
};

#[test]
fn focused_window_handler_stops_dispatch_after_runtime_rejects_ime_cancel() {
    for reject_cancel in [false, true] {
        let mut admission = RuntimeImeInputAdmission::new(true);
        assert!(admission.enable());
        let events = focus_changed_events(&mut admission, ZrRuntimeViewportHandle::new(7), false);
        let mut dispatched = Vec::new();

        dispatch_focus_events(events, |event| {
            dispatched.push((event.kind, event.state));
            !reject_cancel
        });

        let mut expected = vec![(
            ZR_RUNTIME_EVENT_KIND_IME_V1,
            ZR_RUNTIME_IME_STATE_DISABLED_V1,
        )];
        if !reject_cancel {
            expected.push((
                ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
                ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
            ));
        }
        assert_eq!(dispatched, expected);
        assert!(!admission.admits_composition());
    }
}

#[test]
fn native_cancel_failure_still_publishes_background_lifecycle() {
    let mut admission = RuntimeImeInputAdmission::new(true);
    assert!(admission.enable());
    let events = focus_changed_events_with_native_source(
        &mut admission,
        ZrRuntimeViewportHandle::new(11),
        false,
        true,
    );
    let mut dispatched = Vec::new();

    dispatch_focus_events(events, |event| {
        dispatched.push((event.kind, event.state));
        true
    });

    assert_eq!(
        dispatched,
        vec![(
            ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
        )]
    );
    assert!(!admission.admits_composition());
}
