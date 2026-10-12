use super::*;

#[test]
fn contract_mismatch_reports_stage_expected_actual_path_and_hint() {
    let error = PluginLoadError::contract_mismatch(
        "fixture",
        PluginLoadStage::DescriptorProbe,
        "abi_version",
        "3",
        "2",
        Path::new("plugins/fixture/native/fixture.dll"),
        ABI_CONTRACT_HINT,
    );
    let message = error.to_string();

    assert!(message.contains("descriptor-probe"));
    assert!(message.contains("expected 3, actual 2"));
    assert!(message.contains("plugins/fixture/native/fixture.dll"));
    assert!(message.contains(ABI_CONTRACT_HINT));

    match error {
        PluginLoadError::ContractMismatch {
            expected, actual, ..
        } => {
            assert_eq!(expected, "3");
            assert_eq!(actual, "2");
        }
        other => panic!("unexpected plugin load error: {other}"),
    }
}

#[test]
fn invalid_payload_preserves_typed_source() {
    let error = PluginLoadError::invalid_payload(
        "fixture",
        PluginLoadStage::RuntimeEntry,
        "granted_capabilities",
        Path::new("fixture.dll"),
        ABI_CONTRACT_HINT,
        std::ffi::CString::new("invalid\0capability").expect_err("interior NUL must be rejected"),
    );

    assert!(std::error::Error::source(&error).is_some());
    match error {
        PluginLoadError::InvalidPayload {
            expected, actual, ..
        } => {
            assert_eq!(expected, "valid granted_capabilities");
            assert!(actual.contains("nul byte"));
        }
        other => panic!("unexpected plugin load error: {other}"),
    }
}
