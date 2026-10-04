use super::*;

#[test]
fn execution_receipt_rejects_output_over_contract_budget() {
    let command_id = EditorOperationPath::parse("test.command.receipt").unwrap();
    let report = NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: vec!["callback completed".to_owned()],
        payload: Some(vec![1, 2, 3, 4, 5]),
    };

    let receipt = EditorCommandExecutionReceipt::from_report(
        command_id.clone(),
        "fixture.plugin".to_owned(),
        4,
        report,
    );

    assert_eq!(receipt.command_id(), &command_id);
    assert_eq!(receipt.plugin_id(), "fixture.plugin");
    assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert_eq!(receipt.payload(), None);
    assert!(receipt
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("output exceeds budget")));
}

#[test]
fn execution_receipt_preserves_bounded_success_payload_and_diagnostics() {
    let report = NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: vec!["callback completed".to_owned()],
        payload: Some(vec![1, 2, 3]),
    };

    let receipt = EditorCommandExecutionReceipt::from_report(
        EditorOperationPath::parse("test.command.receipt_success").unwrap(),
        "fixture.plugin".to_owned(),
        3,
        report,
    );

    assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(receipt.payload(), Some([1, 2, 3].as_slice()));
    assert_eq!(receipt.diagnostics().len(), 1);
    assert_eq!(receipt.diagnostics()[0], "callback completed");
}

#[test]
fn zero_output_contract_normalizes_empty_callback_payload_to_no_result() {
    let report = NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: Vec::new(),
        payload: Some(Vec::new()),
    };

    let receipt = EditorCommandExecutionReceipt::from_report(
        EditorOperationPath::parse("test.command.no_result").unwrap(),
        "fixture.plugin".to_owned(),
        0,
        report,
    );

    assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(receipt.payload(), None);
}
