use super::*;
use crate::input::{DefaultInputManager, InputButton, TouchPhase};

fn limited(events: usize, bytes: usize) -> InputDriver {
    InputDriver::with_limits(InputIngressLimits {
        events_per_frame: events,
        owned_bytes_per_frame: bytes,
        max_receive_age: Duration::from_secs(1),
    })
    .unwrap()
}

fn submit(
    driver: &InputDriver,
    manager: &DefaultInputManager,
    cap: &InputAttachment,
    event: InputEvent,
) -> InputIngressAdmission {
    driver.submit_batch(manager, cap, vec![event], Instant::now())
}

#[test]
fn runtime220_driver_receiving_requires_real_admission_and_exact_session_capability() {
    let manager = DefaultInputManager::default();
    let driver = InputDriver::default();
    assert_eq!(driver.health().phase, InputIngressPhase::Unattached);
    let first = driver.claim_attachment().unwrap();
    assert_eq!(driver.health().phase, InputIngressPhase::Unattached);
    assert!(matches!(
        driver.claim_attachment(),
        Err(InputIngressRejection::AlreadyAttached)
    ));
    assert!(matches!(
        submit(&driver, &manager, &first, InputEvent::CursorEntered),
        InputIngressAdmission::Accepted(_)
    ));
    assert_eq!(driver.health().phase, InputIngressPhase::Receiving);
    driver.detach(&manager, &first).unwrap();
    let next = driver.claim_attachment().unwrap();
    assert!(next.generation() > first.generation());
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &next,
            InputEvent::ButtonPressed(InputButton::MouseLeft)
        ),
        InputIngressAdmission::Accepted(_)
    ));
    assert_eq!(
        driver.detach(&manager, &first),
        Err(InputIngressRejection::AttachmentMismatch)
    );
    assert_eq!(
        driver.begin_frame(&manager, &first),
        Err(InputIngressRejection::AttachmentMismatch)
    );
    assert!(matches!(
        submit(&driver, &manager, &first, InputEvent::FocusLost),
        InputIngressAdmission::Rejected(InputIngressRejection::AttachmentMismatch)
    ));
    assert!(manager.button_pressed(&InputButton::MouseLeft));
    driver.detach(&manager, &next).unwrap();
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert!(!driver.health().teardown_reset_pending);
}

#[test]
fn runtime220_driver_overloaded_release_uses_reserved_reset_and_gap_never_reattaches() {
    let manager = DefaultInputManager::default();
    let driver = limited(3, 64 * 1024);
    let cap = driver.claim_attachment().unwrap();
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::ButtonPressed(InputButton::MouseLeft)
        ),
        InputIngressAdmission::Accepted(_)
    ));
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::Touch {
                id: 5,
                phase: TouchPhase::Started,
                x: 1.0,
                y: 2.0,
            }
        ),
        InputIngressAdmission::Accepted(_)
    ));
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::Ime(crate::input::ImeEvent::Enabled)
        ),
        InputIngressAdmission::Accepted(_)
    ));
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::ButtonReleased(InputButton::MouseLeft)
        ),
        InputIngressAdmission::Rejected(InputIngressRejection::FrameCapacity {
            reset_completed: true,
            ..
        })
    ));
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert!(manager.frame_snapshot().active_touches.is_empty());
    assert!(!manager.frame_snapshot().ime_enabled);
    assert_eq!(driver.health().phase, InputIngressPhase::Gap);
    assert_eq!(driver.health().gap_resets, 1);
    assert_eq!(manager.drain_events().last(), Some(&InputEvent::FocusLost));
    driver.begin_frame(&manager, &cap).unwrap();
    assert!(matches!(
        submit(&driver, &manager, &cap, InputEvent::CursorEntered),
        InputIngressAdmission::Rejected(InputIngressRejection::Gap)
    ));
    assert!(manager.drain_events().is_empty());
    assert!(matches!(
        driver.claim_attachment(),
        Err(InputIngressRejection::AlreadyAttached)
    ));
    driver.detach(&manager, &cap).unwrap();
}

#[test]
fn runtime220_driver_owned_byte_exhaustion_resets_held_input() {
    let manager = DefaultInputManager::default();
    let driver = limited(8, 2 * size_of::<InputEvent>());
    let cap = driver.claim_attachment().unwrap();
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::ButtonPressed(InputButton::MouseLeft)
        ),
        InputIngressAdmission::Accepted(_)
    ));
    let mut value = String::with_capacity(4096);
    value.push('a');
    assert!(matches!(
        submit(
            &driver,
            &manager,
            &cap,
            InputEvent::Ime(crate::input::ImeEvent::Commit(value),)
        ),
        InputIngressAdmission::Rejected(InputIngressRejection::FrameCapacity {
            reason: InputBatchRejection::OwnedByteCapacity { .. },
            reset_completed: true,
        })
    ));
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert_eq!(driver.health().gap_resets, 1);
}

#[test]
fn runtime220_driver_runtime_receive_clock_rejects_expired_or_future_batches_without_publication() {
    let manager = DefaultInputManager::default();
    let driver = limited(8, 64 * 1024);
    let cap = driver.claim_attachment().unwrap();
    let now = Instant::now();
    assert!(matches!(
        driver.submit_batch(
            &manager,
            &cap,
            vec![InputEvent::CursorEntered],
            now.checked_sub(Duration::from_secs(2)).unwrap()
        ),
        InputIngressAdmission::Rejected(InputIngressRejection::ReceiveExpired)
    ));
    assert!(matches!(
        driver.submit_batch(
            &manager,
            &cap,
            vec![InputEvent::CursorEntered],
            Instant::now().checked_add(Duration::from_secs(2)).unwrap()
        ),
        InputIngressAdmission::Rejected(InputIngressRejection::ReceiveClockAhead)
    ));
    assert!(manager.drain_events().is_empty());
    assert_eq!(driver.health().phase, InputIngressPhase::Unattached);
    assert!(matches!(
        submit(&driver, &manager, &cap, InputEvent::CursorEntered),
        InputIngressAdmission::Accepted(_)
    ));
    assert_eq!(driver.health().frame_events, 1);
}

#[test]
fn runtime220_driver_invalid_batch_does_not_spend_capacity_or_publish_a_prefix() {
    let manager = DefaultInputManager::default();
    let driver = limited(2, 64 * 1024);
    let cap = driver.claim_attachment().unwrap();
    assert!(matches!(
        driver.submit_batch(
            &manager,
            &cap,
            vec![
                InputEvent::ButtonPressed(InputButton::MouseLeft),
                InputEvent::CursorMoved {
                    x: f32::INFINITY,
                    y: 0.0
                },
            ],
            Instant::now()
        ),
        InputIngressAdmission::Rejected(InputIngressRejection::Batch(
            InputBatchRejection::NonFinite { event_index: 1 }
        ))
    ));
    assert!(!manager.button_pressed(&InputButton::MouseLeft));
    assert!(manager.drain_events().is_empty());
    assert_eq!(driver.health().frame_events, 0);
    assert_eq!(driver.health().gap_resets, 0);
    assert!(matches!(
        submit(&driver, &manager, &cap, InputEvent::CursorEntered),
        InputIngressAdmission::Accepted(_)
    ));
}
