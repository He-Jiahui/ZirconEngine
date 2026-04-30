use std::collections::HashSet;

use crate::asset::project::ProjectManifest;
use crate::ExportPackagingStrategy;

use super::default_profile::default_profile;
use super::generated_files::generated_files_for_profile;
use super::native_plugin_load_manifest_template::native_dynamic_package_directory;
use super::ExportBuildPlan;

impl ExportBuildPlan {
    pub fn from_project_manifest(
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<Self, String> {
        let profile = manifest
            .export_profiles
            .iter()
            .find(|profile| profile.name == profile_name)
            .cloned()
            .or_else(|| default_profile(profile_name))
            .ok_or_else(|| format!("missing export profile {profile_name}"))?;

        let enabled_plugins = manifest
            .plugins
            .enabled_for_target(profile.target_mode)
            .collect::<Vec<_>>();
        let project_plugin_selections = manifest.plugins.selections.iter().collect::<Vec<_>>();
        let mut linked_runtime_crate_names = HashSet::new();
        let linked_runtime_crates = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.runtime_crate.is_some()
                    && selection.packaging != ExportPackagingStrategy::NativeDynamic
                    && profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            })
            .map(|selection| selection.runtime_crate_name())
            .filter(|crate_name| linked_runtime_crate_names.insert(crate_name.clone()))
            .collect::<Vec<_>>();
        let mut native_dynamic_package_ids = HashSet::new();
        let mut native_dynamic_package_directories = HashSet::new();
        let mut native_dynamic_packages = Vec::new();
        let mut native_dynamic_diagnostics = Vec::new();
        for selection in enabled_plugins.iter().filter(|selection| {
            selection.packaging == ExportPackagingStrategy::NativeDynamic
                && profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
        }) {
            if !native_dynamic_package_ids.insert(selection.id.clone()) {
                continue;
            }
            let package_directory = native_dynamic_package_directory(&selection.id);
            if !native_dynamic_package_directories.insert(package_directory.clone()) {
                native_dynamic_diagnostics.push(format!(
                    "plugin {} uses NativeDynamic packaging but resolves to duplicate output directory plugins/{package_directory}",
                    selection.id
                ));
                continue;
            }
            native_dynamic_packages.push(selection.id.clone());
        }
        let mut diagnostics = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.packaging == ExportPackagingStrategy::NativeDynamic
                    && !profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
            })
            .map(|selection| {
                format!(
                    "plugin {} uses NativeDynamic packaging but export profile {} does not enable NativeDynamic",
                    selection.id, profile.name
                )
            })
            .collect::<Vec<_>>();
        diagnostics.extend(native_dynamic_diagnostics);
        diagnostics.extend(
            enabled_plugins
                .iter()
                .filter(|selection| {
                    selection.packaging != ExportPackagingStrategy::NativeDynamic
                        && !profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
                        && !profile.uses_strategy(ExportPackagingStrategy::SourceTemplate)
                })
                .map(|selection| {
                    format!(
                        "plugin {} uses LibraryEmbed packaging but export profile {} does not enable LibraryEmbed or SourceTemplate",
                        selection.id, profile.name
                    )
                }),
        );
        let generated_files = generated_files_for_profile(
            manifest,
            &profile,
            &project_plugin_selections,
            &linked_runtime_crates,
            &native_dynamic_packages,
        );

        let mut plan = Self::new(
            profile,
            &enabled_plugins,
            linked_runtime_crates,
            native_dynamic_packages,
            generated_files,
        );
        plan.diagnostics = diagnostics;
        Ok(plan)
    }
}
