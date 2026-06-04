use super::super::super::types::{PendingOptionalFeatureManifest, StaticOptionalFeatureManifest};

pub(in super::super) fn push_optional_feature(
    features: &mut Vec<StaticOptionalFeatureManifest>,
    feature: &mut Option<PendingOptionalFeatureManifest>,
) {
    let Some(mut feature) = feature.take() else {
        return;
    };
    feature.capabilities.sort_unstable();
    feature.dependencies.sort_unstable();
    feature
        .modules
        .sort_unstable_by_key(|module| module.0.clone());
    features.push(StaticOptionalFeatureManifest {
        id: feature.id.expect("optional feature should declare id"),
        display_name: feature
            .display_name
            .expect("optional feature should declare display name"),
        owner_plugin_id: feature
            .owner_plugin_id
            .expect("optional feature should declare owner plugin id"),
        capabilities: feature.capabilities,
        default_packaging: feature.default_packaging,
        enabled_by_default: feature.enabled_by_default.unwrap_or(false),
        dependencies: feature.dependencies,
        modules: feature.modules,
    });
}
