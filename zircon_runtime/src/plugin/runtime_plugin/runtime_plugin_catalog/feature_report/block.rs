#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginFeatureBlock {
    pub feature_id: String,
    pub owner_plugin_id: String,
    pub required: bool,
    pub missing_plugins: Vec<String>,
    pub missing_capabilities: Vec<String>,
    pub target_unsupported: bool,
    pub cycle: bool,
    pub invalid_owner_dependency: bool,
    pub provider_missing: bool,
    pub unknown_feature: bool,
}
