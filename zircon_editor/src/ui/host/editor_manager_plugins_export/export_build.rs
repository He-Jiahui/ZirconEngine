use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{ExportBuildPlan, NativePluginLoader};

use super::super::editor_manager::EditorManager;
use super::super::native_dynamic_export_preparation::{
    cleanup_native_dynamic_preparation, prepare_native_dynamic_packages,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorExportBuildReport {
    pub plan: ExportBuildPlan,
    pub invoked_cargo: bool,
    pub cargo_invocation: Option<EditorExportCargoInvocation>,
    pub native_cargo_invocations: Vec<EditorExportCargoInvocation>,
    pub generated_files: Vec<PathBuf>,
    pub copied_packages: Vec<PathBuf>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorExportCargoInvocation {
    pub command: Vec<String>,
    pub status_code: Option<i32>,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

impl EditorManager {
    pub fn generate_export_plan(
        &self,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<ExportBuildPlan, String> {
        ExportBuildPlan::from_project_manifest(
            &self.complete_project_plugin_manifest(manifest),
            profile_name,
        )
    }

    pub fn generate_native_aware_export_plan(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<ExportBuildPlan, String> {
        ExportBuildPlan::from_project_manifest(
            &self.complete_native_aware_project_plugin_manifest(project_root, manifest),
            profile_name,
        )
    }

    pub fn execute_native_aware_export_build(
        &self,
        project_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<EditorExportBuildReport, String> {
        let native_report =
            NativePluginLoader.discover(self.plugin_directory(project_root.as_ref()));
        let plan =
            self.generate_native_aware_export_plan(project_root.as_ref(), manifest, profile_name)?;
        let native_preparation =
            prepare_native_dynamic_packages(output_root.as_ref(), &plan, &native_report)?;
        let materialized = plan
            .materialize_with_native_packages(&native_preparation.plugin_root, output_root.as_ref())
            .map_err(|error| error.to_string())?;
        let cleanup_diagnostics = cleanup_native_dynamic_preparation(&native_preparation);
        let cargo_invocation = if should_invoke_cargo(&materialized.generated_files) {
            Some(invoke_cargo_build(output_root.as_ref())?)
        } else {
            None
        };
        let mut diagnostics = native_report.diagnostics.clone();
        diagnostics.extend(native_report.descriptor_diagnostics());
        diagnostics.extend(native_report.entry_diagnostics());
        diagnostics.extend(native_preparation.diagnostics);
        for invocation in &native_preparation.cargo_invocations {
            diagnostics.extend(cargo_invocation_diagnostics_with_label(
                invocation,
                "native plugin cargo build",
            ));
        }
        diagnostics.extend(materialized.diagnostics);
        diagnostics.extend(cleanup_diagnostics);
        if should_probe_exported_native_manifest(&materialized.generated_files) {
            let exported_native_report =
                NativePluginLoader.load_from_load_manifest(output_root.as_ref());
            diagnostics.extend(exported_native_report.diagnostics.iter().cloned());
            diagnostics.extend(exported_native_report.descriptor_diagnostics());
            diagnostics.extend(exported_native_report.entry_diagnostics());
        }
        if let Some(cargo_invocation) = &cargo_invocation {
            diagnostics.extend(cargo_invocation_diagnostics(cargo_invocation));
        } else {
            diagnostics.push(
                "export cargo build skipped because no generated Cargo.toml was materialized"
                    .to_string(),
            );
        }
        write_export_diagnostics(output_root.as_ref(), &mut diagnostics);
        Ok(EditorExportBuildReport {
            plan,
            invoked_cargo: cargo_invocation.is_some(),
            cargo_invocation,
            native_cargo_invocations: native_preparation.cargo_invocations,
            generated_files: materialized.generated_files,
            copied_packages: materialized.copied_packages,
            diagnostics,
        })
    }

    pub fn execute_export_build(
        &self,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<EditorExportBuildReport, String> {
        let output_root = output_root.as_ref();
        let plan = self.generate_export_plan(manifest, profile_name)?;
        let materialized = plan
            .materialize(output_root)
            .map_err(|error| error.to_string())?;
        let cargo_invocation = if should_invoke_cargo(&materialized.generated_files) {
            Some(invoke_cargo_build(output_root)?)
        } else {
            None
        };
        let mut diagnostics = cargo_invocation
            .as_ref()
            .map(cargo_invocation_diagnostics)
            .unwrap_or_else(|| {
                vec![
                    "export cargo build skipped because no generated Cargo.toml was materialized"
                        .to_string(),
                ]
            });
        write_export_diagnostics(output_root, &mut diagnostics);
        Ok(EditorExportBuildReport {
            plan,
            invoked_cargo: cargo_invocation.is_some(),
            cargo_invocation,
            native_cargo_invocations: Vec::new(),
            generated_files: materialized.generated_files,
            copied_packages: materialized.copied_packages,
            diagnostics,
        })
    }
}

fn invoke_cargo_build(output_root: &Path) -> Result<EditorExportCargoInvocation, String> {
    let manifest_path = output_root.join("Cargo.toml");
    if !manifest_path.exists() {
        return Ok(EditorExportCargoInvocation {
            command: Vec::new(),
            status_code: None,
            success: false,
            stdout: String::new(),
            stderr: format!(
                "export Cargo manifest is missing: {}",
                manifest_path.display()
            ),
        });
    }

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let args = vec![
        "build".to_string(),
        "--manifest-path".to_string(),
        manifest_path.display().to_string(),
        "--locked".to_string(),
    ];
    let output = Command::new(&cargo)
        .args(&args)
        .current_dir(output_root)
        .output()
        .map_err(|error| format!("failed to invoke cargo for export build: {error}"))?;

    let mut command = Vec::with_capacity(args.len() + 1);
    command.push(cargo);
    command.extend(args);

    Ok(EditorExportCargoInvocation {
        command,
        status_code: output.status.code(),
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn should_invoke_cargo(generated_files: &[PathBuf]) -> bool {
    generated_files
        .iter()
        .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("Cargo.toml"))
}

fn should_probe_exported_native_manifest(generated_files: &[PathBuf]) -> bool {
    generated_files
        .iter()
        .any(|path| path.file_name().and_then(|name| name.to_str()) == Some("native_plugins.toml"))
}

fn write_export_diagnostics(output_root: &Path, diagnostics: &mut Vec<String>) {
    let path = output_root.join("export-diagnostics.txt");
    if let Err(error) = fs::write(&path, diagnostics.join("\n")) {
        diagnostics.push(format!(
            "failed to write export diagnostics {}: {error}",
            path.display()
        ));
    }
}

fn cargo_invocation_diagnostics(invocation: &EditorExportCargoInvocation) -> Vec<String> {
    cargo_invocation_diagnostics_with_label(invocation, "export cargo build")
}

fn cargo_invocation_diagnostics_with_label(
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
