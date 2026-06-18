use std::collections::{HashMap, HashSet, VecDeque};

use crate::rhi::{BufferDesc, TextureDesc};

use super::error::RenderGraphError;
use super::graph::{CompiledRenderGraph, CompiledRenderPass};
use super::types::{
    ExternalResource, PassFlags, QueueLane, RenderGraphAttachmentLoadOp, RenderGraphAttachmentOps,
    RenderGraphAttachmentStoreOp, RenderGraphComputeWorkload, RenderGraphExternalResourceBinding,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderGraphResourceUsageFlags, RenderPassId, RgBufferHandle,
    RgTextureHandle,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ResourceAccessKind {
    Read,
    Write,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ResourceAccess {
    resource: RenderGraphResource,
    kind: ResourceAccessKind,
    attachment_ops: Option<RenderGraphAttachmentOps>,
}

#[derive(Clone, Debug)]
struct RenderPassNode {
    id: RenderPassId,
    name: String,
    declared_queue: QueueLane,
    queue: QueueLane,
    flags: PassFlags,
    executor_id: Option<String>,
    compute_workload: Option<RenderGraphComputeWorkload>,
    dependencies: Vec<RenderPassId>,
    resources: Vec<ResourceAccess>,
}

#[derive(Clone, Debug)]
struct ResourceNode {
    resource: RenderGraphResource,
    name: String,
    desc: RenderGraphResourceDesc,
    external_binding: RenderGraphExternalResourceBinding,
    usage: RenderGraphResourceUsageFlags,
}

#[derive(Clone, Debug)]
pub struct RenderGraphBuilder {
    name: String,
    passes: Vec<RenderPassNode>,
    resources: Vec<ResourceNode>,
    next_texture: usize,
    next_buffer: usize,
    next_external_resource: usize,
}

impl RenderGraphBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: Vec::new(),
            resources: Vec::new(),
            next_texture: 0,
            next_buffer: 0,
            next_external_resource: 0,
        }
    }

    pub fn add_pass(&mut self, name: impl Into<String>, queue: QueueLane) -> RenderPassId {
        self.add_pass_with_executor(name, queue, None::<String>)
    }

    pub fn add_pass_with_executor(
        &mut self,
        name: impl Into<String>,
        queue: QueueLane,
        executor_id: Option<impl Into<String>>,
    ) -> RenderPassId {
        self.add_pass_with_executor_and_declared_queue(name, queue, queue, executor_id)
    }

    pub fn add_pass_with_executor_and_declared_queue(
        &mut self,
        name: impl Into<String>,
        queue: QueueLane,
        declared_queue: QueueLane,
        executor_id: Option<impl Into<String>>,
    ) -> RenderPassId {
        let id = RenderPassId(self.passes.len());
        self.passes.push(RenderPassNode {
            id,
            name: name.into(),
            declared_queue,
            queue,
            flags: PassFlags::default(),
            executor_id: executor_id.map(Into::into),
            compute_workload: None,
            dependencies: Vec::new(),
            resources: Vec::new(),
        });
        id
    }

    pub fn set_compute_workload(
        &mut self,
        pass: RenderPassId,
        workload: RenderGraphComputeWorkload,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.passes[pass.0].compute_workload = Some(workload);
        Ok(())
    }

    pub fn set_pass_flags(
        &mut self,
        pass: RenderPassId,
        flags: PassFlags,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.passes[pass.0].flags = flags;
        Ok(())
    }

    pub fn add_dependency(
        &mut self,
        before: RenderPassId,
        after: RenderPassId,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(before)?;
        self.ensure_pass(after)?;
        let pass = &mut self.passes[after.0];
        if !pass.dependencies.contains(&before) {
            pass.dependencies.push(before);
        }
        Ok(())
    }

    pub fn create_texture(&mut self, desc: TextureDesc) -> RgTextureHandle {
        let id = self.next_texture;
        self.next_texture += 1;
        let handle = RgTextureHandle::from_index(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-texture-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientTexture(handle),
            name,
            desc: RenderGraphResourceDesc::Texture(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    pub fn create_buffer(&mut self, desc: BufferDesc) -> RgBufferHandle {
        let id = self.next_buffer;
        self.next_buffer += 1;
        let handle = RgBufferHandle::from_index(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("rg-buffer-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientBuffer(handle),
            name,
            desc: RenderGraphResourceDesc::Buffer(desc),
            external_binding: RenderGraphExternalResourceBinding::report_only(),
            usage: RenderGraphResourceUsageFlags::default(),
        });
        handle
    }

    pub fn import_external_resource(&mut self, name: impl Into<String>) -> ExternalResource {
        self.import_external_resource_with_usage(name, RenderGraphResourceUsageFlags::present())
    }

    pub fn import_external_resource_with_binding(
        &mut self,
        name: impl Into<String>,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_and_binding(
            name,
            RenderGraphResourceUsageFlags::present(),
            external_binding,
        )
    }

    pub fn import_external_resource_with_usage(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
    ) -> ExternalResource {
        self.import_external_resource_with_usage_and_binding(
            name,
            usage,
            RenderGraphExternalResourceBinding::report_only(),
        )
    }

    pub fn import_external_resource_with_usage_and_binding(
        &mut self,
        name: impl Into<String>,
        usage: RenderGraphResourceUsageFlags,
        external_binding: RenderGraphExternalResourceBinding,
    ) -> ExternalResource {
        let id = self.next_external_resource;
        self.next_external_resource += 1;
        let handle = ExternalResource::from_index(id);
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::External(handle),
            name: name.into(),
            desc: RenderGraphResourceDesc::External,
            external_binding,
            usage,
        });
        handle
    }

    pub fn mark_persistent(&mut self, texture: RgTextureHandle) -> Result<(), RenderGraphError> {
        self.mark_resource_usage(RenderGraphResource::TransientTexture(texture), |usage| {
            usage.persistent = true;
        })
    }

    pub fn mark_readback(&mut self, resource: RenderGraphResource) -> Result<(), RenderGraphError> {
        self.mark_resource_usage(resource, |usage| {
            usage.readback = true;
        })
    }

    pub fn read_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.write_texture_with_ops(pass, texture, RenderGraphAttachmentOps::clear_store())
    }

    pub fn write_storage_texture(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn write_texture_with_ops(
        &mut self,
        pass: RenderPassId,
        texture: RgTextureHandle,
        ops: RenderGraphAttachmentOps,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Write,
            Some(ops),
        )
    }

    pub fn read_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: RgBufferHandle,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn read_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Read,
            None,
        )
    }

    pub fn write_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            Some(RenderGraphAttachmentOps::load_store()),
        )
    }

    pub fn write_storage_external(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            None,
        )
    }

    pub fn write_external_with_ops(
        &mut self,
        pass: RenderPassId,
        external: ExternalResource,
        ops: RenderGraphAttachmentOps,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::External(external),
            ResourceAccessKind::Write,
            Some(ops),
        )
    }

    pub fn compile(self) -> Result<CompiledRenderGraph, RenderGraphError> {
        self.validate_unique_resource_names()?;
        let resource_names = self.resource_names();
        let manual_reachability = self.manual_reachability();
        self.validate_write_dependencies(&manual_reachability, &resource_names)?;

        let manual_dependencies = self
            .passes
            .iter()
            .map(|pass| pass.dependencies.clone())
            .collect::<Vec<_>>();
        let manual_order = self.topological_order(&manual_dependencies)?;
        let inferred_dependencies =
            self.infer_resource_dependencies(&resource_names, &manual_order)?;
        let ordered = self.topological_order(&inferred_dependencies)?;
        self.validate_reads_have_ordered_producers(&ordered, &resource_names)?;
        let culled = self.cull_passes(&ordered)?;
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
                    dependencies: inferred_dependencies[id.0].clone(),
                    culled: culled.contains(id),
                    executor_id: pass.executor_id.clone(),
                    compute_workload: pass.compute_workload.clone(),
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
        let lifetimes = self.resource_lifetimes(&ordered, &culled);

        Ok(CompiledRenderGraph::new(
            self.name,
            compiled_passes,
            resource_declarations,
            lifetimes,
        ))
    }

    fn add_resource_access(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
        attachment_ops: Option<RenderGraphAttachmentOps>,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.ensure_resource(resource)?;
        self.passes[pass.0].resources.push(ResourceAccess {
            resource,
            kind,
            attachment_ops,
        });
        Ok(())
    }

    fn ensure_pass(&self, id: RenderPassId) -> Result<(), RenderGraphError> {
        if id.0 >= self.passes.len() {
            return Err(RenderGraphError::UnknownPass { pass: id.0 });
        }
        Ok(())
    }

    fn ensure_resource(&self, resource: RenderGraphResource) -> Result<(), RenderGraphError> {
        if self.resources.iter().any(|node| node.resource == resource) {
            return Ok(());
        }

        Err(RenderGraphError::UnknownResource {
            resource: format!("{resource:?}"),
        })
    }

    fn mark_resource_usage(
        &mut self,
        resource: RenderGraphResource,
        update: impl FnOnce(&mut RenderGraphResourceUsageFlags),
    ) -> Result<(), RenderGraphError> {
        let Some(node) = self
            .resources
            .iter_mut()
            .find(|node| node.resource == resource)
        else {
            return Err(RenderGraphError::UnknownResource {
                resource: format!("{resource:?}"),
            });
        };

        update(&mut node.usage);
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

    fn manual_reachability(&self) -> Vec<HashSet<RenderPassId>> {
        let mut reachable = vec![HashSet::new(); self.passes.len()];
        for pass in &self.passes {
            for dependency in &pass.dependencies {
                reachable[dependency.0].insert(pass.id);
            }
        }

        for intermediate in 0..self.passes.len() {
            for source in 0..self.passes.len() {
                if reachable[source].contains(&RenderPassId(intermediate)) {
                    let additions = reachable[intermediate].clone();
                    reachable[source].extend(additions);
                }
            }
        }
        reachable
    }

    fn validate_write_dependencies(
        &self,
        manual_reachability: &[HashSet<RenderPassId>],
        resource_names: &HashMap<RenderGraphResource, String>,
    ) -> Result<(), RenderGraphError> {
        let mut writers = HashMap::<RenderGraphResource, Vec<RenderPassId>>::new();
        for pass in &self.passes {
            for access in &pass.resources {
                if access.kind == ResourceAccessKind::Write {
                    writers.entry(access.resource).or_default().push(pass.id);
                }
            }
        }

        for (resource, writer_ids) in writers {
            for (index, first) in writer_ids.iter().enumerate() {
                for second in writer_ids.iter().skip(index + 1) {
                    let ordered = manual_reachability[first.0].contains(second)
                        || manual_reachability[second.0].contains(first);
                    if !ordered {
                        return Err(RenderGraphError::WriteAfterWriteMissingDependency {
                            resource: resource_names
                                .get(&resource)
                                .cloned()
                                .unwrap_or_else(|| format!("{resource:?}")),
                            first_writer: self.passes[first.0].name.clone(),
                            second_writer: self.passes[second.0].name.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn infer_resource_dependencies(
        &self,
        resource_names: &HashMap<RenderGraphResource, String>,
        pass_order: &[RenderPassId],
    ) -> Result<Vec<Vec<RenderPassId>>, RenderGraphError> {
        let mut dependencies = self
            .passes
            .iter()
            .map(|pass| pass.dependencies.clone())
            .collect::<Vec<_>>();
        let mut latest_writer = HashMap::<RenderGraphResource, LatestWriter>::new();

        for pass_id in pass_order {
            let pass = &self.passes[pass_id.0];
            for access in &pass.resources {
                match access.kind {
                    ResourceAccessKind::Read => {
                        if let Some(writer) = latest_writer.get(&access.resource) {
                            if writer.store == RenderGraphAttachmentStoreOp::Discard {
                                return Err(RenderGraphError::ReadAfterDiscardedStore {
                                    resource: resource_name(resource_names, access.resource),
                                    pass: pass.name.clone(),
                                    producer: self.passes[writer.pass.0].name.clone(),
                                });
                            }
                            if writer.pass != pass.id
                                && !dependencies[pass.id.0].contains(&writer.pass)
                            {
                                dependencies[pass.id.0].push(writer.pass);
                            }
                        } else if !matches!(access.resource, RenderGraphResource::External(_)) {
                            return Err(RenderGraphError::ReadBeforeProducer {
                                resource: resource_name(resource_names, access.resource),
                                pass: pass.name.clone(),
                            });
                        }
                    }
                    ResourceAccessKind::Write => {
                        if matches!(access.resource, RenderGraphResource::TransientTexture(_))
                            && access
                                .attachment_ops
                                .is_some_and(|ops| ops.load == RenderGraphAttachmentLoadOp::Load)
                        {
                            match latest_writer.get(&access.resource) {
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
                        latest_writer.insert(
                            access.resource,
                            LatestWriter {
                                pass: pass.id,
                                store: access
                                    .attachment_ops
                                    .map_or(RenderGraphAttachmentStoreOp::Store, |ops| ops.store),
                            },
                        );
                    }
                }
            }
        }

        Ok(dependencies)
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

    fn validate_reads_have_ordered_producers(
        &self,
        ordered: &[RenderPassId],
        resource_names: &HashMap<RenderGraphResource, String>,
    ) -> Result<(), RenderGraphError> {
        let order = ordered
            .iter()
            .enumerate()
            .map(|(index, pass)| (*pass, index))
            .collect::<HashMap<_, _>>();
        for pass in &self.passes {
            for access in &pass.resources {
                if access.kind != ResourceAccessKind::Read
                    || matches!(access.resource, RenderGraphResource::External(_))
                {
                    continue;
                }
                let reader_order = order[&pass.id];
                let has_producer = self.passes.iter().any(|candidate| {
                    order[&candidate.id] <= reader_order
                        && candidate.resources.iter().any(|candidate_access| {
                            candidate_access.resource == access.resource
                                && candidate_access.kind == ResourceAccessKind::Write
                        })
                });
                if !has_producer {
                    return Err(RenderGraphError::ReadBeforeProducer {
                        resource: resource_names
                            .get(&access.resource)
                            .cloned()
                            .unwrap_or_else(|| format!("{:?}", access.resource)),
                        pass: pass.name.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn cull_passes(
        &self,
        ordered: &[RenderPassId],
    ) -> Result<HashSet<RenderPassId>, RenderGraphError> {
        let cull_root_resources = self
            .resources
            .iter()
            .filter(|resource| resource.usage.is_cull_root())
            .map(|resource| resource.resource)
            .collect::<HashSet<_>>();
        let mut needed_resources = HashSet::<RenderGraphResource>::new();
        let mut needed_passes = HashSet::<RenderPassId>::new();
        let mut live_passes = HashSet::<RenderPassId>::new();
        let mut found_cull_root = false;

        for id in ordered.iter().rev() {
            let pass = &self.passes[id.0];
            let writes = pass
                .resources
                .iter()
                .filter(|access| access.kind == ResourceAccessKind::Write)
                .map(|access| access.resource)
                .collect::<Vec<_>>();
            let writes_needed_resource = writes
                .iter()
                .any(|resource| needed_resources.contains(resource));
            let writes_cull_root_resource = writes
                .iter()
                .any(|resource| cull_root_resources.contains(resource));
            let is_explicit_pass_root = !pass.flags.allow_culling
                || pass.flags.has_side_effects
                || writes_cull_root_resource;
            let live =
                needed_passes.contains(id) || is_explicit_pass_root || writes_needed_resource;

            if live {
                found_cull_root |= is_explicit_pass_root;
                live_passes.insert(*id);
                for access in &pass.resources {
                    if access.kind == ResourceAccessKind::Read {
                        needed_resources.insert(access.resource);
                    }
                }
                needed_passes.extend(pass.dependencies.iter().copied());
            }
        }

        if !self.passes.is_empty() && !found_cull_root {
            return Err(RenderGraphError::MissingCullRoot {
                graph_name: self.name.clone(),
            });
        }

        Ok(self
            .passes
            .iter()
            .map(|pass| pass.id)
            .filter(|id| !live_passes.contains(id))
            .collect())
    }

    fn resource_lifetimes(
        &self,
        ordered: &[RenderPassId],
        culled: &HashSet<RenderPassId>,
    ) -> Vec<RenderGraphResourceLifetime> {
        let order = ordered
            .iter()
            .enumerate()
            .map(|(index, pass)| (*pass, index))
            .collect::<HashMap<_, _>>();
        let resource_names = self.resource_names();
        let resource_descs = self
            .resources
            .iter()
            .map(|resource| (resource.resource, resource.desc.clone()))
            .collect::<HashMap<_, _>>();
        let resource_usages = self
            .resources
            .iter()
            .map(|resource| (resource.resource, resource.usage))
            .collect::<HashMap<_, _>>();
        let external_bindings = self
            .resources
            .iter()
            .map(|resource| (resource.resource, resource.external_binding))
            .collect::<HashMap<_, _>>();
        let mut spans = HashMap::<RenderGraphResource, (usize, usize)>::new();

        for pass in &self.passes {
            if culled.contains(&pass.id) {
                continue;
            }
            let pass_order = order[&pass.id];
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

        let mut lifetimes = spans
            .into_iter()
            .map(|(resource, (first_pass, last_pass))| {
                let kind = match resource {
                    RenderGraphResource::TransientTexture(_) => {
                        RenderGraphResourceKind::TransientTexture
                    }
                    RenderGraphResource::TransientBuffer(_) => {
                        RenderGraphResourceKind::TransientBuffer
                    }
                    RenderGraphResource::External(_) => RenderGraphResourceKind::External,
                };
                RenderGraphResourceLifetime {
                    resource,
                    name: resource_names
                        .get(&resource)
                        .cloned()
                        .unwrap_or_else(|| format!("{resource:?}")),
                    kind,
                    desc: resource_descs
                        .get(&resource)
                        .cloned()
                        .unwrap_or(RenderGraphResourceDesc::External),
                    external_binding: external_bindings
                        .get(&resource)
                        .copied()
                        .unwrap_or_default(),
                    first_pass,
                    last_pass,
                    imported: matches!(
                        resource_descs.get(&resource),
                        Some(RenderGraphResourceDesc::External)
                    ),
                    usage: resource_usages.get(&resource).copied().unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();
        lifetimes.sort_by(|a, b| a.name.cmp(&b.name));
        lifetimes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LatestWriter {
    pass: RenderPassId,
    store: RenderGraphAttachmentStoreOp,
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
