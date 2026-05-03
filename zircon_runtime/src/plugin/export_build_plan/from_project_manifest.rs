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
        let platform_policy = profile.target_platform.policy();

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
        let mut native_dynamic_package_ids = HashSet::new();
        let mut native_dynamic_package_directories = HashSet::new();
        let mut native_dynamic_packages = Vec::new();
        let mut native_dynamic_diagnostics = Vec::new();
        let feature_report =
            catalog.feature_dependency_report(&completed_plugins, profile.target_mode);
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
                    push_native_dynamic_package(
                        provider_package_id,
                        &mut native_dynamic_package_ids,
                        &mut native_dynamic_package_directories,
                        &mut native_dynamic_packages,
                        &mut native_dynamic_diagnostics,
                    );
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
            if feature.runtime_crate.is_none()
                || !profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            {
                continue;
            }
            let crate_name = feature.runtime_crate_name();
            if linked_runtime_crate_names.insert(crate_name.clone()) {
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
                    push_native_dynamic_package(
                        provider_package_id,
                        &mut native_dynamic_package_ids,
                        &mut native_dynamic_package_directories,
                        &mut native_dynamic_packages,
                        &mut native_dynamic_diagnostics,
                    );
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
            if feature.runtime_crate.is_some()
                && profile.uses_strategy(ExportPackagingStrategy::LibraryEmbed)
            {
                let crate_name = feature.runtime_crate_name();
                if linked_runtime_crate_names.insert(crate_name.clone()) {
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
                && profile.uses_strategy(ExportPackagingStrategy::NativeDynamic)
                && platform_policy.supports_native_dynamic
        }) {
            push_native_dynamic_package(
                &selection.id,
                &mut native_dynamic_package_ids,
                &mut native_dynamic_package_directories,
                &mut native_dynamic_packages,
                &mut native_dynamic_diagnostics,
            );
        }
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
        diagnostics.extend(feature_packaging_diagnostics);
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

fn external_feature_selections(
    manifest: &crate::plugin::ProjectPluginManifest,
    target: crate::RuntimeTargetMode,
) -> Vec<(
    &crate::plugin::ProjectPluginSelection,
    &crate::plugin::ProjectPluginFeatureSelection,
)> {
    manifest
        .selections
        .iter()
        .filter(move |selection| selection.enabled && selection.supports_target(target))
        .flat_map(move |selection| {
            selection
                .features
                .iter()
                .filter(move |feature| feature.enabled && feature.supports_target(target))
                .filter(move |feature| {
                    feature
                        .external_provider_package_id(&selection.id)
                        .is_some_and(|provider_package_id| {
                            manifest.selections.iter().any(|provider| {
                                provider.id == provider_package_id
                                    && provider.enabled
                                    && provider.supports_target(target)
                            })
                        })
                })
                .map(move |feature| (selection, feature))
        })
        .collect()
}

fn external_feature_selection<'a>(
    manifest: &'a crate::plugin::ProjectPluginManifest,
    feature_id: &str,
    target: crate::RuntimeTargetMode,
) -> Option<(
    &'a crate::plugin::ProjectPluginSelection,
    &'a crate::plugin::ProjectPluginFeatureSelection,
)> {
    manifest.selections.iter().find_map(|selection| {
        if !selection.enabled || !selection.supports_target(target) {
            return None;
        }
        selection
            .features
            .iter()
            .find(|feature| {
                feature.id == feature_id
                    && feature.enabled
                    && feature.supports_target(target)
                    && feature
                        .external_provider_package_id(&selection.id)
                        .is_some_and(|provider_package_id| {
                            manifest.selections.iter().any(|provider| {
                                provider.id == provider_package_id
                                    && provider.enabled
                                    && provider.supports_target(target)
                            })
                        })
            })
            .map(|feature| (selection, feature))
    })
}

fn push_native_dynamic_package(
    package_id: &str,
    native_dynamic_package_ids: &mut HashSet<String>,
    native_dynamic_package_directories: &mut HashSet<String>,
    native_dynamic_packages: &mut Vec<String>,
    native_dynamic_diagnostics: &mut Vec<String>,
) {
    if !native_dynamic_package_ids.insert(package_id.to_string()) {
        return;
    }
    let package_directory = native_dynamic_package_directory(package_id);
    if !native_dynamic_package_directories.insert(package_directory.clone()) {
        native_dynamic_diagnostics.push(format!(
            "plugin {package_id} uses NativeDynamic packaging but resolves to duplicate output directory plugins/{package_directory}"
        ));
        return;
    }
    native_dynamic_packages.push(package_id.to_string());
}
