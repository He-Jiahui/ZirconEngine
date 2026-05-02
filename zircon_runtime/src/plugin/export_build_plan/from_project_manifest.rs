use std::collections::HashSet;

use crate::asset::project::ProjectManifest;
use crate::{plugin::ExportPackagingStrategy, plugin::RuntimePluginCatalog};

use super::cargo_manifest_template::plugin_path_for_runtime_crate;
use super::default_profile::default_profile;
use super::generated_files::generated_files_for_profile;
use super::native_plugin_load_manifest_template::native_dynamic_package_directory;
use super::{ExportBuildPlan, ExportLinkedRuntimeCrate};

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

        let catalog = RuntimePluginCatalog::builtin();
        let completed_plugins = catalog.complete_project_manifest(&manifest.plugins);
        let enabled_plugins = manifest
            .plugins
            .enabled_for_target(profile.target_mode)
            .collect::<Vec<_>>();
        let project_plugin_selections = manifest.plugins.selections.iter().collect::<Vec<_>>();
        let mut linked_runtime_crate_names = HashSet::new();
        let mut linked_runtime_crate_links = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.runtime_crate.is_some()
                    && !selection.is_runtime_builtin_domain()
                    && selection.packaging != ExportPackagingStrategy::NativeDynamic
                    && profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            })
            .map(|selection| selection.runtime_crate_name())
            .filter(|crate_name| linked_runtime_crate_names.insert(crate_name.clone()))
            .map(|crate_name| {
                let path = format!("{}/runtime", plugin_path_for_runtime_crate(&crate_name));
                ExportLinkedRuntimeCrate::runtime_plugin(crate_name, path)
            })
            .collect::<Vec<_>>();
        let feature_report =
            catalog.feature_dependency_report(&completed_plugins, profile.target_mode);
        for feature_id in &feature_report.available_features {
            let Some((owner, feature)) = feature_selection(&completed_plugins, feature_id) else {
                continue;
            };
            if feature.runtime_crate.is_none()
                || feature.packaging == ExportPackagingStrategy::NativeDynamic
                || !profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            {
                continue;
            }
            let crate_name = feature.runtime_crate_name();
            if linked_runtime_crate_names.insert(crate_name.clone()) {
                linked_runtime_crate_links.push(ExportLinkedRuntimeCrate::runtime_feature(
                    crate_name,
                    feature.runtime_crate_path(&owner.id),
                ));
            }
        }
        let linked_runtime_crates = linked_runtime_crate_links
            .iter()
            .map(|linked_crate| linked_crate.crate_name.clone())
            .collect::<Vec<_>>();
        let mut native_dynamic_package_ids = HashSet::new();
        let mut native_dynamic_package_directories = HashSet::new();
        let mut native_dynamic_packages = Vec::new();
        let mut native_dynamic_diagnostics = Vec::new();
        for selection in enabled_plugins.iter().filter(|selection| {
            selection.packaging == ExportPackagingStrategy::NativeDynamic
                && !selection.is_runtime_builtin_domain()
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
        let mut fatal_diagnostics = Vec::new();
        diagnostics.extend(native_dynamic_diagnostics);
        if profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            || profile.uses_strategy(ExportPackagingStrategy::SourceTemplate)
        {
            diagnostics.extend(feature_report.diagnostics.iter().cloned());
            fatal_diagnostics.extend(feature_report.diagnostics.iter().cloned());
            for blocked in &feature_report.blocked_features {
                let diagnostic = blocked.to_diagnostic();
                if blocked.required {
                    fatal_diagnostics.push(diagnostic.clone());
                }
                diagnostics.push(diagnostic);
            }
        }
        diagnostics.extend(
            enabled_plugins
                .iter()
                .filter(|selection| {
                    !selection.is_runtime_builtin_domain()
                        && selection.packaging != ExportPackagingStrategy::NativeDynamic
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
            &linked_runtime_crate_links,
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
        plan.fatal_diagnostics = fatal_diagnostics;
        Ok(plan)
    }
}

fn feature_selection<'a>(
    manifest: &'a crate::plugin::ProjectPluginManifest,
    feature_id: &str,
) -> Option<(
    &'a crate::plugin::ProjectPluginSelection,
    &'a crate::plugin::ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        selection
            .features
            .iter()
            .find(|feature| feature.id == feature_id)
            .map(|feature| (selection, feature))
    })
}
