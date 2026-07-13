use crate::core::framework::project::ProjectPluginFeatureSelection;

use super::key::feature_definition_key;
use super::{FeatureDefinition, FeatureDefinitionMap};

impl FeatureDefinitionMap {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn definition_for_selection(
        &self,
        owner_plugin_id: &str,
        feature: &ProjectPluginFeatureSelection,
    ) -> Option<&FeatureDefinition> {
        let requested_provider = feature
            .provider_package_id
            .as_deref()
            .unwrap_or(owner_plugin_id);
        let preferred_key = feature_definition_key(&feature.id, requested_provider);
        self.definitions.get(&preferred_key)
    }
}

#[cfg(test)]
mod tests {
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
}
