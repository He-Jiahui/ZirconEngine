use crate::asset::project::ProjectManifest;
use crate::{
    core::framework::project::ExportPackagingStrategy, core::framework::project::ExportProfile,
    core::framework::project::ProjectPluginSelection,
};

use super::asset_manifest_template::asset_manifest_template;
use super::cargo_manifest_template::cargo_manifest_template;
use super::native_dynamic_package_plan::NativeDynamicPackageExportPlan;
use super::native_plugin_load_manifest_template::native_plugin_load_manifest_template;
use super::platform_host_files::platform_host_files;
use super::plugin_selection_template::plugin_selection_template;
use super::{ExportGeneratedFile, ExportLinkedRuntimeCrate};

#[cfg(test)]
#[path = "generated_files/capacity_tests.rs"]
mod capacity_tests;

pub(super) fn generated_files_for_profile(
    manifest: &ProjectManifest,
    profile: &ExportProfile,
    project_plugin_selections: &[&ProjectPluginSelection],
    linked_runtime_crates: &[ExportLinkedRuntimeCrate],
    native_dynamic_packages: &[NativeDynamicPackageExportPlan],
) -> Vec<ExportGeneratedFile> {
    let native_dynamic_file = (!native_dynamic_packages.is_empty()).then(|| ExportGeneratedFile {
        path: "plugins/native_plugins.toml".to_string(),
        purpose: "native dynamic plugin loading manifest".to_string(),
        contents: native_plugin_load_manifest_template(native_dynamic_packages),
    });

    if !source_template_enabled(&profile.strategies) {
        return native_dynamic_file.into_iter().collect();
    }

    let has_native_dynamic_plugins = native_dynamic_file.is_some();
    let platform_files = platform_host_files(profile, has_native_dynamic_plugins);
    let mut files = Vec::with_capacity(generated_profile_file_capacity(
        platform_files.len(),
        has_native_dynamic_plugins,
    ));
    files.extend(native_dynamic_file);

    files.extend([
        ExportGeneratedFile {
            path: "Cargo.toml".to_string(),
            purpose: "generated runtime package manifest".to_string(),
            contents: cargo_manifest_template(profile, linked_runtime_crates),
        },
        ExportGeneratedFile {
            path: "src/zircon_plugins.rs".to_string(),
            purpose: "generated plugin selection code".to_string(),
            contents: plugin_selection_template(
                profile,
                project_plugin_selections,
                linked_runtime_crates,
            ),
        },
        ExportGeneratedFile {
            path: "assets/zircon-project.toml".to_string(),
            purpose: "project runtime manifest copy".to_string(),
            contents: asset_manifest_template(manifest),
        },
    ]);
    files.extend(platform_files);
    files
}

fn generated_profile_file_capacity(
    platform_file_count: usize,
    has_native_dynamic_plugins: bool,
) -> usize {
    platform_file_count
        .saturating_add(3)
        .saturating_add(usize::from(has_native_dynamic_plugins))
}

pub(super) fn source_template_enabled(strategies: &[ExportPackagingStrategy]) -> bool {
    strategies.contains(&ExportPackagingStrategy::SourceTemplate)
}
