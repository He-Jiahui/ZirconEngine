use zircon_runtime_interface::ui::template::UiComponentContractDiagnostic;

use super::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};

pub fn map_component_contract_diagnostic(
    diagnostic: UiComponentContractDiagnostic,
) -> UiAssetEditorDiagnostic {
    let mut editor = UiAssetEditorDiagnostic::new(
        diagnostic.code.as_str(),
        UiAssetEditorDiagnosticSeverity::Error,
        diagnostic.message,
        diagnostic.path,
    );
    editor.target_node_id = diagnostic.target_node_id;
    editor.target_control_id = diagnostic.target_control_id;
    editor
}
