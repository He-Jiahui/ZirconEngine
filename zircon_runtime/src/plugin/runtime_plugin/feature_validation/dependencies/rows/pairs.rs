use crate::plugin::PluginFeatureDependency;

use super::super::pairs::validate_runtime_plugin_feature_dependency_pair;

#[derive(Default)]
pub(super) struct FeatureDependencyPairRows<'a> {
    seen: Vec<(&'a str, &'a str)>,
}

impl<'a> FeatureDependencyPairRows<'a> {
    pub(super) fn validate(
        &mut self,
        dependency: &'a PluginFeatureDependency,
        diagnostics: &mut Vec<String>,
    ) {
        validate_runtime_plugin_feature_dependency_pair(dependency, &mut self.seen, diagnostics);
    }
}
