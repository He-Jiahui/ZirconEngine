use super::active::ActiveFeatureSelection;

#[derive(Clone, Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct PendingFeatureSelection<'a> {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) active:
        ActiveFeatureSelection<'a>,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) definition_key: String,
}
