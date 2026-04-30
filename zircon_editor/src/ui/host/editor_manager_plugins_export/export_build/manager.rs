use std::path::Path;

use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{
    ExportBuildPlan, NativePluginLoadReport, NativePluginLoader, RuntimeTargetMode,
};

use super::super::super::editor_manager::EditorManager;
use super::super::super::native_dynamic_export_preparation::{
    cleanup_native_dynamic_preparation, prepare_native_dynamic_packages,
};
use super::cargo_build::invoke_cargo_build;
use super::diagnostics::{
    cargo_invocation_diagnostics, cargo_invocation_diagnostics_with_label,
    finalize_export_diagnostics, skipped_export_cargo_build_diagnostic,
};
use super::generated_files::{should_invoke_cargo, should_probe_exported_native_manifest};
use super::report::EditorExportBuildReport;

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
            let exported_native_report = exported_native_load_report_for_profile(
                output_root.as_ref(),
                plan.profile.target_mode,
            );
            diagnostics.extend(exported_native_report.diagnostics.iter().cloned());
            diagnostics.extend(exported_native_report.descriptor_diagnostics());
            diagnostics.extend(exported_native_report.entry_diagnostics());
        }
        if let Some(cargo_invocation) = &cargo_invocation {
            diagnostics.extend(cargo_invocation_diagnostics(cargo_invocation));
        } else {
            diagnostics.push(skipped_export_cargo_build_diagnostic());
        }
        finalize_export_diagnostics(output_root.as_ref(), &mut diagnostics);
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
        let mut diagnostics = materialized.diagnostics;
        diagnostics.extend(
            cargo_invocation
                .as_ref()
                .map(cargo_invocation_diagnostics)
                .unwrap_or_else(|| vec![skipped_export_cargo_build_diagnostic()]),
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
        })
    }
}

fn exported_native_load_report_for_profile(
    output_root: &Path,
    target_mode: RuntimeTargetMode,
) -> NativePluginLoadReport {
    match target_mode {
        RuntimeTargetMode::ClientRuntime | RuntimeTargetMode::ServerRuntime => {
            NativePluginLoader.load_runtime_from_load_manifest(output_root)
        }
        RuntimeTargetMode::EditorHost => {
            NativePluginLoader.load_editor_from_load_manifest(output_root)
        }
    }
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
        assert!(runtime_report.diagnostics.iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_runtime",
            ))
        }));
        assert!(!runtime_report.diagnostics.iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_editor",
            ))
        }));

        let editor_report =
            exported_native_load_report_for_profile(&root, RuntimeTargetMode::EditorHost);
        assert!(editor_report.diagnostics.iter().any(|message| {
            message.contains(&platform_library_file_name(
                "zircon_plugin_split_tool_editor",
            ))
        }));
        assert!(!editor_report.diagnostics.iter().any(|message| {
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
