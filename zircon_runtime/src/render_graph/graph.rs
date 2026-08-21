use std::collections::{BTreeSet, HashMap};

use super::RenderGraphDump;
use super::types::{
    PassFlags, QueueLane, RenderGraphComputePassMetadata, RenderGraphComputeWorkload,
    RenderGraphPassResourceAccess, RenderGraphResource, RenderGraphResourceAccessKind,
    RenderGraphResourceDeclaration, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime, RenderGraphResourceVersion, RenderPassId,
};
use crate::rhi::{TextureDimension, TextureFormat, TextureResidency};

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
    pub compute_pass_metadata: Option<RenderGraphComputePassMetadata>,
    pub resources: Vec<RenderGraphPassResourceAccess>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum CompiledRenderGraphAccessIndexKind {
    Read,
    Write,
}

impl From<RenderGraphResourceAccessKind> for CompiledRenderGraphAccessIndexKind {
    fn from(access: RenderGraphResourceAccessKind) -> Self {
        match access {
            RenderGraphResourceAccessKind::Read => Self::Read,
            RenderGraphResourceAccessKind::Write => Self::Write,
        }
    }
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
    // Compile-miss algorithmic work counters, not CPU or GPU wall-clock timings.
    pub compile_resource_access_visit_count: usize,
    pub compile_execution_dependency_count: usize,
    pub compile_provenance_dependency_count: usize,
    pub compile_cull_root_count: usize,
    pub compile_cull_dependency_visit_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CompiledRenderGraphCompileWork {
    pub resource_access_visit_count: usize,
    pub execution_dependency_count: usize,
    pub provenance_dependency_count: usize,
    pub cull_root_count: usize,
    pub cull_dependency_visit_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocation {
    pub resource: RenderGraphResource,
    pub resource_name: String,
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub size_bytes: u64,
    pub bucket_key_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientSlotReservation {
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub bytes_reserved: u64,
    pub bucket_key_hash: u64,
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
        let mut matched = self
            .slot_reservations
            .iter()
            .filter(|reservation| reservation.kind == kind && reservation.slot == slot)
            .peekable();
        matched.peek()?;
        Some(
            matched
                .map(|reservation| reservation.bytes_reserved)
                .fold(0_u64, u64::saturating_add),
        )
    }

    pub fn slot_bytes_for_bucket(
        &self,
        kind: RenderGraphResourceKind,
        slot: usize,
        bucket_key_hash: u64,
    ) -> Option<u64> {
        self.slot_reservations
            .iter()
            .find(|reservation| {
                reservation.kind == kind
                    && reservation.slot == slot
                    && reservation.bucket_key_hash == bucket_key_hash
            })
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
    pass_indices: HashMap<RenderPassId, usize>,
    pass_resource_access_indices: HashMap<
        (
            RenderPassId,
            RenderGraphResource,
            CompiledRenderGraphAccessIndexKind,
        ),
        (usize, usize),
    >,
    pass_resource_version_indices: HashMap<
        (
            RenderPassId,
            RenderGraphResource,
            CompiledRenderGraphAccessIndexKind,
        ),
        RenderGraphResourceVersion,
    >,
    resource_declarations: Vec<RenderGraphResourceDeclaration>,
    resource_declaration_indices: HashMap<RenderGraphResource, usize>,
    resource_declaration_indices_by_name: HashMap<String, usize>,
    resource_lifetimes: Vec<RenderGraphResourceLifetime>,
    resource_lifetime_indices: HashMap<RenderGraphResource, usize>,
    transient_allocation_plan: CompiledRenderGraphTransientAllocationPlan,
    // Frame statistics are compiled with the graph so steady-frame diagnostics
    // do not rescan pass, access, and lifetime metadata.
    stats: CompiledRenderGraphStats,
}

impl CompiledRenderGraph {
    pub(crate) fn new(
        name: String,
        passes: Vec<CompiledRenderPass>,
        resource_declarations: Vec<RenderGraphResourceDeclaration>,
        resource_lifetimes: Vec<RenderGraphResourceLifetime>,
        pass_resource_versions: Vec<Vec<RenderGraphResourceVersion>>,
        compile_work: CompiledRenderGraphCompileWork,
    ) -> Self {
        let pass_indices = passes
            .iter()
            .enumerate()
            .map(|(index, pass)| (pass.id, index))
            .collect();
        let resource_declaration_indices = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.resource, index))
            .collect();
        let resource_declaration_indices_by_name = resource_declarations
            .iter()
            .enumerate()
            .map(|(index, declaration)| (declaration.name.clone(), index))
            .collect::<HashMap<_, _>>();
        let mut pass_resource_access_indices = HashMap::new();
        for (pass_index, pass) in passes.iter().enumerate() {
            for (access_index, access) in pass.resources.iter().enumerate() {
                let Some(declaration_index) =
                    resource_declaration_indices_by_name.get(access.name.as_str())
                else {
                    continue;
                };
                let declaration = &resource_declarations[*declaration_index];
                if declaration.kind != access.kind {
                    continue;
                }
                pass_resource_access_indices
                    .entry((pass.id, declaration.resource, access.access.into()))
                    .or_insert((pass_index, access_index));
            }
        }
        let mut pass_resource_version_indices = HashMap::new();
        for (pass_index, pass) in passes.iter().enumerate() {
            let Some(resource_versions) = pass_resource_versions.get(pass_index) else {
                continue;
            };
            for (access, version) in pass.resources.iter().zip(resource_versions) {
                pass_resource_version_indices
                    .entry((pass.id, version.resource(), access.access.into()))
                    .or_insert(*version);
            }
        }
        let resource_lifetime_indices = resource_lifetimes
            .iter()
            .enumerate()
            .map(|(index, lifetime)| (lifetime.resource, index))
            .collect();
        let transient_allocation_plan = build_transient_allocation_plan(&resource_lifetimes);
        let stats = CompiledRenderGraphStats::from_compiled_graph(
            &passes,
            &resource_lifetimes,
            compile_work,
        );
        Self {
            name,
            passes,
            pass_indices,
            pass_resource_access_indices,
            pass_resource_version_indices,
            resource_declarations,
            resource_declaration_indices,
            resource_declaration_indices_by_name,
            resource_lifetimes,
            resource_lifetime_indices,
            transient_allocation_plan,
            stats,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn passes(&self) -> &[CompiledRenderPass] {
        &self.passes
    }

    pub(crate) fn pass(&self, pass: RenderPassId) -> Option<&CompiledRenderPass> {
        self.indexed_pass(pass).map(|(_, pass)| pass)
    }

    pub(crate) fn indexed_pass(&self, pass: RenderPassId) -> Option<(usize, &CompiledRenderPass)> {
        let index = *self.pass_indices.get(&pass)?;
        self.passes.get(index).map(|pass| (index, pass))
    }

    pub(crate) fn pass_resource_access(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<&RenderGraphPassResourceAccess> {
        let (pass_index, access_index) =
            self.pass_resource_access_indices
                .get(&(pass, resource, access.into()))?;
        self.passes
            .get(*pass_index)
            .and_then(|pass| pass.resources.get(*access_index))
    }

    pub fn resource_version_for_access(
        &self,
        pass: RenderPassId,
        resource: RenderGraphResource,
        access: RenderGraphResourceAccessKind,
    ) -> Option<RenderGraphResourceVersion> {
        self.pass_resource_version_indices
            .get(&(pass, resource, access.into()))
            .copied()
    }

    pub fn resource_declarations(&self) -> &[RenderGraphResourceDeclaration] {
        &self.resource_declarations
    }

    pub fn resource_declaration(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declaration_indices
            .get(&resource)
            .and_then(|index| self.resource_declarations.get(*index))
    }

    pub fn resource_declaration_by_name(
        &self,
        name: &str,
    ) -> Option<&RenderGraphResourceDeclaration> {
        self.resource_declaration_indices_by_name
            .get(name)
            .and_then(|index| self.resource_declarations.get(*index))
    }

    pub fn resource_lifetimes(&self) -> &[RenderGraphResourceLifetime] {
        &self.resource_lifetimes
    }

    pub fn resource_lifetime(
        &self,
        resource: RenderGraphResource,
    ) -> Option<&RenderGraphResourceLifetime> {
        self.resource_lifetime_indices
            .get(&resource)
            .and_then(|index| self.resource_lifetimes.get(*index))
    }

    pub fn resource_lifetime_by_name(&self, name: &str) -> Option<&RenderGraphResourceLifetime> {
        self.resource_declaration_by_name(name)
            .and_then(|declaration| self.resource_lifetime(declaration.resource))
    }

    pub fn dump(&self) -> RenderGraphDump {
        RenderGraphDump::from_graph(self)
    }

    pub fn transient_allocation_plan(&self) -> &CompiledRenderGraphTransientAllocationPlan {
        &self.transient_allocation_plan
    }

    pub fn stats(&self) -> CompiledRenderGraphStats {
        self.stats
    }

    pub fn queue_lane_count(&self, queue: QueueLane) -> usize {
        self.stats.queue_lane_count(queue)
    }
}

impl CompiledRenderGraphStats {
    fn from_compiled_graph(
        passes: &[CompiledRenderPass],
        resource_lifetimes: &[RenderGraphResourceLifetime],
        compile_work: CompiledRenderGraphCompileWork,
    ) -> Self {
        let mut stats = Self {
            total_pass_count: passes.len(),
            resource_lifetime_count: resource_lifetimes.len(),
            compile_resource_access_visit_count: compile_work.resource_access_visit_count,
            compile_execution_dependency_count: compile_work.execution_dependency_count,
            compile_provenance_dependency_count: compile_work.provenance_dependency_count,
            compile_cull_root_count: compile_work.cull_root_count,
            compile_cull_dependency_visit_count: compile_work.cull_dependency_visit_count,
            sparse_texture_lifetime_count: resource_lifetimes
                .iter()
                .filter(|lifetime| lifetime.is_sparse_reserved_texture())
                .count(),
            ..Self::default()
        };

        for pass in passes {
            stats.total_dependency_count += pass.dependencies.len();
            for resource in &pass.resources {
                stats.total_resource_access_count += 1;
                match resource.access {
                    RenderGraphResourceAccessKind::Read => stats.read_resource_access_count += 1,
                    RenderGraphResourceAccessKind::Write => {
                        stats.write_resource_access_count += 1;
                        if resource.kind == RenderGraphResourceKind::External {
                            stats.external_output_count += 1;
                        }
                    }
                }
            }

            if pass.culled {
                stats.culled_pass_count += 1;
                continue;
            }

            stats.executable_pass_count += 1;
            if pass.declared_queue != pass.queue {
                stats.queue_fallback_pass_count += 1;
            }
            match pass.queue {
                QueueLane::Graphics => stats.graphics_pass_count += 1,
                QueueLane::AsyncCompute => stats.async_compute_pass_count += 1,
                QueueLane::AsyncCopy => stats.async_copy_pass_count += 1,
            }
        }

        stats
    }
}

fn build_transient_allocation_plan(
    resource_lifetimes: &[RenderGraphResourceLifetime],
) -> CompiledRenderGraphTransientAllocationPlan {
    let mut allocations = allocate_transient_lifetimes_by_bucket(
        resource_lifetimes.iter().filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientTexture
                && !lifetime.usage.persistent
                && !lifetime.is_sparse_reserved_texture()
        }),
        transient_texture_bucket_key,
    );
    let sparse_texture_lifetimes = resource_lifetimes
        .iter()
        .filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientTexture
                && !lifetime.imported
                && !lifetime.usage.persistent
                && lifetime.is_sparse_reserved_texture()
        })
        .collect::<Vec<_>>();
    let sparse_texture_slot_count = sparse_texture_lifetimes.len();
    let sparse_texture_virtual_bytes = sparse_texture_lifetimes
        .iter()
        .copied()
        .map(resource_lifetime_size_bytes)
        .fold(0_u64, u64::saturating_add);
    let mut buffer_allocations = allocate_transient_lifetimes_by_bucket(
        resource_lifetimes.iter().filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientBuffer && !lifetime.usage.persistent
        }),
        transient_buffer_bucket_key,
    );
    allocations.append(&mut buffer_allocations);
    allocations.sort_by(|left, right| left.resource_name.cmp(&right.resource_name));
    let slot_reservations = slot_reservations_for(&allocations);
    let texture_slot_count = slot_reservations
        .iter()
        .filter(|reservation| reservation.kind == RenderGraphResourceKind::TransientTexture)
        .count();
    let buffer_slot_count = slot_reservations
        .iter()
        .filter(|reservation| reservation.kind == RenderGraphResourceKind::TransientBuffer)
        .count();
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

fn allocate_transient_lifetimes<'a>(
    lifetimes: impl Iterator<Item = &'a RenderGraphResourceLifetime>,
    bucket_key_hash: u64,
) -> Vec<CompiledRenderGraphTransientAllocation> {
    let mut lifetimes = lifetimes
        .filter(|lifetime| !lifetime.imported && !lifetime.usage.persistent)
        .collect::<Vec<_>>();
    lifetimes.sort_by(|left, right| {
        left.first_pass
            .cmp(&right.first_pass)
            .then_with(|| left.last_pass.cmp(&right.last_pass))
            .then_with(|| left.name.cmp(&right.name))
    });

    let mut active_slots = BTreeSet::<(usize, usize)>::new();
    let mut free_slots = BTreeSet::<usize>::new();
    let mut next_slot = 0_usize;
    let mut allocations = Vec::new();
    for lifetime in lifetimes {
        while active_slots
            .first()
            .is_some_and(|(last_pass, _)| *last_pass < lifetime.first_pass)
        {
            let Some((_, slot)) = active_slots.pop_first() else {
                break;
            };
            free_slots.insert(slot);
        }
        let slot = free_slots.pop_first().unwrap_or_else(|| {
            let slot = next_slot;
            next_slot += 1;
            slot
        });
        active_slots.insert((lifetime.last_pass, slot));
        allocations.push(CompiledRenderGraphTransientAllocation {
            resource: lifetime.resource,
            resource_name: lifetime.name.clone(),
            kind: lifetime.kind,
            slot,
            size_bytes: resource_lifetime_size_bytes(lifetime),
            bucket_key_hash,
        });
    }

    allocations
}

fn allocate_transient_lifetimes_by_bucket<'a, F>(
    lifetimes: impl Iterator<Item = &'a RenderGraphResourceLifetime>,
    bucket_key_for: F,
) -> Vec<CompiledRenderGraphTransientAllocation>
where
    F: Fn(&RenderGraphResourceLifetime) -> Option<TransientAllocationBucketKey>,
{
    let mut lifetimes_by_bucket = HashMap::<TransientAllocationBucketKey, Vec<_>>::new();
    for lifetime in lifetimes {
        let Some(bucket_key) = bucket_key_for(lifetime) else {
            continue;
        };
        lifetimes_by_bucket
            .entry(bucket_key)
            .or_default()
            .push(lifetime);
    }
    let mut lifetimes_by_bucket = lifetimes_by_bucket.into_iter().collect::<Vec<_>>();
    lifetimes_by_bucket.sort_by(|(left_key, _), (right_key, _)| {
        left_key
            .stable_hash()
            .cmp(&right_key.stable_hash())
            .then_with(|| left_key.cmp(right_key))
    });

    let mut allocations = Vec::new();
    for (bucket_key, lifetimes) in lifetimes_by_bucket {
        allocations.extend(allocate_transient_lifetimes(
            lifetimes.into_iter(),
            bucket_key.stable_hash(),
        ));
    }
    allocations
}

fn slot_reservations_for(
    allocations: &[CompiledRenderGraphTransientAllocation],
) -> Vec<CompiledRenderGraphTransientSlotReservation> {
    let mut reservation_bytes = HashMap::<(RenderGraphResourceKind, usize, u64), u64>::new();

    for allocation in allocations {
        reservation_bytes
            .entry((allocation.kind, allocation.slot, allocation.bucket_key_hash))
            .and_modify(|bytes_reserved| {
                *bytes_reserved = (*bytes_reserved).max(allocation.size_bytes);
            })
            .or_insert(allocation.size_bytes);
    }

    let mut reservations = reservation_bytes
        .into_iter()
        .map(|((kind, slot, bucket_key_hash), bytes_reserved)| {
            CompiledRenderGraphTransientSlotReservation {
                kind,
                slot,
                bytes_reserved,
                bucket_key_hash,
            }
        })
        .collect::<Vec<_>>();

    reservations.sort_by(|left, right| {
        resource_kind_sort_key(left.kind)
            .cmp(&resource_kind_sort_key(right.kind))
            .then_with(|| left.bucket_key_hash.cmp(&right.bucket_key_hash))
            .then_with(|| left.slot.cmp(&right.slot))
    });
    reservations
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum TransientAllocationBucketKey {
    Texture {
        width: u32,
        height: u32,
        depth: u32,
        mip_levels: u32,
        sample_count: u32,
        format_key: u64,
        dimension_key: u64,
        residency_key: u64,
        usage_bits: u32,
    },
    Buffer {
        size_bytes: u64,
        usage_bits: u32,
    },
}

impl TransientAllocationBucketKey {
    fn stable_hash(&self) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        fn mix(hash: &mut u64, value: u64) {
            *hash ^= value;
            *hash = hash.wrapping_mul(FNV_PRIME);
        }

        let mut hash = FNV_OFFSET_BASIS;
        match self {
            Self::Texture {
                width,
                height,
                depth,
                mip_levels,
                sample_count,
                format_key,
                dimension_key,
                residency_key,
                usage_bits,
            } => {
                mix(&mut hash, 1);
                mix(&mut hash, u64::from(*width));
                mix(&mut hash, u64::from(*height));
                mix(&mut hash, u64::from(*depth));
                mix(&mut hash, u64::from(*mip_levels));
                mix(&mut hash, u64::from(*sample_count));
                mix(&mut hash, *format_key);
                mix(&mut hash, *dimension_key);
                mix(&mut hash, *residency_key);
                mix(&mut hash, u64::from(*usage_bits));
            }
            Self::Buffer {
                size_bytes,
                usage_bits,
            } => {
                mix(&mut hash, 2);
                mix(&mut hash, *size_bytes);
                mix(&mut hash, u64::from(*usage_bits));
            }
        }
        hash
    }
}

fn transient_texture_bucket_key(
    lifetime: &RenderGraphResourceLifetime,
) -> Option<TransientAllocationBucketKey> {
    let RenderGraphResourceDesc::Texture(desc) = &lifetime.desc else {
        return None;
    };
    Some(TransientAllocationBucketKey::Texture {
        width: desc.width,
        height: desc.height,
        depth: desc.depth,
        mip_levels: desc.mip_levels,
        sample_count: desc.sample_count,
        format_key: texture_format_key(desc.format),
        dimension_key: texture_dimension_key(desc.dimension),
        residency_key: texture_residency_key(desc.residency),
        usage_bits: desc.usage.bits(),
    })
}

fn transient_buffer_bucket_key(
    lifetime: &RenderGraphResourceLifetime,
) -> Option<TransientAllocationBucketKey> {
    let RenderGraphResourceDesc::Buffer(desc) = &lifetime.desc else {
        return None;
    };
    Some(TransientAllocationBucketKey::Buffer {
        size_bytes: desc.size_bytes,
        usage_bits: desc.usage.bits(),
    })
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

fn texture_format_key(format: TextureFormat) -> u64 {
    match format {
        TextureFormat::R8Unorm => 1,
        TextureFormat::R16Float => 2,
        TextureFormat::R32Float => 3,
        TextureFormat::Rg16Float => 4,
        TextureFormat::Rg11b10Ufloat => 5,
        TextureFormat::Rgba8Unorm => 6,
        TextureFormat::Rgba8UnormSrgb => 7,
        TextureFormat::Bgra8Unorm => 8,
        TextureFormat::Bgra8UnormSrgb => 9,
        TextureFormat::Rgba16Float => 10,
        TextureFormat::Rgba32Float => 11,
        TextureFormat::Depth24Plus => 12,
        TextureFormat::Depth24PlusStencil8 => 13,
        TextureFormat::Depth32Float => 14,
    }
}

fn texture_dimension_key(dimension: TextureDimension) -> u64 {
    match dimension {
        TextureDimension::D1 => 1,
        TextureDimension::D2 => 2,
        TextureDimension::D2Array => 3,
        TextureDimension::D3 => 4,
        TextureDimension::Cube => 5,
    }
}

fn texture_residency_key(residency: TextureResidency) -> u64 {
    match residency {
        TextureResidency::Dense => 1,
        TextureResidency::SparseReserved => 2,
    }
}

const fn resource_kind_sort_key(kind: RenderGraphResourceKind) -> u8 {
    match kind {
        RenderGraphResourceKind::TransientTexture => 0,
        RenderGraphResourceKind::TransientBuffer => 1,
        RenderGraphResourceKind::External => 2,
    }
}
