use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::super::super::export_cargo_process::invoke_cargo_process;
use super::cargo_invocation::EditorExportCargoInvocation;

pub(super) fn invoke_cargo_build(
    output_root: &Path,
) -> Result<EditorExportCargoInvocation, String> {
    invoke_cargo_build_with_cancellation(output_root, None)
}

pub(super) fn invoke_cargo_build_with_cancellation(
    output_root: &Path,
    cancel_requested: Option<&AtomicBool>,
) -> Result<EditorExportCargoInvocation, String> {
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
    invoke_cargo_process(
        cargo,
        args,
        Some(output_root),
        cancel_requested,
        "export build",
    )
}
