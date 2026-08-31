use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::Duration;

use super::{TaskCallbackDispatcher, TaskPool, MAX_CALLBACKS_PER_ENVELOPE, MAX_CALLBACKS_PER_RUN};
use crate::core::runtime::tasks::TaskPoolDescriptor;

#[test]
fn dispatcher_rotates_large_envelopes_before_later_work() {
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let (sender, receiver) = mpsc::channel();
    let (first_callback_started_tx, first_callback_started_rx) = mpsc::sync_channel(1);
    let release_first_callback = Arc::new(Barrier::new(2));
    let mut first_envelope = Vec::new();
    for index in 0..(MAX_CALLBACKS_PER_ENVELOPE * 2) {
        let sender = sender.clone();
        let first_callback_started_tx = first_callback_started_tx.clone();
        let release_first_callback = Arc::clone(&release_first_callback);
        first_envelope.push(Box::new(move || {
            if index == 0 {
                first_callback_started_tx
                    .send(())
                    .expect("first callback should start");
                release_first_callback.wait();
            }
            sender.send(index).expect("first callback result");
        }) as _);
    }
    dispatcher.dispatch(first_envelope, None);
    first_callback_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first envelope should begin before later work is admitted");

    let sender = sender.clone();
    dispatcher.dispatch_one(Box::new(move || {
        sender.send(usize::MAX).expect("later callback result");
    }));
    release_first_callback.wait();

    let delivered = (0..=(MAX_CALLBACKS_PER_ENVELOPE * 2))
        .map(|_| {
            receiver
                .recv_timeout(Duration::from_secs(1))
                .expect("all callbacks should finish")
        })
        .collect::<Vec<_>>();
    let later_position = delivered
        .iter()
        .position(|value| *value == usize::MAX)
        .expect("later envelope should deliver");
    assert!(
        later_position <= MAX_CALLBACKS_PER_ENVELOPE,
        "large envelopes must yield before consuming all delivery turns"
    );
}

#[test]
fn dispatcher_contains_callback_panics_and_runs_later_callbacks() {
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let (sender, receiver) = mpsc::sync_channel(1);
    dispatcher.dispatch(
        vec![
            Box::new(|| panic!("callback failure")),
            Box::new(move || sender.send(()).expect("healthy callback result")),
        ],
        None,
    );

    receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("panic must not prevent later callback delivery");
}

#[test]
fn dispatcher_enforces_the_delivery_run_budget() {
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let (sender, receiver) = mpsc::channel();
    let callback_count = MAX_CALLBACKS_PER_RUN + 1;
    let callbacks = (0..callback_count)
        .map(|index| {
            let sender = sender.clone();
            Box::new(move || sender.send(index).expect("callback result")) as _
        })
        .collect();
    dispatcher.dispatch(callbacks, None);

    for _ in 0..callback_count {
        receiver
            .recv_timeout(Duration::from_secs(1))
            .expect("every callback should be delivered");
    }
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    let metrics = loop {
        let metrics = dispatcher.metrics_snapshot();
        if metrics.delivered_callbacks == callback_count {
            break metrics;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "dispatcher must publish the completed run metrics"
        );
        std::thread::yield_now();
    };

    assert_eq!(metrics.delivery_runs, 2);
    assert_eq!(metrics.max_callbacks_per_run, MAX_CALLBACKS_PER_RUN);
}

#[test]
fn dispatcher_drains_ten_thousand_callback_fanout_with_bounded_runs() {
    const CALLBACK_COUNT: usize = 10_000;
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::compute().with_worker_threads(1),
    ));
    let delivered = Arc::new(AtomicUsize::new(0));
    let callbacks = (0..CALLBACK_COUNT)
        .map(|_| {
            let delivered = Arc::clone(&delivered);
            Box::new(move || {
                delivered.fetch_add(1, Ordering::Relaxed);
            }) as _
        })
        .collect();
    let (completion_tx, completion_rx) = mpsc::sync_channel(1);
    dispatcher.dispatch(
        callbacks,
        Some(Box::new(move || {
            completion_tx.send(()).expect("fan-out completion result");
        })),
    );

    completion_rx
        .recv_timeout(Duration::from_secs(5))
        .expect("dispatcher should drain the complete fan-out");
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    let metrics = loop {
        let metrics = dispatcher.metrics_snapshot();
        if metrics.delivered_callbacks == CALLBACK_COUNT + 1 {
            break metrics;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "dispatcher must publish all fan-out metrics"
        );
        std::thread::yield_now();
    };

    assert_eq!(delivered.load(Ordering::Acquire), CALLBACK_COUNT);
    assert_eq!(
        metrics.delivery_runs,
        CALLBACK_COUNT.div_ceil(MAX_CALLBACKS_PER_RUN)
    );
    assert_eq!(metrics.max_callbacks_per_run, MAX_CALLBACKS_PER_RUN);
}

#[test]
fn process_default_reuses_one_dispatcher_state() {
    let first = TaskCallbackDispatcher::process_default();
    let second = TaskCallbackDispatcher::process_default();

    assert!(
        Arc::ptr_eq(&first.inner, &second.inner),
        "default task callbacks must share the process dispatch budget"
    );
}
