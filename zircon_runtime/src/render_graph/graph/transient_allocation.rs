use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::rhi::{TextureDimension, TextureFormat, TextureResidency};

use super::super::error::RenderGraphError;
use super::super::types::{
    RenderGraphResource, RenderGraphResourceDesc, RenderGraphResourceKind,
    RenderGraphResourceLifetime,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientAllocation {
    /// Collision-free compiler-local identity for one compatible bucket slot.
    /// This is not an RHI physical allocation or backend lease.
    pub allocation_id: CompiledRenderGraphTransientAllocationId,
    pub resource: RenderGraphResource,
    pub resource_name: String,
    pub kind: RenderGraphResourceKind,
    pub slot: usize,
    pub size_bytes: u64,
    /// Inclusive compiled-pass interval. Every allocation sharing `allocation_id` has a
    /// compiler-validated disjoint interval before materialization may reuse its backing.
    pub first_pass: usize,
    pub last_pass: usize,
    pub bucket_key_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CompiledRenderGraphTransientAllocationId(pub(crate) usize);

impl CompiledRenderGraphTransientAllocationId {
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Backend-neutral identity for a physical backing proven by the compiler.
///
/// The collision-free `allocation_id` is the correctness identity. Bucket and
/// slot are retained for diagnostics and materialization admission; neither is
/// sufficient by itself to identify a backing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RenderGraphPhysicalAllocationId {
    allocation_id: CompiledRenderGraphTransientAllocationId,
    kind: RenderGraphResourceKind,
    bucket_key_hash: u64,
    slot: usize,
}

impl RenderGraphPhysicalAllocationId {
    pub(super) const fn from_transient_allocation(
        allocation: &CompiledRenderGraphTransientAllocation,
    ) -> Self {
        Self {
            allocation_id: allocation.allocation_id,
            kind: allocation.kind,
            bucket_key_hash: allocation.bucket_key_hash,
            slot: allocation.slot,
        }
    }

    pub const fn allocation_id(self) -> CompiledRenderGraphTransientAllocationId {
        self.allocation_id
    }

    pub const fn kind(self) -> RenderGraphResourceKind {
        self.kind
    }

    pub const fn bucket_key_hash(self) -> u64 {
        self.bucket_key_hash
    }

    pub const fn slot(self) -> usize {
        self.slot
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompiledRenderGraphTransientSlotReservation {
    pub allocation_id: CompiledRenderGraphTransientAllocationId,
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
    pub fn physical_allocation_id_for_resource(
        &self,
        resource: RenderGraphResource,
    ) -> Option<RenderGraphPhysicalAllocationId> {
        self.allocations
            .iter()
            .find(|allocation| allocation.resource == resource)
            .map(RenderGraphPhysicalAllocationId::from_transient_allocation)
    }

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

    pub fn allocation_id_for(
        &self,
        resource_name: &str,
    ) -> Option<CompiledRenderGraphTransientAllocationId> {
        self.allocations
            .iter()
            .find(|allocation| allocation.resource_name == resource_name)
            .map(|allocation| allocation.allocation_id)
    }

    pub fn slot_bytes_for_allocation(
        &self,
        allocation_id: CompiledRenderGraphTransientAllocationId,
    ) -> Option<u64> {
        self.slot_reservations
            .iter()
            .find(|reservation| reservation.allocation_id == allocation_id)
            .map(|reservation| reservation.bytes_reserved)
    }

    /// Validates the compiler-local interval proof required before a transient backing can be
    /// shared. This is deliberately independent from any RHI physical allocation or lease.
    pub fn validate_transient_allocation_intervals(&self) -> Result<(), RenderGraphError> {
        let mut allocations_by_id = BTreeMap::<
            CompiledRenderGraphTransientAllocationId,
            Vec<&CompiledRenderGraphTransientAllocation>,
        >::new();

        for allocation in &self.allocations {
            if allocation.first_pass > allocation.last_pass {
                return Err(RenderGraphError::TransientAllocationInvalidInterval {
                    allocation_id: allocation.allocation_id.index(),
                    resource: allocation.resource_name.clone(),
                    first_pass: allocation.first_pass,
                    last_pass: allocation.last_pass,
                });
            }
            allocations_by_id
                .entry(allocation.allocation_id)
                .or_default()
                .push(allocation);
        }

        for mut allocations in allocations_by_id.into_values() {
            allocations.sort_by(|left, right| {
                left.first_pass
                    .cmp(&right.first_pass)
                    .then_with(|| left.last_pass.cmp(&right.last_pass))
                    .then_with(|| left.resource_name.cmp(&right.resource_name))
            });
            for pair in allocations.windows(2) {
                let [first, second] = pair else {
                    continue;
                };
                if first.last_pass >= second.first_pass {
                    return Err(RenderGraphError::TransientAllocationIntervalsOverlap {
                        allocation_id: first.allocation_id.index(),
                        first_resource: first.resource_name.clone(),
                        first_start: first.first_pass,
                        first_end: first.last_pass,
                        second_resource: second.resource_name.clone(),
                        second_start: second.first_pass,
                        second_end: second.last_pass,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn total_dense_bytes_reserved(&self) -> u64 {
        self.dense_texture_bytes_reserved
            .saturating_add(self.dense_buffer_bytes_reserved)
    }
}

pub(crate) fn build_transient_allocation_plan(
    resource_lifetimes: &[RenderGraphResourceLifetime],
) -> Result<CompiledRenderGraphTransientAllocationPlan, RenderGraphError> {
    let mut next_allocation_id = 0_usize;
    let mut allocations = allocate_transient_lifetimes_by_bucket(
        resource_lifetimes.iter().filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientTexture
                && !lifetime.usage.persistent
                && !lifetime.is_sparse_reserved_texture()
                && !lifetime.is_texture_view_alias()
        }),
        transient_texture_bucket_key,
        &mut next_allocation_id,
    )?;
    let sparse_texture_lifetimes = resource_lifetimes
        .iter()
        .filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientTexture
                && !lifetime.imported
                && !lifetime.usage.persistent
                && lifetime.is_sparse_reserved_texture()
                && !lifetime.is_texture_view_alias()
        })
        .collect::<Vec<_>>();
    let sparse_texture_slot_count = sparse_texture_lifetimes.len();
    let sparse_texture_virtual_bytes =
        sparse_texture_lifetimes
            .iter()
            .copied()
            .try_fold(0_u64, |total, lifetime| {
                let size_bytes = resource_lifetime_size_bytes(lifetime)?;
                total.checked_add(size_bytes).ok_or(
                    RenderGraphError::TransientAllocationBytesOverflow {
                        kind: RenderGraphResourceKind::TransientTexture,
                    },
                )
            })?;
    let mut buffer_allocations = allocate_transient_lifetimes_by_bucket(
        resource_lifetimes.iter().filter(|lifetime| {
            lifetime.kind == RenderGraphResourceKind::TransientBuffer && !lifetime.usage.persistent
        }),
        transient_buffer_bucket_key,
        &mut next_allocation_id,
    )?;
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
    let dense_texture_bytes_reserved = checked_reservation_bytes(
        &slot_reservations,
        RenderGraphResourceKind::TransientTexture,
    )?;
    let dense_buffer_bytes_reserved =
        checked_reservation_bytes(&slot_reservations, RenderGraphResourceKind::TransientBuffer)?;
    dense_texture_bytes_reserved
        .checked_add(dense_buffer_bytes_reserved)
        .ok_or(RenderGraphError::TransientAllocationTotalBytesOverflow)?;

    Ok(CompiledRenderGraphTransientAllocationPlan {
        allocations,
        slot_reservations,
        texture_slot_count,
        buffer_slot_count,
        sparse_texture_slot_count,
        dense_texture_bytes_reserved,
        dense_buffer_bytes_reserved,
        sparse_texture_virtual_bytes,
    })
}

fn allocate_transient_lifetimes<'a>(
    lifetimes: impl Iterator<Item = &'a RenderGraphResourceLifetime>,
    bucket_key_hash: u64,
    next_allocation_id: &mut usize,
    allocation_ids_by_slot: &mut BTreeMap<usize, CompiledRenderGraphTransientAllocationId>,
) -> Result<Vec<CompiledRenderGraphTransientAllocation>, RenderGraphError> {
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
        let allocation_id = *allocation_ids_by_slot.entry(slot).or_insert_with(|| {
            let allocation_id = CompiledRenderGraphTransientAllocationId(*next_allocation_id);
            *next_allocation_id += 1;
            allocation_id
        });
        active_slots.insert((lifetime.last_pass, slot));
        allocations.push(CompiledRenderGraphTransientAllocation {
            allocation_id,
            resource: lifetime.resource,
            resource_name: lifetime.name.clone(),
            kind: lifetime.kind,
            slot,
            size_bytes: resource_lifetime_size_bytes(lifetime)?,
            first_pass: lifetime.first_pass,
            last_pass: lifetime.last_pass,
            bucket_key_hash,
        });
    }

    Ok(allocations)
}

fn allocate_transient_lifetimes_by_bucket<'a, F>(
    lifetimes: impl Iterator<Item = &'a RenderGraphResourceLifetime>,
    bucket_key_for: F,
    next_allocation_id: &mut usize,
) -> Result<Vec<CompiledRenderGraphTransientAllocation>, RenderGraphError>
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
        let mut allocation_ids_by_slot = BTreeMap::new();
        allocations.extend(allocate_transient_lifetimes(
            lifetimes.into_iter(),
            bucket_key.stable_hash(),
            next_allocation_id,
            &mut allocation_ids_by_slot,
        )?);
    }
    Ok(allocations)
}

pub(crate) fn validate_resource_lifetime_storage_sizes(
    resource_lifetimes: &[RenderGraphResourceLifetime],
) -> Result<(), RenderGraphError> {
    for lifetime in resource_lifetimes {
        let _ = resource_lifetime_size_bytes(lifetime)?;
    }
    Ok(())
}

fn checked_reservation_bytes(
    reservations: &[CompiledRenderGraphTransientSlotReservation],
    kind: RenderGraphResourceKind,
) -> Result<u64, RenderGraphError> {
    reservations
        .iter()
        .filter(|reservation| reservation.kind == kind)
        .try_fold(0_u64, |total, reservation| {
            total
                .checked_add(reservation.bytes_reserved)
                .ok_or(RenderGraphError::TransientAllocationBytesOverflow { kind })
        })
}

fn slot_reservations_for(
    allocations: &[CompiledRenderGraphTransientAllocation],
) -> Vec<CompiledRenderGraphTransientSlotReservation> {
    let mut reservations = BTreeMap::<
        CompiledRenderGraphTransientAllocationId,
        CompiledRenderGraphTransientSlotReservation,
    >::new();

    for allocation in allocations {
        reservations
            .entry(allocation.allocation_id)
            .and_modify(|reservation| {
                reservation.bytes_reserved = reservation.bytes_reserved.max(allocation.size_bytes);
            })
            .or_insert(CompiledRenderGraphTransientSlotReservation {
                allocation_id: allocation.allocation_id,
                kind: allocation.kind,
                slot: allocation.slot,
                bytes_reserved: allocation.size_bytes,
                bucket_key_hash: allocation.bucket_key_hash,
            });
    }

    let mut reservations = reservations.into_values().collect::<Vec<_>>();

    reservations.sort_by(|left, right| {
        left.allocation_id
            .cmp(&right.allocation_id)
            .then_with(|| {
                resource_kind_sort_key(left.kind).cmp(&resource_kind_sort_key(right.kind))
            })
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

fn resource_lifetime_size_bytes(
    lifetime: &RenderGraphResourceLifetime,
) -> Result<u64, RenderGraphError> {
    match &lifetime.desc {
        RenderGraphResourceDesc::Texture(desc) => {
            desc.checked_storage_size_bytes().ok_or_else(|| {
                RenderGraphError::TextureStorageSizeOverflow {
                    resource: lifetime.name.clone(),
                }
            })
        }
        RenderGraphResourceDesc::Buffer(desc) => Ok(desc.size_bytes),
        RenderGraphResourceDesc::External => Ok(0),
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
