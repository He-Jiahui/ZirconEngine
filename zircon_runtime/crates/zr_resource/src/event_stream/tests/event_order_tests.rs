use super::*;
use crate::{ResourceLocator, ResourceScheme};

fn event(id: usize) -> ResourceEvent {
    ResourceEvent {
        kind: ResourceEventKind::Renamed,
        resource_kind: ResourceKind::Texture,
        id: ResourceId::from_stable_label(&format!("terminal-event-{id}")),
        locator: Some(
            ResourceLocator::new(
                ResourceScheme::Res,
                format!("textures/terminal-{id}.png"),
                None,
            )
            .expect("terminal event fixture must use a canonical resource locator"),
        ),
        previous_locator: None,
        revision: 1,
    }
}

#[test]
fn final_sequence_is_published_once_then_all_receive_modes_report_exhaustion() {
    let publisher = ResourceEventPublisher::default();
    publisher.set_next_sequence_for_test(Some(u64::MAX));
    let try_receiver = publisher.subscribe();
    let blocking_receiver = publisher.subscribe();
    let timeout_receiver = publisher.subscribe();

    publisher.publish_for_test(event(1));

    assert_eq!(try_receiver.try_recv().unwrap().id, event(1).id);
    assert_eq!(
        try_receiver.try_recv(),
        Err(ResourceEventTryRecvError::SequenceExhausted)
    );
    assert_eq!(blocking_receiver.recv().unwrap().id, event(1).id);
    assert_eq!(
        blocking_receiver.recv(),
        Err(ResourceEventRecvError::SequenceExhausted)
    );
    assert_eq!(
        timeout_receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap()
            .id,
        event(1).id
    );
    assert_eq!(
        timeout_receiver.recv_timeout(Duration::from_secs(1)),
        Err(ResourceEventRecvTimeoutError::SequenceExhausted)
    );
    assert!(publisher.diagnostics().sequence_exhausted);
}

#[test]
fn terminal_sequence_rejects_new_publication_without_mutating_the_log() {
    let publisher = ResourceEventPublisher::default();
    publisher.set_next_sequence_for_test(None);

    assert_eq!(
        publisher.try_publish_for_test(event(1)),
        Err(ResourceRegistryError::EventSequenceExhausted {
            requested_event_count: 1,
        })
    );

    let diagnostics = publisher.diagnostics();
    assert!(diagnostics.sequence_exhausted);
    assert_eq!(diagnostics.rejected_publish_count, 1);
    assert_eq!(diagnostics.depth, 0);
}

#[test]
fn terminal_lag_reports_that_no_successor_sequence_exists() {
    let publisher = ResourceEventPublisher::default();
    publisher.set_next_sequence_for_test(Some(u64::MAX));
    let receiver = publisher.subscribe();
    publisher.publish_for_test(event(1));
    publisher.drop_all_events_for_test();

    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Lagged(ResourceEventGap {
            expected_sequence: u64::MAX,
            oldest_available_sequence: None,
        }))
    );
    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::SequenceExhausted)
    );
}

#[test]
fn one_reserved_batch_can_consume_the_remaining_sequence_range_exactly() {
    let publisher = ResourceEventPublisher::default();
    publisher.set_next_sequence_for_test(Some(u64::MAX - 1));
    let receiver = publisher.subscribe();
    let permit = publisher.prepare_publish(2).unwrap();

    publisher.publish_permitted(permit, vec![event(1), event(2)]);

    assert_eq!(receiver.try_recv().unwrap().id, event(1).id);
    assert_eq!(receiver.try_recv().unwrap().id, event(2).id);
    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::SequenceExhausted)
    );
}

#[test]
fn receivers_track_the_shared_publisher_lifetime_without_manual_counts() {
    let publisher = ResourceEventPublisher::default();
    let remaining_publisher = publisher.clone();
    let receiver = publisher.subscribe();

    drop(publisher);
    assert_eq!(receiver.try_recv(), Err(ResourceEventTryRecvError::Empty));
    drop(remaining_publisher);
    assert_eq!(
        receiver.try_recv(),
        Err(ResourceEventTryRecvError::Disconnected)
    );
}

#[test]
fn subscription_after_terminal_sequence_starts_exhausted() {
    let publisher = ResourceEventPublisher::default();
    publisher.set_next_sequence_for_test(None);

    assert_eq!(
        publisher.subscribe().try_recv(),
        Err(ResourceEventTryRecvError::SequenceExhausted)
    );
}
