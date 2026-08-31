use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use super::cargo_invocation::EditorExportCargoInvocation;
use zircon_runtime::core::framework::project::ExportPlatformHostKind;
use zircon_runtime::plugin::ExportBuildPlan;

pub(super) fn finalize_export_diagnostics(output_root: &Path, diagnostics: &mut Vec<String>) {
    normalize_export_diagnostics(diagnostics);
    write_export_diagnostics(output_root, diagnostics);
    normalize_export_diagnostics(diagnostics);
}

fn write_export_diagnostics(output_root: &Path, diagnostics: &mut Vec<String>) {
    if let Err(error) = fs::create_dir_all(output_root) {
        diagnostics.push(format!(
            "failed to create export diagnostics directory {}: {error}",
            output_root.display()
        ));
        return;
    }
    let path = output_root.join("export-diagnostics.txt");
    if let Err(error) = fs::write(&path, diagnostics.join("\n")) {
        diagnostics.push(format!(
            "failed to write export diagnostics {}: {error}",
            path.display()
        ));
    }
}

fn normalize_export_diagnostics(diagnostics: &mut Vec<String>) {
    let mut seen = HashSet::new();
    diagnostics.retain(|diagnostic| {
        let diagnostic = diagnostic.trim();
        !diagnostic.is_empty() && seen.insert(diagnostic.to_string())
    });
}

pub(super) fn cargo_invocation_diagnostics(
    invocation: &EditorExportCargoInvocation,
) -> Vec<String> {
    cargo_invocation_diagnostics_with_label(invocation, "export cargo build")
}

pub(super) fn skipped_export_cargo_build_diagnostic(plan: &ExportBuildPlan) -> String {
    match plan.platform_policy.host_kind {
        ExportPlatformHostKind::Desktop => {
            "export cargo build skipped because no generated Cargo.toml was materialized"
                .to_string()
        }
        ExportPlatformHostKind::MobileApp => format!(
            "export cargo build skipped because target platform {} emits a mobile host scaffold that must be built by the platform package toolchain",
            plan.profile.target_platform.as_str()
        ),
        ExportPlatformHostKind::Browser => format!(
            "export cargo build skipped because target platform {} emits a browser host scaffold that must be built by the web/WASM package toolchain",
            plan.profile.target_platform.as_str()
        ),
        ExportPlatformHostKind::Headless => {
            "export cargo build skipped because no generated Cargo.toml was materialized"
                .to_string()
        }
    }
}

pub(in super::super) fn cargo_invocation_diagnostics_with_label(
    invocation: &EditorExportCargoInvocation,
    label: &str,
) -> Vec<String> {
    if invocation.success {
        return vec![successful_cargo_invocation_diagnostic(
            label,
            &invocation.command,
        )];
    }

    let mut diagnostics = vec![failed_cargo_invocation_diagnostic(
        label,
        invocation.status_code,
        &invocation.command,
    )];
    if !invocation.stderr.trim().is_empty() {
        diagnostics.push(invocation.stderr.trim().to_string());
    } else if !invocation.stdout.trim().is_empty() {
        diagnostics.push(invocation.stdout.trim().to_string());
    }
    diagnostics
}

fn successful_cargo_invocation_diagnostic(label: &str, command: &[String]) -> String {
    const SUCCEEDED: &str = " succeeded: ";
    let mut diagnostic =
        String::with_capacity(label.len() + SUCCEEDED.len() + joined_command_len(command));
    diagnostic.push_str(label);
    diagnostic.push_str(SUCCEEDED);
    push_command(&mut diagnostic, command);
    diagnostic
}

fn failed_cargo_invocation_diagnostic(
    label: &str,
    status_code: Option<i32>,
    command: &[String],
) -> String {
    const FAILED: &str = " failed with status ";
    const STATUS_SUFFIX: &str = ": ";
    const MAX_OPTION_I32_DEBUG_LEN: usize = 17;

    let mut diagnostic = String::with_capacity(
        label.len()
            + FAILED.len()
            + MAX_OPTION_I32_DEBUG_LEN
            + STATUS_SUFFIX.len()
            + joined_command_len(command),
    );
    diagnostic.push_str(label);
    diagnostic.push_str(FAILED);
    write!(&mut diagnostic, "{status_code:?}").expect("writing to a String cannot fail");
    diagnostic.push_str(STATUS_SUFFIX);
    push_command(&mut diagnostic, command);
    diagnostic
}

fn joined_command_len(command: &[String]) -> usize {
    command.iter().map(String::len).sum::<usize>() + command.len().saturating_sub(1)
}

fn push_command(diagnostic: &mut String, command: &[String]) {
    if let Some((first, remaining)) = command.split_first() {
        diagnostic.push_str(first);
        for argument in remaining {
            diagnostic.push(' ');
            diagnostic.push_str(argument);
        }
    }
}

#[cfg(test)]
#[path = "diagnostics/command_buffer_tests.rs"]
mod command_buffer_tests;
