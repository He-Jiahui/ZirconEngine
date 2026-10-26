use zircon_plugin_ai_runtime::behavior_tree::{
    standard_node_catalog, BehaviorNodeCatalogError, BehaviorNodeSemantics as Semantics,
    FrozenBehaviorNodeCatalog,
};

use super::types::{
    AiBlackboardDependencies, AiBlackboardKeyReference, AiBlackboardKeyUsage as Usage,
    AiBlackboardProviderSnapshot, AiBlackboardReferenceBlocker as Blocker,
    AiBlackboardReferencePlan, AiBlackboardSchemaInput, AiBlackboardTreeInput,
};

impl<'a> AiBlackboardReferencePlan<'a> {
    /// Reads supplied immutable snapshots only. An empty result with blockers is
    /// unknown, never proof that the key has zero references.
    pub fn scan(
        schema: AiBlackboardSchemaInput<'a>,
        trees: &'a [AiBlackboardTreeInput<'a>],
        providers: AiBlackboardProviderSnapshot<'a>,
        dependencies: Option<AiBlackboardDependencies<'a>>,
    ) -> Result<Self, BehaviorNodeCatalogError> {
        let standard = standard_node_catalog()?;
        let mut plan = Self {
            schema,
            trees,
            providers,
            dependencies,
            references: Vec::new(),
            blockers: Vec::new(),
        };
        plan.check_enumeration();
        for index in 0..trees.len() {
            if plan.check_association(trees[index]) {
                plan.scan_tree(index, &standard);
            }
        }
        Ok(plan)
    }

    pub fn references(&self) -> &[AiBlackboardKeyReference] {
        &self.references
    }

    pub fn blockers(&self) -> &[Blocker] {
        &self.blockers
    }

    fn check_enumeration(&mut self) {
        if self.providers.registry_identity.trim().is_empty() {
            self.blockers.push(Blocker::MissingCatalogIdentity);
        }
        let Some(dependencies) = self.dependencies else {
            self.blockers.push(Blocker::MissingDependencySet);
            return;
        };
        if dependencies.catalog_identity.trim().is_empty() {
            self.blockers.push(Blocker::MissingCatalogIdentity);
        }
        for uri in dependencies.unresolved {
            self.blockers
                .push(Blocker::UnresolvedDependency { uri: uri.clone() });
        }
        for (index, tree) in dependencies.tree_assets.iter().enumerate() {
            if dependencies.tree_assets[..index].contains(tree) {
                self.blockers.push(Blocker::DuplicateTree { tree: *tree });
            }
            if !self
                .trees
                .iter()
                .any(|input| input.source.asset_uuid == *tree)
            {
                self.blockers.push(Blocker::MissingTree { tree: *tree });
            }
        }
        for (index, input) in self.trees.iter().enumerate() {
            let tree = input.source.asset_uuid;
            if !dependencies.tree_assets.contains(&tree) {
                self.blockers.push(Blocker::UnexpectedTree { tree });
            }
            if self.trees[..index]
                .iter()
                .any(|earlier| earlier.source.asset_uuid == tree)
            {
                self.blockers.push(Blocker::DuplicateTree { tree });
            }
        }
    }

    fn check_association(&mut self, input: AiBlackboardTreeInput<'_>) -> bool {
        let tree = input.source.asset_uuid;
        let Some(association) = input.association else {
            self.blockers.push(Blocker::MissingAssociation { tree });
            return false;
        };
        if association.tree != *input.source
            || association.tree_id != input.asset.descriptor().id
            || association.field_path.trim().is_empty()
        {
            self.blockers.push(Blocker::StaleAssociation { tree });
            return false;
        }
        // Matching schema text is insufficient. An explicit other UUID does not
        // reference this schema, even if both schemas use the same business ID.
        if association.schema.asset_uuid != self.schema.source.asset_uuid {
            return false;
        }
        if association.schema != *self.schema.source
            || association.schema_id != self.schema.descriptor.id
        {
            self.blockers.push(Blocker::StaleAssociation { tree });
            return false;
        }
        true
    }

    fn scan_tree(&mut self, tree_index: usize, standard: &FrozenBehaviorNodeCatalog) {
        let input = self.trees[tree_index];
        for (node_index, node) in input.asset.descriptor().nodes.iter().enumerate() {
            // Compiled order is traversal order, not authored descriptor order.
            let expected = input
                .asset
                .compiled()
                .nodes()
                .iter()
                .find(|compiled| compiled.id() == node.id)
                .and_then(|compiled| standard.get(compiled.implementation()));
            let live = expected.and_then(|expected| self.providers.catalog.resolve(expected.id()));
            if expected.is_none()
                || live.and_then(|slot| self.providers.catalog.get(slot)) != expected
                || live.and_then(|slot| self.providers.catalog.owner_for_slot(slot))
                    != Some(self.providers.builtin_provider)
            {
                self.blockers.push(Blocker::UnknownProvider {
                    tree: input.source.asset_uuid,
                    node_id: node.id.clone(),
                });
                continue;
            }
            let expected = expected.expect("checked standard descriptor");
            let semantics = expected.semantics();
            // Script and external nodes can read arbitrary keys. Subtrees need
            // real target resolution; the current Service contract is incomplete.
            if matches!(
                semantics,
                Semantics::ScriptTask
                    | Semantics::External
                    | Semantics::RunSubtree
                    | Semantics::UpdateBlackboardDistance
            ) {
                self.blockers.push(Blocker::OpaqueNode {
                    tree: input.source.asset_uuid,
                    node_id: node.id.clone(),
                    semantics,
                });
                continue;
            }
            for (parameter_index, parameter) in node.parameters.iter().enumerate() {
                if parameter.key != "blackboard_key" || parameter.value.as_string().is_none() {
                    continue;
                }
                let key = parameter
                    .value
                    .as_string()
                    .expect("checked string parameter");
                if !self
                    .schema
                    .descriptor
                    .keys
                    .iter()
                    .any(|entry| entry.key == key)
                {
                    self.blockers.push(Blocker::UnboundKey {
                        tree: input.source.asset_uuid,
                        node_id: node.id.clone(),
                        key: key.to_owned(),
                    });
                    continue;
                }
                let usage = match semantics {
                    Semantics::SetBlackboard => Usage::EffectWrite,
                    Semantics::BlackboardCondition => Usage::ConditionAndObserverSlot,
                    _ => Usage::ObserverSlot,
                };
                self.references.push(AiBlackboardKeyReference {
                    tree: input.source.clone(),
                    node_id: node.id.clone(),
                    implementation: expected.id().to_owned(),
                    provider: self.providers.builtin_provider,
                    original: parameter.clone(),
                    usage,
                    tree_index,
                    node_index,
                    parameter_index,
                });
            }
        }
    }
}
