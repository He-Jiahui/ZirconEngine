#[cfg(test)]
use std::cell::Cell;
use std::collections::HashSet;
use std::sync::Arc;

use crate::asset::project::ProjectManifest;
use crate::{
    core::framework::project::{ExportPackagingStrategy, ExportProfile, ProjectPluginSelection},
    plugin::RuntimePluginCatalog,
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
    ExportProfileSelectionProjection,
};
use super::cargo_manifest_template::plugin_path_for_runtime_crate;
use super::default_profile::default_profile;
use super::export_profile_validation::{
    export_profile_duplicate_name_fatal_diagnostics, export_profile_name_fatal_diagnostics,
    export_profile_output_name_diagnostics, export_profile_runtime_profile_id_fatal_diagnostics,
    export_profile_runtime_profile_target_fatal_diagnostics, export_profile_strategy_diagnostics,
    export_profile_strategy_fatal_diagnostics, sanitize_export_profile_output_name,
    sanitize_export_profile_strategies,
};
use super::generated_files::generated_files_for_profile;
use super::library_embed_compile_plan::library_embed_compile_host_plan;
use super::native_dynamic_package_plan::NativeDynamicPackageAccumulator;
#[cfg(test)]
use super::project_manifest_validation::{
    begin_projection_build_observation, observed_projection_builds,
};
use super::project_manifest_validation::{
    project_duplicate_selection_diagnostics, project_editor_crate_diagnostics,
    project_feature_id_diagnostics, project_feature_provider_package_id_diagnostics,
    project_plugin_package_id_diagnostics, project_plugin_package_id_is_valid,
    project_runtime_crate_diagnostics, project_runtime_crate_name_is_valid,
    project_runtime_crate_override_is_valid, project_target_mode_diagnostics,
    sanitize_invalid_project_crate_overrides, sanitize_invalid_project_provider_package_overrides,
    sanitize_project_identity_rows, sanitize_project_target_mode_rows,
    ProjectPluginManifestValidationProjection,
};
use super::source_template_build_plan::source_template_build_validation_plan;
use super::{ExportBuildPlan, ExportBuildPlanError, ExportLinkedRuntimeCrate};

impl ExportBuildPlan {
    pub fn from_project_manifest(
        manifest: &ProjectManifest,
        profile_name: &str,
    ) -> Result<Self, ExportBuildPlanError> {
        let mut profile = manifest
            .export_profiles
            .iter()
            .find(|profile| profile.name == profile_name)
            .cloned()
            .or_else(|| default_profile(profile_name))
            .ok_or_else(|| ExportBuildPlanError::MissingProfile {
                profile_name: profile_name.to_string(),
            })?;
        let profile_duplicate_name_fatal_diagnostics =
            export_profile_duplicate_name_fatal_diagnostics(
                &manifest.export_profiles,
                profile_name,
            );
        let profile_name_fatal_diagnostics = export_profile_name_fatal_diagnostics(&profile);
        let profile_runtime_profile_id_fatal_diagnostics =
            export_profile_runtime_profile_id_fatal_diagnostics(&profile);
        let profile_output_name_diagnostics = export_profile_output_name_diagnostics(&profile);
        let profile_strategy_diagnostics = export_profile_strategy_diagnostics(&profile);
        let profile_strategy_fatal_diagnostics =
            export_profile_strategy_fatal_diagnostics(&profile);
        sanitize_export_profile_output_name(&mut profile);
        sanitize_export_profile_strategies(&mut profile);
        let platform_policy = profile.target_platform.policy();

        let catalog = builtin_runtime_plugin_catalog();
        let mut completed_plugins = Arc::unwrap_or_clone(
            catalog.complete_project_manifest(&manifest.plugins, profile.target_mode),
        );
        let profile_projection = ExportProfileSelectionProjection::new(&profile);
        let mut completed_manifest_projection =
            ProjectPluginManifestValidationProjection::new(&completed_plugins, profile.target_mode);
        let profile_selection_diagnostics = export_profile_selection_diagnostics(
            &profile,
            &completed_plugins,
            &completed_manifest_projection,
            &profile_projection,
        );
        project_plugins_for_export_profile(&profile_projection, &mut completed_plugins);
        // Keep diagnostics source-faithful while preventing malformed crate tokens from
        // leaking into generated SourceTemplate project metadata.
        sanitize_invalid_project_crate_overrides(&mut completed_plugins, profile.target_mode);
        completed_manifest_projection.refresh(&completed_plugins);
        let source_manifest_projection =
            ProjectPluginManifestValidationProjection::new(&manifest.plugins, profile.target_mode);
        let enabled_plugins = completed_plugins
            .enabled_for_target(profile.target_mode)
            .collect::<Vec<_>>();
        let (project_plugin_id_diagnostics, project_plugin_id_fatal_diagnostics) =
            project_plugin_package_id_diagnostics(&manifest.plugins, &source_manifest_projection);
        let (project_feature_id_diagnostics, project_feature_id_fatal_diagnostics) =
            project_feature_id_diagnostics(&manifest.plugins, &source_manifest_projection);
        let (project_duplicate_diagnostics, project_duplicate_fatal_diagnostics) =
            project_duplicate_selection_diagnostics(&manifest.plugins, &source_manifest_projection);
        let (project_runtime_crate_diagnostics, project_runtime_crate_fatal_diagnostics) =
            project_runtime_crate_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_editor_crate_diagnostics, project_editor_crate_fatal_diagnostics) =
            project_editor_crate_diagnostics(&manifest.plugins, profile.target_mode);
        let (project_target_mode_diagnostics, project_target_mode_fatal_diagnostics) =
            project_target_mode_diagnostics(&manifest.plugins, profile.target_mode);
        let LinkedRuntimePluginProjection {
            crate_names: mut linked_runtime_crate_names,
            package_ids: mut linked_runtime_package_ids,
            crate_links: mut linked_runtime_crate_links,
        } = linked_runtime_plugin_projection(&enabled_plugins, &profile);
        let mut native_dynamic_package_accumulator = NativeDynamicPackageAccumulator::default();
        let feature_report = catalog.feature_dependency_report_for_completed_manifest(
            &completed_plugins,
            profile.target_mode,
        );
        let available_feature_ids = feature_report
            .available_features
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let (project_provider_diagnostics, project_provider_fatal_diagnostics) =
            project_feature_provider_package_id_diagnostics(
                &manifest.plugins,
                &source_manifest_projection,
            );
        let mut feature_packaging_diagnostics = Vec::new();
        let mut feature_packaging_fatal_diagnostics = Vec::new();
        for feature_id in &feature_report.available_features {
            let Some((owner, feature)) = feature_selection(
                &completed_plugins,
                &completed_manifest_projection,
                feature_id,
            ) else {
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
        for (owner, feature) in
            external_feature_selections(&completed_plugins, &completed_manifest_projection)
        {
            if available_feature_ids.contains(feature.id.as_str()) {
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
        diagnostics.extend(profile_runtime_profile_id_fatal_diagnostics.iter().cloned());
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
        fatal_diagnostics.extend(profile_runtime_profile_id_fatal_diagnostics);
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
                        &completed_manifest_projection,
                        &blocked.feature_id,
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
        let profile_runtime_target_fatal_diagnostics = runtime_profile
            .as_ref()
            .map(|runtime_profile| {
                export_profile_runtime_profile_target_fatal_diagnostics(&profile, runtime_profile)
            })
            .unwrap_or_default();
        diagnostics.extend(profile_runtime_target_fatal_diagnostics.iter().cloned());
        fatal_diagnostics.extend(profile_runtime_target_fatal_diagnostics);
        let mut project_plugin_projection = completed_plugins.clone();
        // Generated metadata is a sanitized view; dependency and package projection have already
        // used the completed manifest so invalid identity rows still produce source diagnostics.
        sanitize_project_identity_rows(
            &mut project_plugin_projection,
            &completed_manifest_projection,
        );
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
        let runtime_plugin_availability = runtime_profile
            .map(|runtime_profile| {
                runtime_profile.availability_report_for_catalog_with_provider_membership(
                    catalog,
                    &linked_runtime_package_ids,
                    native_dynamic_packages.iter().map(String::as_str),
                )
            })
            .unwrap_or_default();

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

fn builtin_runtime_plugin_catalog() -> &'static RuntimePluginCatalog {
    #[cfg(test)]
    BUILTIN_CATALOG_BUILD_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    RuntimePluginCatalog::builtin_shared()
}

#[cfg(test)]
thread_local! {
    static BUILTIN_CATALOG_BUILD_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn reset_builtin_catalog_build_count() {
    BUILTIN_CATALOG_BUILD_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
fn builtin_catalog_build_count() -> usize {
    BUILTIN_CATALOG_BUILD_COUNT.with(Cell::get)
}

fn linked_rust_strategy_enabled(profile: &crate::core::framework::project::ExportProfile) -> bool {
    profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
        || profile.uses_strategy(ExportPackagingStrategy::SourceTemplate)
}

#[derive(Default)]
struct LinkedRuntimePluginProjection {
    crate_names: HashSet<String>,
    package_ids: HashSet<String>,
    crate_links: Vec<ExportLinkedRuntimeCrate>,
}

fn linked_runtime_plugin_projection(
    enabled_plugins: &[&ProjectPluginSelection],
    profile: &ExportProfile,
) -> LinkedRuntimePluginProjection {
    if !linked_rust_strategy_enabled(profile) {
        return LinkedRuntimePluginProjection::default();
    }

    let mut crate_names = HashSet::with_capacity(enabled_plugins.len());
    let mut package_ids = HashSet::with_capacity(enabled_plugins.len());
    let mut crate_links = Vec::with_capacity(enabled_plugins.len());
    for &selection in enabled_plugins {
        if selection.runtime_crate.is_none()
            || selection.is_runtime_builtin_domain()
            || !project_plugin_package_id_is_valid(&selection.id)
            || !project_runtime_crate_override_is_valid(selection.runtime_crate.as_deref())
            || selection.packaging == ExportPackagingStrategy::NativeDynamic
        {
            continue;
        }

        package_ids.insert(selection.id.clone());
        let crate_name = selection
            .runtime_crate
            .as_deref()
            .expect("eligible linked runtime plugin must declare a runtime crate");
        if crate_names.contains(crate_name) {
            continue;
        }
        let crate_name = crate_name.to_string();
        crate_names.insert(crate_name.clone());
        let path = format!("{}/runtime", plugin_path_for_runtime_crate(&crate_name));
        crate_links.push(ExportLinkedRuntimeCrate::runtime_plugin(crate_name, path));
    }

    LinkedRuntimePluginProjection {
        crate_names,
        package_ids,
        crate_links,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::AssetUri;
    use crate::core::framework::platform::RuntimeTargetMode;
    use crate::core::framework::project::{ExportProfile, ExportTargetPlatform, RuntimeProfileId};

    #[test]
    fn single_pass_linked_runtime_projection_preserves_contract() {
        let profile = ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate);
        let first = ProjectPluginSelection::runtime_plugin("duplicate-runtime-a", true, false)
            .with_runtime_crate("zircon_plugin_shared_runtime");
        let second = ProjectPluginSelection::runtime_plugin("duplicate-runtime-b", true, false)
            .with_runtime_crate("zircon_plugin_shared_runtime");
        let projection = linked_runtime_plugin_projection(&[&first, &second], &profile);

        assert_eq!(projection.crate_links.len(), 1);
        assert_eq!(
            projection.crate_links[0].crate_name,
            "zircon_plugin_shared_runtime"
        );
        assert_eq!(projection.crate_links[0].path, "shared/runtime");
        assert!(projection.package_ids.contains("duplicate-runtime-a"));
        assert!(projection.package_ids.contains("duplicate-runtime-b"));
    }

    #[test]
    fn export_generation_builds_each_manifest_validation_view_once() {
        let source = include_str!("from_project_manifest.rs");
        let descriptor_rebuild = ["RuntimePluginDescriptor", "::builtin_catalog"].concat();
        let catalog_call = ["builtin_runtime_plugin_catalog", "();"].concat();
        let direct_catalog_build = ["RuntimePluginCatalog", "::builtin"].concat();
        let shared_catalog = ["RuntimePluginCatalog", "::builtin_shared()"].concat();
        assert!(!source.contains(&descriptor_rebuild));
        assert_eq!(source.matches(&catalog_call).count(), 1);
        assert_eq!(source.matches(&direct_catalog_build).count(), 1);
        assert!(source.contains(&shared_catalog));
        reset_builtin_catalog_build_count();
        begin_projection_build_observation();
        let mut manifest = ProjectManifest::new(
            "availability-projection",
            AssetUri::parse("res://scenes/main.scene.toml").expect("fixture asset URI"),
            1,
        );
        manifest.export_profiles = vec![ExportProfile::new(
            "client",
            RuntimeTargetMode::ClientRuntime,
            ExportTargetPlatform::Windows,
            RuntimeProfileId::Client2d,
        )
        .with_strategy(ExportPackagingStrategy::SourceTemplate)];

        let _ = ExportBuildPlan::from_project_manifest(&manifest, "client")
            .expect("export generation should succeed");

        assert_eq!(builtin_catalog_build_count(), 1);
        assert_eq!(observed_projection_builds(), 2);
    }

    #[test]
    fn completed_plugin_manifest_is_reused_for_feature_resolution() {
        let source = include_str!("from_project_manifest.rs");
        let completing_api = ["feature_dependency_report", "(&completed_plugins"].concat();
        let completed_api = ["feature_dependency_report_for_", "completed_manifest"].concat();

        assert!(!source.contains(&completing_api));
        assert!(source.contains(&completed_api));
    }

    #[test]
    fn missing_profile_returns_typed_plan_error() {
        let manifest = ProjectManifest::new(
            "typed-error-contract",
            AssetUri::parse("res://scenes/main.scene.toml").expect("fixture asset URI"),
            1,
        );

        let error = ExportBuildPlan::from_project_manifest(&manifest, "missing-profile")
            .expect_err("unknown profile must fail");

        assert_eq!(
            error,
            ExportBuildPlanError::MissingProfile {
                profile_name: "missing-profile".to_string(),
            }
        );
    }
}
