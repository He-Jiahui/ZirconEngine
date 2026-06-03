mod pairs;
mod primary;

use crate::plugin::PluginFeatureBundleManifest;

use self::{pairs::FeatureDependencyPairRows, primary::FeaturePrimaryDependencyRows};
use super::row;

pub(super) fn validate_runtime_plugin_feature_dependency_rows(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut pairs = FeatureDependencyPairRows::default();
    let mut primary = FeaturePrimaryDependencyRows::default();
    for dependency in &feature.dependencies {
        row::validate_runtime_plugin_feature_dependency_row(dependency, diagnostics);
        pairs.validate(dependency, diagnostics);
        primary.validate(dependency, &feature.owner_plugin_id, diagnostics);
    }
    primary.validate_count(diagnostics);
}
