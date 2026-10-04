use super::{HubEditorReadyReceiptV1, HubEditorStartupMilestoneV1};

#[test]
fn receipt_after_first_present_contains_every_required_milestone() {
    let receipt =
        HubEditorReadyReceiptV1::after_first_present(913, "913-42", 7).expect("valid receipt");

    assert_eq!(receipt.milestones().len(), 5);
    assert!(receipt
        .milestones()
        .contains(&HubEditorStartupMilestoneV1::FirstPresent));
}

#[test]
fn deserialize_rejects_partial_or_path_bearing_ready_receipts() {
    assert!(serde_json::from_str::<HubEditorReadyReceiptV1>(
        r#"{"editor_process_id":913,"editor_instance_id":"913-42","session_generation":7,"milestones":["session_committed","native_window_created"]}"#,
    )
    .is_err());
    assert!(serde_json::from_str::<HubEditorReadyReceiptV1>(
        r#"{"editor_process_id":913,"editor_instance_id":"913-42","session_generation":7,"milestones":["session_committed","native_window_created","first_present","focus_inbox_bound","interactive"],"project":"E:/Projects/Secret"}"#,
    )
    .is_err());
}
