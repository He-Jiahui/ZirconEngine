use super::RuntimePluginFeatureBlock;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginFeatureDependencyReport {
    pub available_features: Vec<String>,
    pub blocked_features: Vec<RuntimePluginFeatureBlock>,
    pub diagnostics: Vec<String>,
}
