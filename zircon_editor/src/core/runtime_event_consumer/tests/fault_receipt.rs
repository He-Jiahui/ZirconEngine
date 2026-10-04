use zircon_runtime_interface::{
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
};

use super::{
    EditorRuntimeEventConsumerCallbackPhase, EditorRuntimeEventConsumerFaultReceiptBudget,
    EditorRuntimeEventConsumerFaultReceiptJournal,
};

#[test]
fn zero_receipt_capacity_keeps_one_fault_record() {
    assert_eq!(
        EditorRuntimeEventConsumerFaultReceiptBudget::new(0, 0).max_receipts(),
        1
    );

    let delivery = ZrRuntimePluginEventDeliveryV1::new(
        7,
        ZrRuntimePluginEventSubscriptionHandle::new(11),
        "tests.events.panic",
        "tests.events.panic.v1",
        1,
        serde_json::json!({ "payload": "must-not-be-copied" }),
    );
    let mut journal = EditorRuntimeEventConsumerFaultReceiptJournal::new(
        EditorRuntimeEventConsumerFaultReceiptBudget::new(0, 0),
    );
    journal.record_callback_panic(
        "tests.consumer.panic",
        7,
        EditorRuntimeEventConsumerCallbackPhase::Consume,
        Some(&delivery),
        None,
        |_| true,
        |_| {},
    );

    let receipts = journal.snapshot();
    assert_eq!(receipts.len(), 1);
    assert!(receipts[0].payload_json().is_none());
    assert!(receipts[0].payload_was_truncated());
    assert!(receipts[0].payload_digest().is_some());
}
