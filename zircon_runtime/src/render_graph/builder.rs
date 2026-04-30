use std::collections::{HashMap, HashSet, VecDeque};

use crate::rhi::{BufferDesc, TextureDesc};

use super::error::RenderGraphError;
use super::graph::{CompiledRenderGraph, CompiledRenderPass};
use super::types::{
    ExternalResource, PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResource,
    RenderGraphResourceAccessKind, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderPassId, TransientBuffer, TransientTexture,
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
}

#[derive(Clone, Debug)]
struct RenderPassNode {
    id: RenderPassId,
    name: String,
    declared_queue: QueueLane,
    queue: QueueLane,
    flags: PassFlags,
    executor_id: Option<String>,
    dependencies: Vec<RenderPassId>,
    resources: Vec<ResourceAccess>,
}

#[derive(Clone, Debug)]
struct ResourceNode {
    resource: RenderGraphResource,
    name: String,
    desc: RenderGraphResourceDesc,
}

#[derive(Clone, Debug)]
pub struct RenderGraphBuilder {
    name: String,
    passes: Vec<RenderPassNode>,
    resources: Vec<ResourceNode>,
    next_transient_texture: usize,
    next_transient_buffer: usize,
    next_external_resource: usize,
}

impl RenderGraphBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passes: Vec::new(),
            resources: Vec::new(),
            next_transient_texture: 0,
            next_transient_buffer: 0,
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
            dependencies: Vec::new(),
            resources: Vec::new(),
        });
        id
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

    pub fn create_transient_texture(&mut self, desc: TextureDesc) -> TransientTexture {
        let id = self.next_transient_texture;
        self.next_transient_texture += 1;
        let handle = TransientTexture(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("transient-texture-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientTexture(handle),
            name,
            desc: RenderGraphResourceDesc::Texture(desc),
        });
        handle
    }

    pub fn create_transient_buffer(&mut self, desc: BufferDesc) -> TransientBuffer {
        let id = self.next_transient_buffer;
        self.next_transient_buffer += 1;
        let handle = TransientBuffer(id);
        let name = desc
            .label
            .clone()
            .unwrap_or_else(|| format!("transient-buffer-{id}"));
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::TransientBuffer(handle),
            name,
            desc: RenderGraphResourceDesc::Buffer(desc),
        });
        handle
    }

    pub fn import_external_resource(&mut self, name: impl Into<String>) -> ExternalResource {
        let id = self.next_external_resource;
        self.next_external_resource += 1;
        let handle = ExternalResource(id);
        self.resources.push(ResourceNode {
            resource: RenderGraphResource::External(handle),
            name: name.into(),
            desc: RenderGraphResourceDesc::External,
        });
        handle
    }

    pub fn read_texture(
        &mut self,
        pass: RenderPassId,
        texture: TransientTexture,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Read,
        )
    }

    pub fn write_texture(
        &mut self,
        pass: RenderPassId,
        texture: TransientTexture,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientTexture(texture),
            ResourceAccessKind::Write,
        )
    }

    pub fn read_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: TransientBuffer,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Read,
        )
    }

    pub fn write_buffer(
        &mut self,
        pass: RenderPassId,
        buffer: TransientBuffer,
    ) -> Result<(), RenderGraphError> {
        self.add_resource_access(
            pass,
            RenderGraphResource::TransientBuffer(buffer),
            ResourceAccessKind::Write,
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
        )
    }

    pub fn compile(self) -> Result<CompiledRenderGraph, RenderGraphError> {
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
        let culled = self.cull_passes(&ordered);
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
                    resources: pass
                        .resources
                        .iter()
                        .map(|access| RenderGraphPassResourceAccess {
                            name: resource_names
                                .get(&access.resource)
                                .cloned()
                                .unwrap_or_else(|| format!("{:?}", access.resource)),
                            kind: render_graph_resource_kind(access.resource),
                            access: render_graph_resource_access_kind(access.kind),
                        })
                        .collect(),
                }
            })
            .collect::<Vec<_>>();
        let lifetimes = self.resource_lifetimes(&ordered, &culled);

        Ok(CompiledRenderGraph::new(
            self.name,
            compiled_passes,
            lifetimes,
        ))
    }

    fn add_resource_access(
        &mut self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        kind: ResourceAccessKind,
    ) -> Result<(), RenderGraphError> {
        self.ensure_pass(pass)?;
        self.ensure_resource(resource)?;
        self.passes[pass.0]
            .resources
            .push(ResourceAccess { resource, kind });
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

    fn resource_names(&self) -> HashMap<RenderGraphResource, String> {
        self.resources
            .iter()
            .map(|resource| (resource.resource, resource.name.clone()))
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
        let mut latest_writer = HashMap::<RenderGraphResource, RenderPassId>::new();

        for pass_id in pass_order {
            let pass = &self.passes[pass_id.0];
            for access in &pass.resources {
                match access.kind {
                    ResourceAccessKind::Read => {
                        if let Some(writer) = latest_writer.get(&access.resource) {
                            if *writer != pass.id && !dependencies[pass.id.0].contains(writer) {
                                dependencies[pass.id.0].push(*writer);
                            }
                        } else if !matches!(access.resource, RenderGraphResource::External(_)) {
                            return Err(RenderGraphError::ReadBeforeProducer {
                                resource: resource_names
                                    .get(&access.resource)
                                    .cloned()
                                    .unwrap_or_else(|| format!("{:?}", access.resource)),
                                pass: pass.name.clone(),
                            });
                        }
                    }
                    ResourceAccessKind::Write => {
                        latest_writer.insert(access.resource, pass.id);
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

    fn cull_passes(&self, ordered: &[RenderPassId]) -> HashSet<RenderPassId> {
        let mut needed_resources = HashSet::<RenderGraphResource>::new();
        let mut needed_passes = HashSet::<RenderPassId>::new();
        let mut live_passes = HashSet::<RenderPassId>::new();

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
            let writes_external = writes
                .iter()
                .any(|resource| matches!(resource, RenderGraphResource::External(_)));
            let has_no_writes = writes.is_empty();
            let live = needed_passes.contains(id)
                || !pass.flags.allow_culling
                || pass.flags.has_side_effects
                || writes_external
                || has_no_writes
                || writes_needed_resource;

            if live {
                live_passes.insert(*id);
                for access in &pass.resources {
                    if access.kind == ResourceAccessKind::Read {
                        needed_resources.insert(access.resource);
                    }
                }
                needed_passes.extend(pass.dependencies.iter().copied());
            }
        }

        self.passes
            .iter()
            .map(|pass| pass.id)
            .filter(|id| !live_passes.contains(id))
            .collect()
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
                    name: resource_names
                        .get(&resource)
                        .cloned()
                        .unwrap_or_else(|| format!("{resource:?}")),
                    kind,
                    desc: resource_descs
                        .get(&resource)
                        .cloned()
                        .unwrap_or(RenderGraphResourceDesc::External),
                    first_pass,
                    last_pass,
                    imported: matches!(
                        resource_descs.get(&resource),
                        Some(RenderGraphResourceDesc::External)
                    ),
                }
            })
            .collect::<Vec<_>>();
        lifetimes.sort_by(|a, b| a.name.cmp(&b.name));
        lifetimes
    }
}

fn render_graph_resource_kind(resource: RenderGraphResource) -> RenderGraphResourceKind {
    match resource {
        RenderGraphResource::TransientTexture(_) => RenderGraphResourceKind::TransientTexture,
        RenderGraphResource::TransientBuffer(_) => RenderGraphResourceKind::TransientBuffer,
        RenderGraphResource::External(_) => RenderGraphResourceKind::External,
    }
}

fn render_graph_resource_access_kind(kind: ResourceAccessKind) -> RenderGraphResourceAccessKind {
    match kind {
        ResourceAccessKind::Read => RenderGraphResourceAccessKind::Read,
        ResourceAccessKind::Write => RenderGraphResourceAccessKind::Write,
    }
}
