use zircon_runtime_interface::ui::template::{UiBindingDiagnostic, UiBindingDiagnosticSeverity};

use super::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};

pub fn map_binding_diagnostic(diagnostic: UiBindingDiagnostic) -> UiAssetEditorDiagnostic {
    let mut editor = UiAssetEditorDiagnostic::new(
        diagnostic.code.as_str(),
        map_binding_severity(diagnostic.severity),
        diagnostic.message,
        diagnostic.path,
    );
    editor.target_node_id = Some(diagnostic.node_id);
    editor.target_binding_id = Some(diagnostic.binding_id);
    editor
}

const fn map_binding_severity(
    severity: UiBindingDiagnosticSeverity,
) -> UiAssetEditorDiagnosticSeverity {
    match severity {
        UiBindingDiagnosticSeverity::Error => UiAssetEditorDiagnosticSeverity::Error,
        UiBindingDiagnosticSeverity::Warning => UiAssetEditorDiagnosticSeverity::Warning,
    }
}
