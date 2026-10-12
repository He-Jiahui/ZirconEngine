use super::*;

fn editor_contribution_behavior(payload: &str) -> NativePluginBehavior {
    NativePluginBehavior {
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: None,
        event_manifest_schema: None,
        registration_manifest_schema: Some(
            SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1.to_string(),
        ),
        command_manifest: None,
        event_manifest: None,
        registration_manifest: Some(payload.to_string()),
        command_table: None,
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
    }
}

#[test]
fn native_entry_payload_error_preserves_granted_capability_source() {
    let source =
        CString::new("native\0capability").expect_err("interior NUL should be rejected by CString");
    let error = PluginLoadError::invalid_payload(
        "fixture",
        PluginLoadStage::RuntimeEntry,
        "granted_capabilities",
        Path::new("fixture.dll"),
        ABI_CONTRACT_HINT,
        source,
    );

    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn native_entry_contract_error_preserves_expected_and_actual_versions() {
    let error = PluginLoadError::contract_mismatch(
        "fixture",
        PluginLoadStage::RuntimeEntry,
        "entry_report.layout_epoch",
        ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH.to_string(),
        "3",
        Path::new("fixture.dll"),
        ABI_CONTRACT_HINT,
    );

    let message = error.to_string();
    assert!(message.contains(&format!(
        "expected {}, actual 3",
        ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH
    )));
    assert!(std::error::Error::source(&error).is_none());
}

#[test]
fn capability_negotiation_reports_missing_required_and_granted_denied_details() {
    let required = vec![
        "runtime.required".to_string(),
        "runtime.available".to_string(),
    ];
    let denied = vec!["runtime.denied".to_string(), "runtime.absent".to_string()];
    let granted = vec![
        "runtime.available".to_string(),
        "runtime.denied".to_string(),
    ];

    let (missing_required, denied) = capability_negotiation_details(&required, &denied, &granted);

    assert_eq!(missing_required, vec!["runtime.required"]);
    assert_eq!(denied, vec!["runtime.denied"]);
}

#[test]
fn editor_contribution_batch_decodes_valid_editor_payload() {
    let behavior = editor_contribution_behavior(
        r#"{
                "package_id": "fixture.editor",
                "contributions": [{
                    "kind": "view",
                    "id": "fixture.editor.view",
                    "schema": "zircon.editor.view/1",
                    "title": "Fixture",
                    "category": "Tests"
                }]
            }"#,
    );

    let batch = editor_contribution_batch_from_behavior(
        "fixture.editor",
        PluginModuleKind::Editor,
        Path::new("fixture.dll"),
        Some(&behavior),
    )
    .expect("valid editor contribution payload should decode")
    .expect("editor contribution schema should produce a batch");

    assert_eq!(batch.package_id(), "fixture.editor");
    assert_eq!(
        batch.contributions()[0].key(),
        ("view", "fixture.editor.view")
    );
}

#[test]
fn editor_contribution_batch_rejects_package_mismatch() {
    let behavior = editor_contribution_behavior(
        r#"{
                "package_id": "foreign.plugin",
                "contributions": []
            }"#,
    );

    let error = editor_contribution_batch_from_behavior(
        "fixture.editor",
        PluginModuleKind::Editor,
        Path::new("fixture.dll"),
        Some(&behavior),
    )
    .expect_err("foreign package payload must be rejected");

    assert!(error
        .to_string()
        .contains("editor_contribution_batch.package_id"));
}

#[test]
fn editor_contribution_batch_is_ignored_for_non_editor_entries() {
    let behavior = editor_contribution_behavior("not JSON");

    let batch = editor_contribution_batch_from_behavior(
        "fixture.runtime",
        PluginModuleKind::Runtime,
        Path::new("fixture.dll"),
        Some(&behavior),
    )
    .expect("runtime entries must not parse an editor-only payload");

    assert!(batch.is_none());
}
