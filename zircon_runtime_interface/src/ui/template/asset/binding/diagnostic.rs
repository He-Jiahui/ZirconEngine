use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingReport {
    #[serde(default)]
    pub diagnostics: Vec<UiBindingDiagnostic>,
}

impl UiBindingReport {
    pub fn is_valid(&self) -> bool {
        self.diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != UiBindingDiagnosticSeverity::Error)
    }

    pub fn first_error(&self) -> Option<&UiBindingDiagnostic> {
        self.diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == UiBindingDiagnosticSeverity::Error)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiBindingDiagnostic {
    pub code: UiBindingDiagnosticCode,
    pub severity: UiBindingDiagnosticSeverity,
    pub path: String,
    pub node_id: String,
    pub binding_id: String,
    pub message: String,
}

impl UiBindingDiagnostic {
    pub const fn error_code(&self) -> &'static str {
        self.code.error_code()
    }

    pub const fn diagnostic_id(&self) -> &'static str {
        self.code.diagnostic_id()
    }

    pub const fn localization_key(&self) -> &'static str {
        self.code.localization_key()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticCode {
    InvalidTarget,
    InvalidValueKind,
    UnresolvedRef,
    UnsupportedOperator,
    UnsupportedBindingMode,
}

#[derive(Clone, Copy)]
struct UiBindingDiagnosticIdentity {
    error_code: &'static str,
    diagnostic_id: &'static str,
    localization_key: &'static str,
}

impl UiBindingDiagnosticCode {
    pub const ALL: [Self; 5] = [
        Self::InvalidTarget,
        Self::InvalidValueKind,
        Self::UnresolvedRef,
        Self::UnsupportedOperator,
        Self::UnsupportedBindingMode,
    ];

    const fn identity(self) -> UiBindingDiagnosticIdentity {
        match self {
            Self::InvalidTarget => UiBindingDiagnosticIdentity {
                error_code: "invalid_target",
                diagnostic_id: "ZUI-BIND-0001",
                localization_key: "diagnostic.ui.binding.invalid_target",
            },
            Self::InvalidValueKind => UiBindingDiagnosticIdentity {
                error_code: "invalid_value_kind",
                diagnostic_id: "ZUI-BIND-0002",
                localization_key: "diagnostic.ui.binding.invalid_value_kind",
            },
            Self::UnresolvedRef => UiBindingDiagnosticIdentity {
                error_code: "unresolved_ref",
                diagnostic_id: "ZUI-BIND-0003",
                localization_key: "diagnostic.ui.binding.unresolved_ref",
            },
            Self::UnsupportedOperator => UiBindingDiagnosticIdentity {
                error_code: "unsupported_operator",
                diagnostic_id: "ZUI-BIND-0004",
                localization_key: "diagnostic.ui.binding.unsupported_operator",
            },
            Self::UnsupportedBindingMode => UiBindingDiagnosticIdentity {
                error_code: "unsupported_binding_mode",
                diagnostic_id: "ZUI-BIND-0005",
                localization_key: "diagnostic.ui.binding.unsupported_binding_mode",
            },
        }
    }

    pub const fn error_code(self) -> &'static str {
        self.identity().error_code
    }

    pub const fn diagnostic_id(self) -> &'static str {
        self.identity().diagnostic_id
    }

    pub const fn localization_key(self) -> &'static str {
        self.identity().localization_key
    }

    pub const fn as_str(self) -> &'static str {
        self.error_code()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiBindingDiagnosticSeverity {
    Error,
    Warning,
}

#[cfg(test)]
mod tests {
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
}
