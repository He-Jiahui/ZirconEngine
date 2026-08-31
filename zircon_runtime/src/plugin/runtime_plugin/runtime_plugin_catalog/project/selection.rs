use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginManifest};
use crate::core::ModuleDescriptor;

use super::super::derived_projection::RuntimePluginCatalogProjection;
use super::super::feature_registration_match::{
    feature_registration_matches_project_selection, project_feature_provider_lookup,
};
use super::super::feature_report::RuntimePluginFeatureDependencyReport;
use super::super::registration::order::order_runtime_plugin_registration_report_refs_for_target;
use super::super::runtime_module_target::runtime_module_names_for_target;
use super::super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

/// Frozen module proposal selected for one compiled project plan.
#[derive(Clone, Debug)]
pub struct RuntimePluginModuleProposal {
    provider_package_id: String,
    feature_id: Option<String>,
    descriptor: ModuleDescriptor,
}

impl RuntimePluginModuleProposal {
    /// Package that owns the proposed runtime module.
    pub fn provider_package_id(&self) -> &str {
        &self.provider_package_id
    }

    /// Selected feature that contributed the module, or `None` for a base plugin module.
    pub fn feature_id(&self) -> Option<&str> {
        self.feature_id.as_deref()
    }

    /// Runtime module descriptor proposed by the selected provider.
    pub fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }
}

#[derive(Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct CompiledRuntimePluginBaseSelection
{
    ordered_plugin_registration_indices: Box<[usize]>,
    effective_enabled_plugins: HashSet<String>,
    available_capabilities: HashSet<String>,
    fatal_diagnostic: Option<String>,
}

#[derive(Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct CompiledRuntimePluginSelection
{
    ordered_plugin_registration_indices: Box<[usize]>,
    linked_provider_package_ids: Box<[String]>,
    native_dynamic_provider_package_ids: Box<[String]>,
    feature_registration_indices: Box<[usize]>,
    module_proposals: Arc<[RuntimePluginModuleProposal]>,
    fatal_diagnostic: Option<String>,
}

impl CompiledRuntimePluginBaseSelection {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn compile(
        registrations: &[RuntimePluginRegistrationReport],
        feature_registrations: &[RuntimePluginFeatureRegistrationReport],
        completed: &ProjectPluginManifest,
        target: RuntimeTargetMode,
    ) -> Self {
        let enabled_plugins = completed
            .enabled_for_target(target)
            .map(|selection| selection.id.clone())
            .collect::<HashSet<_>>();
        let selected_registration_indices = registrations
            .iter()
            .enumerate()
            .filter(|registration| {
                registration.1.project_selection.enabled
                    && registration.1.project_selection.supports_target(target)
                    && enabled_plugins.contains(&registration.1.project_selection.id)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let selected_plugin_ids = selected_registration_indices
            .iter()
            .map(|index| registrations[*index].project_selection.id.as_str())
            .collect::<HashSet<_>>();
        let registered_plugin_ids = registrations
            .iter()
            .map(|registration| registration.project_selection.id.as_str())
            .collect::<HashSet<_>>();
        let concrete_feature_providers = feature_registrations
            .iter()
            .map(|registration| {
                (
                    registration.manifest.id.as_str(),
                    registration.provider_package_id_or_owner(),
                )
            })
            .collect::<HashSet<_>>();
        let mut selected_feature_provider_ids = HashSet::new();
        for selection in &completed.selections {
            for feature in &selection.features {
                let provider_package_id = feature.provider_package_id_or_owner(&selection.id);
                if feature.enabled
                    && feature.supports_target(target)
                    && concrete_feature_providers
                        .contains(&(feature.id.as_str(), provider_package_id))
                {
                    selected_feature_provider_ids.insert(provider_package_id);
                }
            }
        }
        let mut effective_enabled_plugins = enabled_plugins;
        effective_enabled_plugins.retain(|plugin_id| {
            selected_plugin_ids.contains(plugin_id.as_str())
                || (!registered_plugin_ids.contains(plugin_id.as_str())
                    && selected_feature_provider_ids.contains(plugin_id.as_str()))
        });
        let available_capabilities = selected_registration_indices
            .iter()
            .flat_map(|index| registrations[*index].package_manifest.modules.iter())
            .filter(|module| {
                module.target_modes.is_empty() || module.target_modes.contains(&target)
            })
            .flat_map(|module| module.capabilities.iter().cloned())
            .collect::<HashSet<_>>();
        let registration_indices_by_address = selected_registration_indices
            .iter()
            .map(|index| {
                (
                    &registrations[*index] as *const RuntimePluginRegistrationReport,
                    *index,
                )
            })
            .collect::<HashMap<_, _>>();
        let selected_registration_refs = selected_registration_indices
            .iter()
            .map(|index| &registrations[*index])
            .collect::<Vec<_>>();
        let ordered_registrations = match order_runtime_plugin_registration_report_refs_for_target(
            selected_registration_refs,
            target,
        ) {
            Ok(registrations) => registrations,
            Err(error) => {
                return Self {
                    ordered_plugin_registration_indices: Box::new([]),
                    effective_enabled_plugins,
                    available_capabilities,
                    fatal_diagnostic: Some(format!(
                        "runtime plugin module descriptor ordering failed: {error}"
                    )),
                };
            }
        };
        let mut ordered_plugin_registration_indices =
            Vec::with_capacity(ordered_registrations.len());
        for registration in ordered_registrations {
            let address = registration as *const RuntimePluginRegistrationReport;
            let Some(index) = registration_indices_by_address.get(&address).copied() else {
                return Self {
                    ordered_plugin_registration_indices: Box::new([]),
                    effective_enabled_plugins,
                    available_capabilities,
                    fatal_diagnostic: Some(format!(
                        "runtime plugin selection lost registration package `{}` during ordering",
                        registration.package_manifest.id
                    )),
                };
            };
            ordered_plugin_registration_indices.push(index);
        }

        Self {
            ordered_plugin_registration_indices: ordered_plugin_registration_indices.into(),
            effective_enabled_plugins,
            available_capabilities,
            fatal_diagnostic: None,
        }
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn take_feature_dependency_inputs(
        &mut self,
    ) -> (HashSet<String>, HashSet<String>) {
        (
            std::mem::take(&mut self.effective_enabled_plugins),
            std::mem::take(&mut self.available_capabilities),
        )
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn complete(
        self,
        registrations: &[RuntimePluginRegistrationReport],
        feature_registrations: &[RuntimePluginFeatureRegistrationReport],
        projection: &RuntimePluginCatalogProjection,
        completed: &ProjectPluginManifest,
        target: RuntimeTargetMode,
        feature_report: &RuntimePluginFeatureDependencyReport,
    ) -> CompiledRuntimePluginSelection {
        let Self {
            ordered_plugin_registration_indices,
            fatal_diagnostic,
            ..
        } = self;
        if let Some(fatal_diagnostic) = fatal_diagnostic {
            return CompiledRuntimePluginSelection::failed(fatal_diagnostic);
        }

        let selected_providers = project_feature_provider_lookup(completed);
        let feature_registration_indices = feature_report
            .available_features
            .iter()
            .filter_map(|feature_id| {
                projection
                    .feature_registration_indices(feature_id)
                    .iter()
                    .copied()
                    .find(|index| {
                        feature_registration_matches_project_selection(
                            &feature_registrations[*index],
                            &selected_providers,
                            feature_id,
                        )
                    })
            })
            .collect::<Vec<_>>();
        let module_proposals = module_proposals(
            registrations,
            feature_registrations,
            &ordered_plugin_registration_indices,
            &feature_registration_indices,
            target,
        );
        let mut linked_provider_package_ids = Vec::new();
        let mut native_dynamic_provider_package_ids = Vec::new();
        for index in ordered_plugin_registration_indices.iter().copied() {
            let registration = &registrations[index];
            if registration.project_selection.packaging == ExportPackagingStrategy::NativeDynamic {
                native_dynamic_provider_package_ids.push(registration.package_manifest.id.clone());
            } else {
                linked_provider_package_ids.push(registration.package_manifest.id.clone());
            }
        }

        CompiledRuntimePluginSelection {
            ordered_plugin_registration_indices,
            linked_provider_package_ids: linked_provider_package_ids.into(),
            native_dynamic_provider_package_ids: native_dynamic_provider_package_ids.into(),
            feature_registration_indices: feature_registration_indices.into(),
            module_proposals: module_proposals.into(),
            fatal_diagnostic: None,
        }
    }
}

impl CompiledRuntimePluginSelection {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn ordered_plugin_registration_indices(
        &self,
    ) -> &[usize] {
        &self.ordered_plugin_registration_indices
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_registration_indices(
        &self,
    ) -> &[usize] {
        &self.feature_registration_indices
    }

    pub(super) fn linked_provider_package_ids(&self) -> &[String] {
        &self.linked_provider_package_ids
    }

    pub(super) fn native_dynamic_provider_package_ids(&self) -> &[String] {
        &self.native_dynamic_provider_package_ids
    }

    pub(super) fn module_proposals(&self) -> &[RuntimePluginModuleProposal] {
        &self.module_proposals
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn fatal_diagnostic(
        &self,
    ) -> Option<&str> {
        self.fatal_diagnostic.as_deref()
    }

    fn failed(fatal_diagnostic: String) -> Self {
        Self {
            ordered_plugin_registration_indices: Box::new([]),
            linked_provider_package_ids: Box::new([]),
            native_dynamic_provider_package_ids: Box::new([]),
            feature_registration_indices: Box::new([]),
            module_proposals: Arc::from([]),
            fatal_diagnostic: Some(fatal_diagnostic),
        }
    }
}

fn module_proposals(
    registrations: &[RuntimePluginRegistrationReport],
    feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    ordered_plugin_registration_indices: &[usize],
    feature_registration_indices: &[usize],
    target: RuntimeTargetMode,
) -> Vec<RuntimePluginModuleProposal> {
    let plugin_proposals = ordered_plugin_registration_indices
        .iter()
        .flat_map(|index| {
            let registration = &registrations[*index];
            let selected_module_names =
                runtime_module_names_for_target(&registration.package_manifest.modules, target)
                    .collect::<HashSet<_>>();
            registration
                .extensions
                .modules()
                .iter()
                .filter(move |descriptor| selected_module_names.contains(descriptor.name.as_str()))
                .cloned()
                .map(move |descriptor| RuntimePluginModuleProposal {
                    provider_package_id: registration.package_manifest.id.clone(),
                    feature_id: None,
                    descriptor,
                })
        });
    let feature_proposals = feature_registration_indices.iter().flat_map(|index| {
        let registration = &feature_registrations[*index];
        let selected_module_names =
            runtime_module_names_for_target(&registration.manifest.modules, target)
                .collect::<HashSet<_>>();
        registration
            .extensions
            .modules()
            .iter()
            .filter(move |descriptor| selected_module_names.contains(descriptor.name.as_str()))
            .cloned()
            .map(move |descriptor| RuntimePluginModuleProposal {
                provider_package_id: registration.provider_package_id_or_owner().to_string(),
                feature_id: Some(registration.manifest.id.clone()),
                descriptor,
            })
    });
    plugin_proposals.chain(feature_proposals).collect()
}
