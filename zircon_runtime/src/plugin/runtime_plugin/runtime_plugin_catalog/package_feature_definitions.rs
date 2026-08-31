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
        definitions.push(FeatureDefinition::new(feature.clone(), provider_package_id));
    }
    for feature in &package_manifest.feature_extensions {
        definitions.push(FeatureDefinition::new(
            feature.clone(),
            package_manifest.id.clone(),
        ));
    }
    definitions
}

#[cfg(test)]
mod tests {
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
}

#[cfg(test)]
#[path = "package_feature_definitions/capacity_tests.rs"]
mod capacity_tests;
