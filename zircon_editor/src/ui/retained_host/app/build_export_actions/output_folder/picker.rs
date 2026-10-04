mod commands;
mod selection;

use std::path::{Path, PathBuf};
use std::process::Command;

use super::super::DesktopExportActionError;

pub(super) use commands::folder_picker_commands;
pub(super) use selection::parse_selected_folder;
pub(in crate::ui::retained_host::app::build_export_actions) use selection::stable_picker_initial_dir;

pub(in crate::ui::retained_host::app::build_export_actions) fn pick_output_folder(
    initial_dir: &Path,
) -> Result<Option<PathBuf>, DesktopExportActionError> {
    let commands = folder_picker_commands(initial_dir)?;
    let mut missing_commands = Vec::with_capacity(commands.len());
    for (program, args) in commands {
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
                return Err(DesktopExportActionError::PickerExit {
                    program,
                    status_code: output.status.code(),
                    stderr,
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing_commands.push(program);
            }
            Err(error) => {
                return Err(DesktopExportActionError::PickerSpawn {
                    program,
                    source: error,
                });
            }
        }
    }

    Err(DesktopExportActionError::PickerUnavailable {
        programs: missing_commands,
    })
}

#[cfg(test)]
#[path = "tests/picker_optimization_batch_20260830bs_editor_tests.rs"]
mod optimization_batch_20260830bs_editor_tests;
