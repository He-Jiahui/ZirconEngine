use std::collections::HashMap;

use crate::core::framework::project::ProjectPluginFeatureSelection;
use crate::plugin::PluginFeatureBundleManifest;

use super::super::{FeatureDefinition, FeatureDefinitionMap};

#[test]
fn selection_without_provider_does_not_fallback_to_unique_external_definition() {
    let definition = FeatureDefinition::new(
        PluginFeatureBundleManifest::new(
            "sound.timeline_animation_track",
            "Timeline Animation Track",
            "sound",
        ),
        "sound_timeline_animation_track".to_string(),
    );
    let definitions = FeatureDefinitionMap {
        definitions: HashMap::from([(definition.key.clone(), definition)]),
        diagnostics: Vec::new(),
        definition_order: Vec::new(),
    };

    let resolved = definitions.definition_for_selection(
        "sound",
        &ProjectPluginFeatureSelection::new("sound.timeline_animation_track"),
    );

    assert!(
        resolved.is_none(),
        "external providers require an explicit provider identity"
    );
}

#[test]
fn selection_provider_alias_resolves_the_canonical_definition() {
    let definition = FeatureDefinition::new(
        PluginFeatureBundleManifest::new("sound.spatial", "Spatial", "sound"),
        "audio".to_string(),
    );
    let definition_key = definition.key.clone();
    let definitions = FeatureDefinitionMap {
        definitions: HashMap::from([(definition_key.clone(), definition)]),
        diagnostics: Vec::new(),
        definition_order: vec![definition_key],
    };
    let selection =
        ProjectPluginFeatureSelection::new("sound.spatial").with_provider_package_id("sound");

    let resolved = definitions.definition_for_selection("sound", &selection);

    assert_eq!(
        resolved.map(|definition| definition.provider_package_id.as_str()),
        Some("audio")
    );
}
