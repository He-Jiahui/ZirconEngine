use super::UiBindingDiagnosticCode;

#[test]
fn binding_diagnostic_identity_contract_is_unique_and_stable() {
    let expected = [
        (
            UiBindingDiagnosticCode::InvalidTarget,
            "invalid_target",
            "ZUI-BIND-0001",
            "diagnostic.ui.binding.invalid_target",
        ),
        (
            UiBindingDiagnosticCode::InvalidValueKind,
            "invalid_value_kind",
            "ZUI-BIND-0002",
            "diagnostic.ui.binding.invalid_value_kind",
        ),
        (
            UiBindingDiagnosticCode::UnresolvedRef,
            "unresolved_ref",
            "ZUI-BIND-0003",
            "diagnostic.ui.binding.unresolved_ref",
        ),
        (
            UiBindingDiagnosticCode::UnsupportedOperator,
            "unsupported_operator",
            "ZUI-BIND-0004",
            "diagnostic.ui.binding.unsupported_operator",
        ),
        (
            UiBindingDiagnosticCode::UnsupportedBindingMode,
            "unsupported_binding_mode",
            "ZUI-BIND-0005",
            "diagnostic.ui.binding.unsupported_binding_mode",
        ),
    ];

    assert_eq!(UiBindingDiagnosticCode::ALL, expected.map(|entry| entry.0));
    for (index, (code, error_code, diagnostic_id, localization_key)) in
        expected.into_iter().enumerate()
    {
        assert_eq!(code.error_code(), error_code);
        assert_eq!(code.as_str(), error_code);
        assert_eq!(code.diagnostic_id(), diagnostic_id);
        assert_eq!(code.localization_key(), localization_key);

        for other in expected.into_iter().skip(index + 1) {
            assert_ne!(error_code, other.1);
            assert_ne!(diagnostic_id, other.2);
            assert_ne!(localization_key, other.3);
        }
    }
}
