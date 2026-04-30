use std::path::Path;
use std::process::Command;

use super::super::editor_manager_plugins_export::EditorExportCargoInvocation;

pub(super) fn invoke_native_cargo_build(
    manifest_path: &Path,
    target_dir: &Path,
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
    let output = Command::new(&cargo)
        .args(&args)
        .output()
        .map_err(|error| format!("failed to invoke cargo for native dynamic plugin: {error}"))?;

    let mut command = Vec::with_capacity(args.len() + 1);
    command.push(cargo);
    command.extend(args);

    Ok(EditorExportCargoInvocation {
        command,
        status_code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}
