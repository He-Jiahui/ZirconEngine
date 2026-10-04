use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeSessionHandle, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
};

use super::super::session_store::{destroy_session_slot_with_timeout, session_active_actions};
use super::super::{
    destroy_session_slot, insert_session_with_wake, session_is_closing, with_session,
    RuntimeWakeRegistration,
};
use crate::core::framework::input::{FileDragDropEvent, InputEvent};
use crate::dynamic_api::session::ffi;
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;

#[test]
fn owned_event_preserves_status_diagnostics_on_the_calling_thread() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let unsupported = unsafe {
        ffi::handle_event(
            handle,
            ZrRuntimeEventV1::new(
                u32::MAX,
                u32::MAX,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ),
        )
    };
    assert_eq!(unsupported.status_code(), ZrStatusCode::UnsupportedVersion);
    assert_eq!(
        unsafe { unsupported.diagnostics.checked_slice(4096) }.unwrap(),
        b"unsupported runtime ABI version"
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn owned_event_rejects_oversized_and_invalid_payloads_before_dispatch() {
    let invalid_handle = ZrRuntimeSessionHandle::new(0);
    let oversized = vec![b'x'; ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1 + 1];
    let oversized_status = unsafe {
        ffi::handle_event(
            invalid_handle,
            ZrRuntimeEventV1::file_dropped(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                ZrByteSlice {
                    data: oversized.as_ptr(),
                    len: oversized.len(),
                },
            ),
        )
    };
    assert_eq!(oversized_status.status_code(), ZrStatusCode::LimitExceeded);
    assert_eq!(
        unsafe { oversized_status.diagnostics.checked_slice(4096) }.unwrap(),
        b"runtime event payload exceeds limit"
    );

    let invalid_status = unsafe {
        ffi::handle_event(
            invalid_handle,
            ZrRuntimeEventV1::file_dropped(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
                ZrByteSlice {
                    data: ptr::null(),
                    len: 1,
                },
            ),
        )
    };
    assert_eq!(invalid_status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        unsafe { invalid_status.diagnostics.checked_slice(4096) }.unwrap(),
        b"invalid runtime event payload slice"
    );
}

#[test]
fn astra_life_a4_blocked_owner_payload_event_survives_short_destroy_retries() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let input_manager = session.resolve_input_manager().unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let (owner_entered_tx, owner_entered_rx) = mpsc::sync_channel(1);
    let (release_owner_tx, release_owner_rx) = mpsc::sync_channel(1);
    let owner_action = thread::spawn(move || {
        with_session(handle, move |_| {
            owner_entered_tx.send(()).unwrap();
            release_owner_rx.recv().unwrap();
            ZrStatus::ok()
        })
        .status_code()
    });
    owner_entered_rx
        .recv_timeout(Duration::from_secs(2))
        .unwrap();

    let event_action = thread::spawn(move || {
        let path = b"C:/tmp/asset.png".to_vec();
        let event = ZrRuntimeEventV1::file_dropped(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1,
            ZrByteSlice {
                data: path.as_ptr(),
                len: path.len(),
            },
        );
        unsafe { ffi::handle_event(handle, event) }.status_code()
    });
    let admission_deadline = Instant::now() + Duration::from_secs(2);
    while session_active_actions(handle) != Some(2) {
        assert!(
            Instant::now() < admission_deadline,
            "event was not admitted"
        );
        thread::yield_now();
    }
    assert!(!event_action.is_finished());

    let started_at = Instant::now();
    assert_eq!(
        destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code(),
        ZrStatusCode::Error
    );
    assert!(started_at.elapsed() < Duration::from_secs(1));
    assert!(session_is_closing(handle));
    assert_eq!(
        destroy_session_slot_with_timeout(handle, Duration::from_millis(25)).status_code(),
        ZrStatusCode::Error
    );
    assert!(!event_action.is_finished());

    release_owner_tx.send(()).unwrap();
    assert_eq!(owner_action.join().unwrap(), ZrStatusCode::Ok);
    assert_eq!(event_action.join().unwrap(), ZrStatusCode::Ok);
    let expected = InputEvent::FileDragDrop(FileDragDropEvent::Dropped {
        path: "C:/tmp/asset.png".to_owned(),
    });
    assert_eq!(
        input_manager
            .drain_events()
            .iter()
            .filter(|event| *event == &expected)
            .count(),
        1
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}
