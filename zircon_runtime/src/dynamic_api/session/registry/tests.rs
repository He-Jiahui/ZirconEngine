use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use zircon_runtime_interface::{
    ZrRuntimeFrameDemandV1, ZrRuntimePluginEventSubscribeRequestV1, ZrRuntimeWakeSinkV1, ZrStatus,
    ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2,
    ZR_RUNTIME_FRAME_DEMAND_AFTER_V1,
};

use super::allocation_registry::allocation_census;
use super::frame_demand::FrameDemandAccumulator;
use super::{
    destroy_session_slot, insert_session_with_wake, register_runtime_allocation,
    register_runtime_allocation_in_action, release_runtime_allocation, session_is_closing,
    with_session, with_session_activity, with_session_result_finalized, RuntimeAllocationKind,
    RuntimeFrameDemand, RuntimeWakeRegistration, MAX_RUNTIME_FRAME_DEMAND_DELAY,
};
use crate::dynamic_api::session::profile::RuntimeDynamicSessionProfile;
use crate::dynamic_api::session::state::RuntimeDynamicSession;
use crate::scene::{RuntimeEventMirrorRegistration, SceneError};

static WAKE_ENTERED: AtomicBool = AtomicBool::new(false);
static WAKE_RELEASED: AtomicBool = AtomicBool::new(false);
static WAKE_COUNT: AtomicU32 = AtomicU32::new(0);
static REENTRANT_DESTROY_HANDLE: AtomicU64 = AtomicU64::new(0);
static REENTRANT_DESTROY_STATUS: AtomicU32 = AtomicU32::new(u32::MAX);

unsafe extern "C" fn blocking_wake(_token: u64) {
    WAKE_ENTERED.store(true, Ordering::Release);
    while !WAKE_RELEASED.load(Ordering::Acquire) {
        thread::yield_now();
    }
}

unsafe extern "C" fn counting_wake(token: u64) {
    WAKE_COUNT.fetch_add(token as u32, Ordering::AcqRel);
}

unsafe extern "C" fn reentrant_destroy_wake(_token: u64) {
    let handle = zircon_runtime_interface::ZrRuntimeSessionHandle::new(
        REENTRANT_DESTROY_HANDLE.load(Ordering::Acquire),
    );
    REENTRANT_DESTROY_STATUS.store(
        destroy_session_slot(handle).status_code() as u32,
        Ordering::Release,
    );
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
fn wake_registration_channel_callback_invokes_the_runtime_sink() {
    WAKE_COUNT.store(0, Ordering::Release);
    let registration = RuntimeWakeRegistration::from_abi(ZrRuntimeWakeSinkV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        token: 3,
        wake: Some(counting_wake),
    })
    .unwrap();

    registration.channel_wake()();

    assert_eq!(WAKE_COUNT.load(Ordering::Acquire), 3);
}

#[test]
fn wake_callback_reentrant_destroy_is_rejected_without_closing_the_session() {
    REENTRANT_DESTROY_HANDLE.store(0, Ordering::Release);
    REENTRANT_DESTROY_STATUS.store(u32::MAX, Ordering::Release);
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let wake =
        RuntimeWakeRegistration::from_abi(ZrRuntimeWakeSinkV1::new(1, reentrant_destroy_wake))
            .unwrap();
    let trigger = wake.clone();
    let handle = insert_session_with_wake(session, wake);
    REENTRANT_DESTROY_HANDLE.store(handle.raw(), Ordering::Release);

    assert!(trigger.wake());
    assert_eq!(
        REENTRANT_DESTROY_STATUS.load(Ordering::Acquire),
        ZrStatusCode::InvalidArgument as u32,
    );
    assert!(!session_is_closing(handle));
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
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

#[test]
fn destroy_session_slot_preserves_failed_event_mirror_teardown_for_explicit_retry() {
    const EVENT_ID: &str = "dynamic_api.registry.retry_event";
    const PAYLOAD_SCHEMA: &str = "zircon.dynamic_api.registry.retry_event.v1";

    let fail_zero = Arc::new(AtomicBool::new(true));
    let fail_zero_for_registration = Arc::clone(&fail_zero);
    let readers = Arc::new(AtomicU32::new(0));
    let readers_for_registration = Arc::clone(&readers);
    let mut session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None)
        .expect("headless dynamic session");
    session.level.with_world_mut(|world| {
        world
            .register_runtime_event_mirror(
                RuntimeEventMirrorRegistration::typed::<u32>(EVENT_ID, PAYLOAD_SCHEMA)
                    .with_reader_count_callback(move |_world, count| {
                        if count == 0 && fail_zero_for_registration.load(Ordering::SeqCst) {
                            return Err(SceneError::EmptyNodeName);
                        }
                        readers_for_registration.store(count, Ordering::SeqCst);
                        Ok(())
                    }),
            )
            .expect("retry event mirror registration");
    });
    session
        .subscribe_plugin_event(ZrRuntimePluginEventSubscribeRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            EVENT_ID,
            PAYLOAD_SCHEMA,
        ))
        .expect("retry plugin event subscription");
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::Error
    );
    assert!(session_is_closing(handle));
    assert_eq!(
        with_session(handle, |_session| ZrStatus::ok()).status_code(),
        ZrStatusCode::NotFound
    );
    assert_eq!(readers.load(Ordering::SeqCst), 1);

    fail_zero.store(false, Ordering::SeqCst);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
    assert_eq!(readers.load(Ordering::SeqCst), 0);
    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::NotFound
    );
}

#[test]
fn allocation_finalizer_keeps_destroy_in_action_barrier_until_registration_finishes() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let (finalizer_entered_tx, finalizer_entered_rx) = mpsc::channel();
    let (release_finalizer_tx, release_finalizer_rx) = mpsc::channel();
    let (allocation_tx, allocation_rx) = mpsc::channel();

    let producer = thread::spawn(move || {
        let allocation = with_session_result_finalized(
            handle,
            |_session| Ok(vec![0x5a_u8; 32]),
            |active_handle, bytes| {
                finalizer_entered_tx.send(()).unwrap();
                release_finalizer_rx.recv().unwrap();
                register_runtime_allocation_in_action(
                    active_handle,
                    RuntimeAllocationKind::Accessibility,
                    bytes,
                )
                .map(|output| output.allocation)
            },
        )
        .expect("runtime allocation finalizer");
        allocation_tx.send(allocation).unwrap();
    });
    finalizer_entered_rx.recv().unwrap();

    let (destroy_started_tx, destroy_started_rx) = mpsc::channel();
    let (destroy_tx, destroy_rx) = mpsc::channel();
    let destroy = thread::spawn(move || {
        destroy_started_tx.send(()).unwrap();
        destroy_tx
            .send(destroy_session_slot(handle).status_code())
            .unwrap();
    });
    destroy_started_rx.recv().unwrap();

    let closing_deadline = Instant::now() + Duration::from_secs(1);
    while !session_is_closing(handle) && Instant::now() < closing_deadline {
        thread::yield_now();
    }
    let closing = session_is_closing(handle);
    let destroy_wait = destroy_rx.recv_timeout(Duration::from_millis(50));
    release_finalizer_tx.send(()).unwrap();

    assert!(closing, "destroy must enter the closing phase");
    assert_eq!(destroy_wait, Err(mpsc::RecvTimeoutError::Timeout));

    let allocation = allocation_rx.recv().unwrap();
    producer.join().unwrap();
    assert_eq!(destroy_rx.recv().unwrap(), ZrStatusCode::Error);
    destroy.join().unwrap();

    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn runtime_allocation_release_uses_the_session_bound_opaque_id_and_cannot_double_free() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let output = register_runtime_allocation(
        handle,
        RuntimeAllocationKind::WorldSync,
        vec![1_u8, 2, 3, 4],
    )
    .expect("registered runtime allocation");
    let allocation = output.allocation;

    assert_eq!(output.len, 4);
    assert!(!output.data.is_null());
    assert_eq!(allocation_census(handle).outstanding_allocations, 1);
    assert_eq!(allocation_census(handle).outstanding_bytes, 4);
    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(
        release_runtime_allocation(handle, allocation).status_code(),
        ZrStatusCode::NotFound
    );
    assert_eq!(allocation_census(handle).outstanding_allocations, 0);
    assert_eq!(allocation_census(handle).outstanding_bytes, 0);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn forged_runtime_allocation_id_never_changes_the_session_census() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let output =
        register_runtime_allocation(handle, RuntimeAllocationKind::Accessibility, vec![9_u8; 32])
            .expect("registered runtime allocation");
    let before = allocation_census(handle);

    assert_eq!(
        release_runtime_allocation(
            handle,
            zircon_runtime_interface::ZrRuntimeAllocationId::new(u64::MAX),
        )
        .status_code(),
        ZrStatusCode::NotFound
    );
    assert_eq!(allocation_census(handle), before);
    assert_eq!(
        release_runtime_allocation(handle, output.allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn runtime_allocation_release_rejects_a_foreign_session_without_changing_owner_census() {
    let owner = insert_session_with_wake(
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap(),
        RuntimeWakeRegistration::disabled(),
    );
    let foreign = insert_session_with_wake(
        RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap(),
        RuntimeWakeRegistration::disabled(),
    );
    let output =
        register_runtime_allocation(owner, RuntimeAllocationKind::WorldSync, vec![1_u8, 2, 3, 4])
            .expect("registered runtime allocation");
    let owner_census = allocation_census(owner);

    assert_eq!(
        release_runtime_allocation(foreign, output.allocation).status_code(),
        ZrStatusCode::NotFound
    );
    assert_eq!(allocation_census(owner), owner_census);
    assert_eq!(
        destroy_session_slot(foreign).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(
        release_runtime_allocation(owner, output.allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(destroy_session_slot(owner).status_code(), ZrStatusCode::Ok);
}

#[test]
fn concurrent_runtime_allocation_release_reclaims_exactly_once() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let output =
        register_runtime_allocation(handle, RuntimeAllocationKind::Profile, vec![7_u8; 64])
            .expect("registered runtime allocation");
    let allocation = output.allocation;
    let first = thread::spawn(move || release_runtime_allocation(handle, allocation).status_code());
    let second =
        thread::spawn(move || release_runtime_allocation(handle, allocation).status_code());
    let mut statuses = [first.join().unwrap(), second.join().unwrap()];
    statuses.sort_by_key(|status| status.as_raw());

    assert_eq!(statuses, [ZrStatusCode::Ok, ZrStatusCode::NotFound]);
    let census = allocation_census(handle);
    assert_eq!(census.outstanding_allocations, 0);
    assert_eq!(census.outstanding_bytes, 0);
    assert_eq!(census.high_water_allocations, 1);
    assert_eq!(census.high_water_bytes, 64);
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}

#[test]
fn outstanding_runtime_allocation_blocks_destroy_until_release_and_retry() {
    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let output =
        register_runtime_allocation(handle, RuntimeAllocationKind::Frame, vec![255_u8; 16])
            .expect("registered runtime allocation");

    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::Error
    );
    assert!(session_is_closing(handle));
    assert_eq!(allocation_census(handle).outstanding_allocations, 1);
    assert_eq!(
        release_runtime_allocation(handle, output.allocation).status_code(),
        ZrStatusCode::Ok
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
    assert_eq!(
        destroy_session_slot(handle).status_code(),
        ZrStatusCode::NotFound
    );
}

#[test]
fn runtime_allocation_registry_performance_acceptance() {
    const WARMUP_ITERATIONS: usize = 128;
    const MEASURED_ITERATIONS: usize = 2_000;
    const PAYLOAD_BYTES: usize = 4 * 1024;
    const P99_BUDGET: Duration = Duration::from_millis(1);
    const MIN_THROUGHPUT_PER_SECOND: f64 = 10_000.0;

    let session = RuntimeDynamicSession::new(RuntimeDynamicSessionProfile::Headless, None).unwrap();
    let handle = insert_session_with_wake(session, RuntimeWakeRegistration::disabled());
    let payload = vec![0x5a_u8; PAYLOAD_BYTES];

    for _ in 0..WARMUP_ITERATIONS {
        let output = register_runtime_allocation(
            handle,
            RuntimeAllocationKind::HostRequests,
            payload.clone(),
        )
        .expect("warmup runtime allocation");
        assert_eq!(
            release_runtime_allocation(handle, output.allocation).status_code(),
            ZrStatusCode::Ok
        );
    }

    let mut samples = Vec::with_capacity(MEASURED_ITERATIONS);
    for _ in 0..MEASURED_ITERATIONS {
        let started = Instant::now();
        let output = register_runtime_allocation(
            handle,
            RuntimeAllocationKind::HostRequests,
            payload.clone(),
        )
        .expect("measured runtime allocation");
        assert_eq!(
            release_runtime_allocation(handle, output.allocation).status_code(),
            ZrStatusCode::Ok
        );
        samples.push(started.elapsed().as_nanos());
    }
    samples.sort_unstable();

    let percentile = |percent: usize| {
        let rank = samples
            .len()
            .saturating_mul(percent)
            .div_ceil(100)
            .saturating_sub(1)
            .min(samples.len() - 1);
        samples[rank]
    };
    let p50_ns = percentile(50);
    let p95_ns = percentile(95);
    let p99_ns = percentile(99);
    let total_ns = samples.iter().copied().sum::<u128>();
    let throughput = MEASURED_ITERATIONS as f64 * 1_000_000_000.0 / total_ns.max(1) as f64;
    println!(
        "RUNTIME_INTERFACE01_ALLOCATION_REGISTRY_PERF iterations={MEASURED_ITERATIONS} payload_bytes={PAYLOAD_BYTES} p50_ns={p50_ns} p95_ns={p95_ns} p99_ns={p99_ns} throughput_cycles_per_second={throughput:.0}"
    );

    let census = allocation_census(handle);
    assert_eq!(census.outstanding_allocations, 0);
    assert_eq!(census.outstanding_bytes, 0);
    assert_eq!(census.high_water_allocations, 1);
    assert_eq!(census.high_water_bytes, PAYLOAD_BYTES as u64);
    assert!(
        p99_ns <= P99_BUDGET.as_nanos(),
        "allocation registry p99 {p99_ns}ns exceeded {}ns",
        P99_BUDGET.as_nanos()
    );
    assert!(
        throughput >= MIN_THROUGHPUT_PER_SECOND,
        "allocation registry throughput {throughput:.0}/s fell below {MIN_THROUGHPUT_PER_SECOND:.0}/s"
    );
    assert_eq!(destroy_session_slot(handle).status_code(), ZrStatusCode::Ok);
}
