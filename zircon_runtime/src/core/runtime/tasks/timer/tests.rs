use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

use super::super::callback_dispatcher::TaskCallbackDispatcher;
use super::TaskTimer;
use crate::core::runtime::tasks::TaskPool;
use crate::core::runtime::tasks::TaskPoolDescriptor;

#[test]
fn task_timer_rejects_zero_interval() {
    let timer = TaskTimer::process_default().expect("process timer should start");
    let error = timer
        .schedule_interval(Duration::ZERO, || {})
        .expect_err("zero interval must be rejected");
    assert!(error.to_string().contains("must be non-zero"));
}

#[test]
fn dropping_a_subscription_releases_its_bounded_timer_slot() {
    let timer = TaskTimer::new(1).expect("test timer should start");
    let subscription = timer
        .schedule_at(Instant::now() + Duration::from_secs(1), || {})
        .expect("first registration should fit the timer capacity");

    let error = timer
        .schedule_at(Instant::now() + Duration::from_secs(1), || {})
        .expect_err("second registration must observe the hard timer capacity");
    assert!(error.to_string().contains("registration capacity full"));

    drop(subscription);
    let replacement = timer
        .schedule_at(Instant::now() + Duration::from_secs(1), || {})
        .expect("dropping a subscription must release its timer slot");
    drop(replacement);
}

#[test]
fn dropping_the_last_explicit_timer_owner_stops_its_worker() {
    let timer = TaskTimer::new(1).expect("test timer should start");
    let inner = Arc::downgrade(&timer.inner);

    drop(timer);

    assert!(
        inner.upgrade().is_none(),
        "the explicit timer worker must not retain its state after its final owner drops"
    );
}

#[test]
fn dropped_subscription_stops_recurring_callbacks() {
    let timer = TaskTimer::process_default().expect("process timer should start");
    let (first_callback_tx, first_callback_rx) = mpsc::sync_channel(1);
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_for_task = Arc::clone(&callback_count);
    let subscription = timer
        .schedule_interval(Duration::from_millis(1), move || {
            callback_count_for_task.fetch_add(1, Ordering::AcqRel);
            let _ = first_callback_tx.try_send(());
        })
        .expect("timer should accept one recurring callback");

    first_callback_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("recurring callback should run");
    drop(subscription);
    std::thread::sleep(Duration::from_millis(10));
    let settled_count = callback_count.load(Ordering::Acquire);
    std::thread::sleep(Duration::from_millis(10));
    assert_eq!(callback_count.load(Ordering::Acquire), settled_count);
}

#[test]
fn panicking_callback_does_not_stop_later_timer_callbacks() {
    let timer = TaskTimer::process_default().expect("process timer should start");
    let panic_started = Arc::new(AtomicBool::new(false));
    let panic_started_for_task = Arc::clone(&panic_started);
    let panicking_subscription = timer
        .schedule_interval(Duration::from_millis(1), move || {
            if !panic_started_for_task.swap(true, Ordering::AcqRel) {
                panic!("timer callback panic");
            }
        })
        .expect("timer should accept a panicking callback");

    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while !panic_started.load(Ordering::Acquire) {
        assert!(
            std::time::Instant::now() < deadline,
            "panicking callback should run before the healthy subscription"
        );
        std::thread::sleep(Duration::from_millis(1));
    }

    let (healthy_callback_tx, healthy_callback_rx) = mpsc::sync_channel(1);
    let healthy_subscription = timer
        .schedule_interval(Duration::from_millis(1), move || {
            let _ = healthy_callback_tx.try_send(());
        })
        .expect("timer should accept a healthy callback after a panic");
    healthy_callback_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("timer should continue after a callback panic");
    drop(healthy_subscription);
    drop(panicking_subscription);
}

#[test]
fn due_timer_callbacks_do_not_run_on_the_control_thread() {
    let timer = TaskTimer::new_with_callback_dispatcher(
        2,
        TaskCallbackDispatcher::new(TaskPool::new(
            TaskPoolDescriptor::async_compute().with_worker_threads(2),
        )),
    )
    .expect("test timer should start");
    let (slow_started_tx, slow_started_rx) = mpsc::sync_channel(1);
    let release_slow = Arc::new(Barrier::new(2));
    let (healthy_tx, healthy_rx) = mpsc::sync_channel(1);
    let deadline = Instant::now() + Duration::from_millis(10);

    let release_slow_for_callback = Arc::clone(&release_slow);
    let slow_subscription = timer
        .schedule_at(deadline, move || {
            slow_started_tx.send(()).expect("slow callback started");
            release_slow_for_callback.wait();
        })
        .expect("slow callback should fit timer capacity");
    let healthy_subscription = timer
        .schedule_at(deadline, move || {
            healthy_tx.send(()).expect("healthy callback should run");
        })
        .expect("healthy callback should fit timer capacity");

    slow_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("slow callback should start");
    healthy_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("timer control thread should dispatch later callbacks");
    release_slow.wait();
    drop(healthy_subscription);
    drop(slow_subscription);
}

#[test]
fn slow_interval_callback_does_not_accumulate_dispatcher_backlog() {
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::async_compute().with_worker_threads(2),
    ));
    let timer = TaskTimer::new_with_callback_dispatcher(2, dispatcher.clone())
        .expect("test timer should start");
    let (started_tx, started_rx) = mpsc::sync_channel(1);
    let release = Arc::new(Barrier::new(2));
    let release_for_callback = Arc::clone(&release);
    let subscription = timer
        .schedule_interval(Duration::from_millis(1), move || {
            started_tx.send(()).expect("slow callback started");
            release_for_callback.wait();
        })
        .expect("interval callback should fit timer capacity");

    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("slow interval callback should start");
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(
        dispatcher.pending_callback_count(),
        1,
        "a slow interval callback must coalesce later ticks instead of queuing them"
    );

    drop(subscription);
    release.wait();
}

#[test]
fn dropping_timer_suppresses_already_queued_callback_delivery() {
    let dispatcher = TaskCallbackDispatcher::new(TaskPool::new(
        TaskPoolDescriptor::async_compute().with_worker_threads(1),
    ));
    let (blocker_started_tx, blocker_started_rx) = mpsc::sync_channel(1);
    let release_blocker = Arc::new(Barrier::new(2));
    let release_blocker_for_callback = Arc::clone(&release_blocker);
    dispatcher.dispatch_one(Box::new(move || {
        blocker_started_tx
            .send(())
            .expect("dispatcher blocker should start");
        release_blocker_for_callback.wait();
    }));
    blocker_started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("dispatcher blocker should occupy its only runner");

    let timer = TaskTimer::new_with_callback_dispatcher(1, dispatcher.clone())
        .expect("test timer should start");
    let callback_ran = Arc::new(AtomicBool::new(false));
    let callback_ran_for_timer = Arc::clone(&callback_ran);
    let subscription = timer
        .schedule_at(Instant::now() + Duration::from_millis(1), move || {
            callback_ran_for_timer.store(true, Ordering::Release);
        })
        .expect("timer callback should fit the test capacity");

    let deadline = Instant::now() + Duration::from_secs(1);
    while dispatcher.pending_callback_count() < 2 {
        assert!(
            Instant::now() < deadline,
            "timer callback should be queued behind the occupied dispatcher runner"
        );
        std::thread::yield_now();
    }

    drop(timer);
    release_blocker.wait();
    std::thread::sleep(Duration::from_millis(20));
    assert!(
        !callback_ran.load(Ordering::Acquire),
        "dropping a timer must suppress its already queued lifecycle callback"
    );
    drop(subscription);
}
