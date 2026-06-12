use super::types::{
    PassFlags, QueueLane, RenderGraphComputeWorkload, RenderGraphPassResourceAccess,
    RenderGraphResource, RenderGraphResourceAccessKind, RenderGraphResourceDeclaration,
    RenderGraphResourceDesc, RenderGraphResourceKind, RenderGraphResourceLifetime, RenderPassId,
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
    pub compute_workload: Option<RenderGraphComputeWorkload>,
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
    pub sparse_texture_lifetime_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocation {
    pub resource_name: String,
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub size_bytes: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientSlotReservation {
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub bytes_reserved: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocationPlan {
    pub allocations: Vec<CompiledRenderGraphTransientAllocation>,
    pub slot_reservations: Vec<CompiledRenderGraphTransientSlotReservation>,
    pub texture_slot_count: usize,
    pub buffer_slot_count: usize,
    pub sparse_texture_slot_count: usize,
    pub dense_texture_bytes_reserved: u64,
    pub dense_buffer_bytes_reserved: u64,
    pub sparse_texture_virtual_bytes: u64,
}

impl CompiledRenderGraphTransientAllocationPlan {
    pub fn slot_for(&self, resource_name: &str) -> Option<usize> {
        self.allocations
            .iter()
            .find(|allocation| allocation.resource_name == resource_name)
            .map(|allocation| allocation.slot)
    }

    pub fn size_bytes_for(&self, resource_name: &str) -> Option<u64> {
        self.allocations
            .iter()
            .find(|allocation| allocation.resource_name == resource_name)
            .map(|allocation| allocation.size_bytes)
    }

    pub fn slot_bytes(&self, kind: RenderGraphResourceKind, slot: usize) -> Option<u64> {
        self.slot_reservations
            .iter()
            .find(|reservation| reservation.kind == kind && reservation.slot == slot)
            .map(|reservation| reservation.bytes_reserved)
    }

    pub fn total_dense_bytes_reserved(&self) -> u64 {
        self.dense_texture_bytes_reserved
            .saturating_add(self.dense_buffer_bytes_reserved)
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
    resource_declarations: Vec<RenderGraphResourceDeclaration>,
    resource_lifetimes: Vec<RenderGraphResourceLifetime>,
}

impl CompiledRenderGraph {
    pub(crate) fn new(
        name: String,
        passes: Vec<CompiledRenderPass>,
        resource_declarations: Vec<RenderGraphResourceDeclaration>,
        resource_lifetimes: Vec<RenderGraphResourceLifetime>,
    ) -> Self {
        Self {
            name,
            passes,
            resource_declarations,
            resource_lifetimes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }

    pub fn resource_declarations(&self) -> &[RenderGraphResourceDeclaration] {
        &self.resource_declarations
    }

    pub fn resource_declaration(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declarations
            .iter()
            .find(|declaration| declaration.resource == resource)
    }

    pub fn resource_declaration_by_name(
        &self,
        name: &str,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declarations
            .iter()
            .find(|declaration| declaration.name == name)
    }

    pub fn resource_lifetimes(&self) -> &[RenderGraphResourceLifetime] {
        &self.resource_lifetimes
    }

    pub fn resource_lifetime(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceLifetime> {
        self.resource_lifetimes
            .iter()
            .find(|lifetime| lifetime.resource == resource)
    }

    pub fn resource_lifetime_by_name(&self, name: &str) -> Option<&RenderGraphResourceLifetime> {
        self.resource_lifetimes
            .iter()
            .find(|lifetime| lifetime.name == name)
    }

    pub fn transient_allocation_plan(&self) -> CompiledRenderGraphTransientAllocationPlan {
        let mut allocations =
            allocate_transient_lifetimes(self.resource_lifetimes.iter().filter(|lifetime| {
                lifetime.kind == RenderGraphResourceKind::TransientTexture
                    && !lifetime.is_sparse_reserved_texture()
            }));
        let texture_slot_count = allocations
            .iter()
            .map(|allocation| allocation.slot + 1)
            .max()
            .unwrap_or(0);
        let sparse_texture_lifetimes = self
            .resource_lifetimes
            .iter()
            .filter(|lifetime| {
                lifetime.kind == RenderGraphResourceKind::TransientTexture
                    && !lifetime.imported
                    && lifetime.is_sparse_reserved_texture()
            })
            .collect::<Vec<_>>();
        let sparse_texture_slot_count = sparse_texture_lifetimes.len();
        let sparse_texture_virtual_bytes = sparse_texture_lifetimes
            .iter()
            .copied()
            .map(resource_lifetime_size_bytes)
            .fold(0_u64, u64::saturating_add);
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
        let slot_reservations = slot_reservations_for(&allocations);
        let dense_texture_bytes_reserved = slot_reservations
            .iter()
            .filter(|reservation| reservation.kind == RenderGraphResourceKind::TransientTexture)
            .map(|reservation| reservation.bytes_reserved)
            .fold(0_u64, u64::saturating_add);
        let dense_buffer_bytes_reserved = slot_reservations
            .iter()
            .filter(|reservation| reservation.kind == RenderGraphResourceKind::TransientBuffer)
            .map(|reservation| reservation.bytes_reserved)
            .fold(0_u64, u64::saturating_add);

        CompiledRenderGraphTransientAllocationPlan {
            allocations,
            slot_reservations,
            texture_slot_count,
            buffer_slot_count,
            sparse_texture_slot_count,
            dense_texture_bytes_reserved,
            dense_buffer_bytes_reserved,
            sparse_texture_virtual_bytes,
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
        let sparse_texture_lifetime_count = self
            .resource_lifetimes
            .iter()
            .filter(|lifetime| lifetime.is_sparse_reserved_texture())
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
            sparse_texture_lifetime_count,
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
            size_bytes: resource_lifetime_size_bytes(lifetime),
        });
    }

    allocations
}

fn slot_reservations_for(
    allocations: &[CompiledRenderGraphTransientAllocation],
) -> Vec<CompiledRenderGraphTransientSlotReservation> {
    let mut reservations = Vec::<CompiledRenderGraphTransientSlotReservation>::new();

    for allocation in allocations {
        if let Some(reservation) = reservations.iter_mut().find(|reservation| {
            reservation.kind == allocation.kind && reservation.slot == allocation.slot
        }) {
            reservation.bytes_reserved = reservation.bytes_reserved.max(allocation.size_bytes);
        } else {
            reservations.push(CompiledRenderGraphTransientSlotReservation {
                kind: allocation.kind,
                slot: allocation.slot,
                bytes_reserved: allocation.size_bytes,
            });
        }
    }

    reservations.sort_by(|left, right| {
        resource_kind_sort_key(left.kind)
            .cmp(&resource_kind_sort_key(right.kind))
            .then_with(|| left.slot.cmp(&right.slot))
    });
    reservations
}

fn resource_lifetime_size_bytes(lifetime: &RenderGraphResourceLifetime) -> u64 {
    match &lifetime.desc {
        RenderGraphResourceDesc::Texture(desc) => {
            desc.checked_storage_size_bytes().unwrap_or(u64::MAX)
        }
        RenderGraphResourceDesc::Buffer(desc) => desc.size_bytes,
        RenderGraphResourceDesc::External => 0,
    }
}

const fn resource_kind_sort_key(kind: RenderGraphResourceKind) -> u8 {
    match kind {
        RenderGraphResourceKind::TransientTexture => 0,
        RenderGraphResourceKind::TransientBuffer => 1,
        RenderGraphResourceKind::External => 2,
    }
}
