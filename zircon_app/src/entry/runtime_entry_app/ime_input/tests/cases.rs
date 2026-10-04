use winit::event::Ime;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZR_RUNTIME_EVENT_KIND_IME_V1,
    ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1, ZR_RUNTIME_IME_STATE_COMMIT_V1,
    ZR_RUNTIME_IME_STATE_DISABLED_V1, ZR_RUNTIME_IME_STATE_ENABLED_V1,
    ZR_RUNTIME_IME_STATE_PREEDIT_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
};

use super::{
    admission::RuntimeImeInputAdmission, focus_changed_events,
    lifecycle::focus_changed_events_with_native_source, routing::admitted_runtime_ime_event,
};

#[derive(Debug, PartialEq, Eq)]
struct EventSnapshot {
    kind: u32,
    state: u32,
    key_code: u32,
    scan_code: u32,
    payload: Vec<u8>,
}

impl EventSnapshot {
    fn capture(event: ZrRuntimeEventV1) -> Self {
        let payload = unsafe { event.payload.checked_slice(usize::MAX) }
            .expect("test events retain their source payload")
            .to_vec();
        Self {
            kind: event.kind,
            state: event.state,
            key_code: event.key_code,
            scan_code: event.scan_code,
            payload,
        }
    }
}

#[test]
fn focused_window_ime_handler_orders_cancel_and_rejects_the_stale_composition_cycle() {
    let viewport = ZrRuntimeViewportHandle::new(7);
    let mut admission = RuntimeImeInputAdmission::new(true);

    let enabled = admitted_runtime_ime_event(&mut admission, viewport, &Ime::Enabled)
        .map(EventSnapshot::capture);
    assert_eq!(
        enabled,
        Some(EventSnapshot {
            kind: ZR_RUNTIME_EVENT_KIND_IME_V1,
            state: ZR_RUNTIME_IME_STATE_ENABLED_V1,
            key_code: 0,
            scan_code: 0,
            payload: Vec::new(),
        })
    );
    assert!(admitted_runtime_ime_event(
        &mut admission,
        viewport,
        &Ime::Preedit("old".to_string(), Some((1, 2))),
    )
    .is_some());

    let blurred = focus_changed_events(&mut admission, viewport, false)
        .into_iter()
        .flatten()
        .map(EventSnapshot::capture)
        .collect::<Vec<_>>();
    assert_eq!(
        blurred,
        vec![
            EventSnapshot {
                kind: ZR_RUNTIME_EVENT_KIND_IME_V1,
                state: ZR_RUNTIME_IME_STATE_DISABLED_V1,
                key_code: 0,
                scan_code: 0,
                payload: Vec::new(),
            },
            EventSnapshot {
                kind: ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
                state: ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
                key_code: 0,
                scan_code: 0,
                payload: Vec::new(),
            },
        ]
    );

    for late in [
        Ime::Commit("stale-commit".to_string()),
        Ime::Preedit("stale-preedit".to_string(), Some((0, 5))),
        Ime::DeleteSurrounding {
            before_bytes: 2,
            after_bytes: 1,
        },
    ] {
        assert!(admitted_runtime_ime_event(&mut admission, viewport, &late).is_none());
    }

    let refocused = focus_changed_events(&mut admission, viewport, true)
        .into_iter()
        .flatten()
        .map(EventSnapshot::capture)
        .collect::<Vec<_>>();
    assert_eq!(
        refocused,
        vec![EventSnapshot {
            kind: ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1,
            state: ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
            key_code: 0,
            scan_code: 0,
            payload: Vec::new(),
        }]
    );
    assert!(admitted_runtime_ime_event(
        &mut admission,
        viewport,
        &Ime::Commit("old-cycle".to_string()),
    )
    .is_none());

    assert!(admitted_runtime_ime_event(&mut admission, viewport, &Ime::Enabled).is_some());
    let new_preedit = admitted_runtime_ime_event(
        &mut admission,
        viewport,
        &Ime::Preedit("new".to_string(), Some((0, 3))),
    )
    .map(EventSnapshot::capture);
    let new_commit = admitted_runtime_ime_event(
        &mut admission,
        viewport,
        &Ime::Commit("accepted".to_string()),
    )
    .map(EventSnapshot::capture);
    assert_eq!(
        new_preedit,
        Some(EventSnapshot {
            kind: ZR_RUNTIME_EVENT_KIND_IME_V1,
            state: ZR_RUNTIME_IME_STATE_PREEDIT_V1,
            key_code: 0,
            scan_code: 3,
            payload: b"new".to_vec(),
        })
    );
    assert_eq!(
        new_commit,
        Some(EventSnapshot {
            kind: ZR_RUNTIME_EVENT_KIND_IME_V1,
            state: ZR_RUNTIME_IME_STATE_COMMIT_V1,
            key_code: 0,
            scan_code: 0,
            payload: b"accepted".to_vec(),
        })
    );
}

#[test]
fn native_source_replaces_only_the_legacy_ime_cancel_on_focus_loss() {
    let viewport = ZrRuntimeViewportHandle::new(19);
    let mut admission = RuntimeImeInputAdmission::new(true);
    assert!(admission.enable());

    let events = focus_changed_events_with_native_source(&mut admission, viewport, false, true);
    let snapshots = events
        .into_iter()
        .flatten()
        .map(EventSnapshot::capture)
        .collect::<Vec<_>>();

    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].kind, ZR_RUNTIME_EVENT_KIND_LIFECYCLE_V1);
    assert_eq!(snapshots[0].state, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1);
    assert!(!admission.admits_composition());
}
