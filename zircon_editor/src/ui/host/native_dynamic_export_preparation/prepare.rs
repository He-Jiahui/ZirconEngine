use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use zircon_runtime::{
    plugin::ExportBuildPlan, plugin::NativePluginLoadReport, plugin::PluginModuleKind,
};

use super::artifacts::{
    copy_built_native_artifact, copy_native_artifacts, dynamic_library_file_name,
};
use super::cargo_build::invoke_native_cargo_build_with_cancellation;
use super::native_dynamic_preparation::NativeDynamicPreparation;
use super::package_metadata::{module_crate, sanitize_path_component};
use super::staging::stage_native_package_static_files;

pub(in crate::ui::host) fn prepare_native_dynamic_packages_with_cancellation(
    output_root: &Path,
    plan: &ExportBuildPlan,
    native_report: &NativePluginLoadReport,
    cancel_requested: Option<&AtomicBool>,
) -> Result<NativeDynamicPreparation, String> {
    let staging_root = output_root.join(".native-dynamic-staging");
    let build_root = output_root.join(".native-dynamic-build");
    if staging_root.exists() {
        fs::remove_dir_all(&staging_root).map_err(|error| {
            format!(
                "failed to clear native dynamic staging root {}: {error}",
                staging_root.display()
            )
        })?;
    }
    if build_root.exists() {
        fs::remove_dir_all(&build_root).map_err(|error| {
            format!(
                "failed to clear native dynamic build root {}: {error}",
                build_root.display()
            )
        })?;
    }
    fs::create_dir_all(&staging_root).map_err(|error| {
        format!(
            "failed to create native dynamic staging root {}: {error}",
            staging_root.display()
        )
    })?;

    let mut cargo_invocations = Vec::new();
    let mut diagnostics = Vec::new();
    let mut staged_package_directories = HashSet::new();
    for package_id in &plan.native_dynamic_packages {
        if cancellation_requested(cancel_requested) {
            diagnostics.push(
                "native dynamic package preparation cancelled before the next package".to_string(),
            );
            break;
        }
        let Some(candidate) = native_report
            .discovered
            .iter()
            .find(|candidate| &candidate.plugin_id == package_id)
        else {
            diagnostics.push(format!(
                "native dynamic package {package_id} has no discovered package manifest for artifact staging"
            ));
            continue;
        };
        let Some(package_root) = candidate.manifest_path.parent() else {
            diagnostics.push(format!(
                "native dynamic package {package_id} manifest has no parent directory"
            ));
            continue;
        };
        let package_directory = sanitize_path_component(package_id);
        if !staged_package_directories.insert(package_directory.clone()) {
            diagnostics.push(format!(
                "native dynamic package {package_id} resolves to duplicate staging directory {package_directory}"
            ));
            continue;
        }
        let staged_package = staging_root.join(&package_directory);
        stage_native_package_static_files(package_root, &staged_package).map_err(|error| {
            format!(
                "failed to stage native dynamic package {package_id} into {}: {error}",
                staged_package.display()
            )
        })?;
        let artifact_count =
            copy_native_artifacts(&package_root.join("native"), &staged_package.join("native"))
                .map_err(|error| {
                    format!("failed to stage native dynamic artifacts for {package_id}: {error}")
                })?;
        if artifact_count > 0 {
            diagnostics.push(format!(
                "native dynamic package {package_id} staged {artifact_count} existing native artifact(s)"
            ));
            continue;
        }

        let native_manifest_path = package_root.join("native/Cargo.toml");
        if !native_manifest_path.exists() {
            continue;
        }
        let Some(crate_name) = module_crate(&candidate.package_manifest, PluginModuleKind::Runtime)
            .or_else(|| module_crate(&candidate.package_manifest, PluginModuleKind::Editor))
        else {
            diagnostics.push(format!(
                "native dynamic package {package_id} has native Cargo.toml but no runtime or editor crate name"
            ));
            continue;
        };
        let build_target = build_root.join(&package_directory);
        let invocation = invoke_native_cargo_build_with_cancellation(
            &native_manifest_path,
            &build_target,
            cancel_requested,
        )?;
        if invocation.success {
            let artifact = build_target
                .join("debug")
                .join(dynamic_library_file_name(&crate_name));
            if artifact.exists() {
                copy_built_native_artifact(&artifact, &staged_package.join("native")).map_err(
                    |error| {
                        format!(
                            "failed to stage built native artifact {}: {error}",
                            artifact.display()
                        )
                    },
                )?;
            } else {
                diagnostics.push(format!(
                    "native dynamic package {package_id} cargo build succeeded but artifact was missing: {}",
                    artifact.display()
                ));
            }
        }
        cargo_invocations.push(invocation);
        if cancellation_requested(cancel_requested) {
            diagnostics.push(
                "native dynamic package preparation cancelled after Cargo returned".to_string(),
            );
            break;
        }
    }

    Ok(NativeDynamicPreparation {
        plugin_root: staging_root,
        build_root,
        cargo_invocations,
        diagnostics,
    })
}

fn cancellation_requested(cancel_requested: Option<&AtomicBool>) -> bool {
    cancel_requested.is_some_and(|cancel_requested| cancel_requested.load(Ordering::SeqCst))
}
