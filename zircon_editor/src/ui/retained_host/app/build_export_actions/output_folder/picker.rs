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
mod optimization_batch_20260830bs_editor_tests {
    use std::time::Instant;

    const SAMPLE_PAIRS: usize = 17;
    const COMMANDS_PER_SAMPLE: usize = 2;

    #[test]
    fn output_folder_picker_reserves_missing_command_capacity() {
        let source = include_str!("picker.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(implementation.contains("let commands = folder_picker_commands(initial_dir)?;"));
        assert!(implementation.contains("Vec::with_capacity(commands.len())"));
        assert!(!implementation.contains("let mut missing_commands = Vec::new()"));
    }

    #[test]
    fn output_folder_picker_keeps_missing_programs_in_command_order() {
        let source = include_str!("picker.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        let reserve = implementation
            .find("Vec::with_capacity(commands.len())")
            .expect("missing command capacity reservation");
        let loop_start = implementation
            .find("for (program, args) in commands")
            .expect("picker command loop");
        let push = implementation
            .find("missing_commands.push(program)")
            .expect("missing command push");
        assert!(reserve < loop_start);
        assert!(loop_start < push);
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830bs_editor_output_folder_capacity_p95() {
        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(false));
                optimized.push(measure(true));
            } else {
                optimized.push(measure(true));
                legacy.push(measure(false));
            }
        }
        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "EDITOR317_OUTPUT_FOLDER_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} commands_per_sample={COMMANDS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            sample_csv(&legacy),
            sample_csv(&optimized),
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0usize;
        for _ in 0..256 {
            let mut missing = if optimized {
                Vec::with_capacity(COMMANDS_PER_SAMPLE)
            } else {
                Vec::new()
            };
            for index in 0..COMMANDS_PER_SAMPLE {
                missing.push(index);
            }
            checksum ^= missing.len();
        }
        std::hint::black_box(checksum);
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn sample_csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
