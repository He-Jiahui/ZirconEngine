use crate::plugin::runtime_plugin::{
    embedded_feature_provider_identity::runtime_plugin_package_feature_provider_package_id,
    package_validation::EmbeddedFeatureKind,
};
use crate::plugin::PluginPackageManifest;

use super::feature_definitions::FeatureDefinition;

fn package_feature_definition_capacity(package_manifest: &PluginPackageManifest) -> usize {
    package_manifest
        .optional_features
        .len()
        .saturating_add(package_manifest.feature_extensions.len())
}

pub(super) fn package_feature_definitions(
    package_manifest: &PluginPackageManifest,
) -> Vec<FeatureDefinition> {
    let mut definitions = Vec::with_capacity(package_feature_definition_capacity(package_manifest));
    visit_package_feature_definitions(package_manifest, |definition| definitions.push(definition));
    definitions
}

/// 遍历顺序为 optional_features 后 feature_extensions；普通 owner 包保留显式 provider，扩展包以自身登记 ID 作为提供者。
/// Visits package-owned feature definitions in the canonical optional-first order.
///
/// The materializing helper above remains available for callers that need an owned list, while
/// catalog merges can consume this visitor directly and avoid one temporary vector per package.
pub(super) fn visit_package_feature_definitions(
    package_manifest: &PluginPackageManifest,
    mut visit: impl FnMut(FeatureDefinition),
) {
    for feature in &package_manifest.optional_features {
        let provider_package_id = runtime_plugin_package_feature_provider_package_id(
            package_manifest,
            feature,
            EmbeddedFeatureKind::Optional,
        )
        .to_owned();
        visit(FeatureDefinition::new(feature.clone(), provider_package_id));
    }
    for feature in &package_manifest.feature_extensions {
        visit(FeatureDefinition::new(
            feature.clone(),
            runtime_plugin_package_feature_provider_package_id(
                package_manifest,
                feature,
                EmbeddedFeatureKind::Extension,
            )
            .to_owned(),
        ));
    }
}

#[cfg(test)]
#[path = "tests/package_feature_definitions.rs"]
mod tests;

#[cfg(test)]
#[path = "../../catalog/tests/package_feature_definitions_capacity.rs"]
mod capacity_tests;
