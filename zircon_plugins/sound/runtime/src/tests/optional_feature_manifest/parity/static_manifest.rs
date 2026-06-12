use super::super::support::{optional_features_from_plugin_toml, STATIC_SOUND_PLUGIN_MANIFEST};
use super::helpers::{
    sorted_runtime_optional_feature_signatures, sorted_static_optional_feature_signatures,
};

#[test]
fn static_plugin_manifest_keeps_optional_feature_manifests_in_sync() {
    let static_features = sorted_static_optional_feature_signatures(
        optional_features_from_plugin_toml(STATIC_SOUND_PLUGIN_MANIFEST),
    );
    let runtime_features =
        sorted_runtime_optional_feature_signatures(crate::package_manifest().optional_features);

    assert_eq!(static_features, runtime_features);
}
