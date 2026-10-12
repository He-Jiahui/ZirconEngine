use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::package_feature_definitions;

#[test]
fn ordinary_owner_package_preserves_explicit_external_feature_provider() {
    let manifest = PluginPackageManifest::new("sound", "Sound").with_optional_feature(
        PluginFeatureBundleManifest::new(
            "sound.timeline_animation_track",
            "Timeline Animation Track",
            "sound",
        )
        .with_provider_package_id("sound_timeline_animation_track"),
    );

    let definitions = package_feature_definitions(&manifest);

    assert_eq!(definitions.len(), 1);
    assert_eq!(
        definitions[0].key,
        "sound.timeline_animation_track@sound_timeline_animation_track"
    );
    assert_eq!(
        definitions[0].provider_package_id,
        "sound_timeline_animation_track"
    );
}

#[test]
fn ordinary_owner_package_defaults_feature_provider_to_owner() {
    let manifest = PluginPackageManifest::new("sound", "Sound").with_optional_feature(
        PluginFeatureBundleManifest::new("sound.spatial_audio", "Spatial Audio", "sound"),
    );

    let definitions = package_feature_definitions(&manifest);

    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].key, "sound.spatial_audio@sound");
    assert_eq!(definitions[0].provider_package_id, "sound");
}
