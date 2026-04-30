use std::path::Path;
use std::process::Command;

use super::cargo_invocation::EditorExportCargoInvocation;

pub(super) fn invoke_cargo_build(
    output_root: &Path,
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
    let output = Command::new(&cargo)
        .args(&args)
        .current_dir(output_root)
        .output()
        .map_err(|error| format!("failed to invoke cargo for export build: {error}"))?;

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
