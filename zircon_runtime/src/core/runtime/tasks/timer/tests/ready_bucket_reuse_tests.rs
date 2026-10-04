use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::{
    lock_timer_state, next_callbacks, TaskCallbackDispatcher, TaskTimer, TaskTimerInner,
    TaskTimerState, TimerDeliveryPending, TimerRegistration, TimerSchedule,
};

const PROFILE_MARKER: &str = "RUNTIME158_TIMER_READY_BUCKET_REUSE_BENCH_V1";
const WARMUP_PAIRS: usize = 5;
const SAMPLE_PAIRS: usize = 31;
const BUCKETS_PER_SAMPLE: usize = 64;
const MAX_NEW_P95_PERCENT: u128 = 110;
const INTERVAL: Duration = Duration::from_secs(1);
const FUTURE_OFFSET: Duration = Duration::from_secs(3600);

#[derive(Clone, Copy, Debug)]
enum Workload {
    Once,
    MixedCancellation,
    CoalescedIntervals,
}

#[test]
fn runtime158_ready_once_bucket_reuses_storage_and_preserves_registration_order() {
    let (current, legacy, new, old) = run_comparison(8, Workload::Once);
    assert_eq!(ids(&new.callbacks[0]), (0..8).collect::<Vec<_>>());
    assert_reused_buffer(&current[0], &new.callbacks[0]);
    assert_ne!(legacy[0].buffer, old.callbacks[0].as_ptr());
    assert!(old.callbacks[0].capacity() >= 8);
}

#[test]
fn runtime158_mixed_due_bucket_preserves_cancellation_renewal_and_pending_coalescing() {
    let (current, _legacy, new, _old) = run_comparison(8, Workload::MixedCancellation);
    assert_eq!(ids(&new.callbacks[0]), vec![1, 3, 5, 7]);
    assert_reused_buffer(&current[0], &new.callbacks[0]);
    let state = lock_timer_state(&current[0].timer);
    let mut scheduled = state
        .scheduled_deadlines
        .keys()
        .copied()
        .collect::<Vec<_>>();
    scheduled.sort_unstable();
    assert_eq!(scheduled, vec![2, 3, 6, 7, 8]);
    assert!(current[0].registrations[0]
        .cancelled
        .load(Ordering::Acquire));
    assert!(current[0].registrations[2]
        .delivery_pending
        .load(Ordering::Acquire));
}

#[test]
fn runtime158_coalesced_interval_bucket_keeps_storage_and_can_resume_delivery() {
    let (current, _legacy, new, _old) = run_comparison(8, Workload::CoalescedIntervals);
    assert!(new.callbacks[0].is_empty());
    assert_reused_buffer(&current[0], &new.callbacks[0]);
    let fixture = &current[0];
    for registration in &fixture.registrations {
        drop(TimerDeliveryPending::new(Arc::clone(registration)));
    }
    // Advance this test fixture's renewed bucket to an already-due deadline without sleeping.
    // Dispatch/coalescing still runs through the production function on its real state.
    let due = Instant::now() - INTERVAL;
    {
        let mut state = lock_timer_state(&fixture.timer);
        let renewed = *state.scheduled_deadlines.get(&0).unwrap();
        let registrations = state.deadlines.remove(&renewed).unwrap();
        for registration in &registrations {
            state.scheduled_deadlines.insert(registration.id, due);
        }
        state.deadlines.insert(due, registrations);
    }
    let callbacks = next_callbacks(&fixture.timer).unwrap();
    assert_eq!(ids(&callbacks), (0..8).collect::<Vec<_>>());
    assert!(callbacks
        .iter()
        .all(|registration| registration.delivery_pending.load(Ordering::Acquire)));
}

#[test]
fn runtime158_closed_timer_leaves_pending_buckets_and_registration_flags_unchanged() {
    let anchor = Instant::now();
    let current = Fixture::new(8, Workload::MixedCancellation, anchor);
    let legacy = Fixture::new(8, Workload::MixedCancellation, anchor);
    current.timer.closing.store(true, Ordering::Release);
    legacy.timer.closing.store(true, Ordering::Release);
    let current_before = state_shape(&current);
    let legacy_before = state_shape(&legacy);
    assert!(next_callbacks(&current.timer).is_none());
    assert!(legacy_next_callbacks(&legacy.timer).is_none());
    assert_eq!(state_shape(&current), current_before);
    assert_eq!(state_shape(&legacy), legacy_before);
    assert_eq!(current_before, legacy_before);
}

#[test]
fn runtime158_real_timer_worker_delivers_a_shared_deadline_in_order_after_cancellation() {
    let timer =
        TaskTimer::new_with_callback_dispatcher(4, TaskCallbackDispatcher::inline()).unwrap();
    let (send, receive) = mpsc::channel();
    let deadline = Instant::now() + FUTURE_OFFSET;
    let mut subscriptions = (0..4)
        .map(|id| {
            let send = send.clone();
            timer
                .schedule_at(deadline, move || send.send(id).unwrap())
                .unwrap()
        })
        .collect::<Vec<_>>();
    drop(subscriptions.remove(1));
    // Make the already-admitted bucket due only after production cancellation completed.
    // The real worker handles selection and delivery; the test needs no scheduling sleep.
    {
        let mut state = lock_timer_state(&timer.inner);
        let registrations = state.deadlines.remove(&deadline).unwrap();
        let due = Instant::now();
        for registration in &registrations {
            state.scheduled_deadlines.insert(registration.id, due);
        }
        state.deadlines.insert(due, registrations);
    }
    timer.inner.changed.notify_one();
    for expected in [0, 2, 3] {
        assert_eq!(
            receive.recv_timeout(Duration::from_secs(2)).unwrap(),
            expected
        );
    }
    assert!(receive.try_recv().is_err());
    drop(subscriptions);
    drop(timer);
}

#[test]
#[ignore = "managed Windows Release profile of actual ready-bucket processing"]
fn runtime158_timer_ready_bucket_reuse_release_profile() {
    assert!(!cfg!(debug_assertions), "run this profile with --release");
    eprintln!(
        "{PROFILE_MARKER} os={} arch={} crate={} processor={:?} profile=release warmup_pairs={WARMUP_PAIRS} sample_pairs={SAMPLE_PAIRS} buckets_per_sample={BUCKETS_PER_SAMPLE} max_new_p95_percent={MAX_NEW_P95_PERCENT} guard=no_regression_with_noise_tolerance",
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
        std::env::var("PROCESSOR_IDENTIFIER").ok(),
    );
    for count in [1, 8, 64, 512] {
        for workload in [
            Workload::Once,
            Workload::MixedCancellation,
            Workload::CoalescedIntervals,
        ] {
            let mut old_ns = Vec::with_capacity(SAMPLE_PAIRS);
            let mut new_ns = Vec::with_capacity(SAMPLE_PAIRS);
            for round in 0..WARMUP_PAIRS + SAMPLE_PAIRS {
                let anchor = Instant::now();
                let current = fixtures(count, workload, anchor, BUCKETS_PER_SAMPLE);
                let legacy = fixtures(count, workload, anchor, BUCKETS_PER_SAMPLE);
                let (old, new) = if round % 2 == 0 {
                    (
                        measure(&legacy, legacy_next_callbacks),
                        measure(&current, next_callbacks),
                    )
                } else {
                    let new = measure(&current, next_callbacks);
                    let old = measure(&legacy, legacy_next_callbacks);
                    (old, new)
                };
                verify_pair(&current, &legacy, &new, &old);
                if round >= WARMUP_PAIRS {
                    old_ns.push(old.elapsed_ns);
                    new_ns.push(new.elapsed_ns);
                }
                // The output buffers, registrations, fixtures and dispatcher are dropped
                // after both clocks stopped and all semantic/storage checks completed.
            }
            let old_p50_ns = percentile(&old_ns, 50);
            let old_p95_ns = percentile(&old_ns, 95);
            let old_p99_ns = percentile(&old_ns, 99);
            let new_p50_ns = percentile(&new_ns, 50);
            let new_p95_ns = percentile(&new_ns, 95);
            let new_p99_ns = percentile(&new_ns, 99);
            let old_buckets_per_second =
                BUCKETS_PER_SAMPLE as u128 * 1_000_000_000 / old_p50_ns.max(1);
            let new_buckets_per_second =
                BUCKETS_PER_SAMPLE as u128 * 1_000_000_000 / new_p50_ns.max(1);
            eprintln!(
                "{PROFILE_MARKER} registrations_per_bucket={count} workload={workload:?} pair_order=alternating owned_buffer_reused=true legacy_second_buffer=true old_p50_ns={old_p50_ns} old_p95_ns={old_p95_ns} old_p99_ns={old_p99_ns} new_p50_ns={new_p50_ns} new_p95_ns={new_p95_ns} new_p99_ns={new_p99_ns} old_buckets_per_second={old_buckets_per_second} new_buckets_per_second={new_buckets_per_second} old_ns={old_ns:?} new_ns={new_ns:?}"
            );
            assert!(old_p95_ns > 0);
            assert!(
                new_p95_ns.saturating_mul(100) <= old_p95_ns.saturating_mul(MAX_NEW_P95_PERCENT),
                "{workload:?}/{count}: new P95 {new_p95_ns} ns exceeds old {old_p95_ns} ns plus the 10% noise tolerance"
            );
        }
    }
}

type CallbackBatch = Vec<Arc<TimerRegistration>>;
type NextCallbacks = fn(&TaskTimerInner) -> Option<CallbackBatch>;

struct Measurement {
    started: Instant,
    finished: Instant,
    elapsed_ns: u128,
    callbacks: Vec<CallbackBatch>,
}

fn measure(fixtures: &[Fixture], operation: NextCallbacks) -> Measurement {
    let operation = black_box(operation);
    let fixtures = black_box(fixtures);
    let mut callbacks = Vec::with_capacity(fixtures.len());
    let started = Instant::now();
    for fixture in fixtures {
        callbacks.push(operation(&fixture.timer).expect("fixture has an already-due bucket"));
    }
    let finished = Instant::now();
    Measurement {
        started,
        finished,
        elapsed_ns: finished.duration_since(started).as_nanos(),
        callbacks,
    }
}

fn run_comparison(
    count: usize,
    workload: Workload,
) -> (Vec<Fixture>, Vec<Fixture>, Measurement, Measurement) {
    let anchor = Instant::now();
    let current = fixtures(count, workload, anchor, 1);
    let legacy = fixtures(count, workload, anchor, 1);
    let new = measure(&current, next_callbacks);
    let old = measure(&legacy, legacy_next_callbacks);
    verify_pair(&current, &legacy, &new, &old);
    (current, legacy, new, old)
}

fn verify_pair(current: &[Fixture], legacy: &[Fixture], new: &Measurement, old: &Measurement) {
    assert_eq!(current.len(), legacy.len());
    for i in 0..current.len() {
        verify_result(&current[i], &new.callbacks[i], new.started, new.finished);
        verify_result(&legacy[i], &old.callbacks[i], old.started, old.finished);
        assert_eq!(ids(&new.callbacks[i]), ids(&old.callbacks[i]));
        assert_eq!(state_shape(&current[i]), state_shape(&legacy[i]));
        assert_reused_buffer(&current[i], &new.callbacks[i]);
        assert_ne!(legacy[i].buffer, old.callbacks[i].as_ptr());
        assert!(old.callbacks[i].capacity() >= legacy[i].registrations.len());
    }
}

fn verify_result(
    fixture: &Fixture,
    callbacks: &[Arc<TimerRegistration>],
    before: Instant,
    after: Instant,
) {
    let state = lock_timer_state(&fixture.timer);
    assert!(!state.deadlines.contains_key(&fixture.due));
    assert_eq!(
        state.scheduled_deadlines.get(&fixture.future.id),
        Some(&fixture.future_deadline)
    );
    assert_eq!(
        ids(state.deadlines.get(&fixture.future_deadline).unwrap()),
        vec![fixture.future.id]
    );
    let mut scheduled = 1;
    let mut expected_callbacks = Vec::new();
    for (i, registration) in fixture.registrations.iter().enumerate() {
        let (schedule, cancelled, pending) = specification(fixture.workload, i);
        if !cancelled && !pending {
            expected_callbacks.push(registration.id);
        }
        assert_eq!(registration.cancelled.load(Ordering::Acquire), cancelled);
        assert_eq!(
            registration.delivery_pending.load(Ordering::Acquire),
            !cancelled || pending
        );
        if !cancelled && matches!(schedule, TimerSchedule::Interval(_)) {
            scheduled += 1;
            let deadline = *state.scheduled_deadlines.get(&registration.id).unwrap();
            assert!(deadline >= before.checked_add(INTERVAL).unwrap());
            assert!(deadline <= after.checked_add(INTERVAL).unwrap());
            let bucket = state.deadlines.get(&deadline).unwrap();
            assert_eq!(
                bucket
                    .iter()
                    .filter(|candidate| candidate.id == registration.id)
                    .count(),
                1
            );
            assert!(bucket
                .iter()
                .any(|candidate| Arc::ptr_eq(candidate, registration)));
        } else {
            assert!(!state.scheduled_deadlines.contains_key(&registration.id));
        }
    }
    assert_eq!(state.scheduled_deadlines.len(), scheduled);
    assert_eq!(
        state.deadlines.values().map(Vec::len).sum::<usize>(),
        scheduled
    );
    assert_eq!(ids(callbacks), expected_callbacks);
    for registration in callbacks {
        assert!(Arc::ptr_eq(
            registration,
            &fixture.registrations[registration.id as usize]
        ));
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DeadlineKind {
    OriginalDue,
    Renewed,
    PreservedFuture,
}

#[derive(Debug, PartialEq, Eq)]
struct StateShape {
    buckets: Vec<(DeadlineKind, Vec<u64>)>,
    scheduled: Vec<(u64, DeadlineKind)>,
    flags: Vec<(u64, bool, bool)>,
    next_id: u64,
    capacity: usize,
}

fn state_shape(fixture: &Fixture) -> StateShape {
    let state = lock_timer_state(&fixture.timer);
    let classify = |deadline| {
        if deadline == fixture.due {
            DeadlineKind::OriginalDue
        } else if deadline == fixture.future_deadline {
            DeadlineKind::PreservedFuture
        } else {
            DeadlineKind::Renewed
        }
    };
    let buckets = state
        .deadlines
        .iter()
        .map(|(deadline, bucket)| (classify(*deadline), ids(bucket)))
        .collect();
    let mut scheduled = state
        .scheduled_deadlines
        .iter()
        .map(|(id, deadline)| (*id, classify(*deadline)))
        .collect::<Vec<_>>();
    scheduled.sort_unstable();
    let flags = fixture
        .registrations
        .iter()
        .chain(std::iter::once(&fixture.future))
        .map(|registration| {
            (
                registration.id,
                registration.cancelled.load(Ordering::Acquire),
                registration.delivery_pending.load(Ordering::Acquire),
            )
        })
        .collect();
    StateShape {
        buckets,
        scheduled,
        flags,
        next_id: state.next_id,
        capacity: state.capacity,
    }
}

struct Fixture {
    timer: TaskTimerInner,
    registrations: Vec<Arc<TimerRegistration>>,
    future: Arc<TimerRegistration>,
    due: Instant,
    future_deadline: Instant,
    buffer: *const Arc<TimerRegistration>,
    capacity: usize,
    workload: Workload,
}

impl Fixture {
    fn new(count: usize, workload: Workload, anchor: Instant) -> Self {
        let due = anchor - INTERVAL;
        let future_deadline = anchor + FUTURE_OFFSET;
        let registrations = (0..count)
            .map(|i| {
                let (schedule, cancelled, pending) = specification(workload, i);
                Arc::new(TimerRegistration {
                    id: i as u64,
                    schedule,
                    cancelled: AtomicBool::new(cancelled),
                    delivery_pending: AtomicBool::new(pending),
                    callback: Box::new(|| {}),
                })
            })
            .collect::<Vec<_>>();
        // Spare capacity proves reuse rather than coincidental output sizing.
        let mut bucket = Vec::with_capacity(count + 7);
        bucket.extend(registrations.iter().cloned());
        let buffer = bucket.as_ptr();
        let capacity = bucket.capacity();
        let future = Arc::new(TimerRegistration {
            id: count as u64,
            schedule: TimerSchedule::Once,
            cancelled: AtomicBool::new(false),
            delivery_pending: AtomicBool::new(false),
            callback: Box::new(|| {}),
        });
        let mut scheduled_deadlines = registrations
            .iter()
            .map(|registration| (registration.id, due))
            .collect::<HashMap<_, _>>();
        scheduled_deadlines.insert(future.id, future_deadline);
        let deadlines =
            BTreeMap::from([(due, bucket), (future_deadline, vec![Arc::clone(&future)])]);
        Self {
            timer: TaskTimerInner {
                callback_dispatcher: TaskCallbackDispatcher::inline(),
                state: Mutex::new(TaskTimerState {
                    next_id: count as u64 + 1,
                    capacity: count + 1,
                    deadlines,
                    scheduled_deadlines,
                }),
                changed: Condvar::new(),
                owners: AtomicUsize::new(1),
                closing: AtomicBool::new(false),
            },
            registrations,
            future,
            due,
            future_deadline,
            buffer,
            capacity,
            workload,
        }
    }
}

fn specification(workload: Workload, index: usize) -> (TimerSchedule, bool, bool) {
    match workload {
        Workload::Once => (TimerSchedule::Once, false, false),
        Workload::CoalescedIntervals => (TimerSchedule::Interval(INTERVAL), false, true),
        Workload::MixedCancellation => match index % 4 {
            0 => (TimerSchedule::Once, true, false),
            1 => (TimerSchedule::Once, false, false),
            2 => (TimerSchedule::Interval(INTERVAL), false, true),
            _ => (TimerSchedule::Interval(INTERVAL), false, false),
        },
    }
}

fn fixtures(count: usize, workload: Workload, anchor: Instant, copies: usize) -> Vec<Fixture> {
    (0..copies)
        .map(|_| Fixture::new(count, workload, anchor))
        .collect()
}

fn ids(registrations: &[Arc<TimerRegistration>]) -> Vec<u64> {
    registrations
        .iter()
        .map(|registration| registration.id)
        .collect()
}

fn assert_reused_buffer(fixture: &Fixture, callbacks: &CallbackBatch) {
    assert_eq!(callbacks.as_ptr(), fixture.buffer);
    assert_eq!(callbacks.capacity(), fixture.capacity);
}

fn percentile(samples: &[u128], percent: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percent).div_ceil(100).saturating_sub(1)]
}

// Full pre-change next_callbacks is appended below; only its function name changes.

fn legacy_next_callbacks(timer: &TaskTimerInner) -> Option<Vec<Arc<TimerRegistration>>> {
    let mut state = lock_timer_state(timer);
    loop {
        if timer.closing.load(Ordering::Acquire) {
            return None;
        }
        let Some((&deadline, _)) = state.deadlines.first_key_value() else {
            state = timer
                .changed
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            continue;
        };
        let now = Instant::now();
        if now < deadline {
            let wait = deadline.saturating_duration_since(now);
            let (next_state, _) = timer
                .changed
                .wait_timeout(state, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next_state;
            continue;
        }

        let registrations = state
            .deadlines
            .remove(&deadline)
            .expect("timer deadline exists while selected");
        let next_deadline = Instant::now();
        let mut callbacks = Vec::with_capacity(registrations.len());
        for registration in registrations {
            if state.scheduled_deadlines.remove(&registration.id).is_none()
                || registration.cancelled.load(Ordering::Acquire)
            {
                continue;
            }
            if let TimerSchedule::Interval(interval) = registration.schedule {
                let Some(deadline) = next_deadline.checked_add(interval) else {
                    registration.cancelled.store(true, Ordering::Release);
                    continue;
                };
                state
                    .deadlines
                    .entry(deadline)
                    .or_default()
                    .push(Arc::clone(&registration));
                state.scheduled_deadlines.insert(registration.id, deadline);
            }
            // A slow periodic delivery coalesces later ticks instead of building a callback backlog.
            if !registration.delivery_pending.swap(true, Ordering::AcqRel) {
                callbacks.push(registration);
            }
        }
        return Some(callbacks);
    }
}
