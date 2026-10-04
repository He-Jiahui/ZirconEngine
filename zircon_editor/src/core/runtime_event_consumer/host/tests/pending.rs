use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
};

use super::{EditorRuntimeEventConsumerDeliveryDisposition, PendingDeliveryBatch};

#[test]
fn page_bytes_are_partitioned_across_pending_deliveries() {
    let deliveries = (1..=3)
        .map(|sequence| {
            ZrRuntimePluginEventDeliveryV1::new(
                7,
                ZrRuntimePluginEventSubscriptionHandle::new(11),
                "tests.events.pending",
                "tests.events.pending.v1",
                sequence,
                serde_json::json!({ "value": sequence }),
            )
        })
        .collect();
    let mut batch = PendingDeliveryBatch::from_page(deliveries, 10);

    assert_eq!(batch.retained_bytes_upper_bound(), 10);
    assert_eq!(
        batch.begin_current().unwrap().retained_bytes_upper_bound(),
        4
    );
    batch.complete_current(EditorRuntimeEventConsumerDeliveryDisposition::Applied);
    assert_eq!(batch.retained_bytes_upper_bound(), 6);
    assert_eq!(batch.first_sequence(), Some(2));

    batch.begin_current();
    batch.retry_current();
    let retrying = batch.begin_current().unwrap();
    assert_eq!(
        retrying.disposition(),
        Some(EditorRuntimeEventConsumerDeliveryDisposition::Retryable)
    );
    assert_eq!(retrying.retry_count(), 1);
}
