use std::collections::HashSet;

use crate::asset::project::ProjectManifest;
use crate::{
    plugin::ExportPackagingStrategy, plugin::RuntimePluginCatalog, plugin::RuntimePluginDescriptor,
};

mod feature_selection;
mod profile;
mod profile_projection;

use self::feature_selection::{
    external_feature_selection, external_feature_selections, feature_selection,
};
use self::profile::runtime_profile_for_export_profile;
use self::profile_projection::{
    export_profile_selection_diagnostics, project_plugins_for_export_profile,
};
use super::cargo_manifest_template::plugin_path_for_runtime_crate;
use super::default_profile::default_profile;
use super::export_profile_validation::{
    export_profile_duplicate_name_fatal_diagnostics, export_profile_name_fatal_diagnostics,
    export_profile_output_name_diagnostics,
    export_profile_runtime_profile_target_fatal_diagnostics, export_profile_strategy_diagnostics,
    export_profile_strategy_fatal_diagnostics, sanitize_export_profile_output_name,
    sanitize_export_profile_strategies,
};
use super::generated_files::generated_files_for_profile;
use super::library_embed_compile_plan::library_embed_compile_host_plan;
use super::native_dynamic_package_plan::NativeDynamicPackageAccumulator;
use super::project_manifest_validation::{
    project_duplicate_selection_diagnostics, project_editor_crate_diagnostics,
    project_feature_id_diagnostics, project_feature_provider_package_id_diagnostics,
    project_plugin_package_id_diagnostics, project_plugin_package_id_is_valid,
    project_runtime_crate_diagnostics, project_runtime_crate_name_is_valid,
    project_runtime_crate_override_is_valid, project_target_mode_diagnostics,
    sanitize_invalid_project_crate_overrides, sanitize_invalid_project_provider_package_overrides,
    sanitize_project_identity_rows, sanitize_project_target_mode_rows,
};
use super::source_template_build_plan::source_template_build_validation_plan;
use super::{ExportBuildPlan, ExportLinkedRuntimeCrate};

impl ExportBuildPlan {
    pub fn from_project_manifest(
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<Self, String> {
        let mut profile = manifest
            .export_profiles
            .iter()
            .find(|profile| profile.name == profile_name)
            .cloned()
            .or_else(|| default_profile(profile_name))
            .ok_or_else(|| format!("missing export profile {profile_name}"))?;
        let profile_duplicate_name_fatal_diagnostics =
            export_profile_duplicate_name_fatal_diagnostics(
                &manifest.export_profiles,
                profile_name,
            );
        let profile_name_fatal_diagnostics = export_profile_name_fatal_diagnostics(&profile);
        let profile_output_name_diagnostics = export_profile_output_name_diagnostics(&profile);
        let profile_strategy_diagnostics = export_profile_strategy_diagnostics(&profile);
        let profile_strategy_fatal_diagnostics =
            export_profile_strategy_fatal_diagnostics(&profile);
        sanitize_export_profile_output_name(&mut profile);
        sanitize_export_profile_strategies(&mut profile);
        let platform_policy = profile.target_platform.policy();

        let catalog = RuntimePluginCatalog::builtin();
        let mut completed_plugins = catalog.complete_project_manifest(&manifest.plugins);
        let profile_selection_diagnostics =
            export_profile_selection_diagnostics(&profile, &completed_plugins);
        project_plugins_for_export_profile(&profile, &mut completed_plugins);
        // Keep diagnostics source-faithful while preventing malformed crate tokens from
        // leaking into generated SourceTemplate project metadata.
        sanitize_invalid_project_crate_overrides(&mut completed_plugins, profile.target_mode);
        let enabled_plugins = completed_plugins
            .enabled_for_target(profile.target_mode)
            .collect::<Vec<_>>();
        let (project_plugin_id_diagnostics, project_plugin_id_fatal_diagnostics) =
            project_plugin_package_id_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_feature_id_diagnostics, project_feature_id_fatal_diagnostics) =
            project_feature_id_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_duplicate_diagnostics, project_duplicate_fatal_diagnostics) =
            project_duplicate_selection_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_runtime_crate_diagnostics, project_runtime_crate_fatal_diagnostics) =
            project_runtime_crate_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_editor_crate_diagnostics, project_editor_crate_fatal_diagnostics) =
            project_editor_crate_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_target_mode_diagnostics, project_target_mode_fatal_diagnostics) =
            project_target_mode_diagnostics(&manifest.plugins, profile.target_mode);
        let mut linked_runtime_crate_names = HashSet::new();
        let mut linked_runtime_package_ids = HashSet::new();
        let mut linked_runtime_crate_links = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.runtime_crate.is_some()
                    && !selection.is_runtime_builtin_domain()
                    && project_plugin_package_id_is_valid(&selection.id)
                    && project_runtime_crate_override_is_valid(selection.runtime_crate.as_deref())
                    && selection.packaging != ExportPackagingStrategy::NativeDynamic
                    && linked_rust_strategy_enabled(&profile)
            })
            .map(|selection| selection.runtime_crate_name())
            .filter(|crate_name| linked_runtime_crate_names.insert(crate_name.clone()))
            .map(|crate_name| {
                let path = format!("{}/runtime", plugin_path_for_runtime_crate(&crate_name));
                ExportLinkedRuntimeCrate::runtime_plugin(crate_name, path)
            })
            .collect::<Vec<_>>();
        for selection in enabled_plugins.iter().filter(|selection| {
            selection.runtime_crate.is_some()
                && !selection.is_runtime_builtin_domain()
                && project_plugin_package_id_is_valid(&selection.id)
                && project_runtime_crate_override_is_valid(selection.runtime_crate.as_deref())
                && selection.packaging != ExportPackagingStrategy::NativeDynamic
                && linked_rust_strategy_enabled(&profile)
        }) {
            linked_runtime_package_ids.insert(selection.id.clone());
        }
        let mut native_dynamic_package_accumulator = NativeDynamicPackageAccumulator::default();
        let feature_report =
            catalog.feature_dependency_report(&completed_plugins, profile.target_mode);
        let (project_provider_diagnostics, project_provider_fatal_diagnostics) =
            project_feature_provider_package_id_diagnostics(&manifest.plugins, profile.target_mode);
        let mut feature_packaging_diagnostics = Vec::new();
        let mut feature_packaging_fatal_diagnostics = Vec::new();
        for feature_id in &feature_report.available_features {
            let Some((owner, feature)) = feature_selection(&completed_plugins, feature_id) else {
                continue;
            };
            let provider_package_id = feature.provider_package_id_or_owner(&owner.id);
            let external_provider = provider_package_id != owner.id;
            if feature.packaging == ExportPackagingStrategy::NativeDynamic {
                let diagnostic = if !platform_policy.supports_native_dynamic {
                    Some(format!(
                        "optional feature {} uses NativeDynamic packaging but target platform {} does not support NativeDynamic; use LibraryEmbed, SourceTemplate, or VM packaging",
                        feature.id,
                        profile.target_platform.as_str()
                    ))
                } else if !profile.uses_strategy(ExportPackagingStrategy::NativeDynamic) {
                    Some(format!(
                        "optional feature {} uses NativeDynamic packaging but export profile {} does not enable NativeDynamic",
                        feature.id, profile.name
                    ))
                } else if external_provider {
                    native_dynamic_package_accumulator.push(provider_package_id);
                    None
                } else if owner.packaging != ExportPackagingStrategy::NativeDynamic {
                    Some(format!(
                        "optional feature {} uses NativeDynamic packaging but owner plugin {} is not NativeDynamic; native dynamic feature packages are exported through their owner plugin package",
                        feature.id, owner.id
                    ))
                } else {
                    None
                };
                if let Some(diagnostic) = diagnostic {
                    if feature.required {
                        feature_packaging_fatal_diagnostics.push(diagnostic.clone());
                    }
                    feature_packaging_diagnostics.push(diagnostic);
                }
                continue;
            }
            if feature.runtime_crate.is_none() || !linked_rust_strategy_enabled(&profile) {
                continue;
            }
            let crate_name = feature.runtime_crate_name();
            if !project_runtime_crate_name_is_valid(&crate_name) {
                continue;
            }
            if linked_runtime_crate_names.insert(crate_name.clone()) {
                linked_runtime_package_ids.insert(provider_package_id.to_string());
                linked_runtime_crate_links.push(
                    ExportLinkedRuntimeCrate::runtime_feature_with_provider(
                        crate_name,
                        feature.runtime_crate_path(&owner.id),
                        external_provider.then(|| provider_package_id.to_string()),
                    ),
                );
            }
        }
        for (owner, feature) in external_feature_selections(&completed_plugins, profile.target_mode)
        {
            if feature_report
                .available_features
                .iter()
                .any(|feature_id| feature_id == &feature.id)
            {
                continue;
            }
            let Some(provider_package_id) = feature.external_provider_package_id(&owner.id) else {
                continue;
            };
            if feature.packaging == ExportPackagingStrategy::NativeDynamic {
                let diagnostic = if !platform_policy.supports_native_dynamic {
                    Some(format!(
                        "optional feature {} uses NativeDynamic packaging but target platform {} does not support NativeDynamic; use LibraryEmbed, SourceTemplate, or VM packaging",
                        feature.id,
                        profile.target_platform.as_str()
                    ))
                } else if !profile.uses_strategy(ExportPackagingStrategy::NativeDynamic) {
                    Some(format!(
                        "optional feature {} uses NativeDynamic packaging but export profile {} does not enable NativeDynamic",
                        feature.id, profile.name
                    ))
                } else {
                    native_dynamic_package_accumulator.push(provider_package_id);
                    None
                };
                if let Some(diagnostic) = diagnostic {
                    if feature.required {
                        feature_packaging_fatal_diagnostics.push(diagnostic.clone());
                    }
                    feature_packaging_diagnostics.push(diagnostic);
                }
                continue;
            }
            if feature.runtime_crate.is_some() && linked_rust_strategy_enabled(&profile) {
                let crate_name = feature.runtime_crate_name();
                if !project_runtime_crate_name_is_valid(&crate_name) {
                    continue;
                }
                if linked_runtime_crate_names.insert(crate_name.clone()) {
                    linked_runtime_package_ids.insert(provider_package_id.to_string());
                    linked_runtime_crate_links.push(
                        ExportLinkedRuntimeCrate::runtime_feature_with_provider(
                            crate_name,
                            feature.runtime_crate_path(&owner.id),
                            Some(provider_package_id.to_string()),
                        ),
                    );
                }
            }
        }
        let linked_runtime_crates = linked_runtime_crate_links
            .iter()
            .map(|linked_crate| linked_crate.crate_name.clone())
            .collect::<Vec<_>>();
        for selection in enabled_plugins.iter().filter(|selection| {
            selection.packaging == ExportPackagingStrategy::NativeDynamic
                && !selection.is_runtime_builtin_domain()
                && project_plugin_package_id_is_valid(&selection.id)
                && profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
                && platform_policy.supports_native_dynamic
        }) {
            native_dynamic_package_accumulator.push(&selection.id);
        }
        let native_dynamic_package_plan = native_dynamic_package_accumulator.finish();
        let native_dynamic_packages = native_dynamic_package_plan.packages;
        let native_dynamic_package_exports = native_dynamic_package_plan.package_exports;
        let native_dynamic_diagnostics = native_dynamic_package_plan.diagnostics;
        let mut diagnostics = enabled_plugins
            .iter()
            .filter(|selection| {
                selection.packaging == ExportPackagingStrategy::NativeDynamic
                    && !selection.is_runtime_builtin_domain()
                    && !profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
                    && platform_policy.supports_native_dynamic
            })
            .map(|selection| {
                format!(
                    "plugin {} uses NativeDynamic packaging but export profile {} does not enable NativeDynamic",
                    selection.id, profile.name
                )
            })
            .collect::<Vec<_>>();
        let mut fatal_diagnostics = Vec::new();
        if profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
            && !platform_policy.supports_native_dynamic
        {
            let diagnostic = format!(
                "export profile {} enables NativeDynamic but target platform {} does not support dynamic libraries; use LibraryEmbed, SourceTemplate, or VM packaging",
                profile.name,
                profile.target_platform.as_str()
            );
            diagnostics.push(diagnostic.clone());
            fatal_diagnostics.push(diagnostic);
        }
        for selection in enabled_plugins.iter().filter(|selection| {
            selection.packaging == ExportPackagingStrategy::NativeDynamic
                && !selection.is_runtime_builtin_domain()
                && !platform_policy.supports_native_dynamic
        }) {
            let diagnostic = format!(
                "plugin {} uses NativeDynamic packaging but target platform {} does not support dynamic libraries",
                selection.id,
                profile.target_platform.as_str()
            );
            if selection.required {
                fatal_diagnostics.push(diagnostic.clone());
            }
            diagnostics.push(diagnostic);
        }
        diagnostics.extend(native_dynamic_diagnostics);
        diagnostics.extend(project_plugin_id_diagnostics);
        diagnostics.extend(project_feature_id_diagnostics);
        diagnostics.extend(project_duplicate_diagnostics);
        diagnostics.extend(project_runtime_crate_diagnostics);
        diagnostics.extend(project_editor_crate_diagnostics);
        diagnostics.extend(project_target_mode_diagnostics);
        diagnostics.extend(project_provider_diagnostics);
        diagnostics.extend(profile_duplicate_name_fatal_diagnostics.iter().cloned());
        diagnostics.extend(profile_name_fatal_diagnostics.iter().cloned());
        diagnostics.extend(profile_output_name_diagnostics);
        diagnostics.extend(profile_strategy_diagnostics);
        diagnostics.extend(profile_strategy_fatal_diagnostics.iter().cloned());
        diagnostics.extend(profile_selection_diagnostics.diagnostics);
        diagnostics.extend(feature_packaging_diagnostics);
        fatal_diagnostics.extend(project_plugin_id_fatal_diagnostics);
        fatal_diagnostics.extend(project_feature_id_fatal_diagnostics);
        fatal_diagnostics.extend(project_duplicate_fatal_diagnostics);
        fatal_diagnostics.extend(project_runtime_crate_fatal_diagnostics);
        fatal_diagnostics.extend(project_editor_crate_fatal_diagnostics);
        fatal_diagnostics.extend(project_target_mode_fatal_diagnostics);
        fatal_diagnostics.extend(project_provider_fatal_diagnostics);
        fatal_diagnostics.extend(profile_duplicate_name_fatal_diagnostics);
        fatal_diagnostics.extend(profile_name_fatal_diagnostics);
        fatal_diagnostics.extend(profile_strategy_fatal_diagnostics);
        fatal_diagnostics.extend(profile_selection_diagnostics.fatal_diagnostics);
        fatal_diagnostics.extend(feature_packaging_fatal_diagnostics);
        if profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            || profile.uses_strategy(ExportPackagingStrategy::SourceTemplate)
        {
            diagnostics.extend(feature_report.diagnostics.iter().cloned());
            fatal_diagnostics.extend(feature_report.diagnostics.iter().cloned());
            for blocked in &feature_report.blocked_features {
                if blocked.unknown_feature
                    && external_feature_selection(
                        &completed_plugins,
                        &blocked.feature_id,
                        profile.target_mode,
                    )
                    .is_some()
                {
                    continue;
                }
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
        let runtime_profile = runtime_profile_for_export_profile(&profile);
        let profile_runtime_target_fatal_diagnostics =
            export_profile_runtime_profile_target_fatal_diagnostics(&profile, &runtime_profile);
        diagnostics.extend(profile_runtime_target_fatal_diagnostics.iter().cloned());
        fatal_diagnostics.extend(profile_runtime_target_fatal_diagnostics);
        let mut project_plugin_projection = completed_plugins.clone();
        // Generated metadata is a sanitized view; dependency and package projection have already
        // used the completed manifest so invalid identity rows still produce source diagnostics.
        sanitize_project_identity_rows(&mut project_plugin_projection, profile.target_mode);
        sanitize_project_target_mode_rows(&mut project_plugin_projection, profile.target_mode);
        sanitize_invalid_project_provider_package_overrides(
            &mut project_plugin_projection,
            profile.target_mode,
        );
        let project_plugin_selections = project_plugin_projection
            .selections
            .iter()
            .collect::<Vec<_>>();
        let generated_files = generated_files_for_profile(
            manifest,
            &profile,
            &project_plugin_selections,
            &linked_runtime_crate_links,
            &native_dynamic_package_exports,
        );
        let runtime_plugin_descriptors = RuntimePluginDescriptor::builtin_catalog();
        let runtime_plugin_availability = runtime_profile.availability_report_with_providers(
            runtime_plugin_descriptors.iter(),
            linked_runtime_package_ids.iter(),
            native_dynamic_packages.iter(),
        );

        let mut plan = Self::new(
            profile,
            &enabled_plugins,
            linked_runtime_crates,
            native_dynamic_packages,
            native_dynamic_package_exports,
            runtime_plugin_availability,
            generated_files,
        );
        plan.diagnostics = diagnostics;
        plan.fatal_diagnostics = fatal_diagnostics;
        let compile_host_plan = library_embed_compile_host_plan(&plan, &linked_runtime_crate_links);
        plan.set_library_embed_compile_host_plan(compile_host_plan);
        let source_template_build_plan = source_template_build_validation_plan(&plan);
        plan.set_source_template_build_validation_plan(source_template_build_plan);
        Ok(plan)
    }
}

fn linked_rust_strategy_enabled(profile: &crate::plugin::ExportProfile) -> bool {
    profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
        || profile.uses_strategy(ExportPackagingStrategy::SourceTemplate)
}
