use std::sync::mpsc;
use std::time::Duration;

use super::{BackgroundTask, BackgroundTaskPoll, BackgroundTaskShutdown};

#[test]
fn completed_background_load_wakes_and_returns_value() {
    let (wake_sender, wake_receiver) = mpsc::channel();
    let mut task = BackgroundTask::spawn(
        "viewer-background-load-success-test",
        |_| Ok(42_u32),
        move || {
            let _ = wake_sender.send(());
        },
    )
    .expect("background task should start");

    wake_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("background task should wake the event loop");
    match task.try_take() {
        BackgroundTaskPoll::Completed(Ok(value)) => assert_eq!(value, 42),
        _ => panic!("background task did not return its completed value"),
    }
}

#[test]
fn panicking_background_load_wakes_and_returns_error() {
    let (wake_sender, wake_receiver) = mpsc::channel();
    let mut task = BackgroundTask::<u32>::spawn(
        "viewer-background-load-panic-test",
        |_| panic!("test panic"),
        move || {
            let _ = wake_sender.send(());
        },
    )
    .expect("background task should start");

    wake_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("panicking task should still wake the event loop");
    match task.try_take() {
        BackgroundTaskPoll::Completed(Err(message)) => {
            assert!(message.contains("test panic"));
        }
        _ => panic!("background task panic was not returned as an error"),
    }
}

#[test]
fn cancellation_signal_is_observed_and_the_loader_is_joined() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (wake_sender, wake_receiver) = mpsc::channel();
    let task = BackgroundTask::spawn(
        "viewer-background-load-cancellation-test",
        move |cancellation| {
            started_sender
                .send(())
                .expect("test should observe the task start");
            while !cancellation.is_cancel_requested() {
                std::thread::yield_now();
            }
            Ok(42_u32)
        },
        move || {
            let _ = wake_sender.send(());
        },
    )
    .expect("background task should start");

    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("background task should begin before cancellation");
    assert!(task.request_cancel());
    assert!(task.is_cancellation_requested());
    assert_eq!(
        task.cancel_and_join(Duration::from_secs(2)),
        BackgroundTaskShutdown::CancelledAndJoined
    );
    wake_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("cancelled task should wake the event loop after joining");
}

#[test]
fn shutdown_timeout_is_distinct_from_a_joined_cancellation() {
    let (started_sender, started_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::sync_channel(1);
    let (wake_sender, wake_receiver) = mpsc::channel();
    let task = BackgroundTask::spawn(
        "viewer-background-load-timeout-test",
        move |_| {
            started_sender
                .send(())
                .expect("test should observe the task start");
            release_receiver
                .recv()
                .expect("test should release the non-cooperative task");
            Ok(())
        },
        move || {
            let _ = wake_sender.send(());
        },
    )
    .expect("background task should start");

    started_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("background task should begin before cancellation");
    assert!(task.request_cancel());
    assert_eq!(
        task.cancel_and_join(Duration::ZERO),
        BackgroundTaskShutdown::TimedOut
    );
    release_sender
        .send(())
        .expect("test should release the timed-out task");
    wake_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("timed-out task should eventually finish after release");
}
