use zircon_plugin_ai_runtime::DefaultAiManager;
use zircon_runtime::core::framework::ai::{AiBehaviorNodeParameter, AiManager, AiManagerError};

use super::super::{AiBehaviorTreeAsset, AiBehaviorTreeEdit};
use super::types::{
    AiBlackboardReferencePlan, AiBlackboardRenameCandidate, AiBlackboardRenameError as Error,
    AiBlackboardTreeRewrite,
};

impl AiBlackboardReferencePlan<'_> {
    /// Builds business values and inverses only. The caller still owes real source
    /// closure, memory admission, revision/hash CAS, history and atomic host publication.
    pub fn rename_key(
        &self,
        old: &str,
        new: &str,
    ) -> Result<Option<AiBlackboardRenameCandidate>, Error> {
        if !self.blockers.is_empty() {
            return Err(Error::Blocked(self.blockers.clone()));
        }
        if !self.schema.writable {
            return Err(Error::ReadOnly {
                asset: self.schema.source.asset_uuid,
            });
        }
        let schema = self.schema.descriptor;
        let key_index = schema
            .keys
            .iter()
            .position(|key| key.key == old)
            .ok_or_else(|| Error::MissingKey(old.to_owned()))?;
        // Use the actual public runtime validation in a fresh isolated manager,
        // without replacing any schema or obtaining a live world's manager.
        validate_schema(schema).map_err(Error::InvalidSchema)?;
        if old == new {
            return Ok(None);
        }
        if schema.keys.iter().any(|key| key.key == new) {
            return Err(Error::DuplicateKey(new.to_owned()));
        }
        let mut schema_after = schema.clone();
        schema_after.keys[key_index].key = new.to_owned();
        validate_schema(&schema_after).map_err(Error::InvalidSchema)?;

        let mut trees = Vec::new();
        for (tree_index, input) in self.trees.iter().enumerate() {
            let references = self.references.iter().filter(|reference| {
                reference.tree_index == tree_index
                    && reference.original.value.as_string() == Some(old)
            });
            let mut descriptor = None;
            let mut forward = Vec::new();
            let mut inverse = Vec::new();
            for reference in references {
                if !input.writable {
                    return Err(Error::ReadOnly {
                        asset: input.source.asset_uuid,
                    });
                }
                let descriptor = descriptor.get_or_insert_with(|| input.asset.descriptor().clone());
                let replacement = AiBehaviorNodeParameter::new("blackboard_key", new);
                descriptor.nodes[reference.node_index].parameters[reference.parameter_index]
                    .value
                    .clone_from(&replacement.value);
                forward.push(AiBehaviorTreeEdit::SetParameter {
                    node_id: reference.node_id.clone(),
                    parameter: replacement,
                });
                inverse.push(AiBehaviorTreeEdit::SetParameter {
                    node_id: reference.node_id.clone(),
                    parameter: reference.original.clone(),
                });
            }
            let Some(descriptor) = descriptor else {
                continue;
            };
            // Validate the whole rewritten tree once; this retains set_blackboard's
            // UTF-8 byte limit without imposing that limit on every schema key.
            let candidate = AiBehaviorTreeAsset::from_descriptor(descriptor).map_err(|error| {
                Error::InvalidTree {
                    asset: input.source.asset_uuid,
                    error,
                }
            })?;
            inverse.reverse();
            trees.push(AiBlackboardTreeRewrite {
                source: input.source.clone(),
                candidate,
                forward,
                inverse,
            });
        }
        let dependencies = self
            .dependencies
            .expect("missing dependency set is blocked");
        Ok(Some(AiBlackboardRenameCandidate {
            schema_source: self.schema.source.clone(),
            schema_before: schema.clone(),
            schema_after,
            trees,
            source_preconditions: self
                .trees
                .iter()
                .map(|input| input.source.clone())
                .collect(),
            association_preconditions: self
                .trees
                .iter()
                .filter_map(|input| input.association.cloned())
                .collect(),
            provider_registry: self.providers.registry_identity.to_owned(),
            provider_revision: self.providers.revision,
            builtin_provider: self.providers.builtin_provider,
            dependency_catalog: dependencies.catalog_identity.to_owned(),
            dependency_revision: dependencies.revision,
            dependency_tree_assets: dependencies.tree_assets.to_vec(),
        }))
    }
}

pub(super) fn validate_schema(
    descriptor: &zircon_runtime::core::framework::ai::AiBlackboardSchemaDescriptor,
) -> Result<(), AiManagerError> {
    DefaultAiManager::default()
        .register_blackboard_schema(descriptor.clone())
        .map(|_| ())
}
