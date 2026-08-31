use std::ffi::OsString;
use std::path::Path;

use super::super::super::DesktopExportActionError;

pub(in crate::ui::retained_host::app::build_export_actions::output_folder) fn folder_picker_commands(
    initial_dir: &Path,
) -> Result<Vec<(&'static str, Vec<OsString>)>, DesktopExportActionError> {
    #[cfg(target_os = "windows")]
    {
        let selected_path = powershell_single_quoted(&initial_dir.to_string_lossy());
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $dialog = New-Object System.Windows.Forms.FolderBrowserDialog; \
             $dialog.Description = 'Choose Zircon desktop export folder'; \
             $dialog.SelectedPath = {selected_path}; \
             if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {{ \
                 [Console]::Out.WriteLine($dialog.SelectedPath) \
             }}"
        );
        return Ok(vec![(
            "powershell.exe",
            vec![
                "-NoProfile".into(),
                "-STA".into(),
                "-Command".into(),
                script.into(),
            ],
        )]);
    }

    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "POSIX path of (choose folder with prompt \"Choose Zircon desktop export folder\" default location POSIX file {})",
            apple_script_string(&initial_dir.to_string_lossy())
        );
        return Ok(vec![("osascript", vec!["-e".into(), script.into()])]);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let mut filename = initial_dir.as_os_str().to_os_string();
        filename.push(std::path::MAIN_SEPARATOR.to_string());
        return Ok(vec![
            (
                "zenity",
                vec![
                    "--file-selection".into(),
                    "--directory".into(),
                    "--filename".into(),
                    filename,
                ],
            ),
            (
                "kdialog",
                vec![
                    "--getexistingdirectory".into(),
                    initial_dir.as_os_str().to_os_string(),
                ],
            ),
        ]);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        let _ = initial_dir;
        Err(DesktopExportActionError::PickerUnsupported)
    }
}

#[cfg(target_os = "windows")]
fn powershell_single_quoted(value: &str) -> String {
    let capacity = value
        .bytes()
        .filter(|byte| *byte == b'\'')
        .count()
        .saturating_add(value.len())
        .saturating_add(2);
    let mut quoted = String::with_capacity(capacity);
    quoted.push('\'');
    for character in value.chars() {
        if character == '\'' {
            quoted.push_str("''");
        } else {
            quoted.push(character);
        }
    }
    quoted.push('\'');
    quoted
}

#[cfg(target_os = "macos")]
fn apple_script_string(value: &str) -> String {
    let capacity = value
        .chars()
        .filter(|character| matches!(character, '\\' | '"'))
        .count()
        .saturating_add(value.len())
        .saturating_add(2);
    let mut quoted = String::with_capacity(capacity);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            _ => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
#[path = "commands/single_buffer_quoting_tests.rs"]
mod single_buffer_quoting_tests;
