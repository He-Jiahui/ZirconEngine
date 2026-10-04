use super::picker::{folder_picker_commands, parse_selected_folder, stable_picker_initial_dir};
use super::reveal::reveal_path_command;
use std::path::{Path, PathBuf};

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
