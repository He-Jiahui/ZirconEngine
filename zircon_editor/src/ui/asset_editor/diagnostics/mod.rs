mod binding;
mod contract;
mod localization;

pub use binding::map_binding_diagnostic;
pub use contract::map_component_contract_diagnostic;
pub use localization::map_localization_diagnostic;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiAssetEditorDiagnosticSeverity {
    #[default]
    Error,
    Warning,
}

impl UiAssetEditorDiagnosticSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorDiagnostic {
    pub code: String,
    pub severity: UiAssetEditorDiagnosticSeverity,
    pub message: String,
    pub source_path: String,
    pub target_node_id: Option<String>,
    pub target_control_id: Option<String>,
    pub target_binding_id: Option<String>,
}

impl UiAssetEditorDiagnostic {
    pub fn new(
        code: impl Into<String>,
        severity: UiAssetEditorDiagnosticSeverity,
        message: impl Into<String>,
        source_path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity,
            message: message.into(),
            source_path: source_path.into(),
            target_node_id: None,
            target_control_id: None,
            target_binding_id: None,
        }
    }
}
