use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use crate::core::jobs::{CancellationToken, EditorJobSystem};
use zircon_runtime::plugin::native::NativePluginLoadReport;
use zircon_runtime::plugin::{ExportBuildPlan, PluginModuleKind};

use super::artifacts::{
    copy_built_native_artifact, copy_native_artifacts, dynamic_library_file_name,
};
use super::cargo_build::invoke_native_cargo_build_with_cancellation;
use super::cleanup::cleanup_native_dynamic_roots;
use super::native_dynamic_preparation::NativeDynamicPreparation;
use super::package_metadata::{module_crate, sanitize_path_component};
use super::staging::stage_native_package_static_files;
use super::NativeDynamicPreparationError;

pub(in crate::ui::host) fn prepare_native_dynamic_packages_with_cancellation(
    output_root: &Path,
    plan: &ExportBuildPlan,
    native_report: &NativePluginLoadReport,
    jobs: &EditorJobSystem,
    cancel: &CancellationToken,
) -> Result<NativeDynamicPreparation, NativeDynamicPreparationError> {
    let staging_root = output_root.join(".native-dynamic-staging");
    let build_root = output_root.join(".native-dynamic-build");
    let mut cleanup_guard =
        NativeDynamicPreparationGuard::new(staging_root.clone(), build_root.clone());
    let result = (|| -> Result<NativeDynamicPreparation, NativeDynamicPreparationError> {
        cleanup_native_dynamic_roots(&staging_root, &build_root)?;
        fs::create_dir_all(&staging_root).map_err(|error| {
            NativeDynamicPreparationError::io(
                "failed to create staging root",
                "<staging-root>",
                Some(staging_root.clone()),
                error,
            )
        })?;

        let mut cargo_invocations = Vec::new();
        let mut diagnostics = Vec::new();
        let mut staged_package_directories = HashSet::new();
        let discovered_by_plugin_id = native_report
            .discovered
            .iter()
            .map(|candidate| (candidate.plugin_id.as_str(), candidate))
            .collect::<HashMap<_, _>>();
        for package_id in &plan.native_dynamic_packages {
            if cancel.is_cancelled() {
                diagnostics.push(
                    "native dynamic package preparation cancelled before the next package"
                        .to_string(),
                );
                break;
            }
            let Some(candidate) = discovered_by_plugin_id.get(package_id.as_str()).copied() else {
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
                NativeDynamicPreparationError::io(
                    "failed to stage static files",
                    package_id,
                    Some(staged_package.clone()),
                    error,
                )
            })?;
            let artifact_count =
                copy_native_artifacts(&package_root.join("native"), &staged_package.join("native"))
                    .map_err(|error| {
                        NativeDynamicPreparationError::io(
                            "failed to stage existing artifacts",
                            package_id,
                            Some(staged_package.join("native")),
                            error,
                        )
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
            let Some(crate_name) =
                module_crate(&candidate.package_manifest, PluginModuleKind::Runtime).or_else(
                    || module_crate(&candidate.package_manifest, PluginModuleKind::Editor),
                )
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
                jobs,
                cancel,
            )?;
            if invocation.success {
                let artifact = build_target
                    .join("debug")
                    .join(dynamic_library_file_name(&crate_name));
                if artifact.exists() {
                    copy_built_native_artifact(&artifact, &staged_package.join("native")).map_err(
                        |error| {
                            NativeDynamicPreparationError::io(
                                "failed to stage built artifact",
                                package_id,
                                Some(artifact.clone()),
                                error,
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
            if cancel.is_cancelled() {
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
    })();

    match result {
        Ok(preparation) => {
            cleanup_guard.disarm();
            Ok(preparation)
        }
        Err(source) => match cleanup_guard.cleanup() {
            Ok(()) => Err(source),
            Err(cleanup) => Err(source.with_cleanup_failure(cleanup)),
        },
    }
}

struct NativeDynamicPreparationGuard {
    staging_root: PathBuf,
    build_root: PathBuf,
    armed: bool,
}

impl NativeDynamicPreparationGuard {
    fn new(staging_root: PathBuf, build_root: PathBuf) -> Self {
        Self {
            staging_root,
            build_root,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }

    fn cleanup(mut self) -> Result<(), NativeDynamicPreparationError> {
        self.armed = false;
        cleanup_native_dynamic_roots(&self.staging_root, &self.build_root)
    }
}

impl Drop for NativeDynamicPreparationGuard {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let _ = cleanup_native_dynamic_roots(&self.staging_root, &self.build_root);
    }
}

#[cfg(test)]
mod performance_tests {
    #[test]
    fn native_package_preparation_indexes_discovery_once() {
        let source = include_str!("prepare.rs");
        let body = source
            .split("fn prepare_native_dynamic_packages_with_cancellation")
            .nth(1)
            .expect("native package preparation")
            .split("struct NativeDynamicPreparationGuard")
            .next()
            .expect("native package preparation body");
        let repeated_scan = ["native_report", ".discovered", ".iter()", ".find"].concat();

        assert!(body.contains("discovered_by_plugin_id"));
        assert!(!body.contains(&repeated_scan));
    }
}
