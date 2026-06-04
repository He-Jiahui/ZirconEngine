use super::support::{
    optional_feature_signature, optional_features_from_plugin_toml, STATIC_SOUND_PLUGIN_MANIFEST,
};

#[test]
fn static_plugin_manifest_keeps_optional_feature_manifests_in_sync() {
    let mut static_features = optional_features_from_plugin_toml(STATIC_SOUND_PLUGIN_MANIFEST);
    let mut runtime_features = crate::package_manifest()
        .optional_features
        .iter()
        .map(optional_feature_signature)
        .collect::<Vec<_>>();
    static_features.sort_unstable_by(|left, right| left.id().cmp(right.id()));
    runtime_features.sort_unstable_by(|left, right| left.id().cmp(right.id()));

    assert_eq!(static_features, runtime_features);
}
