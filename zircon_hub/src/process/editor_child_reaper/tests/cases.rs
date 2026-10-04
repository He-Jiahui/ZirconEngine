use std::process::Command;
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};

use super::EditorChildReaper;
use crate::process::editor_child_receipt::EditorChildDisposition;
use crate::process::SupervisedChild;

#[test]
fn exited_child_delivers_one_attempt_bound_receipt_after_reap() {
    let (sender, receiver) = mpsc::channel();
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        sender
            .send(receipt)
            .expect("terminal receipt receiver stays alive");
    })
    .expect("start child reaper");
    let child = exited_child();
    let process_id = child.id();
    reaper
        .register(41, child)
        .expect("register supervised child");

    let receipt = receiver
        .recv_timeout(Duration::from_secs(3))
        .expect("receive terminal receipt");
    assert_eq!(receipt.attempt_id, 41);
    assert_eq!(receipt.process_id, process_id);
    assert_eq!(
        receipt.disposition,
        EditorChildDisposition::Exited {
            code: Some(7),
            signal: None,
        }
    );
    assert_eq!(receipt.cleanup_error, None);
    assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
    reaper
        .shutdown_and_join_until(Instant::now() + Duration::from_secs(3))
        .expect("join reaper after terminal receipt");
}

#[test]
fn shutdown_terminates_registered_child_and_delivers_one_receipt() {
    let (sender, receiver) = mpsc::channel();
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        sender
            .send(receipt)
            .expect("terminal receipt receiver stays alive");
    })
    .expect("start child reaper");
    let child = long_running_child();
    let process_id = child.id();
    reaper
        .register(42, child)
        .expect("register supervised child");

    reaper
        .shutdown_and_join_until(Instant::now() + Duration::from_secs(3))
        .expect("shutdown and join child reaper");
    let receipt = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("receive shutdown terminal receipt");
    assert_eq!(receipt.attempt_id, 42);
    assert_eq!(receipt.process_id, process_id);
    assert_eq!(receipt.disposition, EditorChildDisposition::StoppedByHub);
    assert_eq!(receipt.cleanup_error, None);
    assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
}

#[test]
fn ready_child_registered_after_shutdown_gets_stopped_receipt() {
    let (sender, receiver) = mpsc::channel();
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        sender.send(receipt).expect("receipt receiver stays alive");
    })
    .expect("start child reaper");
    reaper
        .shutdown_and_join_until(Instant::now() + Duration::from_secs(1))
        .expect("stop reaper before registration");
    let child = long_running_child();
    let process_id = child.id();

    reaper
        .register(43, child)
        .expect("late Ready child is stopped with a receipt");

    let receipt = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("stopped receipt");
    assert_eq!(receipt.attempt_id, 43);
    assert_eq!(receipt.process_id, process_id);
    assert_eq!(receipt.disposition, EditorChildDisposition::StoppedByHub);
    assert_eq!(receipt.cleanup_error, None);
    assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
}

#[test]
fn pre_ready_cancel_reaps_child_with_attempt_and_pid_receipt() {
    let (sender, receiver) = mpsc::channel();
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        sender.send(receipt).expect("receipt receiver stays alive");
    })
    .expect("start child reaper");
    let child = long_running_child();
    let process_id = child.id();

    reaper
        .cancel_before_ready(44, child)
        .expect("cancel and reap pre-Ready child");

    let receipt = receiver
        .recv_timeout(Duration::from_secs(1))
        .expect("terminal receipt");
    assert_eq!(receipt.attempt_id, 44);
    assert_eq!(receipt.process_id, process_id);
    assert_eq!(
        receipt.disposition,
        EditorChildDisposition::StoppedBeforeReady
    );
    assert_eq!(receipt.cleanup_error, None);
    assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
    reaper
        .shutdown_and_join_until(Instant::now() + Duration::from_secs(1))
        .unwrap();
}

#[test]
fn shutdown_keeps_the_first_absolute_deadline_across_cleanup_and_join() {
    let reaper = EditorChildReaper::start().expect("start child reaper");
    let first_deadline = Instant::now() + Duration::from_secs(1);
    reaper.request_shutdown_until(first_deadline);
    reaper.request_shutdown_until(first_deadline + Duration::from_secs(2));

    assert_eq!(
        *reaper
            .inner
            .shutdown_deadline
            .lock()
            .expect("shutdown deadline lock"),
        Some(first_deadline),
        "later shutdown calls must not restart child cleanup's clock"
    );
    reaper
        .shutdown_and_join_until(first_deadline + Duration::from_secs(2))
        .expect("join reaper");
    assert_eq!(
        *reaper
            .inner
            .shutdown_deadline
            .lock()
            .expect("shutdown deadline lock"),
        Some(first_deadline),
        "joining must preserve the original child cleanup deadline"
    );
}

#[test]
fn second_live_child_keeps_the_shutdown_deadline_after_first_receipt_stalls() {
    let (receipt_sender, receipt_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let release_receiver = Mutex::new(release_receiver);
    let reaper = EditorChildReaper::start_with_observer(move |receipt| {
        let first = receipt.attempt_id == 51;
        receipt_sender
            .send(receipt)
            .expect("receipt receiver stays alive");
        if first {
            release_receiver
                .lock()
                .expect("release receiver lock")
                .recv()
                .expect("release first receipt callback");
        }
    })
    .expect("start child reaper");
    let first = long_running_child();
    let second = long_running_child();
    let second_pid = second.id();
    reaper.register(51, first).expect("register first child");
    reaper.register(52, second).expect("register second child");

    let deadline = Instant::now() + Duration::from_secs(1);
    reaper.request_shutdown_until(deadline);
    assert_eq!(
        receipt_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("first child terminal")
            .attempt_id,
        51
    );
    assert!(
        reaper.shutdown_and_join_until(deadline).is_err(),
        "the blocked first receipt must leave the second child owned at the deadline"
    );
    release_sender.send(()).expect("resume reaper");
    let second_receipt = receipt_receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("second child terminal");
    assert_eq!(second_receipt.attempt_id, 52);
    assert_eq!(second_receipt.process_id, second_pid);
    assert_eq!(
        second_receipt.disposition,
        EditorChildDisposition::StoppedByHub
    );
    assert_eq!(
        *reaper
            .inner
            .shutdown_deadline
            .lock()
            .expect("shutdown deadline lock"),
        Some(deadline),
        "the second live child's cleanup must not start a new deadline"
    );
    let join_deadline = Instant::now() + Duration::from_secs(2);
    while !reaper
        .worker
        .lock()
        .expect("reaper worker lock")
        .as_ref()
        .is_none_or(std::thread::JoinHandle::is_finished)
        && Instant::now() < join_deadline
    {
        std::thread::sleep(Duration::from_millis(10));
    }
    reaper
        .shutdown_and_join_until(deadline)
        .expect("retained owner can join a reaper that completed after its deadline");
}

#[cfg(windows)]
fn exited_child() -> SupervisedChild {
    let mut command = Command::new("cmd");
    command.args(["/C", "exit", "/B", "7"]);
    SupervisedChild::spawn(&mut command, "exited Editor fixture").expect("spawn fixture")
}

#[cfg(unix)]
fn exited_child() -> SupervisedChild {
    let mut command = Command::new("sh");
    command.args(["-c", "exit 7"]);
    SupervisedChild::spawn(&mut command, "exited Editor fixture").expect("spawn fixture")
}

fn long_running_child() -> SupervisedChild {
    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command.args([
        "--exact",
        "process::editor_child_reaper::terminal_tests::long_running_fixture",
        "--ignored",
    ]);
    SupervisedChild::spawn(&mut command, "long-running Editor fixture").expect("spawn fixture")
}

#[test]
#[ignore = "child fixture for shutdown_terminates_registered_child_and_delivers_one_receipt"]
fn long_running_fixture() {
    std::thread::sleep(Duration::from_secs(30));
}
