use super::{
    UiNumberInputCommitMethod, UiNumberInputCommitStatus, UiNumberInputParseStatus,
    UiNumberInputReceiptV1, UI_NUMBER_INPUT_RECEIPT_VERSION_V1,
};

#[test]
fn number_input_receipt_roundtrips_without_source_text() {
    let receipt = UiNumberInputReceiptV1 {
        parse_status: UiNumberInputParseStatus::OutOfRange,
        commit_method: UiNumberInputCommitMethod::Enter,
        commit_status: UiNumberInputCommitStatus::Clamped,
        ..UiNumberInputReceiptV1::default()
    };

    let json = serde_json::to_string(&receipt).expect("receipt serializes");
    let roundtrip: UiNumberInputReceiptV1 =
        serde_json::from_str(&json).expect("receipt deserializes");

    assert_eq!(roundtrip, receipt);
    assert!(roundtrip.validate());
    assert_eq!(roundtrip.version, UI_NUMBER_INPUT_RECEIPT_VERSION_V1);
    assert!(!json.contains("123456"));
}

#[test]
fn keyboard_step_receipt_has_a_stable_snake_case_wire_value() {
    let receipt = UiNumberInputReceiptV1 {
        parse_status: UiNumberInputParseStatus::Valid,
        commit_method: UiNumberInputCommitMethod::KeyboardStep,
        commit_status: UiNumberInputCommitStatus::Applied,
        ..UiNumberInputReceiptV1::default()
    };

    let json = serde_json::to_string(&receipt).expect("receipt serializes");

    assert!(json.contains("\"commit_method\":\"keyboard_step\""));
}
