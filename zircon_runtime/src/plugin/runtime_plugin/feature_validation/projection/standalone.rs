use std::collections::HashSet;

use crate::plugin::PluginFeatureBundleManifest;

#[derive(Default)]
pub(super) struct StandaloneFeatureValidationProjection {
    duplicate_capabilities: HashSet<usize>,
    duplicate_dependencies: HashSet<usize>,
    duplicate_module_names: HashSet<usize>,
    duplicate_module_capabilities: HashSet<(usize, usize)>,
}

impl StandaloneFeatureValidationProjection {
    pub(super) fn build(feature: &PluginFeatureBundleManifest) -> Self {
        let mut projection = Self::default();

        let mut capabilities = HashSet::new();
        for (index, capability) in feature.capabilities.iter().enumerate() {
            if !capabilities.insert(capability.as_str()) {
                projection.duplicate_capabilities.insert(index);
            }
        }

        let mut dependencies = HashSet::new();
        for (index, dependency) in feature.dependencies.iter().enumerate() {
            if !dependencies.insert((
                dependency.plugin_id.as_str(),
                dependency.capability.as_str(),
            )) {
                projection.duplicate_dependencies.insert(index);
            }
        }

        let mut module_names = HashSet::new();
        for (module_index, module) in feature.modules.iter().enumerate() {
            if !module_names.insert(module.name.as_str()) {
                projection.duplicate_module_names.insert(module_index);
            }
            let mut module_capabilities = HashSet::new();
            for (capability_index, capability) in module.capabilities.iter().enumerate() {
                if !module_capabilities.insert(capability.as_str()) {
                    projection
                        .duplicate_module_capabilities
                        .insert((module_index, capability_index));
                }
            }
        }

        projection
    }

    pub(super) fn capability_is_duplicate(&self, capability: usize) -> bool {
        self.duplicate_capabilities.contains(&capability)
    }

    pub(super) fn dependency_is_duplicate(&self, dependency: usize) -> bool {
        self.duplicate_dependencies.contains(&dependency)
    }

    pub(super) fn module_name_is_duplicate(&self, module: usize) -> bool {
        self.duplicate_module_names.contains(&module)
    }

    pub(super) fn module_capability_is_duplicate(&self, module: usize, capability: usize) -> bool {
        self.duplicate_module_capabilities
            .contains(&(module, capability))
    }
}
