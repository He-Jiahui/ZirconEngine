use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::cargo_invocation::EditorExportCargoInvocation;
use zircon_runtime::plugin::{ExportBuildPlan, ExportPlatformHostKind};

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
            "export cargo build skipped because no generated Cargo.toml was materialized".to_string()
        }
        ExportPlatformHostKind::MobileApp => format!(
            "export cargo build skipped because target platform {} emits a mobile host scaffold that must be built by the platform package toolchain",
            plan.profile.target_platform.as_str()
        ),
        ExportPlatformHostKind::Browser => format!(
            "export cargo build skipped because target platform {} emits a browser host scaffold that must be built by the web/WASM package toolchain",
            plan.profile.target_platform.as_str()
        ),
    }
}

pub(in super::super) fn cargo_invocation_diagnostics_with_label(
    invocation: &EditorExportCargoInvocation,
    label: &str,
) -> Vec<String> {
    if invocation.success {
        return vec![format!(
            "{label} succeeded: {}",
            invocation.command.join(" ")
        )];
    }

    let mut diagnostics = vec![format!(
        "{label} failed with status {:?}: {}",
        invocation.status_code,
        invocation.command.join(" ")
    )];
    if !invocation.stderr.trim().is_empty() {
        diagnostics.push(invocation.stderr.trim().to_string());
    } else if !invocation.stdout.trim().is_empty() {
        diagnostics.push(invocation.stdout.trim().to_string());
    }
    diagnostics
}
