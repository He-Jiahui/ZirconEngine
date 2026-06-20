mod commands;
mod selection;

use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) use commands::folder_picker_commands;
pub(super) use selection::parse_selected_folder;
pub(in crate::ui::retained_host::app::build_export_actions) use selection::stable_picker_initial_dir;

pub(in crate::ui::retained_host::app::build_export_actions) fn pick_output_folder(
    initial_dir: &Path,
) -> Result<Option<PathBuf>, String> {
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
