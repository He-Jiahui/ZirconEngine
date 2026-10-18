use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::JobTicket;
use crate::core::jobs::{JobError, JobId};

#[test]
fn deadline_wait_consumes_a_ready_result_once() {
    let (sender, receiver) = mpsc::channel();
    sender.send(Ok(7_u32)).unwrap();
    let ticket = JobTicket::new(JobId::new(1), receiver);

    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(1)),
        Some(Ok(7))
    );
    assert_eq!(ticket.try_take(), None);
}

#[test]
fn deadline_wait_timeout_preserves_the_result_receiver_for_retry() {
    let (sender, receiver) = mpsc::channel();
    let ticket = JobTicket::new(JobId::new(2), receiver);
    let started = Instant::now();

    assert_eq!(ticket.wait_until(started + Duration::from_millis(10)), None);
    assert!(started.elapsed() < Duration::from_millis(250));

    sender.send(Ok::<_, JobError>(11_u32)).unwrap();
    assert_eq!(ticket.try_take(), Some(Ok(11)));
}

#[test]
fn deadline_wait_reports_a_disconnected_result_channel() {
    let (sender, receiver) = mpsc::channel::<Result<u32, JobError>>();
    drop(sender);
    let ticket = JobTicket::new(JobId::new(3), receiver);

    assert_eq!(
        ticket.wait_until(Instant::now() + Duration::from_secs(1)),
        Some(Err(JobError::ResultChannelClosed))
    );
    assert_eq!(ticket.try_take(), None);
}

#[test]
fn one_deadline_waiter_does_not_hold_the_ticket_lock_until_its_deadline() {
    let (sender, receiver) = mpsc::channel();
    let ticket = Arc::new(JobTicket::new(JobId::new(4), receiver));
    let waiter_ticket = Arc::clone(&ticket);
    let waiter = std::thread::spawn(move || {
        waiter_ticket.wait_until(Instant::now() + Duration::from_secs(1))
    });
    while ticket.lock_result().is_some() {
        std::thread::yield_now();
    }

    let probe_started = Instant::now();
    assert_eq!(ticket.wait_until(Instant::now()), None);
    assert!(probe_started.elapsed() < Duration::from_millis(50));

    sender.send(Ok::<_, JobError>(13_u32)).unwrap();
    assert_eq!(waiter.join().unwrap(), Some(Ok(13)));
}

#[test]
fn owned_ticket_does_not_consume_ready_result_before_real_runtime_terminal() {
    use super::{JobExecution, JobExecutionState};
    use crate::core::jobs::test_job_scheduler;
    let execution = Arc::new(JobExecution::new());
    let (sender, receiver) = mpsc::channel();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let handle = test_job_scheduler().schedule(move || {
        sender.send(Ok::<_, JobError>(29_u32)).unwrap();
        started_tx.send(()).unwrap();
        release_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    });
    *execution.lock() = JobExecutionState::Running(handle.clone());
    let ticket = JobTicket::new_owned(JobId::new(29), receiver, execution);
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!handle.is_complete());
    assert!(!ticket.is_terminal());
    assert_eq!(ticket.try_take_terminal(), None);
    release_tx.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ticket.is_terminal() {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(ticket.try_take_terminal(), Some(Ok(29)));
    assert_eq!(ticket.try_take_terminal(), None);
}

#[test]
fn submission_failure_has_one_typed_terminal_without_a_published_result() {
    use super::JobExecution;
    let execution = Arc::new(JobExecution::new());
    let (_sender, receiver) = mpsc::channel::<Result<(), JobError>>();
    let ticket = JobTicket::new_owned(JobId::new(30), receiver, Arc::clone(&execution));
    assert!(!ticket.is_terminal());
    execution.fail_submission("real admission did not install a task");
    assert!(ticket.is_terminal());
    assert!(matches!(
        ticket.try_take_terminal(),
        Some(Err(JobError::Panicked(_)))
    ));
    assert_eq!(ticket.try_take_terminal(), None);
}
