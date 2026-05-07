use zircon_runtime_interface::ui::template::{
    UiLocalizationDiagnostic, UiLocalizationDiagnosticSeverity,
};

use super::{UiAssetEditorDiagnostic, UiAssetEditorDiagnosticSeverity};

const LOCALIZATION_INVALID_REF_CODE: &str = "localization_invalid_ref";

pub fn map_localization_diagnostic(
    diagnostic: UiLocalizationDiagnostic,
) -> UiAssetEditorDiagnostic {
    let code = editor_localization_code(&diagnostic).to_string();
    let source_path = diagnostic.path;
    let mut editor = UiAssetEditorDiagnostic::new(
        code,
        map_localization_severity(diagnostic.severity),
        diagnostic.message,
        source_path.clone(),
    );
    editor.target_node_id = node_id_from_localization_path(&source_path);
    editor
}

fn editor_localization_code(diagnostic: &UiLocalizationDiagnostic) -> &str {
    match diagnostic.code.as_str() {
        "" | "empty_localized_text_key" => LOCALIZATION_INVALID_REF_CODE,
        code => code,
    }
}

const fn map_localization_severity(
    severity: UiLocalizationDiagnosticSeverity,
) -> UiAssetEditorDiagnosticSeverity {
    match severity {
        UiLocalizationDiagnosticSeverity::Error => UiAssetEditorDiagnosticSeverity::Error,
        UiLocalizationDiagnosticSeverity::Warning => UiAssetEditorDiagnosticSeverity::Warning,
    }
}

fn node_id_from_localization_path(path: &str) -> Option<String> {
    let mut segments = path.split('.');
    match (segments.next(), segments.next()) {
        (Some("nodes"), Some(node_id)) if !node_id.is_empty() => Some(node_id.to_string()),
        _ => None,
    }
}
