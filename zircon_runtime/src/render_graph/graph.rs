use super::types::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind, RenderGraphResourceLifetime, RenderPassId,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderPass {
    pub id: RenderPassId,
    pub name: String,
    pub declared_queue: QueueLane,
    pub queue: QueueLane,
    pub flags: PassFlags,
    pub dependencies: Vec<RenderPassId>,
    pub culled: bool,
    pub executor_id: Option<String>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphStats {
    pub total_pass_count: usize,
    pub executable_pass_count: usize,
    pub culled_pass_count: usize,
    pub graphics_pass_count: usize,
    pub async_compute_pass_count: usize,
    pub async_copy_pass_count: usize,
    pub queue_fallback_pass_count: usize,
    pub resource_lifetime_count: usize,
    pub total_resource_access_count: usize,
    pub read_resource_access_count: usize,
    pub write_resource_access_count: usize,
    pub total_dependency_count: usize,
    pub external_output_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocation {
    pub resource_name: String,
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocationPlan {
    pub allocations: Vec<CompiledRenderGraphTransientAllocation>,
    pub texture_slot_count: usize,
    pub buffer_slot_count: usize,
}

impl CompiledRenderGraphTransientAllocationPlan {
    pub fn slot_for(&self, resource_name: &str) -> Option<usize> {
        self.allocations
            .iter()
            .find(|allocation| allocation.resource_name == resource_name)
            .map(|allocation| allocation.slot)
    }
}

impl CompiledRenderGraphStats {
    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        match queue {
            QueueLane::Graphics => self.graphics_pass_count,
            QueueLane::AsyncCompute => self.async_compute_pass_count,
            QueueLane::AsyncCopy => self.async_copy_pass_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraph {
    name: String,
    passes: Vec<CompiledRenderPass>,
    resource_lifetimes: Vec<RenderGraphResourceLifetime>,
}

impl CompiledRenderGraph {
    pub(crate) fn new(
        name: String,
        passes: Vec<CompiledRenderPass>,
        resource_lifetimes: Vec<RenderGraphResourceLifetime>,
    ) -> Self {
        Self {
            name,
            passes,
            resource_lifetimes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }

    pub fn resource_lifetimes(&self) -> &[RenderGraphResourceLifetime] {
        &self.resource_lifetimes
    }

    pub fn transient_allocation_plan(&self) -> CompiledRenderGraphTransientAllocationPlan {
        let mut allocations = allocate_transient_lifetimes(
            self.resource_lifetimes
                .iter()
                .filter(|lifetime| lifetime.kind == RenderGraphResourceKind::TransientTexture),
        );
        let texture_slot_count = allocations
            .iter()
            .map(|allocation| allocation.slot + 1)
            .max()
            .unwrap_or(0);
        let mut buffer_allocations = allocate_transient_lifetimes(
            self.resource_lifetimes
                .iter()
                .filter(|lifetime| lifetime.kind == RenderGraphResourceKind::TransientBuffer),
        );
        let buffer_slot_count = buffer_allocations
            .iter()
            .map(|allocation| allocation.slot + 1)
            .max()
            .unwrap_or(0);
        allocations.append(&mut buffer_allocations);
        allocations.sort_by(|left, right| left.resource_name.cmp(&right.resource_name));

        CompiledRenderGraphTransientAllocationPlan {
            allocations,
            texture_slot_count,
            buffer_slot_count,
        }
    }

    pub fn stats(&self) -> CompiledRenderGraphStats {
        let total_pass_count = self.passes.len();
        let culled_pass_count = self.passes.iter().filter(|pass| pass.culled).count();
        let total_resource_access_count = self
            .passes
            .iter()
            .map(|pass| pass.resources.len())
            .sum::<usize>();
        let read_resource_access_count = self
            .passes
            .iter()
            .flat_map(|pass| &pass.resources)
            .filter(|resource| resource.access == RenderGraphResourceAccessKind::Read)
            .count();
        let write_resource_access_count = self
            .passes
            .iter()
            .flat_map(|pass| &pass.resources)
            .filter(|resource| resource.access == RenderGraphResourceAccessKind::Write)
            .count();
        let total_dependency_count = self.passes.iter().map(|pass| pass.dependencies.len()).sum();
        let external_output_count = self
            .passes
            .iter()
            .flat_map(|pass| &pass.resources)
            .filter(|resource| {
                resource.kind == RenderGraphResourceKind::External
                    && resource.access == RenderGraphResourceAccessKind::Write
            })
            .count();
        let queue_fallback_pass_count = self
            .passes
            .iter()
            .filter(|pass| pass.declared_queue != pass.queue && !pass.culled)
            .count();
        CompiledRenderGraphStats {
            total_pass_count,
            executable_pass_count: total_pass_count - culled_pass_count,
            culled_pass_count,
            graphics_pass_count: self.queue_lane_count(QueueLane::Graphics),
            async_compute_pass_count: self.queue_lane_count(QueueLane::AsyncCompute),
            async_copy_pass_count: self.queue_lane_count(QueueLane::AsyncCopy),
            queue_fallback_pass_count,
            resource_lifetime_count: self.resource_lifetimes.len(),
            total_resource_access_count,
            read_resource_access_count,
            write_resource_access_count,
            total_dependency_count,
            external_output_count,
        }
    }

    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        self.passes
            .iter()
            .filter(|pass| pass.queue == queue && !pass.culled)
            .count()
    }
}

fn allocate_transient_lifetimes<'a>(
    lifetimes: impl Iterator<Item = &'a RenderGraphResourceLifetime>,
) -> Vec<CompiledRenderGraphTransientAllocation> {
    let mut lifetimes = lifetimes
        .filter(|lifetime| !lifetime.imported)
        .collect::<Vec<_>>();
    lifetimes.sort_by(|left, right| {
        left.first_pass
            .cmp(&right.first_pass)
            .then_with(|| left.last_pass.cmp(&right.last_pass))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut slot_last_passes = Vec::<usize>::new();
    let mut allocations = Vec::new();
    for lifetime in lifetimes {
        let slot = slot_last_passes
            .iter()
            .position(|last_pass| *last_pass < lifetime.first_pass)
            .unwrap_or_else(|| {
                slot_last_passes.push(0);
                slot_last_passes.len() - 1
            });
        slot_last_passes[slot] = lifetime.last_pass;
        allocations.push(CompiledRenderGraphTransientAllocation {
            resource_name: lifetime.name.clone(),
            kind: lifetime.kind,
            slot,
        });
    }

    allocations
}
