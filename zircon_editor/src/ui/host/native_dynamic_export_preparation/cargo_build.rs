use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::super::editor_manager_plugins_export::EditorExportCargoInvocation;
use super::super::export_cargo_process::invoke_cargo_process;

pub(super) fn invoke_native_cargo_build_with_cancellation(
    manifest_path: &Path,
    target_dir: &Path,
    cancel_requested: Option<&AtomicBool>,
) -> Result<EditorExportCargoInvocation, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "--locked".to_string(),
        "--target-dir".to_string(),
        target_dir.display().to_string(),
    ];
    invoke_cargo_process(cargo, args, None, cancel_requested, "native dynamic plugin")
}
