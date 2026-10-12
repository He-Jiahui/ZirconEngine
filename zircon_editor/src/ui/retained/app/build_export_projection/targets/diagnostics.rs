use std::path::Path;

pub(super) fn prepend_desktop_export_output_diagnostic(
    output_root: &Path,
    diagnostics: impl Into<String>,
) -> String {
    let diagnostics = diagnostics.into();
    if diagnostics.is_empty() {
        format!("Output: {}", output_root.display())
    } else {
        format!("Output: {}\n{diagnostics}", output_root.display())
    }
}
