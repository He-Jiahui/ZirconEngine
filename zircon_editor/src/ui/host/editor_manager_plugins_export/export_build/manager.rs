use std::path::Path;

use crate::core::jobs::CancellationToken;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::plugin::native::{
    discovery::{
        discover_native_plugins, load_native_editor_from_load_manifest,
        load_native_runtime_from_load_manifest,
    },
    NativePluginLoadReport,
};
use zircon_runtime::plugin::ExportBuildPlan;

use super::super::super::editor_manager::EditorManager;
use super::super::super::native_dynamic_export_preparation::prepare_native_dynamic_packages_with_cancellation;
use super::cargo_build::{invoke_cargo_build, invoke_cargo_build_with_cancellation};
use super::diagnostics::{
    cargo_invocation_diagnostics, cargo_invocation_diagnostics_with_label,
    finalize_export_diagnostics, skipped_export_cargo_build_diagnostic,
};
use super::error::EditorExportBuildError;
use super::generated_files::{should_invoke_cargo, should_probe_exported_native_manifest};
use super::progress::EditorExportBuildProgress;
use super::report::EditorExportBuildReport;

impl EditorManager {
    pub fn generate_export_plan(
        &self,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<ExportBuildPlan, EditorExportBuildError> {
        ExportBuildPlan::from_project_manifest(
            &self.complete_project_plugin_manifest(manifest),
            profile_name,
        )
        .map_err(EditorExportBuildError::from)
    }

    pub fn generate_native_aware_export_plan(
        &self,
        project_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<ExportBuildPlan, EditorExportBuildError> {
        ExportBuildPlan::from_project_manifest(
            &self.complete_native_aware_project_plugin_manifest(project_root, manifest),
            profile_name,
        )
        .map_err(EditorExportBuildError::from)
    }

    pub fn execute_native_aware_export_build(
        &self,
        project_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        let cancel = CancellationToken::default();
        self.execute_native_aware_export_build_with_cancellation(
            project_root,
            output_root,
            manifest,
            profile_name,
            &cancel,
        )
    }

    pub(crate) fn execute_native_aware_export_build_with_cancellation(
        &self,
        project_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
        cancel: &CancellationToken,
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        self.execute_native_aware_export_build_with_cancellation_and_progress(
            project_root,
            output_root,
            manifest,
            profile_name,
            cancel,
            |_| {},
        )
    }

    pub(crate) fn execute_native_aware_export_build_with_cancellation_and_progress<F>(
        &self,
        project_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
        cancel: &CancellationToken,
        mut progress: F,
    ) -> Result<EditorExportBuildReport, EditorExportBuildError>
    where
        F: FnMut(EditorExportBuildProgress),
    {
        emit_export_progress(
            &mut progress,
            "discover-native-packages",
            5,
            "Discovering native dynamic plugin packages",
        );
        let native_report = discover_native_plugins(self.plugin_directory(project_root.as_ref()));
        let native_projection = native_report.projection();
        emit_export_progress(
            &mut progress,
            "resolve-export-plan",
            12,
            format!("Resolving desktop export plan {profile_name}"),
        );
        let plan = ExportBuildPlan::from_project_manifest(
            &self.complete_project_plugin_manifest_with_native_projection(
                manifest,
                &native_projection,
            ),
            profile_name,
        )?;
        if plan.has_fatal_diagnostics() {
            return blocked_native_aware_export_build_report(
                output_root.as_ref(),
                plan,
                &mut progress,
            );
        }
        emit_export_progress(
            &mut progress,
            "prepare-native-packages",
            25,
            "Preparing native dynamic plugin packages",
        );
        let mut native_preparation = prepare_native_dynamic_packages_with_cancellation(
            output_root.as_ref(),
            &plan,
            &native_report,
            self.context().jobs(),
            cancel,
        )?;
        if cancel.is_cancelled() {
            return Err(EditorExportBuildError::cancelled(
                "native dynamic package preparation",
            ));
        }
        emit_export_progress(
            &mut progress,
            "materialize-export",
            45,
            format!(
                "Writing export files after native staging copied {} file(s), {} byte(s), removed {} file(s)",
                native_preparation.staging_stats.copied_files,
                native_preparation.staging_stats.copied_bytes,
                native_preparation.staging_stats.removed_files,
            ),
        );
        let materialized = match plan
            .materialize_with_native_packages(&native_preparation.plugin_root, output_root.as_ref())
        {
            Ok(materialized) => materialized,
            Err(source) => {
                return Err(EditorExportBuildError::materialize(source));
            }
        };
        if cancel.is_cancelled() {
            return Err(EditorExportBuildError::cancelled("export materialization"));
        }
        let cargo_invocation = if should_invoke_cargo(&plan, &materialized.generated_files) {
            emit_export_progress(
                &mut progress,
                "cargo-build",
                72,
                "Running generated SourceTemplate Cargo build",
            );
            Some(invoke_cargo_build_with_cancellation(
                output_root.as_ref(),
                self.context().jobs(),
                cancel,
            )?)
        } else {
            emit_export_progress(
                &mut progress,
                "cargo-build-skipped",
                72,
                "Skipping Cargo build because no generated Cargo.toml was materialized",
            );
            None
        };
        let mut diagnostics = native_report.diagnostics().to_vec();
        diagnostics.extend(native_projection.descriptor_diagnostics().iter().cloned());
        diagnostics.extend(native_projection.entry_diagnostics().iter().cloned());
        diagnostics.extend(std::mem::take(&mut native_preparation.diagnostics));
        for invocation in &native_preparation.cargo_invocations {
            diagnostics.extend(cargo_invocation_diagnostics_with_label(
                invocation,
                "native plugin cargo build",
            ));
        }
        diagnostics.extend(materialized.diagnostics);
        let fatal_diagnostics = materialized.fatal_diagnostics;
        if should_probe_exported_native_manifest(&materialized.generated_files) {
            emit_export_progress(
                &mut progress,
                "probe-exported-native-manifest",
                88,
                "Validating exported native plugin load manifest",
            );
            let exported_native_report = exported_native_load_report_for_profile(
                output_root.as_ref(),
                plan.profile.target_mode,
            );
            let exported_native_projection = exported_native_report.projection();
            diagnostics.extend(exported_native_report.diagnostics().iter().cloned());
            diagnostics.extend(
                exported_native_projection
                    .descriptor_diagnostics()
                    .iter()
                    .cloned(),
            );
            diagnostics.extend(
                exported_native_projection
                    .entry_diagnostics()
                    .iter()
                    .cloned(),
            );
        }
        if let Some(cargo_invocation) = &cargo_invocation {
            diagnostics.extend(cargo_invocation_diagnostics(cargo_invocation));
        } else {
            diagnostics.push(skipped_export_cargo_build_diagnostic(&plan));
        }
        emit_export_progress(
            &mut progress,
            "write-export-diagnostics",
            96,
            "Writing export diagnostics report",
        );
        finalize_export_diagnostics(output_root.as_ref(), &mut diagnostics);
        emit_export_progress(
            &mut progress,
            "complete",
            100,
            "Desktop export build finished",
        );
        let report = EditorExportBuildReport {
            plan,
            invoked_cargo: cargo_invocation.is_some(),
            cargo_invocation,
            native_cargo_invocations: std::mem::take(&mut native_preparation.cargo_invocations),
            generated_files: materialized.generated_files,
            copied_packages: materialized.copied_packages,
            diagnostics,
            fatal_diagnostics,
        };
        drop(native_preparation);
        Ok(report)
    }

    pub fn execute_export_build(
        &self,
        output_root: impl AsRef<Path>,
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<EditorExportBuildReport, EditorExportBuildError> {
        let output_root = output_root.as_ref();
        let plan = self.generate_export_plan(manifest, profile_name)?;
        let materialized = plan
            .materialize(output_root)
            .map_err(EditorExportBuildError::materialize)?;
        let cargo_invocation = if should_invoke_cargo(&plan, &materialized.generated_files) {
            Some(invoke_cargo_build(output_root, self.context().jobs())?)
        } else {
            None
        };
        let mut diagnostics = materialized.diagnostics;
        let fatal_diagnostics = materialized.fatal_diagnostics;
        diagnostics.extend(
            cargo_invocation
                .as_ref()
                .map(cargo_invocation_diagnostics)
                .unwrap_or_else(|| vec![skipped_export_cargo_build_diagnostic(&plan)]),
        );
        finalize_export_diagnostics(output_root, &mut diagnostics);
        Ok(EditorExportBuildReport {
            plan,
            invoked_cargo: cargo_invocation.is_some(),
            cargo_invocation,
            native_cargo_invocations: Vec::new(),
            generated_files: materialized.generated_files,
            copied_packages: materialized.copied_packages,
            diagnostics,
            fatal_diagnostics,
        })
    }
}

fn emit_export_progress(
    progress: &mut impl FnMut(EditorExportBuildProgress),
    stage: impl Into<String>,
    percent: u8,
    message: impl Into<String>,
) {
    progress(EditorExportBuildProgress::new(stage, percent, message));
}

fn exported_native_load_report_for_profile(
    output_root: &Path,
    target_mode: RuntimeTargetMode,
) -> NativePluginLoadReport {
    match target_mode {
        RuntimeTargetMode::ClientRuntime | RuntimeTargetMode::ServerRuntime => {
            load_native_runtime_from_load_manifest(output_root)
        }
        RuntimeTargetMode::EditorHost => load_native_editor_from_load_manifest(output_root),
    }
}

fn blocked_native_aware_export_build_report(
    output_root: &Path,
    plan: ExportBuildPlan,
    progress: &mut impl FnMut(EditorExportBuildProgress),
) -> Result<EditorExportBuildReport, EditorExportBuildError> {
    emit_export_progress(
        progress,
        "materialize-export",
        45,
        "Skipping export materialization because the export plan has fatal diagnostics",
    );
    let materialized = plan
        .materialize(output_root)
        .map_err(EditorExportBuildError::materialize)?;
    let mut diagnostics = materialized.diagnostics;
    let fatal_diagnostics = materialized.fatal_diagnostics;
    diagnostics.push(skipped_export_cargo_build_diagnostic(&plan));
    emit_export_progress(
        progress,
        "write-export-diagnostics",
        96,
        "Writing export diagnostics report",
    );
    finalize_export_diagnostics(output_root, &mut diagnostics);
    emit_export_progress(
        progress,
        "complete",
        100,
        "Desktop export build finished with fatal diagnostics",
    );

    Ok(EditorExportBuildReport {
        plan,
        invoked_cargo: false,
        cargo_invocation: None,
        native_cargo_invocations: Vec::new(),
        generated_files: materialized.generated_files,
        copied_packages: materialized.copied_packages,
        diagnostics,
        fatal_diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn exported_native_probe_uses_target_mode_specific_loader() {
        let root = temp_export_root("editor-export-native-target-mode-probe");
        let package_root = root.join("plugins/split_tool");
        fs::create_dir_all(&package_root).unwrap();
        fs::write(
            package_root.join("plugin.toml"),
            split_native_plugin_manifest(),
        )
        .unwrap();
        fs::write(
            root.join("plugins/native_plugins.toml"),
            r#"
[[plugins]]
id = "split_tool"
path = "plugins/split_tool"
manifest = "plugins/split_tool/plugin.toml"
"#,
        )
        .unwrap();

        let runtime_report =
            exported_native_load_report_for_profile(&root, RuntimeTargetMode::ClientRuntime);
        assert!(runtime_report.diagnostics().iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_runtime",
            ))
        }));
        assert!(!runtime_report.diagnostics().iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_editor",
            ))
        }));

        let editor_report =
            exported_native_load_report_for_profile(&root, RuntimeTargetMode::EditorHost);
        assert!(editor_report.diagnostics().iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_editor",
            ))
        }));
        assert!(!editor_report.diagnostics().iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_runtime",
            ))
        }));

        let _ = fs::remove_dir_all(root);
    }

    fn split_native_plugin_manifest() -> &'static str {
        r#"
id = "split_tool"
version = "0.1.0"
display_name = "Split Tool"

[[modules]]
name = "split_tool.runtime"
kind = "runtime"
crate_name = "zircon_plugin_split_tool_runtime"

[[modules]]
name = "split_tool.editor"
kind = "editor"
crate_name = "zircon_plugin_split_tool_editor"
"#
    }

    fn temp_export_root(label: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zircon-{label}-{stamp}"))
    }

    fn platform_library_file_name(crate_name: &str) -> String {
        #[cfg(target_os = "windows")]
        {
            format!("{crate_name}.dll")
        }
        #[cfg(target_os = "macos")]
        {
            format!("lib{crate_name}.dylib")
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            format!("lib{crate_name}.so")
        }
    }
}
