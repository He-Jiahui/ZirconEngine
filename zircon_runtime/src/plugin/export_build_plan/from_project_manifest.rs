use crate::asset::project::ProjectManifest;
use crate::ExportPackagingStrategy;

use super::default_profile::default_profile;
use super::generated_files::generated_files_for_profile;
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
        let linked_runtime_crates = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.runtime_crate.is_some()
                    && selection.packaging != ExportPackagingStrategy::NativeDynamic
                    && (profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
                        || profile.uses_strategy(ExportPackagingStrategy::SourceTemplate))
            })
            .map(|selection| selection.runtime_crate_name())
            .collect::<Vec<_>>();
        let native_dynamic_packages = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.packaging == ExportPackagingStrategy::NativeDynamic
                    || profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
            })
            .map(|selection| selection.id.clone())
            .collect::<Vec<_>>();
        let generated_files = generated_files_for_profile(
            manifest,
            &profile,
            &enabled_plugins,
            &project_plugin_selections,
            &linked_runtime_crates,
            &native_dynamic_packages,
        );

        Ok(Self::new(
            profile,
            &enabled_plugins,
            linked_runtime_crates,
            native_dynamic_packages,
            generated_files,
        ))
    }
}
