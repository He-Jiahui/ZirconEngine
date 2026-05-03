use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn reveal_path_in_file_browser(path: &Path) -> Result<(), String> {
    let (program, args) = reveal_path_command(path)?;
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            format!(
                "failed to open desktop export output folder {}: {error}",
                path.display()
            )
        })
}

pub(super) fn pick_output_folder(initial_dir: &Path) -> Result<Option<PathBuf>, String> {
    let mut missing_commands = Vec::new();
    for (program, args) in folder_picker_commands(initial_dir)? {
        match Command::new(program).args(args).output() {
            Ok(output) if output.status.success() => {
                return Ok(parse_selected_folder(&output.stdout));
            }
            Ok(output) => {
                if output.stdout.is_empty() && output.stderr.is_empty() {
                    return Ok(None);
                }
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    return Ok(None);
                }
                return Err(format!("folder picker failed: {stderr}"));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing_commands.push(program);
            }
            Err(error) => {
                return Err(format!(
                    "failed to start folder picker `{program}`: {error}"
                ));
            }
        }
    }

    Err(format!(
        "no desktop folder picker command was available ({})",
        missing_commands.join(", ")
    ))
}

pub(super) fn stable_picker_initial_dir(preferred: &Path, fallback: &Path) -> PathBuf {
    preferred
        .ancestors()
        .find(|ancestor| ancestor.is_dir())
        .unwrap_or(fallback)
        .to_path_buf()
}

fn parse_selected_folder(stdout: &[u8]) -> Option<PathBuf> {
    let selected = String::from_utf8_lossy(stdout).trim().to_string();
    (!selected.is_empty()).then(|| PathBuf::from(selected))
}

fn reveal_path_command(path: &Path) -> Result<(&'static str, Vec<OsString>), String> {
    #[cfg(target_os = "windows")]
    {
        return Ok(("explorer.exe", vec![path.as_os_str().to_os_string()]));
    }
    #[cfg(target_os = "macos")]
    {
        return Ok(("open", vec![path.as_os_str().to_os_string()]));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return Ok(("xdg-open", vec![path.as_os_str().to_os_string()]));
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    {
        let _ = path;
        Err("opening desktop export output folders is unsupported on this host".to_string())
    }
}

fn folder_picker_commands(
    initial_dir: &Path,
) -> Result<Vec<(&'static str, Vec<OsString>)>, String> {
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
        Err("choosing desktop export output folders is unsupported on this host".to_string())
    }
}

#[cfg(target_os = "windows")]
fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

#[cfg(target_os = "macos")]
fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_export_reveal_output_uses_host_file_browser_command() {
        let (program, args) = reveal_path_command(Path::new("Builds/zircon/desktop_windows"))
            .expect("host should expose a file-browser command");
        #[cfg(target_os = "windows")]
        assert_eq!(program, "explorer.exe");
        #[cfg(target_os = "macos")]
        assert_eq!(program, "open");
        #[cfg(all(unix, not(target_os = "macos")))]
        assert_eq!(program, "xdg-open");
        assert_eq!(args.len(), 1);
        assert!(args[0].to_string_lossy().contains("Builds"));
    }

    #[test]
    fn desktop_export_folder_picker_uses_native_host_command() {
        let commands = folder_picker_commands(Path::new("Builds/zircon/desktop_windows"))
            .expect("host should expose at least one folder picker command");
        #[cfg(target_os = "windows")]
        assert_eq!(commands[0].0, "powershell.exe");
        #[cfg(target_os = "macos")]
        assert_eq!(commands[0].0, "osascript");
        #[cfg(all(unix, not(target_os = "macos")))]
        assert!(commands
            .iter()
            .any(|(program, _)| *program == "zenity" || *program == "kdialog"));
        assert!(!commands[0].1.is_empty());
    }

    #[test]
    fn desktop_export_folder_picker_parses_selected_folder() {
        assert_eq!(
            parse_selected_folder(b"Builds/zircon/desktop_windows\n"),
            Some(PathBuf::from("Builds/zircon/desktop_windows"))
        );
        assert_eq!(parse_selected_folder(b"\n"), None);
    }

    #[test]
    fn desktop_export_folder_picker_starts_from_existing_parent() {
        let fallback = std::env::current_dir().expect("test should have a current directory");
        let missing_child = fallback
            .join("missing-export-picker-parent")
            .join("missing-export-picker-child");

        assert_eq!(
            stable_picker_initial_dir(&missing_child, &fallback),
            fallback
        );
    }
}
