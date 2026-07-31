use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::PluginModuleKind;

use super::bridge_dependencies::{bridge_dependency_diagnostics, RuntimePluginBridgeDependent};
use super::feature_capabilities::feature_capabilities_for_target;
use super::feature_definition_collection::feature_definition_map;
use super::feature_definitions::{FeatureDefinition, FeatureDefinitionMap};
use super::{RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport};

#[derive(Clone, Debug, Default)]
pub(super) struct RuntimePluginCatalogProjection {
    feature_definitions: FeatureDefinitionMap,
    registration_indices_by_package: HashMap<String, usize>,
    feature_registration_indices_by_id: HashMap<String, Vec<usize>>,
    feature_definition_keys_by_owner: HashMap<String, Vec<String>>,
    runtime_modules_by_provider: HashMap<String, Vec<String>>,
    providers_by_runtime_module: HashMap<String, String>,
    base_capabilities_by_package_and_target: [HashMap<String, Vec<String>>; 3],
    feature_capability_providers_by_target: [HashMap<String, Vec<String>>; 3],
    bridge_dependents_by_provider: HashMap<String, Vec<RuntimePluginBridgeDependent>>,
    bridge_dependency_diagnostics: Vec<String>,
    metrics: RuntimePluginCatalogProjectionMetrics,
    #[cfg(test)]
    stats: RuntimePluginCatalogProjectionStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginCatalogProjectionMetrics {
    pub catalog_generation: u64,
    pub projection_builds: u64,
    pub build_elapsed_ns: u64,
    pub indexed_entry_count: usize,
    pub indexed_string_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RuntimePluginCatalogProjectionStats {
    pub(super) projection_builds: usize,
    pub(super) registrations_scanned: usize,
    pub(super) feature_registrations_scanned: usize,
    pub(super) feature_definitions_projected: usize,
    pub(super) runtime_modules_indexed: usize,
    pub(super) feature_dependency_edges_indexed: usize,
}

impl RuntimePluginCatalogProjection {
    pub(super) fn build(
        registrations: &[RuntimePluginRegistrationReport],
        feature_registrations: &[RuntimePluginFeatureRegistrationReport],
        catalog_generation: u64,
        projection_builds: u64,
    ) -> Self {
        let build_started = Instant::now();
        let feature_definitions = feature_definition_map(registrations, feature_registrations);
        let mut projection = Self {
            #[cfg(test)]
            stats: RuntimePluginCatalogProjectionStats {
                projection_builds: projection_builds as usize,
                registrations_scanned: registrations.len(),
                feature_registrations_scanned: feature_registrations.len(),
                feature_definitions_projected: feature_definitions.definition_order.len(),
                feature_dependency_edges_indexed: feature_definitions
                    .definitions
                    .values()
                    .map(|definition| definition.manifest.dependencies.len())
                    .sum(),
                ..RuntimePluginCatalogProjectionStats::default()
            },
            feature_definitions,
            bridge_dependency_diagnostics: bridge_dependency_diagnostics(registrations),
            ..Self::default()
        };
        projection.index_registrations(registrations);
        projection.index_feature_registrations(feature_registrations);
        projection.index_feature_definitions();
        projection.index_bridge_dependents(registrations);
        projection.metrics = RuntimePluginCatalogProjectionMetrics {
            catalog_generation,
            projection_builds,
            build_elapsed_ns: elapsed_nanos(build_started),
            indexed_entry_count: projection.indexed_entry_count(),
            indexed_string_bytes: projection.indexed_string_bytes(),
        };
        tracing::debug!(
            target: "zircon_runtime::plugin",
            catalog_generation = projection.metrics.catalog_generation,
            projection_builds = projection.metrics.projection_builds,
            build_elapsed_ns = projection.metrics.build_elapsed_ns,
            indexed_entry_count = projection.metrics.indexed_entry_count,
            indexed_string_bytes = projection.metrics.indexed_string_bytes,
            "runtime plugin catalog derived projection rebuilt"
        );
        projection
    }

    pub(super) fn feature_definitions(&self) -> &FeatureDefinitionMap {
        &self.feature_definitions
    }

    pub(super) fn definition_keys_for_owner(&self, owner_plugin_id: &str) -> &[String] {
        self.feature_definition_keys_by_owner
            .get(owner_plugin_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn registration_index_for_package(&self, package_id: &str) -> Option<usize> {
        self.registration_indices_by_package
            .get(package_id)
            .copied()
    }

    pub(super) fn feature_registration_indices(&self, feature_id: &str) -> &[usize] {
        self.feature_registration_indices_by_id
            .get(feature_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn runtime_modules_for_provider(&self, provider_package_id: &str) -> &[String] {
        self.runtime_modules_by_provider
            .get(provider_package_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn provider_for_runtime_module(&self, runtime_module_name: &str) -> Option<&str> {
        self.providers_by_runtime_module
            .get(runtime_module_name)
            .map(String::as_str)
    }

    pub(super) fn capability_has_unresolved_provider(
        &self,
        capability: &str,
        unresolved_definition_keys: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> bool {
        self.feature_capability_providers_by_target[target_index(target)]
            .get(capability)
            .is_some_and(|providers| {
                providers
                    .iter()
                    .any(|provider| unresolved_definition_keys.contains(provider.as_str()))
            })
    }

    pub(super) fn base_capabilities_for_target(
        &self,
        enabled_plugins: &HashSet<String>,
        target: RuntimeTargetMode,
    ) -> HashSet<String> {
        let capabilities_by_package =
            &self.base_capabilities_by_package_and_target[target_index(target)];
        enabled_plugins
            .iter()
            .filter_map(|package_id| capabilities_by_package.get(package_id))
            .flatten()
            .cloned()
            .collect()
    }

    pub(super) fn bridge_dependents_for_provider(
        &self,
        provider_package_id: &str,
    ) -> &[RuntimePluginBridgeDependent] {
        self.bridge_dependents_by_provider
            .get(provider_package_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub(super) fn bridge_dependency_diagnostics(&self) -> &[String] {
        &self.bridge_dependency_diagnostics
    }

    pub(super) fn feature_definition_diagnostics(&self) -> &[String] {
        &self.feature_definitions.diagnostics
    }

    pub(super) fn metrics(&self) -> RuntimePluginCatalogProjectionMetrics {
        self.metrics
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> RuntimePluginCatalogProjectionStats {
        self.stats
    }

    fn index_registrations(&mut self, registrations: &[RuntimePluginRegistrationReport]) {
        let mut base_capability_seen_by_package_and_target =
            <[HashMap<String, HashSet<String>>; 3]>::default();
        for (index, registration) in registrations.iter().enumerate() {
            let package_id = &registration.package_manifest.id;
            if let std::collections::hash_map::Entry::Vacant(entry) = self
                .registration_indices_by_package
                .entry(package_id.clone())
            {
                entry.insert(index);
                self.index_runtime_modules(registration);
            }
            self.index_base_capabilities(
                registration,
                &mut base_capability_seen_by_package_and_target,
            );
        }
    }

    fn index_runtime_modules(&mut self, registration: &RuntimePluginRegistrationReport) {
        let package_id = &registration.package_manifest.id;
        let mut runtime_modules = registration
            .package_manifest
            .modules
            .iter()
            .filter(|module| module.kind == PluginModuleKind::Runtime)
            .map(|module| module.name.clone())
            .collect::<Vec<_>>();
        if runtime_modules.is_empty() {
            runtime_modules.push(format!("{package_id}.runtime"));
        }
        runtime_modules.sort();
        runtime_modules.dedup();
        #[cfg(test)]
        {
            self.stats.runtime_modules_indexed += runtime_modules.len();
        }
        for module_name in &runtime_modules {
            self.providers_by_runtime_module
                .entry(module_name.clone())
                .or_insert_with(|| package_id.clone());
        }
        self.runtime_modules_by_provider
            .insert(package_id.clone(), runtime_modules);
    }

    fn index_base_capabilities(
        &mut self,
        registration: &RuntimePluginRegistrationReport,
        seen_by_package_and_target: &mut [HashMap<String, HashSet<String>>; 3],
    ) {
        for target in [
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::ServerRuntime,
            RuntimeTargetMode::EditorHost,
        ] {
            let target_index = target_index(target);
            let package_id = &registration.package_manifest.id;
            let seen = seen_by_package_and_target[target_index]
                .entry(package_id.clone())
                .or_default();
            let capabilities = self.base_capabilities_by_package_and_target[target_index]
                .entry(package_id.clone())
                .or_default();
            for capability in registration
                .package_manifest
                .modules
                .iter()
                .filter(|module| {
                    module.target_modes.is_empty() || module.target_modes.contains(&target)
                })
                .flat_map(|module| module.capabilities.iter())
            {
                if seen.insert(capability.clone()) {
                    capabilities.push(capability.clone());
                }
            }
        }
    }

    fn index_feature_registrations(
        &mut self,
        feature_registrations: &[RuntimePluginFeatureRegistrationReport],
    ) {
        for (index, registration) in feature_registrations.iter().enumerate() {
            self.feature_registration_indices_by_id
                .entry(registration.manifest.id.clone())
                .or_default()
                .push(index);
        }
    }

    fn index_feature_definitions(&mut self) {
        let definitions = &self.feature_definitions;
        let keys_by_owner = &mut self.feature_definition_keys_by_owner;
        let capability_providers = &mut self.feature_capability_providers_by_target;
        for definition_key in &self.feature_definitions.definition_order {
            let Some(definition) = definitions.definitions.get(definition_key) else {
                continue;
            };
            keys_by_owner
                .entry(definition.manifest.owner_plugin_id.clone())
                .or_default()
                .push(definition_key.clone());
            index_feature_capabilities(capability_providers, definition);
        }
    }

    fn index_bridge_dependents(&mut self, registrations: &[RuntimePluginRegistrationReport]) {
        for registration in registrations {
            let dependent_package_id = &registration.package_manifest.id;
            let mut interfaces_by_provider = HashMap::<&str, Vec<String>>::new();
            for dependency in &registration.package_manifest.dependencies {
                if dependency.required && !dependency.interfaces.is_empty() {
                    interfaces_by_provider
                        .entry(dependency.id.as_str())
                        .or_default()
                        .extend(dependency.interfaces.iter().cloned());
                }
            }
            for (provider_package_id, mut interface_ids) in interfaces_by_provider {
                interface_ids.sort();
                interface_ids.dedup();
                self.bridge_dependents_by_provider
                    .entry(provider_package_id.to_string())
                    .or_default()
                    .push(RuntimePluginBridgeDependent {
                        package_id: dependent_package_id.clone(),
                        interface_ids,
                    });
            }
        }
        for dependents in self.bridge_dependents_by_provider.values_mut() {
            dependents.sort_by(|left, right| left.package_id.cmp(&right.package_id));
        }
    }

    fn indexed_entry_count(&self) -> usize {
        self.feature_definitions.definitions.len()
            + self.feature_definitions.definition_order.len()
            + self.registration_indices_by_package.len()
            + self.feature_registration_indices_by_id.len()
            + self
                .feature_registration_indices_by_id
                .values()
                .map(Vec::len)
                .sum::<usize>()
            + self.feature_definition_keys_by_owner.len()
            + self
                .feature_definition_keys_by_owner
                .values()
                .map(Vec::len)
                .sum::<usize>()
            + self.runtime_modules_by_provider.len()
            + self
                .runtime_modules_by_provider
                .values()
                .map(Vec::len)
                .sum::<usize>()
            + self.providers_by_runtime_module.len()
            + self
                .base_capabilities_by_package_and_target
                .iter()
                .map(|by_package| {
                    by_package.len() + by_package.values().map(Vec::len).sum::<usize>()
                })
                .sum::<usize>()
            + self
                .feature_capability_providers_by_target
                .iter()
                .map(|by_capability| {
                    by_capability.len() + by_capability.values().map(Vec::len).sum::<usize>()
                })
                .sum::<usize>()
            + self.bridge_dependents_by_provider.len()
            + self
                .bridge_dependents_by_provider
                .values()
                .map(Vec::len)
                .sum::<usize>()
            + self.bridge_dependency_diagnostics.len()
    }

    fn indexed_string_bytes(&self) -> usize {
        map_string_key_bytes(&self.feature_definitions.definitions)
            + self
                .feature_definitions
                .definition_order
                .iter()
                .map(String::len)
                .sum::<usize>()
            + map_string_key_bytes(&self.registration_indices_by_package)
            + map_string_vec_bytes(&self.feature_registration_indices_by_id, |_| 0)
            + map_string_vec_bytes(&self.feature_definition_keys_by_owner, String::len)
            + map_string_vec_bytes(&self.runtime_modules_by_provider, String::len)
            + self
                .providers_by_runtime_module
                .iter()
                .map(|(module, provider)| module.len() + provider.len())
                .sum::<usize>()
            + self
                .base_capabilities_by_package_and_target
                .iter()
                .map(|by_package| map_string_vec_bytes(by_package, String::len))
                .sum::<usize>()
            + self
                .feature_capability_providers_by_target
                .iter()
                .map(|by_capability| map_string_vec_bytes(by_capability, String::len))
                .sum::<usize>()
            + self
                .bridge_dependents_by_provider
                .iter()
                .map(|(provider, dependents)| {
                    provider.len()
                        + dependents
                            .iter()
                            .map(|dependent| {
                                dependent.package_id.len()
                                    + dependent
                                        .interface_ids
                                        .iter()
                                        .map(String::len)
                                        .sum::<usize>()
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self
                .bridge_dependency_diagnostics
                .iter()
                .map(String::len)
                .sum::<usize>()
    }
}

fn map_string_key_bytes<T>(map: &HashMap<String, T>) -> usize {
    map.keys().map(String::len).sum()
}

fn map_string_vec_bytes<T>(
    map: &HashMap<String, Vec<T>>,
    value_bytes: impl Fn(&T) -> usize,
) -> usize {
    map.iter()
        .map(|(key, values)| {
            key.len() + values.iter().map(|value| value_bytes(value)).sum::<usize>()
        })
        .sum()
}

fn elapsed_nanos(started: Instant) -> u64 {
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn index_feature_capabilities(
    providers_by_target: &mut [HashMap<String, Vec<String>>; 3],
    definition: &FeatureDefinition,
) {
    for target in [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::ServerRuntime,
        RuntimeTargetMode::EditorHost,
    ] {
        let mut seen = HashSet::new();
        for capability in feature_capabilities_for_target(&definition.manifest, target) {
            if !seen.insert(capability) {
                continue;
            }
            providers_by_target[target_index(target)]
                .entry(capability.to_string())
                .or_default()
                .push(definition.key.clone());
        }
    }
}

fn target_index(target: RuntimeTargetMode) -> usize {
    match target {
        RuntimeTargetMode::ClientRuntime => 0,
        RuntimeTargetMode::ServerRuntime => 1,
        RuntimeTargetMode::EditorHost => 2,
    }
}

#[cfg(test)]
mod tests;
