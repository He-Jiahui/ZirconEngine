use crate::core::framework::project::{
    ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection,
};
use crate::plugin::{PluginFeatureBundleManifest, RuntimePluginFeatureRegistrationReport};

use super::{feature_registration_matches_project_selection, project_feature_provider_lookup};

#[test]
fn provider_lookup_keeps_raw_manifest_spelling_while_matching_alias_registration() {
    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin("audio", true, false)
            .with_feature(ProjectPluginFeatureSelection::new("sound.spatial"))],
    };
    let providers = project_feature_provider_lookup(&manifest);
    assert_eq!(providers["sound.spatial"], "audio");

    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        PluginFeatureBundleManifest::new("sound.spatial", "Spatial", "sound"),
        Some("sound".to_string()),
    );
    assert!(feature_registration_matches_project_selection(
        &registration,
        &providers,
        "sound.spatial",
    ));
}

#[test]
fn provider_match_rejects_unknown_and_malformed_ids() {
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin("unknown", true, false)
                .with_feature(ProjectPluginFeatureSelection::new("sound.spatial")),
        ],
    };
    let providers = project_feature_provider_lookup(&manifest);
    let registration = RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        PluginFeatureBundleManifest::new("sound.spatial", "Spatial", "sound"),
        Some("bad/id".to_string()),
    );

    assert!(!feature_registration_matches_project_selection(
        &registration,
        &providers,
        "sound.spatial",
    ));
}
