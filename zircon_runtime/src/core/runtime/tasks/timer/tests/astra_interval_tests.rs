use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn astra_m26_first_deadline_overflow_is_typed_and_releases_capacity() {
    let timer = TaskTimer::new(1).expect("test timer should start");
    let error = timer
        .schedule_interval(Duration::MAX, || {})
        .expect_err("unrepresentable deadline must fail admission");
    assert_eq!(error, crate::core::CoreError::DeadlineOutOfRange);
    let subscription = timer
        .schedule_at(Instant::now() + Duration::from_secs(1), || {})
        .expect("overflow rejection must not retain a slot");
    drop(subscription);
}

#[test]
fn astra_m26_recurring_overflow_retires_without_immediate_repeat() {
    let timer = TaskTimer::new_with_callback_dispatcher(1, TaskCallbackDispatcher::inline())
        .expect("timer should start");
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_for_timer = Arc::clone(&callback_count);
    let subscription = timer
        .schedule(
            TimerSchedule::Interval(Duration::MAX),
            Instant::now(),
            move || {
                callback_count_for_timer.fetch_add(1, Ordering::SeqCst);
            },
        )
        .expect("the due overflow probe should be admitted");
    let deadline = Instant::now() + Duration::from_secs(1);
    while !subscription.registration.cancelled.load(Ordering::Acquire) {
        assert!(
            Instant::now() < deadline,
            "the timer worker should retire the overflowing interval"
        );
        std::thread::yield_now();
    }
    assert_eq!(callback_count.load(Ordering::SeqCst), 0);
    let replacement = timer
        .schedule_at(Instant::now() + Duration::from_secs(1), || {})
        .expect("recurring overflow must release its timer slot while its subscription is alive");
    drop(replacement);
    assert!(
        timer.shutdown_until(Instant::now() + Duration::from_secs(1)),
        "the timer worker should finish before checking the final callback count"
    );
    assert_eq!(callback_count.load(Ordering::SeqCst), 0);
    drop(subscription);
}
