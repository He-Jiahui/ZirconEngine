use std::collections::{HashMap, HashSet, VecDeque};

use super::super::error::RenderGraphError;
use super::super::graph::{
    CompiledRenderGraph, CompiledRenderGraphCompileWork, CompiledRenderPass,
};
use super::super::types::{
    ComputeBindingKind, RenderGraphAttachmentLoadOp, RenderGraphAttachmentStoreOp,
    RenderGraphComputeDispatchExtent, RenderGraphComputeShaderSource,
    RenderGraphExternalResourceType, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDeclaration, RenderGraphResourceDesc,
    RenderGraphResourceLifetime, RenderGraphResourceVersion, RenderPassId,
};
use super::{RenderGraphBuilder, ResourceAccessKind};

impl RenderGraphBuilder {
    pub fn compile(self) -> Result<CompiledRenderGraph, RenderGraphError> {
        self.validate_unique_pass_names()?;
        self.validate_unique_resource_names()?;
        self.validate_compute_dispatch_resources()?;
        self.validate_compute_pass_metadata()?;
        let resource_names = self.resource_names();
        let manual_dependencies = self
            .passes
            .iter()
            .map(|pass| pass.dependencies.clone())
            .collect::<Vec<_>>();
        let manual_order = self.topological_order(&manual_dependencies)?;
        let inferred_dependencies =
            self.infer_resource_dependencies(&resource_names, &manual_order)?;
        let ordered = self.topological_order(&inferred_dependencies.execution)?;
        let culling = self.cull_passes(
            &inferred_dependencies.culling,
            &inferred_dependencies.cull_roots,
        )?;
        let compile_work = CompiledRenderGraphCompileWork {
            resource_access_visit_count: inferred_dependencies.resource_access_visit_count,
            execution_dependency_count: inferred_dependencies.execution_dependency_count,
            provenance_dependency_count: inferred_dependencies.provenance_dependency_count,
            cull_root_count: culling.root_count,
            cull_dependency_visit_count: culling.dependency_visit_count,
        };
        let compiled_passes = ordered
            .iter()
            .map(|id| {
                let pass = &self.passes[id.0];
                CompiledRenderPass {
                    id: *id,
                    name: pass.name.clone(),
                    declared_queue: pass.declared_queue,
                    queue: pass.queue,
                    flags: pass.flags,
                    dependencies: inferred_dependencies.execution[id.0]
                        .iter()
                        .copied()
                        .filter(|dependency| !culling.culled.contains(dependency))
                        .collect(),
                    culled: culling.culled.contains(id),
                    executor_id: pass.executor_id.clone(),
                    compute_workload: pass.compute_workload.clone(),
                    compute_pass_metadata: pass.compute_pass_metadata.clone(),
                    resources: pass
                        .resources
                        .iter()
                        .map(|access| RenderGraphPassResourceAccess {
                            name: resource_names
                                .get(&access.resource)
                                .cloned()
                                .unwrap_or_else(|| format!("{:?}", access.resource)),
                            kind: access.resource.kind(),
                            access: render_graph_resource_access_kind(access.kind),
                            attachment_ops: access.attachment_ops,
                        })
                        .collect(),
                }
            })
            .collect::<Vec<_>>();
        let resource_declarations = self.resource_declarations();
        let lifetimes =
            self.resource_lifetimes(&ordered, &culling.culled, &resource_declarations)?;
        let pass_resource_versions = ordered
            .iter()
            .map(|id| inferred_dependencies.resource_access_versions[id.0].clone())
            .collect();

        Ok(CompiledRenderGraph::new(
            self.name,
            compiled_passes,
            resource_declarations,
            lifetimes,
            pass_resource_versions,
            compile_work,
        ))
    }

    fn validate_unique_pass_names(&self) -> Result<(), RenderGraphError> {
        let mut seen = HashSet::new();
        for pass in &self.passes {
            if !seen.insert(pass.name.as_str()) {
                return Err(RenderGraphError::DuplicatePassName {
                    pass: pass.name.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_unique_resource_names(&self) -> Result<(), RenderGraphError> {
        let mut seen = HashSet::new();
        for resource in &self.resources {
            if !seen.insert(resource.name.as_str()) {
                return Err(RenderGraphError::DuplicateResourceName {
                    resource: resource.name.clone(),
                });
            }
        }
        Ok(())
    }

    fn validate_compute_dispatch_resources(&self) -> Result<(), RenderGraphError> {
        let resources_by_name = self
            .resources
            .iter()
            .map(|resource| (resource.name.as_str(), resource))
            .collect::<HashMap<_, _>>();
        for pass in &self.passes {
            let Some(workload) = &pass.compute_workload else {
                continue;
            };
            match &workload.dispatch_extent {
                RenderGraphComputeDispatchExtent::FromBuffer { buffer, .. } => {
                    let is_declared_read_buffer = resources_by_name
                        .get(buffer.as_str())
                        .is_some_and(|resource| {
                            let is_buffer =
                                matches!(&resource.desc, RenderGraphResourceDesc::Buffer(_))
                                    || resource.external_binding.resource_type
                                        == RenderGraphExternalResourceType::Buffer;
                            is_buffer
                                && pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Read
                                })
                        });
                    if !is_declared_read_buffer {
                        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
                            pass: pass.name.clone(),
                            resource: buffer.clone(),
                            required_access: "read buffer",
                        });
                    }
                }
                RenderGraphComputeDispatchExtent::PerPixel { target, .. } => {
                    let is_declared_texture =
                        resources_by_name
                            .get(target.as_str())
                            .is_some_and(|resource| {
                                let is_texture =
                                    matches!(&resource.desc, RenderGraphResourceDesc::Texture(_))
                                        || resource.external_binding.resource_type
                                            == RenderGraphExternalResourceType::Texture;
                                is_texture
                                    && pass
                                        .resources
                                        .iter()
                                        .any(|access| access.resource == resource.resource)
                            });
                    if !is_declared_texture {
                        return Err(RenderGraphError::ComputeDispatchResourceNotDeclared {
                            pass: pass.name.clone(),
                            resource: target.clone(),
                            required_access: "read or write texture",
                        });
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn validate_compute_pass_metadata(&self) -> Result<(), RenderGraphError> {
        const INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES: u64 = 4;
        let resources_by_name = self
            .resources
            .iter()
            .map(|resource| (resource.name.as_str(), resource))
            .collect::<HashMap<_, _>>();

        for pass in &self.passes {
            let Some(metadata) = &pass.compute_pass_metadata else {
                continue;
            };
            let Some(workload) = &pass.compute_workload else {
                return Err(RenderGraphError::ComputePassMetadataMissingWorkload {
                    pass: pass.name.clone(),
                });
            };
            if metadata.entry_point.trim().is_empty() {
                return Err(RenderGraphError::ComputePassEntryPointEmpty {
                    pass: pass.name.clone(),
                });
            }
            if matches!(
                &metadata.shader,
                RenderGraphComputeShaderSource::Wgsl { source, .. } if source.trim().is_empty()
            ) {
                return Err(RenderGraphError::ComputePassShaderSourceEmpty {
                    pass: pass.name.clone(),
                });
            }
            if workload.workgroup_size.contains(&0) {
                return Err(RenderGraphError::InvalidComputeWorkgroupSize {
                    pass: pass.name.clone(),
                });
            }
            let mut declared_bindings = HashSet::with_capacity(metadata.bindings.len());
            for binding in &metadata.bindings {
                if !declared_bindings.insert(binding.binding) {
                    return Err(RenderGraphError::DuplicateComputeBinding {
                        pass: pass.name.clone(),
                        binding: binding.binding,
                    });
                }
                if let Some(mip_level) = binding.texture_mip_level {
                    if !matches!(
                        binding.kind,
                        ComputeBindingKind::SampledTexture
                            | ComputeBindingKind::StorageTextureWrite
                    ) {
                        return Err(RenderGraphError::ComputeTextureMipBindingNotTexture {
                            pass: pass.name.clone(),
                            binding: binding.binding,
                            mip_level,
                        });
                    }
                    if let Some(resource) = resources_by_name.get(binding.resource.as_str()) {
                        match &resource.desc {
                            RenderGraphResourceDesc::Texture(desc)
                                if mip_level >= desc.mip_levels =>
                            {
                                return Err(RenderGraphError::ComputeTextureMipOutOfRange {
                                    pass: pass.name.clone(),
                                    binding: binding.binding,
                                    resource: binding.resource.clone(),
                                    mip_level,
                                    mip_levels: desc.mip_levels,
                                });
                            }
                            RenderGraphResourceDesc::External => {
                                return Err(
                                    RenderGraphError::ComputeTextureMipRequiresTransientTexture {
                                        pass: pass.name.clone(),
                                        binding: binding.binding,
                                        resource: binding.resource.clone(),
                                        mip_level,
                                    },
                                );
                            }
                            _ => {}
                        }
                    }
                }
                if let Some(offset) = binding.buffer_offset {
                    if !matches!(
                        binding.kind,
                        ComputeBindingKind::UniformBuffer
                            | ComputeBindingKind::StorageBufferRead
                            | ComputeBindingKind::StorageBufferReadWrite
                    ) {
                        return Err(RenderGraphError::ComputeBufferOffsetBindingNotBuffer {
                            pass: pass.name.clone(),
                            binding: binding.binding,
                            offset,
                        });
                    }
                }
                let (requires_buffer, requires_read, requires_write, required_access) =
                    match binding.kind {
                        ComputeBindingKind::UniformBuffer
                        | ComputeBindingKind::StorageBufferRead => {
                            (true, true, false, "read buffer")
                        }
                        ComputeBindingKind::StorageBufferReadWrite => {
                            (true, true, true, "read/write buffer")
                        }
                        ComputeBindingKind::SampledTexture => (false, true, false, "read texture"),
                        ComputeBindingKind::StorageTextureWrite => {
                            (false, false, true, "write texture")
                        }
                    };
                let is_declared = resources_by_name
                    .get(binding.resource.as_str())
                    .is_some_and(|resource| {
                        let resource_type_matches = if requires_buffer {
                            matches!(&resource.desc, RenderGraphResourceDesc::Buffer(_))
                                || resource.external_binding.resource_type
                                    == RenderGraphExternalResourceType::Buffer
                        } else {
                            matches!(&resource.desc, RenderGraphResourceDesc::Texture(_))
                                || resource.external_binding.resource_type
                                    == RenderGraphExternalResourceType::Texture
                        };
                        resource_type_matches
                            && (!requires_read
                                || pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Read
                                }))
                            && (!requires_write
                                || pass.resources.iter().any(|access| {
                                    access.resource == resource.resource
                                        && access.kind == ResourceAccessKind::Write
                                }))
                    });
                if !is_declared {
                    return Err(RenderGraphError::ComputeBindingResourceNotDeclared {
                        pass: pass.name.clone(),
                        binding: binding.binding,
                        resource: binding.resource.clone(),
                        required_access,
                    });
                }
            }
            match &workload.dispatch_extent {
                RenderGraphComputeDispatchExtent::FromBuffer { offset, .. }
                    if offset % INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES != 0 =>
                {
                    return Err(RenderGraphError::ComputeIndirectDispatchOffsetUnaligned {
                        pass: pass.name.clone(),
                        offset: *offset,
                        alignment: INDIRECT_DISPATCH_OFFSET_ALIGNMENT_BYTES,
                    });
                }
                RenderGraphComputeDispatchExtent::PerPixel { local_size, .. }
                    if *local_size != [workload.workgroup_size[0], workload.workgroup_size[1]] =>
                {
                    return Err(RenderGraphError::PerPixelComputeWorkgroupMismatch {
                        pass: pass.name.clone(),
                        local_size: *local_size,
                        workgroup_size: workload.workgroup_size,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn resource_names(&self) -> HashMap<RenderGraphResource, String> {
        self.resources
            .iter()
            .map(|resource| (resource.resource, resource.name.clone()))
            .collect()
    }

    fn resource_declarations(&self) -> Vec<RenderGraphResourceDeclaration> {
        self.resources
            .iter()
            .map(|resource| RenderGraphResourceDeclaration {
                resource: resource.resource,
                name: resource.name.clone(),
                kind: resource.resource.kind(),
                desc: resource.desc.clone(),
                external_binding: resource.external_binding,
                imported: matches!(&resource.desc, RenderGraphResourceDesc::External),
                usage: resource.usage,
            })
            .collect()
    }

    fn infer_resource_dependencies(
        &self,
        resource_names: &HashMap<RenderGraphResource, String>,
        pass_order: &[RenderPassId],
    ) -> Result<InferredResourceDependencies, RenderGraphError> {
        let manual_dependencies = self
            .passes
            .iter()
            .map(|pass| pass.dependencies.clone())
            .collect::<Vec<_>>();
        let mut execution_dependencies =
            DependencyAdjacency::from_manual_dependencies(manual_dependencies.clone());
        let mut culling_dependencies =
            DependencyAdjacency::from_manual_dependencies(manual_dependencies);
        let resource_access_identities = self.resource_access_identities()?;
        let mut resource_accesses = HashMap::<usize, ResourceAccessHistory>::new();
        let mut resource_access_versions = vec![Vec::new(); self.passes.len()];
        let mut resource_access_visit_count = 0;

        for pass_id in pass_order {
            let pass = &self.passes[pass_id.0];
            for access in &pass.resources {
                resource_access_visit_count += 1;
                let access_identity = resource_access_identities
                    .get(&access.resource)
                    .copied()
                    .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                        resource: resource_name(resource_names, access.resource),
                    })?;
                let history = resource_accesses.entry(access_identity).or_default();
                match access.kind {
                    ResourceAccessKind::Read => {
                        if let Some(writer) = history.latest_writer {
                            if writer.store == RenderGraphAttachmentStoreOp::Discard {
                                return Err(RenderGraphError::ReadAfterDiscardedStore {
                                    resource: resource_name(resource_names, access.resource),
                                    pass: pass.name.clone(),
                                    producer: self.passes[writer.pass.0].name.clone(),
                                });
                            }
                            execution_dependencies.add_dependency(writer.pass, pass.id);
                            culling_dependencies.add_dependency(writer.pass, pass.id);
                        } else if !matches!(access.resource, RenderGraphResource::External(_)) {
                            return Err(RenderGraphError::ReadBeforeProducer {
                                resource: resource_name(resource_names, access.resource),
                                pass: pass.name.clone(),
                            });
                        }
                        resource_access_versions[pass.id.0].push(RenderGraphResourceVersion::new(
                            access.resource,
                            history.latest_version_ordinal,
                        ));
                        history.readers_since_last_write.push(pass.id);
                    }
                    ResourceAccessKind::Write => {
                        let loads_previous_version = access
                            .attachment_ops
                            .is_some_and(|ops| ops.load == RenderGraphAttachmentLoadOp::Load);
                        if matches!(access.resource, RenderGraphResource::TransientTexture(_))
                            && loads_previous_version
                        {
                            match history.latest_writer {
                                Some(writer)
                                    if writer.store == RenderGraphAttachmentStoreOp::Store => {}
                                Some(writer) => {
                                    return Err(RenderGraphError::ReadAfterDiscardedStore {
                                        resource: resource_name(resource_names, access.resource),
                                        pass: pass.name.clone(),
                                        producer: self.passes[writer.pass.0].name.clone(),
                                    });
                                }
                                None => {
                                    return Err(RenderGraphError::LoadBeforeProducer {
                                        resource: resource_name(resource_names, access.resource),
                                        pass: pass.name.clone(),
                                    });
                                }
                            }
                        }
                        if let Some(writer) = history.latest_writer {
                            // WAW ordering preserves the physical resource hazard, but a clear
                            // creates a new logical value and must not keep the old producer live.
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
                        history.readers_since_last_write.clear();
                        history.latest_version_ordinal = history
                            .latest_version_ordinal
                            .checked_add(1)
                            .ok_or_else(|| RenderGraphError::ResourceVersionExhausted {
                                resource: resource_name(resource_names, access.resource),
                            })?;
                        history.latest_writer = Some(LatestWriter {
                            pass: pass.id,
                            store: access
                                .attachment_ops
                                .map_or(RenderGraphAttachmentStoreOp::Store, |ops| ops.store),
                        });
                        resource_access_versions[pass.id.0].push(RenderGraphResourceVersion::new(
                            access.resource,
                            history.latest_version_ordinal,
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
            let Some(writer) = resource_accesses
                .get(&access_identity)
                .and_then(|history| history.latest_writer)
            else {
                continue;
            };
            if seen_cull_roots.insert(writer.pass) {
                cull_roots.push(writer.pass);
            }
        }
        let execution_dependency_count = execution_dependencies.dependency_count();
        let provenance_dependency_count = culling_dependencies.dependency_count();

        Ok(InferredResourceDependencies {
            execution: execution_dependencies.into_dependencies(),
            culling: culling_dependencies.into_dependencies(),
            cull_roots,
            resource_access_versions,
            resource_access_visit_count,
            execution_dependency_count,
            provenance_dependency_count,
        })
    }

    fn resource_access_identities(
        &self,
    ) -> Result<HashMap<RenderGraphResource, usize>, RenderGraphError> {
        let mut identities = HashMap::with_capacity(self.resources.len());
        let mut aliases = HashMap::<&str, (usize, RenderGraphExternalResourceType)>::new();
        let mut next_identity = 0;

        for resource in &self.resources {
            let identity = if let Some(alias_group) = resource.external_alias_group.as_deref() {
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

    fn topological_order(
        &self,
        dependencies: &[Vec<RenderPassId>],
    ) -> Result<Vec<RenderPassId>, RenderGraphError> {
        let mut indegree = vec![0_usize; self.passes.len()];
        let mut dependents = vec![Vec::new(); self.passes.len()];

        for pass in &self.passes {
            indegree[pass.id.0] = dependencies[pass.id.0].len();
            for dependency in &dependencies[pass.id.0] {
                dependents[dependency.0].push(pass.id);
            }
        }

        let mut ready = VecDeque::new();
        for pass in &self.passes {
            if indegree[pass.id.0] == 0 {
                ready.push_back(pass.id);
            }
        }

        let mut ordered = Vec::with_capacity(self.passes.len());
        while let Some(id) = ready.pop_front() {
            ordered.push(id);

            for dependent in &dependents[id.0] {
                indegree[dependent.0] -= 1;
                if indegree[dependent.0] == 0 {
                    ready.push_back(*dependent);
                }
            }
        }

        if ordered.len() != self.passes.len() {
            return Err(RenderGraphError::CycleDetected {
                graph_name: self.name.clone(),
            });
        }

        Ok(ordered)
    }

    fn cull_passes(
        &self,
        dependencies: &[Vec<RenderPassId>],
        cull_roots: &[RenderPassId],
    ) -> Result<CullingResult, RenderGraphError> {
        let mut live_passes = HashSet::<RenderPassId>::new();
        let mut pending = cull_roots.to_vec();
        pending.extend(self.passes.iter().filter_map(|pass| {
            (!pass.flags.allow_culling || pass.flags.has_side_effects).then_some(pass.id)
        }));
        let root_count = pending.len();

        if !self.passes.is_empty() && pending.is_empty() {
            return Err(RenderGraphError::MissingCullRoot {
                graph_name: self.name.clone(),
            });
        }

        // Walk only semantic provenance edges. Execution still carries WAW/WAR edges to
        // serialize reused backing resources, but those hazards do not make overwritten data live.
        let mut dependency_visit_count = 0;
        while let Some(pass) = pending.pop() {
            if live_passes.insert(pass) {
                dependency_visit_count += dependencies[pass.0].len();
                pending.extend(dependencies[pass.0].iter().copied());
            }
        }

        let culled = self
            .passes
            .iter()
            .map(|pass| pass.id)
            .filter(|id| !live_passes.contains(id))
            .collect();
        Ok(CullingResult {
            culled,
            root_count,
            dependency_visit_count,
        })
    }

    fn resource_lifetimes(
        &self,
        ordered: &[RenderPassId],
        culled: &HashSet<RenderPassId>,
        resource_declarations: &[RenderGraphResourceDeclaration],
    ) -> Result<Vec<RenderGraphResourceLifetime>, RenderGraphError> {
        let resource_declaration_indices = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.resource, index))
            .collect::<HashMap<_, _>>();
        let mut spans = HashMap::<RenderGraphResource, (usize, usize)>::new();

        for (pass_order, pass_id) in ordered.iter().enumerate() {
            if culled.contains(pass_id) {
                continue;
            }
            let pass = &self.passes[pass_id.0];
            for access in &pass.resources {
                spans
                    .entry(access.resource)
                    .and_modify(|span| {
                        span.0 = span.0.min(pass_order);
                        span.1 = span.1.max(pass_order);
                    })
                    .or_insert((pass_order, pass_order));
            }
        }

        let mut lifetimes = Vec::with_capacity(spans.len());
        for (resource, (first_pass, last_pass)) in spans {
            let declaration = resource_declaration_indices
                .get(&resource)
                .and_then(|index| resource_declarations.get(*index))
                .ok_or_else(|| RenderGraphError::ResourceDeclarationMissing {
                    resource: format!("{resource:?}"),
                })?;
            let usage = declaration.usage;
            // Readback consumers run after graph recording, so their backing must
            // remain unique and live through the terminal compiled pass.
            let last_pass = if usage.readback {
                ordered.len().saturating_sub(1)
            } else {
                last_pass
            };
            lifetimes.push(RenderGraphResourceLifetime {
                resource,
                name: declaration.name.clone(),
                kind: declaration.kind,
                desc: declaration.desc.clone(),
                external_binding: declaration.external_binding,
                first_pass,
                last_pass,
                imported: declaration.imported,
                usage,
            });
        }
        lifetimes.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(lifetimes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LatestWriter {
    pass: RenderPassId,
    store: RenderGraphAttachmentStoreOp,
}

struct InferredResourceDependencies {
    execution: Vec<Vec<RenderPassId>>,
    culling: Vec<Vec<RenderPassId>>,
    cull_roots: Vec<RenderPassId>,
    resource_access_versions: Vec<Vec<RenderGraphResourceVersion>>,
    resource_access_visit_count: usize,
    execution_dependency_count: usize,
    provenance_dependency_count: usize,
}

struct CullingResult {
    culled: HashSet<RenderPassId>,
    root_count: usize,
    dependency_visit_count: usize,
}

// Each write forms a new logical value. Readers are retained only until that value is replaced.
#[derive(Default)]
struct ResourceAccessHistory {
    latest_writer: Option<LatestWriter>,
    latest_version_ordinal: u64,
    readers_since_last_write: Vec<RenderPassId>,
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

fn resource_name(
    resource_names: &HashMap<RenderGraphResource, String>,
    resource: RenderGraphResource,
) -> String {
    resource_names
        .get(&resource)
        .cloned()
        .unwrap_or_else(|| format!("{resource:?}"))
}

fn render_graph_resource_access_kind(kind: ResourceAccessKind) -> RenderGraphResourceAccessKind {
    match kind {
        ResourceAccessKind::Read => RenderGraphResourceAccessKind::Read,
        ResourceAccessKind::Write => RenderGraphResourceAccessKind::Write,
    }
}
