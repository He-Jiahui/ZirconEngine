use std::path::PathBuf;

use crate::error::HubError;

pub struct FolderPickerRequest {
    title: String,
    initial_dir: Option<PathBuf>,
}

impl FolderPickerRequest {
    pub fn new(title: impl Into<String>, initial_dir: Option<PathBuf>) -> Self {
        Self {
            title: title.into(),
            initial_dir,
        }
    }
}

#[cfg(windows)]
pub fn pick_folder(request: &FolderPickerRequest) -> Result<Option<PathBuf>, HubError> {
    let script = r#"
Add-Type -AssemblyName System.Windows.Forms
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.Description = $env:ZIRCON_HUB_PICKER_TITLE
$dialog.ShowNewFolderButton = $true
$initial = $env:ZIRCON_HUB_PICKER_INITIAL
if ($initial -and [System.IO.Directory]::Exists($initial)) {
    $dialog.SelectedPath = $initial
}
$result = $dialog.ShowDialog()
if ($result -eq [System.Windows.Forms.DialogResult]::OK) {
    Write-Output $dialog.SelectedPath
    exit 0
}
exit 2
"#;
    let mut command = std::process::Command::new("powershell");
    command
        .arg("-NoProfile")
        .arg("-STA")
        .arg("-Command")
        .arg(script)
        .env("ZIRCON_HUB_PICKER_TITLE", &request.title);
    if let Some(initial_dir) = &request.initial_dir {
        command.env("ZIRCON_HUB_PICKER_INITIAL", initial_dir);
    }
    let output = command.output()?;
    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            return Ok(None);
        }
        return Ok(Some(PathBuf::from(selected)));
    }
    if output.status.code() == Some(2) {
        return Ok(None);
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(HubError::message(if stderr.is_empty() {
        "Folder picker failed"
    } else {
        stderr.as_str()
    }))
}

#[cfg(not(windows))]
pub fn pick_folder(_request: &FolderPickerRequest) -> Result<Option<PathBuf>, HubError> {
    Err(HubError::message(
        "Folder picker is not implemented on this platform yet",
    ))
}
