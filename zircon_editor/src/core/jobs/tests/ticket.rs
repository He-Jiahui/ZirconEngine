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
