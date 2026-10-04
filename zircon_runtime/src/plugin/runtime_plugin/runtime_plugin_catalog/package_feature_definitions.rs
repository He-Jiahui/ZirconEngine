use crate::plugin::{PluginPackageKind, PluginPackageManifest};

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
        let provider_package_id = if package_manifest.package_kind
            == PluginPackageKind::FeatureExtension
            || feature.owner_plugin_id != package_manifest.id
        {
            package_manifest.id.clone()
        } else {
            feature
                .provider_package_id
                .clone()
                .unwrap_or_else(|| feature.owner_plugin_id.clone())
        };
        visit(FeatureDefinition::new(feature.clone(), provider_package_id));
    }
    for feature in &package_manifest.feature_extensions {
        visit(FeatureDefinition::new(
            feature.clone(),
            package_manifest.id.clone(),
        ));
    }
}

#[cfg(test)]
#[path = "tests/package_feature_definitions.rs"]
mod tests;

#[cfg(test)]
#[path = "package_feature_definitions/tests/capacity_tests.rs"]
mod capacity_tests;
