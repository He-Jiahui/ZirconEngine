use std::collections::{HashMap, HashSet};

use super::super::error::RenderGraphError;
use super::super::types::{
    RenderGraphAttachmentLoadOp, RenderGraphAttachmentStoreOp, RenderGraphExternalResourceType,
    RenderGraphResource, RenderGraphResourceVersion, RenderGraphResourceVersionToken, RenderPassId,
};
use super::super::RenderGraphResourceAccessMetadata;
use super::access_scope_tracker::{
    token_covers_scope, AccessScopeTracker, LatestWriter, PassScopeConflictTracker,
    ResourceAccessHistory,
};
use super::{RenderGraphBuilder, RenderPassNode, ResourceAccessKind};

/// Compiler-only dependency and version facts lowered from declared resource accesses.
pub(super) struct InferredResourceDependencies {
    pub(super) execution: Vec<Vec<RenderPassId>>,
    pub(super) culling: Vec<Vec<RenderPassId>>,
    pub(super) cull_roots: Vec<RenderPassId>,
    pub(super) resource_access_versions: Vec<Vec<RenderGraphResourceVersion>>,
    pub(super) resource_access_metadata: Vec<Vec<RenderGraphResourceAccessMetadata>>,
    pub(super) resource_access_visit_count: usize,
    pub(super) execution_dependency_count: usize,
    pub(super) provenance_dependency_count: usize,
}

impl RenderGraphBuilder {
    pub(super) fn add_explicit_version_dependencies(
        &self,
        dependencies: &mut [Vec<RenderPassId>],
    ) -> Result<(), RenderGraphError> {
        for pass in &self.passes {
            for access in &pass.resources {
                let Some(token) = access.input_version else {
                    continue;
                };
                self.validate_resource_version_token(pass.id, access.resource, token)?;
                let producer = token.producer_pass();
                if producer == pass.id {
                    return Err(RenderGraphError::ResourceVersionSelfDependency {
                        pass: pass.name.clone(),
                        resource: self.resource_name(access.resource),
                    });
                }
                if !dependencies[pass.id.0].contains(&producer) {
                    dependencies[pass.id.0].push(producer);
                }
            }
        }
        Ok(())
    }

    pub(super) fn resolve_compiled_input_version(
        &self,
        token: RenderGraphResourceVersionToken,
        inferred: &InferredResourceDependencies,
    ) -> Result<RenderGraphResourceVersion, RenderGraphError> {
        let Some(producer_pass) = self.passes.get(token.producer_pass().index()) else {
            return Err(RenderGraphError::ResourceVersionProducerMissing {
                producer_pass: token.producer_pass().index(),
                producer_access: token.producer_access_index(),
            });
        };
        inferred.resource_access_versions[token.producer_pass().index()]
            .get(token.producer_access_index())
            .copied()
            .ok_or_else(|| RenderGraphError::ResourceVersionUnavailable {
                pass: producer_pass.name.clone(),
                resource: self.resource_name(token.resource()),
                producer: producer_pass.name.clone(),
            })
    }

    pub(super) fn infer_resource_dependencies(
        &self,
        resource_names: &HashMap<RenderGraphResource, &str>,
        pass_order: &[RenderPassId],
        manual_dependencies: Vec<Vec<RenderPassId>>,
    ) -> Result<InferredResourceDependencies, RenderGraphError> {
        let mut execution_dependencies =
            DependencyAdjacency::from_manual_dependencies(manual_dependencies.clone());
        let mut culling_dependencies =
            DependencyAdjacency::from_manual_dependencies(manual_dependencies);
        let resource_access_identities = self.resource_access_identities()?;
        let mut resource_accesses = AccessScopeTracker::new(&self.resources);
        let mut resource_access_versions = vec![Vec::new(); self.passes.len()];
        let mut resource_access_metadata = vec![Vec::new(); self.passes.len()];
        let mut resource_access_visit_count = 0;

        for pass_id in pass_order {
            let pass = &self.passes[pass_id.0];
            let mut same_pass_conflicts = PassScopeConflictTracker::default();
            for (access_index, access) in pass.resources.iter().enumerate() {
                resource_access_visit_count += 1;
                let access_identity = resource_access_identities
                    .get(&access.resource)
                    .copied()
                    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                        resource: resource_name(resource_names, access.resource),
                    })?;
                let scope = resource_accesses.prepare_scope(
                    access_identity,
                    access.resource,
                    access.metadata,
                )?;
                resource_access_metadata[pass.id.0].push(scope.metadata());
                if let Some(first_access) = same_pass_conflicts.register(
                    &scope,
                    matches!(access.kind, ResourceAccessKind::Write),
                    access_index,
                ) {
                    let first_resource = pass
                        .resources
                        .get(first_access)
                        .map(|access| access.resource)
                        .ok_or(RenderGraphError::AccessScopeTrackerStateMismatch {
                            identity: access_identity,
                        })?;
                    return Err(RenderGraphError::OverlappingPassResourceAccessScope {
                        pass: pass.name.clone(),
                        first_resource: resource_name(resource_names, first_resource),
                        first_access,
                        second_resource: resource_name(resource_names, access.resource),
                        second_access: access_index,
                        access: match access.kind {
                            ResourceAccessKind::Read => {
                                super::super::types::RenderGraphResourceAccessKind::Read
                            }
                            ResourceAccessKind::Write => {
                                super::super::types::RenderGraphResourceAccessKind::Write
                            }
                        },
                    });
                }
                let histories = resource_accesses.histories_for(&scope)?;
                match access.kind {
                    ResourceAccessKind::Read => {
                        if let Some(token) = access.input_version {
                            self.validate_current_resource_version(
                                resource_names,
                                pass,
                                access.resource,
                                token,
                                &histories,
                                scope.is_precise(),
                            )?;
                        }
                        let mut latest_version_ordinal = 0;
                        let mut has_unwritten_scope = false;
                        for history in &histories {
                            latest_version_ordinal =
                                latest_version_ordinal.max(history.latest_version_ordinal);
                            let Some(writer) = history.latest_writer else {
                                has_unwritten_scope = true;
                                continue;
                            };
                            if writer.store == RenderGraphAttachmentStoreOp::Discard {
                                return Err(RenderGraphError::ReadAfterDiscardedStore {
                                    resource: resource_name(resource_names, access.resource),
                                    pass: pass.name.clone(),
                                    producer: self.passes[writer.pass.0].name.clone(),
                                });
                            }
                            execution_dependencies.add_dependency(writer.pass, pass.id);
                            culling_dependencies.add_dependency(writer.pass, pass.id);
                        }
                        if has_unwritten_scope
                            && !matches!(access.resource, RenderGraphResource::External(_))
                        {
                            return Err(RenderGraphError::ReadBeforeProducer {
                                resource: resource_name(resource_names, access.resource),
                                pass: pass.name.clone(),
                            });
                        }
                        resource_access_versions[pass.id.0].push(RenderGraphResourceVersion::new(
                            access.resource,
                            latest_version_ordinal,
                        ));
                        resource_accesses.mutate_histories(&scope, |history| {
                            history.readers_since_last_write.push(pass.id);
                        })?;
                    }
                    ResourceAccessKind::Write => {
                        let loads_previous_version = access
                            .attachment_ops
                            .is_some_and(|ops| ops.load == RenderGraphAttachmentLoadOp::Load);
                        if let Some(token) = access.input_version {
                            if !loads_previous_version {
                                return Err(
                                    RenderGraphError::ResourceVersionRequiresAttachmentLoad {
                                        pass: pass.name.clone(),
                                        resource: resource_name(resource_names, access.resource),
                                    },
                                );
                            }
                            self.validate_current_resource_version(
                                resource_names,
                                pass,
                                access.resource,
                                token,
                                &histories,
                                scope.is_precise(),
                            )?;
                        }
                        if matches!(access.resource, RenderGraphResource::TransientTexture(_))
                            && loads_previous_version
                        {
                            for history in &histories {
                                match history.latest_writer {
                                    Some(writer)
                                        if writer.store == RenderGraphAttachmentStoreOp::Store => {}
                                    Some(writer) => {
                                        return Err(RenderGraphError::ReadAfterDiscardedStore {
                                            resource: resource_name(
                                                resource_names,
                                                access.resource,
                                            ),
                                            pass: pass.name.clone(),
                                            producer: self.passes[writer.pass.0].name.clone(),
                                        });
                                    }
                                    None => {
                                        return Err(RenderGraphError::LoadBeforeProducer {
                                            resource: resource_name(
                                                resource_names,
                                                access.resource,
                                            ),
                                            pass: pass.name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                        for history in &histories {
                            if let Some(writer) = history.latest_writer {
                                // WAW ordering preserves physical hazards. A clear creates a new
                                // logical value and therefore keeps provenance only for Load.
                                execution_dependencies.add_dependency(writer.pass, pass.id);
                                if loads_previous_version {
                                    culling_dependencies.add_dependency(writer.pass, pass.id);
                                }
                            }
                            for reader in history.readers_since_last_write.iter().copied() {
                                if reader != pass.id {
                                    execution_dependencies.add_dependency(reader, pass.id);
                                }
                            }
                        }
                        let next_version_ordinal = resource_accesses.next_write_version(
                            access_identity,
                            &resource_name(resource_names, access.resource),
                        )?;
                        let writer = LatestWriter {
                            pass: pass.id,
                            access_index,
                            store: access
                                .attachment_ops
                                .map_or(RenderGraphAttachmentStoreOp::Store, |ops| ops.store),
                        };
                        resource_accesses.mutate_histories(&scope, |history| {
                            history.readers_since_last_write.clear();
                            history.latest_version_ordinal = next_version_ordinal;
                            history.latest_writer = Some(writer);
                        })?;
                        resource_access_versions[pass.id.0].push(RenderGraphResourceVersion::new(
                            access.resource,
                            next_version_ordinal,
                        ));
                    }
                }
            }
        }

        let mut cull_roots = Vec::new();
        let mut seen_cull_roots = HashSet::new();
        for resource in self
            .resources
            .iter()
            .filter(|resource| resource.usage.is_cull_root())
        {
            let access_identity = resource_access_identities
                .get(&resource.resource)
                .copied()
                .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                    resource: resource.name.clone(),
                })?;
            for writer in
                resource_accesses.cull_root_writers_for(access_identity, resource.resource)?
            {
                if seen_cull_roots.insert(writer.pass) {
                    cull_roots.push(writer.pass);
                }
            }
        }
        let execution_dependency_count = execution_dependencies.dependency_count();
        let provenance_dependency_count = culling_dependencies.dependency_count();

        Ok(InferredResourceDependencies {
            execution: execution_dependencies.into_dependencies(),
            culling: culling_dependencies.into_dependencies(),
            cull_roots,
            resource_access_versions,
            resource_access_metadata,
            resource_access_visit_count,
            execution_dependency_count,
            provenance_dependency_count,
        })
    }

    fn validate_current_resource_version(
        &self,
        resource_names: &HashMap<RenderGraphResource, &str>,
        consumer: &RenderPassNode,
        resource: RenderGraphResource,
        token: RenderGraphResourceVersionToken,
        histories: &[ResourceAccessHistory],
        precise_scope: bool,
    ) -> Result<(), RenderGraphError> {
        let Some(producer) = self.passes.get(token.producer_pass().index()) else {
            return Err(RenderGraphError::ResourceVersionProducerMissing {
                producer_pass: token.producer_pass().index(),
                producer_access: token.producer_access_index(),
            });
        };
        if token_covers_scope(histories, token) {
            return Ok(());
        }
        if precise_scope {
            return Err(RenderGraphError::ResourceVersionScopeNotCovered {
                pass: consumer.name.clone(),
                resource: resource_name(resource_names, resource),
                producer: producer.name.clone(),
            });
        }
        match histories.iter().find_map(|history| {
            history.latest_writer.filter(|writer| {
                writer.pass != token.producer_pass()
                    || writer.access_index != token.producer_access_index()
            })
        }) {
            Some(writer) => Err(RenderGraphError::ResourceVersionNotCurrent {
                pass: consumer.name.clone(),
                resource: resource_name(resource_names, resource),
                producer: producer.name.clone(),
                latest_producer: self.passes[writer.pass.index()].name.clone(),
            }),
            None => Err(RenderGraphError::ResourceVersionUnavailable {
                pass: consumer.name.clone(),
                resource: resource_name(resource_names, resource),
                producer: producer.name.clone(),
            }),
        }
    }

    fn resource_access_identities(
        &self,
    ) -> Result<HashMap<RenderGraphResource, usize>, RenderGraphError> {
        let mut identities = HashMap::with_capacity(self.resources.len());
        let mut aliases = HashMap::<&str, (usize, RenderGraphExternalResourceType)>::new();
        let mut next_identity = 0;

        for resource in &self.resources {
            let identity = if let Some(texture_view_alias) = resource.texture_view_alias {
                identities
                    .get(&RenderGraphResource::TransientTexture(
                        texture_view_alias.parent,
                    ))
                    .copied()
                    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                        resource: resource.name.clone(),
                    })?
            } else if let Some(alias_group) = resource.external_alias_group.as_deref() {
                if let Some(&(identity, expected_type)) = aliases.get(alias_group) {
                    if expected_type != resource.external_binding.resource_type {
                        return Err(RenderGraphError::ExternalAliasResourceTypeMismatch {
                            alias_group: alias_group.to_owned(),
                            expected: expected_type,
                            found: resource.external_binding.resource_type,
                        });
                    }
                    identity
                } else {
                    let identity = next_identity;
                    next_identity += 1;
                    aliases.insert(
                        alias_group,
                        (identity, resource.external_binding.resource_type),
                    );
                    identity
                }
            } else {
                let identity = next_identity;
                next_identity += 1;
                identity
            };
            identities.insert(resource.resource, identity);
        }

        Ok(identities)
    }
}

// The same builder records both execution hazards and semantic provenance; consumers choose the
// adjacency appropriate for scheduling or culling.
struct DependencyAdjacency {
    dependencies: Vec<Vec<RenderPassId>>,
    membership: Vec<HashSet<RenderPassId>>,
    dependency_count: usize,
}

impl DependencyAdjacency {
    fn from_manual_dependencies(dependencies: Vec<Vec<RenderPassId>>) -> Self {
        let membership = dependencies
            .iter()
            .map(|incoming| incoming.iter().copied().collect())
            .collect::<Vec<HashSet<_>>>();
        let dependency_count = membership.iter().map(HashSet::len).sum();
        Self {
            dependencies,
            membership,
            dependency_count,
        }
    }

    fn add_dependency(&mut self, before: RenderPassId, after: RenderPassId) {
        if before != after && self.membership[after.0].insert(before) {
            self.dependencies[after.0].push(before);
            self.dependency_count += 1;
        }
    }

    fn dependency_count(&self) -> usize {
        self.dependency_count
    }

    fn into_dependencies(self) -> Vec<Vec<RenderPassId>> {
        self.dependencies
    }
}

pub(super) fn resource_name(
    resource_names: &HashMap<RenderGraphResource, &str>,
    resource: RenderGraphResource,
) -> String {
    resource_names
        .get(&resource)
        .map(|name| (*name).to_owned())
        .unwrap_or_else(|| format!("{resource:?}"))
}
