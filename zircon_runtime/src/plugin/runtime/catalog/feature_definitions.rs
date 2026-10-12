#[path = "feature_definitions/definition.rs"]
mod definition;
#[path = "feature_definitions/definition_map.rs"]
mod definition_map;
#[path = "feature_definitions/key.rs"]
mod key;
#[path = "feature_definitions/lookup.rs"]
mod lookup;

pub(super) use definition::FeatureDefinition;
pub(super) use definition_map::FeatureDefinitionMap;
