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

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn mark_target_unsupported(
        &mut self,
    ) {
        self.target_unsupported = true;
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn add_missing_plugin(
        &mut self,
        plugin_id: String,
    ) {
        push_unique(&mut self.missing_plugins, plugin_id);
    }

    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn add_missing_capability(
        &mut self,
        capability: String,
    ) {
        push_unique(&mut self.missing_capabilities, capability);
    }
}

fn push_unique(collection: &mut Vec<String>, value: String) {
    if !collection.contains(&value) {
        collection.push(value);
    }
}
