use super::*;

#[test]
fn drain_page_reports_remaining_deliveries_and_oldest_pending_age() {
    let payload = vec![b'x'; RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES];
    let observed_at = Instant::now();
    let mut queue = RuntimeEventMirrorQueue {
        pending: VecDeque::from([
            QueuedRuntimeEventPayload {
                payload: payload.clone(),
                enqueued_at: observed_at
                    .checked_sub(Duration::from_millis(12))
                    .expect("test instant supports a recent offset"),
            },
            QueuedRuntimeEventPayload {
                payload,
                enqueued_at: observed_at
                    .checked_sub(Duration::from_millis(7))
                    .expect("test instant supports a recent offset"),
            },
        ]),
        pending_payload_bytes: RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES * 2,
        failure: None,
    };

    let page = queue
        .drain_page(RUNTIME_EVENT_MIRROR_PAGE_MAX_EVENTS)
        .expect("queue drain page");

    assert_eq!(page.payloads.len(), 1);
    assert_eq!(page.remaining_deliveries, 1);
    assert!(page.oldest_pending_age_millis >= 7);
}

#[test]
fn drain_page_limit_leaves_unconsumed_payloads_in_subscription_authority() {
    let mut queue = RuntimeEventMirrorQueue {
        pending: VecDeque::from([
            QueuedRuntimeEventPayload {
                payload: b"first".to_vec(),
                enqueued_at: Instant::now(),
            },
            QueuedRuntimeEventPayload {
                payload: b"second".to_vec(),
                enqueued_at: Instant::now(),
            },
        ]),
        pending_payload_bytes: b"first".len() + b"second".len(),
        failure: None,
    };

    let first = queue.drain_page(1).expect("limited first queue page");
    assert_eq!(first.payloads.len(), 1);
    assert_eq!(first.remaining_deliveries, 1);

    let deferred = queue.drain_page(0).expect("zero-limit queue page");
    assert!(deferred.payloads.is_empty());
    assert_eq!(deferred.remaining_deliveries, 1);

    let second = queue.drain_page(1).expect("limited second queue page");
    assert_eq!(second.payloads.len(), 1);
    assert_eq!(second.remaining_deliveries, 0);
}

#[test]
fn writer_stops_serialization_at_the_payload_byte_budget() {
    let oversized = "x".repeat(RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES * 2);
    let mut writer = BoundedRuntimeEventPayloadWriter::new(
        RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
        ZR_RUNTIME_PLUGIN_EVENT_OUTPUT_LIMIT_V1.max_nesting_depth,
        u64::MAX,
    );

    let result = serde_json::to_writer(&mut writer, &oversized);
    let failure = writer
        .finish(result)
        .expect_err("oversized event payload must stop at the writer boundary");
    assert!(matches!(
        failure,
        RuntimeEventMirrorQueueFailure::PayloadTooLarge {
            max_payload_bytes: RUNTIME_EVENT_MIRROR_PAGE_MAX_PAYLOAD_BYTES,
            ..
        }
    ));
}
