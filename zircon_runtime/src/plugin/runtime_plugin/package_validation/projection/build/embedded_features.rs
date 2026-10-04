use std::collections::HashSet;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::super::duplicate_identity::DuplicateIdentity;
use super::super::duplicate_occurrence::{DuplicateOccurrence, EmbeddedFeatureKind};
use super::index_identity;

/// 统一索引两类内嵌特性，供特性校验借用包视图，避免逐特性重建重复集合。
/// 提供者身份按特性与提供包组合去重，缺省提供包沿用宿主包。
pub(super) fn index_embedded_features<'a>(
    manifest: &'a PluginPackageManifest,
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    count: &mut usize,
) {
    for (kind, features) in [
        (
            EmbeddedFeatureKind::Optional,
            manifest.optional_features.as_slice(),
        ),
        (
            EmbeddedFeatureKind::Extension,
            manifest.feature_extensions.as_slice(),
        ),
    ] {
        for (feature_index, feature) in features.iter().enumerate() {
            let provider = feature
                .provider_package_id
                .as_deref()
                .unwrap_or(manifest.id.as_str());
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::EmbeddedFeatureProvider {
                    feature: &feature.id,
                    provider,
                },
                DuplicateOccurrence::EmbeddedFeatureProvider {
                    kind,
                    feature: feature_index,
                },
                count,
            );
            index_feature_rows(kind, feature_index, feature, seen, duplicates, count);
        }
    }
}

// 特性内部身份带上类别和原清单行号，因此两个不同特性可以合法使用相同的局部名称。
fn index_feature_rows<'a>(
    kind: EmbeddedFeatureKind,
    feature_index: usize,
    feature: &'a PluginFeatureBundleManifest,
    seen: &mut HashSet<DuplicateIdentity<'a>>,
    duplicates: &mut HashSet<DuplicateOccurrence>,
    count: &mut usize,
) {
    for (capability_index, capability) in feature.capabilities.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::FeatureCapability {
                kind,
                feature: feature_index,
                value: capability,
            },
            DuplicateOccurrence::FeatureCapability {
                kind,
                feature: feature_index,
                capability: capability_index,
            },
            count,
        );
    }
    for (dependency_index, dependency) in feature.dependencies.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::FeatureDependency {
                kind,
                feature: feature_index,
                provider: &dependency.plugin_id,
                capability: &dependency.capability,
            },
            DuplicateOccurrence::FeatureDependency {
                kind,
                feature: feature_index,
                dependency: dependency_index,
            },
            count,
        );
    }
    for (module_index, module) in feature.modules.iter().enumerate() {
        index_identity(
            seen,
            duplicates,
            DuplicateIdentity::FeatureModuleName {
                kind,
                feature: feature_index,
                value: &module.name,
            },
            DuplicateOccurrence::FeatureModuleName {
                kind,
                feature: feature_index,
                module: module_index,
            },
            count,
        );
        for (capability_index, capability) in module.capabilities.iter().enumerate() {
            index_identity(
                seen,
                duplicates,
                DuplicateIdentity::FeatureModuleCapability {
                    kind,
                    feature: feature_index,
                    module: module_index,
                    value: capability,
                },
                DuplicateOccurrence::FeatureModuleCapability {
                    kind,
                    feature: feature_index,
                    module: module_index,
                    capability: capability_index,
                },
                count,
            );
        }
    }
}
