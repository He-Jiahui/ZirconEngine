use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::core::export::ExportGenerationInventory;
use crate::core::jobs::{CancellationToken, EditorJobSystem};
use zircon_runtime::plugin::native::NativePluginLoadReport;
use zircon_runtime::plugin::{ExportBuildPlan, PluginModuleKind};

use super::artifacts::{dynamic_library_file_name, sync_built_native_artifact};
use super::cargo_build::invoke_native_cargo_build_with_cancellation;
use super::native_dynamic_preparation::NativeDynamicPreparation;
use super::package_metadata::{module_crate, sanitize_path_component};
use super::staging::{prune_stale_packages, sync_native_package, NativeStagingStats};
use super::NativeDynamicPreparationError;

const NATIVE_DYNAMIC_CACHE_ROOT: &str = ".zircon/cache/export/native-dynamic";
const EXPORT_FILE_INVENTORY_CACHE: &str = ".zircon/cache/export/file-inventory-v1.json";

pub(in crate::ui::host) fn prepare_native_dynamic_packages_with_cancellation(
    output_root: &Path,
    plan: &ExportBuildPlan,
    native_report: &NativePluginLoadReport,
    jobs: &EditorJobSystem,
    cancel: &CancellationToken,
) -> Result<NativeDynamicPreparation, NativeDynamicPreparationError> {
    let selected_package_ids = plan
        .native_dynamic_packages
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut discovered_by_plugin_id = HashMap::with_capacity(native_report.discovered().len());
    for candidate in native_report.discovered() {
        let role = candidate.package_manifest.package_role;
        if selected_package_ids.contains(candidate.plugin_id.as_str())
            && !role.is_product_catalog_eligible()
        {
            return Err(NativeDynamicPreparationError::IneligibleProductPackage {
                package_id: candidate.plugin_id.clone(),
                role,
            });
        }
        discovered_by_plugin_id.insert(candidate.plugin_id.as_str(), candidate);
    }

    let cache_root = output_root.join(NATIVE_DYNAMIC_CACHE_ROOT);
    let staging_root = cache_root.join("packages");
    let manifests_root = cache_root.join("manifests");
    let build_root = cache_root.join("build");
    fs::create_dir_all(&staging_root).map_err(|error| {
        NativeDynamicPreparationError::io(
            "failed to create persistent staging root",
            "<staging-root>",
            Some(staging_root.clone()),
            error,
        )
    })?;
    fs::create_dir_all(&manifests_root).map_err(|error| {
        NativeDynamicPreparationError::io(
            "failed to create persistent staging manifest root",
            "<manifest-root>",
            Some(manifests_root.clone()),
            error,
        )
    })?;

    let mut inventory = ExportGenerationInventory::with_persistent_cache(
        output_root.join(EXPORT_FILE_INVENTORY_CACHE),
    );
    let mut cargo_invocations = Vec::with_capacity(plan.native_dynamic_packages.len());
    let mut diagnostics = Vec::new();
    let mut staging_stats = NativeStagingStats::default();
    let mut staged_package_directories = BTreeSet::new();
    for package_id in &plan.native_dynamic_packages {
        if cancel.is_cancelled() {
            diagnostics.push(
                "native dynamic package preparation cancelled before the next package".to_string(),
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
        let package_stats = sync_native_package(
            package_root,
            &staged_package,
            &manifests_root.join(format!("{package_directory}.json")),
            &mut inventory,
        )
        .map_err(|error| {
            NativeDynamicPreparationError::io(
                "failed to synchronize native package staging delta",
                package_id,
                Some(staged_package.clone()),
                error,
            )
        })?;
        let artifact_count = package_stats.existing_artifact_count;
        staging_stats.merge(package_stats);
        if artifact_count > 0 {
            diagnostics.push(format!(
                "native dynamic package {package_id} has {artifact_count} staged native artifact(s)"
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
            jobs,
            cancel,
        )?;
        if invocation.success {
            let artifact = build_target
                .join("debug")
                .join(dynamic_library_file_name(&crate_name));
            if artifact.exists() {
                let artifact_stats = sync_built_native_artifact(
                    &artifact,
                    &staged_package.join("native"),
                    &mut inventory,
                )
                .map_err(|error| {
                    NativeDynamicPreparationError::io(
                        "failed to synchronize built native artifact",
                        package_id,
                        Some(artifact.clone()),
                        error,
                    )
                })?;
                staging_stats.merge(artifact_stats);
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

    if !cancel.is_cancelled() {
        let removed =
            prune_stale_packages(&staging_root, &manifests_root, &staged_package_directories)
                .map_err(|error| {
                    NativeDynamicPreparationError::io(
                        "failed to prune stale native package staging entries",
                        "<staging-root>",
                        Some(staging_root.clone()),
                        error,
                    )
                })?;
        staging_stats.merge(removed);
    }
    diagnostics.push(format!(
        "native dynamic staging delta copied_files={} copied_bytes={} removed_files={}",
        staging_stats.copied_files, staging_stats.copied_bytes, staging_stats.removed_files
    ));
    Ok(NativeDynamicPreparation {
        plugin_root: staging_root,
        build_root,
        cargo_invocations,
        diagnostics,
        staging_stats,
    })
}

#[cfg(test)]
#[path = "tests/prepare_admission_tests.rs"]
mod admission_tests;

#[cfg(test)]
#[path = "tests/prepare_performance_tests.rs"]
mod performance_tests;
