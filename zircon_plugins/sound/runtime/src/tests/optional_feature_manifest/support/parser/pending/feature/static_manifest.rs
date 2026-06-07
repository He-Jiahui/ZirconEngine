mod required;

use super::super::super::super::types::{
    PendingOptionalFeatureManifest, StaticOptionalFeatureManifest,
};

pub(super) fn static_optional_feature_manifest(
    feature: PendingOptionalFeatureManifest,
) -> StaticOptionalFeatureManifest {
    StaticOptionalFeatureManifest {
        id: required::take_required_id(feature.id),
        display_name: required::take_required_display_name(feature.display_name),
        owner_plugin_id: required::take_required_owner_plugin_id(feature.owner_plugin_id),
        capabilities: feature.capabilities,
        default_packaging: feature.default_packaging,
        enabled_by_default: feature.enabled_by_default.unwrap_or(false),
        dependencies: feature.dependencies,
        modules: feature.modules,
    }
}
