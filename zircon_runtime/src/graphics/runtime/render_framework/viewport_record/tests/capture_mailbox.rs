use super::super::capture::capture_generation_is_newer;
use super::{
    viewport_capture_pending_limit, ViewportAsyncCaptureMailbox, ViewportAsyncCaptureReservation,
};

#[test]
fn completed_capture_generation_never_moves_backwards() {
    assert!(capture_generation_is_newer(None, 8));
    assert!(capture_generation_is_newer(Some(8), 9));
    assert!(!capture_generation_is_newer(Some(8), 8));
    assert!(!capture_generation_is_newer(Some(8), 7));
}

#[test]
fn completion_before_registration_remains_available_while_armed() {
    let mut mailbox = ViewportAsyncCaptureMailbox::default();

    mailbox.arm(8);
    mailbox.complete(8, Ok(vec![1, 2, 3, 4]));

    assert!(mailbox.armed.contains(&8));
    assert!(mailbox.completed.contains_key(&8));
}

#[test]
fn late_completion_for_trimmed_generation_is_discarded() {
    let mut mailbox = ViewportAsyncCaptureMailbox::default();
    let limit = viewport_capture_pending_limit();

    for generation in 0..=limit as u64 {
        mailbox.arm(generation);
    }
    assert_eq!(mailbox.armed.len(), limit);
    assert!(!mailbox.armed.contains(&0));

    mailbox.complete(0, Ok(vec![0; 1024 * 1024]));

    assert!(!mailbox.completed.contains_key(&0));
    assert_eq!(mailbox.armed.len(), limit);
}

#[test]
fn repeated_late_completions_do_not_grow_completed_storage() {
    let mut mailbox = ViewportAsyncCaptureMailbox::default();
    let limit = viewport_capture_pending_limit();

    for generation in 0..(limit as u64 * 2) {
        mailbox.arm(generation);
    }
    for generation in 0..limit as u64 {
        mailbox.complete(generation, Ok(vec![0; 4096]));
    }

    assert_eq!(mailbox.armed.len(), limit);
    assert!(mailbox.completed.is_empty());
}

#[test]
fn cancelled_generation_rejects_late_completion() {
    let mut mailbox = ViewportAsyncCaptureMailbox::default();

    mailbox.arm(21);
    mailbox.cancel(21);
    mailbox.complete(21, Ok(vec![7; 4096]));

    assert!(!mailbox.armed.contains(&21));
    assert!(!mailbox.completed.contains_key(&21));
}

#[test]
fn dropped_capture_request_cancels_its_reservation() {
    let mailbox =
        std::sync::Arc::new(std::sync::Mutex::new(ViewportAsyncCaptureMailbox::default()));
    let reservation = ViewportAsyncCaptureReservation::new(std::sync::Arc::clone(&mailbox), 34);
    assert!(mailbox.lock().unwrap().armed.contains(&34));

    drop(reservation);

    assert!(!mailbox.lock().unwrap().armed.contains(&34));
}
