use std::collections::HashSet;
use std::sync::Arc;

use crate::asset::AssetImporterRegistry;
use crate::engine_module::EngineModule;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{
    ProjectPluginManifest, RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport, RuntimeProfileDescriptor, RuntimeProfileId,
};

use super::availability::{
    runtime_profile_availability, runtime_profile_manifest_availability,
    target_manifest_availability, target_manifest_availability_for_registration_reports,
};
use super::core_modules::{
    minimal_profile_runtime_modules, runtime_core_modules_for_target_with_render_features,
};
use super::extensions::asset_importers_from_extension_registries;
use super::manifest::manifest_with_mode_baseline;
use super::plugin_modules::{
    builtin_runtime_domain_is_available, builtin_runtime_domain_message,
    linked_plugin_is_available, module_for_plugin,
};
use super::{RuntimeModuleLoadReport, RuntimeRequiredPluginMissing, RuntimeTargetMode};

pub fn builtin_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None).modules
}

pub fn runtime_modules_for_target(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_linked_plugins(
        target,
        manifest_override,
        std::iter::empty::<String>(),
    )
}

pub fn runtime_modules_for_target_with_linked_plugins(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        manifest_override,
        linked_plugin_ids,
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
    )
}

pub fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect::<Vec<_>>();
    let linked_plugin_ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    let render_features = registrations
        .iter()
        .flat_map(|registration| registration.extensions.render_features().iter().cloned())
        .collect::<Vec<_>>();
    let render_pass_executors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .render_pass_executors()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let runtime_prepare_collectors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let hybrid_gi_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .hybrid_gi_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let solari_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let virtual_geometry_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) = asset_importers_from_extension_registries(
        registrations
            .iter()
            .map(|registration| &registration.extensions),
    );
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let mut report =
        runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
            target,
            &manifest,
            linked_plugin_ids,
            &asset_importers,
            &render_features,
            &render_pass_executors,
            &runtime_prepare_collectors,
            &hybrid_gi_runtime_providers,
            &solari_runtime_providers,
            &virtual_geometry_runtime_providers,
        );
    report.errors.extend(asset_importer_errors);
    report.runtime_plugin_availability =
        target_manifest_availability_for_registration_reports(target, &manifest, registrations);
    report
}

pub fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    if profile_id == RuntimeProfileId::Minimal {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_availability(&profile));
    }

    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
        profile.target_mode,
        &manifest,
        std::iter::empty::<String>(),
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
    )
    .with_runtime_plugin_availability(runtime_profile_availability(&profile))
}

pub fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
        profile_id,
        &profile.project_manifest(),
        registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
                &profile,
                manifest,
                registrations.iter().copied(),
            ));
    }

    runtime_modules_for_profile_manifest_with_plugin_registration_reports(
        &profile,
        profile.target_mode,
        manifest,
        registrations.iter().copied(),
    )
}

pub fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
        profile_id,
        &profile.project_manifest(),
        registrations,
        feature_registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
                &profile,
                manifest,
                registrations.iter(),
            ));
    }

    let mut report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter(),
        feature_registrations.iter(),
    );
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(&profile, manifest, registrations.iter());
    report
}

fn runtime_modules_for_profile_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect::<Vec<_>>();
    let linked_plugin_ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    let render_features = registrations
        .iter()
        .flat_map(|registration| registration.extensions.render_features().iter().cloned())
        .collect::<Vec<_>>();
    let render_pass_executors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .render_pass_executors()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let runtime_prepare_collectors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let hybrid_gi_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .hybrid_gi_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let solari_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let virtual_geometry_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) = asset_importers_from_extension_registries(
        registrations
            .iter()
            .map(|registration| &registration.extensions),
    );
    let mut report =
        runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
            target,
            manifest,
            linked_plugin_ids,
            &asset_importers,
            &render_features,
            &render_pass_executors,
            &runtime_prepare_collectors,
            &hybrid_gi_runtime_providers,
            &solari_runtime_providers,
            &virtual_geometry_runtime_providers,
        );
    report.errors.extend(asset_importer_errors);
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(profile, manifest, registrations.iter().copied());
    report
}

pub fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.clone(),
        feature_registrations.clone(),
    );
    let active_registrations = registrations
        .iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect::<Vec<_>>();
    let feature_report = catalog.feature_dependency_report(&manifest, target);
    let active_feature_registrations = feature_registrations
        .iter()
        .filter(|registration| {
            feature_report
                .available_features
                .iter()
                .any(|id| id == &registration.manifest.id)
        })
        .collect::<Vec<_>>();
    let linked_plugin_ids = active_registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    let render_features = active_registrations
        .iter()
        .flat_map(|registration| registration.extensions.render_features().iter().cloned())
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| registration.extensions.render_features().iter().cloned()),
        )
        .collect::<Vec<_>>();
    let render_pass_executors = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .render_pass_executors()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .render_pass_executors()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let runtime_prepare_collectors = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .runtime_prepare_collectors()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let hybrid_gi_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .hybrid_gi_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .hybrid_gi_runtime_providers()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let solari_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .solari_runtime_providers()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let virtual_geometry_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .virtual_geometry_runtime_providers()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) = asset_importers_from_extension_registries(
        active_registrations
            .iter()
            .map(|registration| &registration.extensions)
            .chain(
                active_feature_registrations
                    .iter()
                    .map(|registration| &registration.extensions),
            ),
    );
    let mut report = runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        Some(&manifest),
        linked_plugin_ids,
        &asset_importers,
        &render_features,
        &render_pass_executors,
        &runtime_prepare_collectors,
        &hybrid_gi_runtime_providers,
        &solari_runtime_providers,
        &virtual_geometry_runtime_providers,
    );
    for blocked in feature_report.blocked_features {
        if blocked.required {
            report.errors.push(blocked.to_diagnostic());
        } else {
            report.warnings.push(blocked.to_diagnostic());
        }
    }
    report.errors.extend(feature_report.diagnostics);
    report.errors.extend(asset_importer_errors);
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter(),
    );
    report
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> RuntimeModuleLoadReport {
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
        target,
        &manifest,
        linked_plugin_ids,
        asset_importers,
        render_features,
        render_pass_executors,
        runtime_prepare_collectors,
        hybrid_gi_runtime_providers,
        solari_runtime_providers,
        virtual_geometry_runtime_providers,
    )
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> RuntimeModuleLoadReport {
    let linked_plugin_ids = linked_plugin_ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .collect::<HashSet<_>>();
    let mut report =
        RuntimeModuleLoadReport::new(runtime_core_modules_for_target_with_render_features(
            target,
            asset_importers,
            render_features,
            render_pass_executors,
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            solari_runtime_providers,
            virtual_geometry_runtime_providers,
        ));
    report.runtime_plugin_availability =
        target_manifest_availability(target, manifest, linked_plugin_ids.iter());

    for selection in manifest.enabled_for_target(target) {
        let Some(runtime_id) = selection.runtime_id() else {
            let reason = format!("plugin {} has no known runtime id", selection.id);
            if selection.required {
                report.errors.push(format!(
                    "required runtime plugin {} is unavailable: {}",
                    selection.id, reason
                ));
            } else {
                report.warnings.push(reason);
            }
            continue;
        };
        if builtin_runtime_domain_is_available(runtime_id) {
            report
                .warnings
                .push(builtin_runtime_domain_message(runtime_id.key()));
            continue;
        }
        if linked_plugin_is_available(selection, runtime_id, &linked_plugin_ids) {
            continue;
        }
        let warning_start = report.warnings.len();
        if let Some(module) = module_for_plugin(runtime_id, &mut report.warnings) {
            report.modules.push(module);
            continue;
        }
        if selection.required {
            let reason = report.warnings[warning_start..]
                .last()
                .cloned()
                .unwrap_or_else(|| format!("plugin {} is unavailable", runtime_id.label()));
            let message = format!(
                "required runtime plugin {} is unavailable: {}",
                runtime_id.label(),
                reason.clone()
            );
            report.required_missing.push(RuntimeRequiredPluginMissing {
                id: runtime_id,
                reason,
            });
            report.errors.push(message);
        }
    }
    report
}
