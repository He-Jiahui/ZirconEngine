use crate::plugin::PluginFeatureDependency;

use super::super::{owner, primary_count};

#[derive(Default)]
pub(super) struct FeaturePrimaryDependencyRows {
    primary_count: usize,
}

impl FeaturePrimaryDependencyRows {
    pub(super) fn validate(
        &mut self,
        dependency: &PluginFeatureDependency,
        owner_plugin_id: &str,
        diagnostics: &mut Vec<String>,
    ) {
        self.primary_count += owner::validate_runtime_plugin_feature_primary_dependency_owner(
            dependency,
            owner_plugin_id,
            diagnostics,
        );
    }

    pub(super) fn validate_count(self, diagnostics: &mut Vec<String>) {
        primary_count::validate_runtime_plugin_feature_primary_dependency_count(
            self.primary_count,
            diagnostics,
        );
    }
}
