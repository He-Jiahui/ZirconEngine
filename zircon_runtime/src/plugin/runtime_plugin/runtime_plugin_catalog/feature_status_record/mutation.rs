use super::FeatureStatus;

impl FeatureStatus {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn mark_cycle(&mut self) {
        self.cycle = true;
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn mark_invalid_owner_dependency(
        &mut self,
    ) {
        self.invalid_owner_dependency = true;
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn mark_provider_missing(
        &mut self,
    ) {
        self.provider_missing = true;
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn mark_target_unsupported(
        &mut self,
    ) {
        self.target_unsupported = true;
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn add_missing_plugin(
        &mut self,
        plugin_id: String,
    ) {
        if self.missing_plugin_membership.insert(plugin_id.clone()) {
            self.missing_plugins.push(plugin_id);
        }
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn add_missing_capability(
        &mut self,
        capability: String,
    ) {
        if self
            .missing_capability_membership
            .insert(capability.clone())
        {
            self.missing_capabilities.push(capability);
        }
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn resolve_missing_capability(
        &mut self,
        capability: &str,
    ) -> bool {
        self.missing_capability_membership.remove(capability)
    }
}
