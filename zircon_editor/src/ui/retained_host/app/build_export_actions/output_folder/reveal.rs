use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

use super::super::DesktopExportActionError;

pub(in crate::ui::retained_host::app::build_export_actions) fn reveal_path_in_file_browser(
    path: &Path,
) -> Result<(), DesktopExportActionError> {
    let (program, args) = reveal_path_command(path)?;
    Command::new(program)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|source| DesktopExportActionError::RevealSpawn {
            path: path.to_path_buf(),
            source,
        })
}

pub(super) fn reveal_path_command(
    path: &Path,
) -> Result<(&'static str, Vec<OsString>), DesktopExportActionError> {
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
        Err(DesktopExportActionError::RevealUnsupported)
    }
}
