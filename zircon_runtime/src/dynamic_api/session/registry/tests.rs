use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZrRuntimeWakeSinkV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2, ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
};

use super::frame_demand::FrameDemandAccumulator;
use super::{
    destroy_session_slot, insert_session_with_wake, session_is_closing, with_session_activity,
    RuntimeFrameDemand, RuntimeWakeRegistration, MAX_RUNTIME_FRAME_DEMAND_DELAY,
};
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;

static WAKE_ENTERED: AtomicBool = AtomicBool::new(false);
static WAKE_RELEASED: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn blocking_wake(_token: u64) {
    WAKE_ENTERED.store(true, Ordering::Release);
    while !WAKE_RELEASED.load(Ordering::Acquire) {
        thread::yield_now();
    }
}

#[test]
fn frame_demand_merge_immediate_dominates_earliest_delay_and_consume_resets_idle() {
    let mut accumulator = FrameDemandAccumulator::default();
    accumulator.merge(RuntimeFrameDemand::After(Duration::from_millis(30)));
    accumulator.merge(RuntimeFrameDemand::Idle);
    accumulator.merge(RuntimeFrameDemand::After(Duration::from_millis(10)));

    assert_eq!(
        accumulator.consume(),
        RuntimeFrameDemand::After(Duration::from_millis(10))
    );
    assert_eq!(accumulator.consume(), RuntimeFrameDemand::Idle);

    accumulator.merge(RuntimeFrameDemand::After(Duration::from_millis(1)));
    accumulator.merge(RuntimeFrameDemand::Immediate);
    accumulator.merge(RuntimeFrameDemand::After(Duration::ZERO));
    assert_eq!(accumulator.consume(), RuntimeFrameDemand::Immediate);
}

#[test]
fn frame_demand_checked_conversion_rejects_unknown_kind_and_clamps_after_delay() {
    assert_eq!(
        RuntimeFrameDemand::try_from(ZrRuntimeFrameDemandV1::after(1_000)).unwrap(),
        RuntimeFrameDemand::After(Duration::from_nanos(1_000))
    );
    assert!(matches!(
        RuntimeFrameDemand::try_from(ZrRuntimeFrameDemandV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            kind: zircon_runtime_interface::ZR_RUNTIME_FRAME_DEMAND_IDLE_V1,
            delay_nanoseconds: 0,
        }),
        Err(super::frame_demand::InvalidRuntimeFrameDemand::UnsupportedVersion)
    ));

    let unknown = ZrRuntimeFrameDemandV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: u32::MAX,
        delay_nanoseconds: 0,
    };
    assert!(RuntimeFrameDemand::try_from(unknown).is_err());

    let oversized = RuntimeFrameDemand::After(MAX_RUNTIME_FRAME_DEMAND_DELAY * 2);
    assert_eq!(
        oversized.into_abi(),
        ZrRuntimeFrameDemandV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            kind: ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
            delay_nanoseconds: MAX_RUNTIME_FRAME_DEMAND_DELAY.as_nanos() as u64,
        }
    );
}

#[test]
fn wake_registration_rejects_bad_pairs_and_unsupported_version() {
    for sink in [
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            token: 0,
            wake: None,
        },
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 7,
            wake: None,
        },
        ZrRuntimeWakeSinkV1 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            token: 0,
            wake: Some(blocking_wake),
        },
    ] {
        assert!(RuntimeWakeRegistration::from_abi(sink).is_err());
    }
}

#[test]
fn destroy_is_a_quiescence_barrier_for_actions_and_wake_callbacks() {
    WAKE_ENTERED.store(false, Ordering::Release);
    WAKE_RELEASED.store(false, Ordering::Release);

    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let wake = RuntimeWakeRegistration::from_abi(ZrRuntimeWakeSinkV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        token: 7,
        wake: Some(blocking_wake),
    })
    .unwrap();
    let handle = insert_session_with_wake(session, wake);
    let (action_entered_tx, action_entered_rx) = mpsc::channel();
    let (release_action_tx, release_action_rx) = mpsc::channel();
    let (wake_tx, wake_rx) = mpsc::channel();

    let action = thread::spawn(move || {
        let status = with_session_activity(handle, |_session, activity| {
            wake_tx.send(activity.wake_registration()).unwrap();
            action_entered_tx.send(()).unwrap();
            release_action_rx.recv().unwrap();
            ZrStatus::ok()
        });
        assert_eq!(status.status_code(), ZrStatusCode::Ok);
    });
    action_entered_rx.recv().unwrap();
    let wake_registration = wake_rx.recv().unwrap();
    let wake_thread = thread::spawn(move || wake_registration.wake());
    while !WAKE_ENTERED.load(Ordering::Acquire) {
        thread::yield_now();
    }

    let (destroy_tx, destroy_rx) = mpsc::channel();
    let destroy = thread::spawn(move || {
        destroy_tx
            .send(destroy_session_slot(handle).status_code())
            .unwrap();
    });

    while !session_is_closing(handle) {
        thread::yield_now();
    }
    assert_eq!(
        with_session_activity(handle, |_session, _activity| ZrStatus::ok()).status_code(),
        ZrStatusCode::NotFound
    );
    assert!(destroy_rx.try_recv().is_err());

    release_action_tx.send(()).unwrap();
    action.join().unwrap();
    assert!(destroy_rx.try_recv().is_err());

    WAKE_RELEASED.store(true, Ordering::Release);
    assert!(wake_thread.join().unwrap());
    assert_eq!(destroy_rx.recv().unwrap(), ZrStatusCode::Ok);
    destroy.join().unwrap();

    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::NotFound
    );
}
