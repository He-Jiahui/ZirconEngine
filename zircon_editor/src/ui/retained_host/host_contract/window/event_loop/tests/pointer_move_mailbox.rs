use super::*;

fn pending(sequence: u64) -> PendingIdlePointerMove {
    PendingIdlePointerMove {
        device_id: None,
        metadata: UiInputEventMetadata::new(
            zircon_runtime_interface::ui::dispatch::UiInputTimestamp::from_micros(sequence),
            UiInputSequence::new(sequence),
        ),
        fallback_position: PhysicalPosition::new(sequence as f64, 0.0),
    }
}

#[test]
fn mailbox_replaces_only_the_previous_latest_move() {
    let mut mailbox = UiIdlePointerMoveMailbox::default();

    mailbox.replace(pending(1));
    mailbox.replace(pending(2));
    let latest = mailbox.take().expect("latest pointer move");
    assert_eq!(latest.pending.metadata.sequence, UiInputSequence::new(2));
    assert_eq!(latest.received_count, 2);
    assert_eq!(
        latest.coalesced,
        Some(UiCoalescedInputRange::single(UiInputSequence::new(1)))
    );
    assert!(mailbox.take().is_none());
}

#[test]
fn mailbox_coalesces_consecutive_sequences_into_one_bounded_range() {
    let mut mailbox = UiIdlePointerMoveMailbox::default();

    for sequence in 7..=10 {
        mailbox.replace(pending(sequence));
    }
    let batch = mailbox.take().expect("pointer move batch");

    assert_eq!(batch.received_count, 4);
    assert_eq!(
        batch.coalesced,
        Some(UiCoalescedInputRange {
            first_sequence: UiInputSequence::new(7),
            last_sequence: UiInputSequence::new(9),
            count: 3,
        })
    );
    assert_eq!(batch.pending.metadata.sequence, UiInputSequence::new(10));
}
