use std::path::Path;

use crate::core::jobs::{CancellationToken, EditorJobSystem};

use super::super::editor_manager_plugins_export::EditorExportCargoInvocation;
use super::super::export_cargo_process::invoke_cargo_process;
use super::NativeDynamicPreparationError;

pub(super) fn invoke_native_cargo_build_with_cancellation(
    manifest_path: &Path,
    target_dir: &Path,
    jobs: &EditorJobSystem,
    cancel: &CancellationToken,
) -> Result<EditorExportCargoInvocation, NativeDynamicPreparationError> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "--locked".to_string(),
        "--target-dir".to_string(),
        target_dir.display().to_string(),
    ];
    invoke_cargo_process(jobs, cargo, args, None, cancel, "native dynamic plugin")
        .map_err(NativeDynamicPreparationError::from)
}
