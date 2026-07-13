use crate::core::framework::project::ProjectPluginManifest;

use super::super::feature_definitions::FeatureDefinitionMap;
use super::super::feature_report::RuntimePluginFeatureBlock;
use super::active::active_feature_selections;
use super::pending::PendingFeatureSelection;

#[derive(Clone, Debug)]
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) struct FeatureSelectionPartition<'a> {
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) pending:
        Vec<PendingFeatureSelection<'a>>,
    pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) unknown_feature_blocks:
        Vec<RuntimePluginFeatureBlock>,
}

/// Unknown feature rows are blocked before capability resolution so the loop only sees catalog-backed definitions.
pub(in crate::plugin::runtime_plugin::runtime_plugin_catalog) fn feature_selection_partition<'a>(
    manifest: &'a ProjectPluginManifest,
    feature_definitions: &FeatureDefinitionMap,
) -> FeatureSelectionPartition<'a> {
    let mut pending = Vec::new();
    let mut unknown_feature_blocks = Vec::new();
    for active in active_feature_selections(manifest) {
        if let Some(feature_definition) =
            feature_definitions.definition_for_selection(&active.owner_plugin_id, active.feature)
        {
            pending.push(PendingFeatureSelection {
                active,
                definition_key: feature_definition.key.clone(),
            });
        } else {
            unknown_feature_blocks.push(RuntimePluginFeatureBlock {
                feature_id: active.feature.id.clone(),
                owner_plugin_id: active.owner_plugin_id.clone(),
                required: active.feature.required,
                unknown_feature: true,
                ..RuntimePluginFeatureBlock::default()
            });
        }
    }
    FeatureSelectionPartition {
        pending,
        unknown_feature_blocks,
    }
}
