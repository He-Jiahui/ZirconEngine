use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;

use crate::core::resource::ResourceId;
use crate::core::framework::render::RenderCapabilitySummary;
use crate::graphics::scene::resources::GpuTextureResource;

/// Stable owner-provided identity for an entry in the material texture table.
///
/// The resource streamer derives this from the prepared texture's asset identity and revision.
/// Keeping the logical key separate from a `wgpu::TextureView` lets mip streaming replace a view
/// in-place without invalidating material uniform slot indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum BindlessTextureKey {
    Resource { id: ResourceId, revision: u64 },
    Opaque(u64),
}

impl BindlessTextureKey {
    /// Creates an opaque owner key for synthetic fallback resources and isolated tests.
    pub(crate) const fn new(value: u64) -> Self {
        Self::Opaque(value)
    }

    /// Creates a collision-free key for a streamed asset texture revision.
    pub(crate) const fn from_resource(id: ResourceId, revision: u64) -> Self {
        Self::Resource { id, revision }
    }
}

/// Index written to the material payload and consumed by the bindless shader variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindlessSlotIndex(u32);

impl BindlessSlotIndex {
    pub(crate) const FALLBACK: Self = Self(0);

    pub(crate) const fn new(value: u32) -> Self {
        Self(value)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }
}

/// Owner-side lease for a material texture slot.
///
/// The shader only receives [`BindlessSlotIndex`]. The generation stays on the CPU so a delayed
/// material teardown cannot release a slot that has already been recycled for another texture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct BindlessSlotLease {
    index: BindlessSlotIndex,
    generation: u32,
}

impl BindlessSlotLease {
    pub(crate) const fn slot_index(self) -> BindlessSlotIndex {
        self.index
    }

    pub(crate) const fn is_fallback(self) -> bool {
        self.index.get() == BindlessSlotIndex::FALLBACK.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BindlessMaterialSlabError {
    ZeroCapacity,
    Exhausted { capacity: u32 },
    ReferenceCountOverflow { slot: BindlessSlotIndex },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BindlessMaterialBindingTableError {
    Slab(BindlessMaterialSlabError),
}

impl From<BindlessMaterialSlabError> for BindlessMaterialBindingTableError {
    fn from(error: BindlessMaterialSlabError) -> Self {
        Self::Slab(error)
    }
}

#[derive(Clone, Debug, Default)]
struct SlotState {
    texture: Option<BindlessTextureKey>,
    generation: u32,
    reference_count: u32,
}

/// Fixed-capacity material texture slot allocator for the bindless group-2 layout.
///
/// Slot zero is permanently bound to `fallback_texture`. Every vacant slot is represented by the
/// same fallback key in [`Self::binding_table`], so the eventual WGPU binding array stays valid on
/// platforms that reject partially populated arrays. Allocation and release are O(1) expected
/// time and never grow the preallocated table on the render hot path.
#[derive(Debug)]
pub(crate) struct BindlessMaterialSlab {
    fallback_texture: BindlessTextureKey,
    binding_table: Vec<BindlessTextureKey>,
    slots: Vec<SlotState>,
    free_slots: Vec<u32>,
    texture_slots: HashMap<BindlessTextureKey, BindlessSlotIndex>,
}

impl BindlessMaterialSlab {
    /// Returns the exact capacity negotiated for the global texture/sampler arrays.
    ///
    /// Slot zero is reserved for the fallback texture, so the framework capability gate already
    /// guarantees that the returned value leaves room for at least one dynamic material slot.
    pub(crate) fn capacity_for_capabilities(capabilities: &RenderCapabilitySummary) -> Option<u32> {
        capabilities
            .bindless_material_supported()
            .then(|| capabilities.bindless_material_slot_capacity())
    }

    pub(crate) fn new(
        capacity: u32,
        fallback_texture: BindlessTextureKey,
    ) -> Result<Self, BindlessMaterialSlabError> {
        if capacity == 0 {
            return Err(BindlessMaterialSlabError::ZeroCapacity);
        }

        let capacity_usize = capacity as usize;
        let mut free_slots = Vec::with_capacity(capacity_usize.saturating_sub(1));
        for slot in (1..capacity).rev() {
            free_slots.push(slot);
        }
        Ok(Self {
            fallback_texture,
            binding_table: vec![fallback_texture; capacity_usize],
            slots: vec![SlotState::default(); capacity_usize],
            free_slots,
            texture_slots: HashMap::with_capacity(capacity_usize.saturating_sub(1)),
        })
    }

    pub(crate) fn allocate(
        &mut self,
        texture: BindlessTextureKey,
    ) -> Result<BindlessSlotLease, BindlessMaterialSlabError> {
        if texture == self.fallback_texture {
            return Ok(self.fallback_lease());
        }
        if let Some(index) = self.texture_slots.get(&texture).copied() {
            let state = &mut self.slots[index.get() as usize];
            state.reference_count = state
                .reference_count
                .checked_add(1)
                .ok_or(BindlessMaterialSlabError::ReferenceCountOverflow { slot: index })?;
            return Ok(BindlessSlotLease {
                index,
                generation: state.generation,
            });
        }

        let index = self.free_slots.pop().map(BindlessSlotIndex).ok_or(
            BindlessMaterialSlabError::Exhausted {
                capacity: self.capacity(),
            },
        )?;
        let state = &mut self.slots[index.get() as usize];
        debug_assert!(state.texture.is_none());
        debug_assert_eq!(state.reference_count, 0);
        state.texture = Some(texture);
        state.reference_count = 1;
        self.binding_table[index.get() as usize] = texture;
        self.texture_slots.insert(texture, index);

        Ok(BindlessSlotLease {
            index,
            generation: state.generation,
        })
    }

    /// Releases one material reference. Returns false for the fallback or a stale lease.
    pub(crate) fn release(&mut self, lease: BindlessSlotLease) -> bool {
        if lease.is_fallback() {
            return false;
        }
        let slot_index = lease.index.get() as usize;
        let Some(state) = self.slots.get_mut(slot_index) else {
            return false;
        };
        if state.generation != lease.generation || state.reference_count == 0 {
            return false;
        }

        state.reference_count -= 1;
        if state.reference_count != 0 {
            return true;
        }

        let texture = state
            .texture
            .take()
            .expect("active bindless material slot must retain its texture key");
        state.generation = state.generation.wrapping_add(1);
        self.binding_table[slot_index] = self.fallback_texture;
        self.texture_slots.remove(&texture);
        self.free_slots.push(lease.index.get());
        true
    }

    pub(crate) const fn capacity(&self) -> u32 {
        self.binding_table.len() as u32
    }

    pub(crate) const fn fallback_slot(&self) -> BindlessSlotIndex {
        BindlessSlotIndex::FALLBACK
    }

    pub(crate) fn active_slot_count(&self) -> u32 {
        self.texture_slots.len().min(u32::MAX as usize) as u32
    }

    pub(crate) fn binding_table(&self) -> &[BindlessTextureKey] {
        &self.binding_table
    }

    fn contains_current_lease(&self, lease: BindlessSlotLease) -> bool {
        if lease.is_fallback() {
            return true;
        }
        self.slots
            .get(lease.index.get() as usize)
            .is_some_and(|state| state.generation == lease.generation && state.reference_count != 0)
    }

    fn fallback_lease(&self) -> BindlessSlotLease {
        BindlessSlotLease {
            index: BindlessSlotIndex::FALLBACK,
            generation: 0,
        }
    }
}

/// Owns the fixed group-2 WGPU binding arrays used by the bindless material variant.
///
/// The table deliberately retains `Arc<GpuTextureResource>` values instead of bare views. That
/// keeps texture and sampler lifetimes coupled while a material slot is live, and lets mip
/// streaming replace a view without changing the shader-visible slot index. Rebinding is a
/// prepare-time operation only; no draw-path allocation or bind-group reconstruction occurs.
pub(crate) struct BindlessMaterialBindingTable {
    fallback_texture_key: BindlessTextureKey,
    fallback_texture: Arc<GpuTextureResource>,
    slab: BindlessMaterialSlab,
    slot_textures: Vec<Arc<GpuTextureResource>>,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    bind_group_generation: u64,
}

impl BindlessMaterialBindingTable {
    pub(crate) fn new(
        device: &wgpu::Device,
        capacity: u32,
        fallback_texture_key: BindlessTextureKey,
        fallback_texture: Arc<GpuTextureResource>,
    ) -> Result<Self, BindlessMaterialBindingTableError> {
        let slab = BindlessMaterialSlab::new(capacity, fallback_texture_key)?;
        let capacity = NonZeroU32::new(capacity)
            .expect("BindlessMaterialSlab rejects zero capacity before creating a table");
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-bindless-material-layout"),
            entries: &bindless_material_binding_array_layout_entries(capacity),
        });
        let slot_textures = vec![Arc::clone(&fallback_texture); capacity.get() as usize];
        let bind_group = create_bindless_material_bind_group(device, &layout, &slot_textures);

        Ok(Self {
            fallback_texture_key,
            fallback_texture,
            slab,
            slot_textures,
            layout,
            bind_group,
            bind_group_generation: 1,
        })
    }

    pub(crate) fn capacity_for_capabilities(capabilities: &RenderCapabilitySummary) -> Option<u32> {
        BindlessMaterialSlab::capacity_for_capabilities(capabilities)
    }

    pub(crate) fn allocate(
        &mut self,
        device: &wgpu::Device,
        texture_key: BindlessTextureKey,
        texture: Arc<GpuTextureResource>,
    ) -> Result<BindlessSlotLease, BindlessMaterialBindingTableError> {
        let lease = self.slab.allocate(texture_key)?;
        if lease.is_fallback() {
            return Ok(lease);
        }

        let slot_index = lease.slot_index().get() as usize;
        if !Arc::ptr_eq(&self.slot_textures[slot_index], &texture) {
            self.slot_textures[slot_index] = texture;
            self.rebuild_bind_group(device);
        }
        Ok(lease)
    }

    /// Replaces the resource behind an active slot without allocating another shader-visible
    /// index. Mip-streaming calls this after it publishes a newly resident texture view.
    pub(crate) fn replace_texture(
        &mut self,
        device: &wgpu::Device,
        lease: BindlessSlotLease,
        texture: Arc<GpuTextureResource>,
    ) -> bool {
        if !self.slab.contains_current_lease(lease) || lease.is_fallback() {
            return false;
        }

        let slot_index = lease.slot_index().get() as usize;
        if Arc::ptr_eq(&self.slot_textures[slot_index], &texture) {
            return true;
        }
        self.slot_textures[slot_index] = texture;
        self.rebuild_bind_group(device);
        true
    }

    /// Releases a material reference and only rebuilds the WGPU array when a slot becomes vacant.
    pub(crate) fn release(&mut self, device: &wgpu::Device, lease: BindlessSlotLease) -> bool {
        let slot = lease.slot_index();
        if !self.slab.release(lease) {
            return false;
        }
        if slot == BindlessSlotIndex::FALLBACK
            || self.slab.binding_table()[slot.get() as usize] != self.fallback_texture_key
        {
            return true;
        }

        self.slot_textures[slot.get() as usize] = Arc::clone(&self.fallback_texture);
        self.rebuild_bind_group(device);
        true
    }

    pub(crate) fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub(crate) fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub(crate) const fn capacity(&self) -> u32 {
        self.slab.capacity()
    }

    pub(crate) fn active_slot_count(&self) -> u32 {
        self.slab.active_slot_count()
    }

    pub(crate) const fn bind_group_generation(&self) -> u64 {
        self.bind_group_generation
    }

    fn rebuild_bind_group(&mut self, device: &wgpu::Device) {
        self.bind_group =
            create_bindless_material_bind_group(device, &self.layout, &self.slot_textures);
        self.bind_group_generation = self.bind_group_generation.saturating_add(1);
    }
}

fn bindless_material_binding_array_layout_entries(
    capacity: NonZeroU32,
) -> [wgpu::BindGroupLayoutEntry; 2] {
    [
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: Some(capacity),
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: Some(capacity),
        },
    ]
}

fn create_bindless_material_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    slot_textures: &[Arc<GpuTextureResource>],
) -> wgpu::BindGroup {
    let texture_views = slot_textures
        .iter()
        .map(|texture| texture.view())
        .collect::<Vec<_>>();
    let samplers = slot_textures
        .iter()
        .map(|texture| texture.sampler())
        .collect::<Vec<_>>();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-bindless-material-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureViewArray(&texture_views),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::SamplerArray(&samplers),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::{
        BindlessMaterialSlab, BindlessMaterialSlabError, BindlessSlotIndex, BindlessTextureKey,
        bindless_material_binding_array_layout_entries,
    };
    use crate::core::framework::render::RenderCapabilitySummary;
    use crate::core::resource::ResourceId;
    use std::num::NonZeroU32;

    const FALLBACK: BindlessTextureKey = BindlessTextureKey::new(1);
    const BASE_COLOR: BindlessTextureKey = BindlessTextureKey::new(2);
    const NORMAL: BindlessTextureKey = BindlessTextureKey::new(3);
    const EMISSIVE: BindlessTextureKey = BindlessTextureKey::new(4);

    #[test]
    fn vacant_slots_are_prepopulated_with_the_fallback_texture() {
        let slab = BindlessMaterialSlab::new(4, FALLBACK).expect("valid slab");

        assert_eq!(slab.fallback_slot(), BindlessSlotIndex::FALLBACK);
        assert_eq!(
            slab.binding_table(),
            &[FALLBACK, FALLBACK, FALLBACK, FALLBACK]
        );
        assert_eq!(slab.active_slot_count(), 0);
    }

    #[test]
    fn identical_textures_deduplicate_without_consuming_another_slot() {
        let mut slab = BindlessMaterialSlab::new(3, FALLBACK).expect("valid slab");
        let first = slab.allocate(BASE_COLOR).expect("first allocation");
        let second = slab.allocate(BASE_COLOR).expect("deduplicated allocation");

        assert_eq!(first.slot_index(), second.slot_index());
        assert_eq!(first.slot_index().get(), 1);
        assert_eq!(slab.active_slot_count(), 1);
        assert_eq!(slab.binding_table(), &[FALLBACK, BASE_COLOR, FALLBACK]);

        assert!(slab.release(first));
        assert_eq!(slab.active_slot_count(), 1);
        assert!(slab.release(second));
        assert_eq!(slab.binding_table(), &[FALLBACK, FALLBACK, FALLBACK]);
    }

    #[test]
    fn stale_lease_cannot_release_a_recycled_slot() {
        let mut slab = BindlessMaterialSlab::new(2, FALLBACK).expect("valid slab");
        let old = slab.allocate(BASE_COLOR).expect("first allocation");
        assert!(slab.release(old));
        let replacement = slab.allocate(NORMAL).expect("recycled allocation");

        assert_eq!(old.slot_index(), replacement.slot_index());
        assert!(!slab.release(old));
        assert_eq!(slab.binding_table(), &[FALLBACK, NORMAL]);
        assert!(slab.release(replacement));
    }

    #[test]
    fn capacity_excludes_the_reserved_fallback_slot_and_recovers_after_release() {
        let mut slab = BindlessMaterialSlab::new(3, FALLBACK).expect("valid slab");
        let base_color = slab.allocate(BASE_COLOR).expect("base color allocation");
        let normal = slab.allocate(NORMAL).expect("normal allocation");

        assert_eq!(
            slab.allocate(EMISSIVE),
            Err(BindlessMaterialSlabError::Exhausted { capacity: 3 })
        );
        assert!(slab.release(normal));
        assert_eq!(
            slab.allocate(EMISSIVE)
                .expect("recovered slot")
                .slot_index()
                .get(),
            2
        );
        assert!(slab.release(base_color));
    }

    #[test]
    fn fallback_allocations_do_not_consume_a_dynamic_slot() {
        let mut slab = BindlessMaterialSlab::new(2, FALLBACK).expect("valid slab");
        let fallback = slab.allocate(FALLBACK).expect("fallback slot");
        let base_color = slab.allocate(BASE_COLOR).expect("dynamic slot");

        assert!(fallback.is_fallback());
        assert!(!slab.release(fallback));
        assert_eq!(base_color.slot_index().get(), 1);
    }

    #[test]
    fn resource_texture_keys_keep_revisions_distinct_without_hash_identity() {
        let resource = ResourceId::from_stable_label("bindless-texture-key-test");

        assert_eq!(
            BindlessTextureKey::from_resource(resource, 2),
            BindlessTextureKey::from_resource(resource, 2)
        );
        assert_ne!(
            BindlessTextureKey::from_resource(resource, 2),
            BindlessTextureKey::from_resource(resource, 3)
        );
        assert_ne!(
            BindlessTextureKey::from_resource(resource, 2),
            BindlessTextureKey::new(2)
        );
    }

    #[test]
    fn zero_capacity_is_rejected() {
        assert_eq!(
            BindlessMaterialSlab::new(0, FALLBACK),
            Err(BindlessMaterialSlabError::ZeroCapacity)
        );
    }

    #[test]
    fn negotiated_capability_capacity_uses_the_texture_and_sampler_lower_bound() {
        let capabilities = RenderCapabilitySummary {
            supports_texture_binding_array: true,
            supports_partially_bound_binding_array: true,
            supports_non_uniform_resource_indexing: true,
            max_binding_array_elements_per_shader_stage: 512,
            max_binding_array_sampler_elements_per_shader_stage: 64,
            ..RenderCapabilitySummary::default()
        };

        assert_eq!(
            BindlessMaterialSlab::capacity_for_capabilities(&capabilities),
            Some(64)
        );
    }

    #[test]
    fn missing_bindless_capability_has_no_slab_capacity() {
        assert_eq!(
            BindlessMaterialSlab::capacity_for_capabilities(&RenderCapabilitySummary::default()),
            None
        );
    }

    #[test]
    fn bindless_layout_uses_fixed_texture_and_sampler_arrays() {
        let entries = bindless_material_binding_array_layout_entries(
            NonZeroU32::new(64).expect("nonzero capacity"),
        );

        assert_eq!(entries[0].binding, 0);
        assert_eq!(entries[0].count.map(NonZeroU32::get), Some(64));
        assert!(matches!(entries[0].ty, wgpu::BindingType::Texture { .. }));
        assert_eq!(entries[1].binding, 1);
        assert_eq!(entries[1].count.map(NonZeroU32::get), Some(64));
        assert!(matches!(entries[1].ty, wgpu::BindingType::Sampler(_)));
    }
}
