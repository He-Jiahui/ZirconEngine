use std::hint::black_box;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use super::*;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::jobs::{
    EditorJobLimits, EditorJobProgressObserver, EditorJobProgressSource, EditorJobSystem,
};
use zircon_runtime::core::runtime::tasks::{EngineTaskGraph, EngineTaskGraphOptions, JobScheduler};

#[derive(Default)]
struct Observer {
    events: Mutex<Vec<(u8, u64)>>,
    reentrant: Mutex<Weak<EditorJobSystemInner>>,
    panic_on_admit: AtomicBool,
    panic_on_resync: AtomicBool,
}

impl EditorJobProgressObserver for Observer {
    fn job_admitted(&self, id: JobId, _: &EditorJobProgressSource) {
        assert!(
            !self.panic_on_admit.swap(false, Ordering::Relaxed),
            "admit failure"
        );
        self.events.lock().unwrap().push((0, id.value()));
        if let Some(inner) = self.reentrant.lock().unwrap().upgrade() {
            inner.enqueue_progress_observer_event(ProgressObserverEvent::Finished(id));
            inner.deliver_progress_observer_events();
        }
    }

    fn job_finished(&self, id: JobId, _: &EditorJobProgressSource) {
        self.events.lock().unwrap().push((1, id.value()));
    }

    fn jobs_resynchronized(&self, _: &EditorJobProgressSource) {
        assert!(
            !self.panic_on_resync.swap(false, Ordering::Relaxed),
            "resync failure"
        );
        self.events.lock().unwrap().push((2, 0));
    }
}

fn fixture(observer: Arc<Observer>) -> (EngineTaskGraph, EditorJobSystem) {
    let graph = EngineTaskGraph::try_new(EngineTaskGraphOptions::with_worker_threads(1)).unwrap();
    let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
        JobScheduler::from_pool(graph.worker_pool().clone()),
        SharedEditorMessageBus::default(),
        EditorJobLimits::default(),
        observer,
    );
    (graph, jobs)
}

fn buffer_capacities(dispatch: &ProgressObserverDispatch) -> [usize; 2] {
    let mut buffers = [dispatch.events.capacity(), dispatch.spare_events.capacity()];
    buffers.sort_unstable();
    buffers
}

#[test]
fn astra_m1_progress_delivery_reuses_buffers_and_preserves_reentrant_order() {
    for count in [1, 1_000] {
        let observer = Arc::new(Observer::default());
        let (graph, jobs) = fixture(Arc::clone(&observer));
        *observer.reentrant.lock().unwrap() = Arc::downgrade(&jobs.inner);
        let mut expected_buffers = None;
        for _ in 0..16 {
            for id in 0..count {
                jobs.inner
                    .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(JobId::new(
                        id,
                    )));
            }
            jobs.inner.deliver_progress_observer_events();
            let observed = std::mem::take(&mut *observer.events.lock().unwrap());
            let expected = (0..count)
                .map(|id| (0, id))
                .chain((0..count).map(|id| (1, id)))
                .collect::<Vec<_>>();
            assert_eq!(observed, expected);
            let dispatch = jobs.inner.progress_observer_dispatch.lock().unwrap();
            assert!(!dispatch.delivering);
            assert!(dispatch.events.is_empty());
            assert!(dispatch.spare_events.is_empty());
            let buffers = buffer_capacities(&dispatch);
            if let Some(expected) = expected_buffers {
                assert_eq!(buffers, expected, "warm delivery must retain both buffers");
            }
            expected_buffers = Some(buffers);
        }
        graph.shutdown(Duration::from_secs(2)).unwrap();
    }
}

#[test]
fn astra_m1_progress_overflow_and_panic_recovery_keep_dispatch_usable() {
    let observer = Arc::new(Observer::default());
    let (graph, jobs) = fixture(Arc::clone(&observer));
    for id in 0..10_000 {
        jobs.inner
            .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(JobId::new(id)));
    }
    assert_eq!(
        jobs.inner
            .progress_observer_dispatch
            .lock()
            .unwrap()
            .events
            .len(),
        1
    );
    jobs.inner.deliver_progress_observer_events();
    assert_eq!(*observer.events.lock().unwrap(), [(2, 0)]);
    observer.events.lock().unwrap().clear();
    observer.panic_on_admit.store(true, Ordering::Relaxed);
    observer.panic_on_resync.store(true, Ordering::Relaxed);
    jobs.inner
        .enqueue_progress_observer_event(ProgressObserverEvent::Admitted(JobId::new(10_000)));
    jobs.inner.deliver_progress_observer_events();
    {
        let dispatch = jobs.inner.progress_observer_dispatch.lock().unwrap();
        assert!(!dispatch.delivering);
        assert!(dispatch.resynchronize_queued);
        assert_eq!(dispatch.events.len(), 1);
        assert!(dispatch.spare_events.is_empty());
    }
    jobs.inner.deliver_progress_observer_events();
    jobs.inner
        .enqueue_progress_observer_event(ProgressObserverEvent::Finished(JobId::new(10_000)));
    jobs.inner.deliver_progress_observer_events();
    assert_eq!(*observer.events.lock().unwrap(), [(2, 0), (1, 10_000)]);
    graph.shutdown(Duration::from_secs(2)).unwrap();
}

fn transfer_workload(
    dispatch: &mut ProgressObserverDispatch,
    count: usize,
    batches: usize,
    reuse: bool,
) -> u64 {
    let mut total = 0;
    for _ in 0..batches {
        for id in 0..count {
            dispatch.push(ProgressObserverEvent::Admitted(JobId::new(id as u64)));
        }
        let mut events = if reuse {
            dispatch.take_all()
        } else {
            std::mem::take(&mut dispatch.events)
        };
        while let Some(event) = events.pop_front() {
            if let ProgressObserverEvent::Admitted(id) = event {
                total += black_box(id.value());
            }
        }
        if reuse {
            dispatch.recycle(events);
            let empty = dispatch.take_all();
            assert!(empty.is_empty());
            dispatch.recycle(empty);
        }
    }
    total
}

fn percentiles(samples: &mut [u128]) -> [u128; 3] {
    samples.sort_unstable();
    [50, 95, 99].map(|p| samples[(samples.len() * p).div_ceil(100) - 1])
}

#[test]
#[ignore = "Windows release evidence through the coordinator"]
fn astra_m1_progress_buffer_release_evidence() {
    assert!(
        !cfg!(debug_assertions),
        "performance evidence requires release"
    );
    const SAMPLES: usize = 101;
    const BATCHES: usize = 2_000;
    for count in [1, 1_000] {
        let mut before_dispatch = ProgressObserverDispatch::default();
        let mut after_dispatch = ProgressObserverDispatch::default();
        let expected = transfer_workload(&mut before_dispatch, count, BATCHES, false);
        assert_eq!(
            transfer_workload(&mut after_dispatch, count, BATCHES, true),
            expected
        );
        for _ in 0..8 {
            black_box(transfer_workload(
                &mut before_dispatch,
                count,
                BATCHES,
                false,
            ));
            black_box(transfer_workload(&mut after_dispatch, count, BATCHES, true));
        }
        let mut before = Vec::with_capacity(SAMPLES);
        let mut after = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            for reuse in if sample % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let started = Instant::now();
                assert_eq!(
                    black_box(transfer_workload(
                        if reuse {
                            &mut after_dispatch
                        } else {
                            &mut before_dispatch
                        },
                        black_box(count),
                        BATCHES,
                        reuse
                    )),
                    expected
                );
                let elapsed = started.elapsed().as_nanos();
                if reuse {
                    after.push(elapsed);
                } else {
                    before.push(elapsed);
                }
            }
        }
        let before = percentiles(&mut before);
        let after = percentiles(&mut after);
        println!(
            "ASTRA_M1_PROGRESS_BUFFER profile=release scope=queue_transfer events={count} batches={BATCHES} samples={SAMPLES} warmup=8 before_p50_p95_p99_ns={before:?} after_p50_p95_p99_ns={after:?} p95_limit_percent=105"
        );
        assert!(
            after[1] * 100 <= before[1] * 105,
            "queue transfer p95 regression: {before:?} -> {after:?}"
        );
    }
}
