use super::*;

#[test]
fn result_codec_id_requires_a_namespaced_positive_version() {
    let codec = EditorCommandResultCodecId::parse("zircon.editor.command-result.v1")
        .expect("versioned result codec id should parse");
    assert_eq!(codec.as_str(), "zircon.editor.command-result.v1");
    for invalid in [
        "zircon.editor.result",
        "zircon.editor.result.v0",
        "zircon.editor.result.v01",
        "Zircon.editor.result.v1",
        "zircon..result.v1",
    ] {
        assert!(
            EditorCommandResultCodecId::parse(invalid).is_err(),
            "codec id `{invalid}` must be rejected"
        );
    }
}

#[test]
fn resource_budget_enforces_finite_input_output_and_time_limits() {
    let budget = EditorCommandResourceBudget::new(
        MAX_EDITOR_COMMAND_INPUT_BYTES,
        0,
        MAX_EDITOR_COMMAND_EXECUTION_TIME_MS,
    )
    .expect("zero output is valid for a command without a result payload");
    assert_eq!(budget.max_output_bytes(), 0);
    assert!(matches!(
        EditorCommandResourceBudget::new(0, 0, 0),
        Err(EditorCommandResourceBudgetError::ExecutionTimeZero)
    ));
    assert!(matches!(
        EditorCommandResourceBudget::new(MAX_EDITOR_COMMAND_INPUT_BYTES + 1, 0, 1),
        Err(EditorCommandResourceBudgetError::InputLimitTooLarge { .. })
    ));
    assert!(matches!(
        EditorCommandResourceBudget::new(0, MAX_EDITOR_COMMAND_OUTPUT_BYTES + 1, 1),
        Err(EditorCommandResourceBudgetError::OutputLimitTooLarge { .. })
    ));
    assert!(matches!(
        EditorCommandResourceBudget::new(0, 0, MAX_EDITOR_COMMAND_EXECUTION_TIME_MS + 1),
        Err(EditorCommandResourceBudgetError::ExecutionTimeTooLong { .. })
    ));
}

#[test]
fn execution_contract_roundtrips_and_rejects_invalid_budget_on_decode() {
    let contract = EditorCommandExecutionContract::new(
        EditorCommandResultCodecId::parse("zircon.editor.command-result.v1").unwrap(),
        EditorCommandResourceBudget::new(4096, 8192, 250).unwrap(),
    );
    let encoded = serde_json::to_vec(&contract).expect("contract should serialize");
    let decoded: EditorCommandExecutionContract =
        serde_json::from_slice(&encoded).expect("contract should deserialize");
    assert_eq!(decoded, contract);

    let invalid = br#"{
            "result_codec":"zircon.editor.command-result.v1",
            "resource_budget":{"max_input_bytes":0,"max_output_bytes":0,"max_execution_time_ms":0}
        }"#;
    assert!(serde_json::from_slice::<EditorCommandExecutionContract>(invalid).is_err());
}
