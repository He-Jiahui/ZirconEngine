use std::path::Path;

use crate::core::jobs::{CancellationToken, EditorJobSystem};

use super::super::super::export_cargo_process::invoke_cargo_process;
use super::cargo_invocation::EditorExportCargoInvocation;
use super::error::EditorExportBuildError;

pub(super) fn invoke_cargo_build(
    output_root: &Path,
    jobs: &EditorJobSystem,
) -> Result<EditorExportCargoInvocation, EditorExportBuildError> {
    let cancel = CancellationToken::default();
    invoke_cargo_build_with_cancellation(output_root, jobs, &cancel)
}

pub(super) fn invoke_cargo_build_with_cancellation(
    output_root: &Path,
    jobs: &EditorJobSystem,
    cancel: &CancellationToken,
) -> Result<EditorExportCargoInvocation, EditorExportBuildError> {
    let manifest_path = output_root.join("Cargo.toml");
    if !manifest_path.exists() {
        return Ok(EditorExportCargoInvocation {
            command: Vec::new(),
            status_code: None,
            success: false,
            stdout: String::new(),
            stderr: format!(
                "export Cargo manifest is missing: {}",
                manifest_path.display()
            ),
        });
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "--locked".to_string(),
    ];
    invoke_cargo_process(jobs, cargo, args, Some(output_root), cancel, "export build")
        .map_err(EditorExportBuildError::cargo)
}
