use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::framework::render::GpuLightData;
use wgpu::util::DeviceExt;

use super::binding::{
    GpuSceneVisibleInstanceRemapParams, create_gpu_scene_bind_group,
    create_gpu_scene_bind_group_layout,
};
use super::id_allocator::GpuSceneIdAllocator;
use super::layout::{
    GPU_INSTANCE_DATA_STRIDE, GPU_PRIMITIVE_DATA_STRIDE, GpuInstanceData, GpuMorphDelta,
    GpuMorphPayload, GpuMorphWeight, GpuPrimitiveData, GpuVirtualGeometryClusterWord,
    GpuVirtualGeometryPage,
};
use super::prev_skinned_palette::GpuSceneSkinnedJointPaletteState;
use super::prev_skinned_source::GpuSceneSkinnedGpuSourceState;
use super::skinned_palette_arena::GpuSceneSkinnedPaletteArena;
use super::staging_ring::GpuSceneStagingRing;
use super::update_queue::GpuSceneUpdateQueue;

pub(crate) const GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY: u32 = 64;
pub(crate) const GPU_SCENE_INITIAL_INSTANCE_CAPACITY: u32 = 64;
pub(crate) const GPU_SCENE_INITIAL_LIGHT_CAPACITY: u32 = 1;
pub(crate) const GPU_SCENE_INITIAL_MORPH_CAPACITY: u32 = 1;
pub(crate) const GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY: u32 = 1;

const GPU_SCENE_VISIBLE_INSTANCE_REMAP_FALLBACK_BYTES: u64 = 4;
const GPU_SCENE_MORPH_FALLBACK_BYTES: u64 = 16;
const GPU_SCENE_VIRTUAL_GEOMETRY_FALLBACK_BYTES: u64 = 16;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneEntry {
    pub(crate) stable_instance_key: u64,
    pub(crate) primitive_index: u32,
    pub(crate) first_instance_index: u32,
    pub(crate) instance_count: u32,
    pub(crate) last_transform_revision: u64,
    pub(crate) has_rolled_previous_transform: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneStats {
    pub(crate) primitive_count: u32,
    pub(crate) instance_count: u32,
    pub(crate) light_count: u32,
    pub(crate) dirty_entry_count: usize,
    pub(crate) uploaded_bytes: u64,
    pub(crate) primitive_capacity: u32,
    pub(crate) instance_capacity: u32,
    pub(crate) light_capacity: u32,
    pub(crate) free_span_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum GpuSceneUploadPath {
    #[default]
    DirectQueueWrite,
    StagingCopy,
}

impl GpuSceneUploadPath {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::DirectQueueWrite => "direct_queue_write",
            Self::StagingCopy => "staging_copy",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneUploadReport {
    pub(crate) upload_path: GpuSceneUploadPath,
    pub(crate) uploaded_bytes: u64,
    pub(crate) primitive_upload_range_count: usize,
    pub(crate) instance_upload_range_count: usize,
    pub(crate) light_upload_range_count: usize,
}

impl GpuSceneUploadReport {
    pub(crate) fn with_additional_uploaded_bytes(mut self, uploaded_bytes: u64) -> Self {
        self.uploaded_bytes = self.uploaded_bytes.saturating_add(uploaded_bytes);
        self
    }
}

/// Owns the GPUScene storage buffers and CPU mirrors before frame-path wiring.
///
/// Registration keeps stable primitive/instance indices until explicit
/// unregister. Uploads use coordinator-batched range writes for small merged
/// ranges and a persistent staging-copy ring for large frames; callers can
/// feed full-frame extracts every frame without re-uploading unchanged entries.
pub(crate) struct GpuScene {
    pub(super) primitive_buffer: wgpu::Buffer,
    pub(super) instance_buffer: wgpu::Buffer,
    pub(super) light_buffer: wgpu::Buffer,
    pub(super) skinned_palette_arena: GpuSceneSkinnedPaletteArena,
    direct_visible_instance_remap_buffer: wgpu::Buffer,
    pub(super) direct_visible_instance_remap_params_buffer: wgpu::Buffer,
    pub(super) remapped_visible_instance_remap_params_buffer: wgpu::Buffer,
    pub(super) morph_deltas_buffer: wgpu::Buffer,
    pub(super) morph_weights_buffer: wgpu::Buffer,
    pub(super) virtual_geometry_pages_buffer: wgpu::Buffer,
    pub(super) virtual_geometry_clusters_buffer: wgpu::Buffer,
    pub(super) morph_payloads_buffer: wgpu::Buffer,
    scene_bind_group_layout: wgpu::BindGroupLayout,
    scene_bind_groups: [wgpu::BindGroup; 2],
    pub(super) primitive_shadow: Vec<GpuPrimitiveData>,
    pub(super) instance_shadow: Vec<GpuInstanceData>,
    pub(super) light_shadow: Vec<GpuLightData>,
    pub(super) morph_deltas_shadow: Vec<GpuMorphDelta>,
    pub(super) morph_weights_shadow: Vec<GpuMorphWeight>,
    pub(super) virtual_geometry_pages_shadow: Vec<GpuVirtualGeometryPage>,
    pub(super) virtual_geometry_clusters_shadow: Vec<GpuVirtualGeometryClusterWord>,
    pub(super) morph_payloads_shadow: Vec<GpuMorphPayload>,
    pub(super) primitive_ids: GpuSceneIdAllocator,
    pub(super) instance_ids: GpuSceneIdAllocator,
    pub(super) entries: HashMap<u64, GpuSceneEntry>,
    pub(super) pending_prev_transform_rolls: HashSet<u64>,
    pub(super) current_skinned_joint_palettes: HashMap<u64, GpuSceneSkinnedJointPaletteState>,
    pub(super) previous_skinned_joint_palettes: HashMap<u64, GpuSceneSkinnedJointPaletteState>,
    pub(super) current_skinned_gpu_sources: HashMap<u64, GpuSceneSkinnedGpuSourceState>,
    pub(super) previous_skinned_gpu_sources: HashMap<u64, GpuSceneSkinnedGpuSourceState>,
    pub(super) current_morph_weights: HashMap<u64, Arc<[f32]>>,
    pub(super) previous_morph_weights: HashMap<u64, Arc<[f32]>>,
    pub(super) updates: GpuSceneUpdateQueue,
    pub(super) staging_ring: GpuSceneStagingRing,
    stats: GpuSceneStats,
    primitive_capacity: u32,
    instance_capacity: u32,
    light_capacity: u32,
    pub(super) morph_payloads_capacity: u32,
    pub(super) morph_deltas_capacity: u32,
    pub(super) morph_weights_capacity: u32,
    pub(super) virtual_geometry_pages_capacity: u32,
    pub(super) virtual_geometry_clusters_capacity: u32,
    pub(super) morph_payloads_require_full_upload: bool,
    pub(super) morph_deltas_require_full_upload: bool,
    pub(super) morph_weights_require_full_upload: bool,
    pub(super) virtual_geometry_pages_require_full_upload: bool,
    pub(super) virtual_geometry_clusters_require_full_upload: bool,
    pub(super) upload_transaction_owner: Arc<()>,
    pub(super) morph_preparation_reservation: Arc<std::sync::atomic::AtomicBool>,
    pub(super) virtual_geometry_preparation_reservation: Arc<std::sync::atomic::AtomicBool>,
    pub(super) force_full_primitive_upload: bool,
    pub(super) force_full_instance_upload: bool,
    pub(super) force_full_light_upload: bool,
    pub(super) uploaded_scene_data_counts: Option<[u32; 3]>,
}

impl GpuScene {
    pub(crate) fn new(
        device: &wgpu::Device,
        initial_skinned_joint_palette_arena_buffer: Arc<wgpu::Buffer>,
        skinned_joint_palette_min_binding_size: wgpu::BufferSize,
    ) -> Self {
        let primitive_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-primitive-data",
            u64::from(GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY) * GPU_PRIMITIVE_DATA_STRIDE as u64,
        );
        let instance_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-instance-data",
            u64::from(GPU_SCENE_INITIAL_INSTANCE_CAPACITY) * GPU_INSTANCE_DATA_STRIDE as u64,
        );
        let light_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-light-data",
            u64::from(GPU_SCENE_INITIAL_LIGHT_CAPACITY) * GpuLightData::STRIDE as u64,
        );
        let direct_visible_instance_remap_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-visible-instance-remap-fallback",
            GPU_SCENE_VISIBLE_INSTANCE_REMAP_FALLBACK_BYTES,
        );
        let direct_visible_instance_remap_params_buffer = create_remap_params_buffer(
            device,
            "zircon-gpu-scene-visible-instance-remap-direct-params",
            GpuSceneVisibleInstanceRemapParams::direct(),
        );
        let remapped_visible_instance_remap_params_buffer = create_remap_params_buffer(
            device,
            "zircon-gpu-scene-visible-instance-remap-enabled-params",
            GpuSceneVisibleInstanceRemapParams::remapped(),
        );
        let morph_deltas_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-morph-deltas-fallback",
            GPU_SCENE_MORPH_FALLBACK_BYTES,
        );
        let morph_weights_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-morph-weights-fallback",
            GPU_SCENE_MORPH_FALLBACK_BYTES,
        );
        let virtual_geometry_pages_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-virtual-geometry-pages-fallback",
            GPU_SCENE_VIRTUAL_GEOMETRY_FALLBACK_BYTES,
        );
        let virtual_geometry_clusters_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-virtual-geometry-clusters-fallback",
            GPU_SCENE_VIRTUAL_GEOMETRY_FALLBACK_BYTES,
        );
        let morph_payloads_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-morph-payloads-fallback",
            GPU_SCENE_MORPH_FALLBACK_BYTES,
        );
        let scene_bind_group_layout =
            create_gpu_scene_bind_group_layout(device, skinned_joint_palette_min_binding_size);
        let skinned_palette_arena = GpuSceneSkinnedPaletteArena::new(
            device,
            initial_skinned_joint_palette_arena_buffer,
            skinned_joint_palette_min_binding_size,
        );
        let scene_bind_groups = std::array::from_fn(|current_slot| {
            let (current_palette, previous_palette) =
                skinned_palette_arena.buffers_for_current_slot(current_slot);
            create_gpu_scene_bind_group(
                device,
                &scene_bind_group_layout,
                &primitive_buffer,
                &instance_buffer,
                &light_buffer,
                current_palette,
                previous_palette,
                &direct_visible_instance_remap_buffer,
                &direct_visible_instance_remap_params_buffer,
                &morph_deltas_buffer,
                &morph_weights_buffer,
                &virtual_geometry_pages_buffer,
                &virtual_geometry_clusters_buffer,
                &morph_payloads_buffer,
            )
        });
        Self {
            primitive_buffer,
            instance_buffer,
            light_buffer,
            skinned_palette_arena,
            direct_visible_instance_remap_buffer,
            direct_visible_instance_remap_params_buffer,
            remapped_visible_instance_remap_params_buffer,
            morph_deltas_buffer,
            morph_weights_buffer,
            virtual_geometry_pages_buffer,
            virtual_geometry_clusters_buffer,
            morph_payloads_buffer,
            scene_bind_group_layout,
            scene_bind_groups,
            primitive_shadow: Vec::new(),
            instance_shadow: Vec::new(),
            light_shadow: Vec::new(),
            morph_deltas_shadow: Vec::new(),
            morph_weights_shadow: Vec::new(),
            virtual_geometry_pages_shadow: Vec::new(),
            virtual_geometry_clusters_shadow: Vec::new(),
            morph_payloads_shadow: Vec::new(),
            primitive_ids: GpuSceneIdAllocator::new(),
            instance_ids: GpuSceneIdAllocator::new(),
            entries: HashMap::new(),
            pending_prev_transform_rolls: HashSet::new(),
            current_skinned_joint_palettes: HashMap::new(),
            previous_skinned_joint_palettes: HashMap::new(),
            current_skinned_gpu_sources: HashMap::new(),
            previous_skinned_gpu_sources: HashMap::new(),
            current_morph_weights: HashMap::new(),
            previous_morph_weights: HashMap::new(),
            updates: GpuSceneUpdateQueue::new(),
            staging_ring: GpuSceneStagingRing::default(),
            stats: GpuSceneStats {
                primitive_capacity: GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY,
                instance_capacity: GPU_SCENE_INITIAL_INSTANCE_CAPACITY,
                light_capacity: GPU_SCENE_INITIAL_LIGHT_CAPACITY,
                ..GpuSceneStats::default()
            },
            primitive_capacity: GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY,
            instance_capacity: GPU_SCENE_INITIAL_INSTANCE_CAPACITY,
            light_capacity: GPU_SCENE_INITIAL_LIGHT_CAPACITY,
            morph_payloads_capacity: 0,
            morph_deltas_capacity: 0,
            morph_weights_capacity: 0,
            virtual_geometry_pages_capacity: GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
            virtual_geometry_clusters_capacity: GPU_SCENE_INITIAL_VIRTUAL_GEOMETRY_CAPACITY,
            morph_payloads_require_full_upload: false,
            morph_deltas_require_full_upload: false,
            morph_weights_require_full_upload: false,
            virtual_geometry_pages_require_full_upload: false,
            virtual_geometry_clusters_require_full_upload: false,
            upload_transaction_owner: Arc::new(()),
            morph_preparation_reservation: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            virtual_geometry_preparation_reservation: Arc::new(std::sync::atomic::AtomicBool::new(
                false,
            )),
            force_full_primitive_upload: true,
            force_full_instance_upload: true,
            force_full_light_upload: true,
            uploaded_scene_data_counts: None,
        }
    }

    pub(crate) fn register(
        &mut self,
        device: &wgpu::Device,
        stable_instance_key: u64,
        instance_count: u32,
    ) -> GpuSceneEntry {
        assert!(
            instance_count > 0,
            "gpu scene entries must have at least one instance"
        );

        if let Some(existing) = self.entries.get(&stable_instance_key).copied() {
            if existing.instance_count == instance_count {
                return existing;
            }
            self.unregister(stable_instance_key);
        }

        let primitive_index = self.primitive_ids.allocate();
        let instance_span = self.instance_ids.allocate_span(instance_count);
        self.ensure_capacity(device);
        self.ensure_shadow_len(
            self.primitive_ids.high_water(),
            self.instance_ids.high_water(),
        );

        let entry = GpuSceneEntry {
            stable_instance_key,
            primitive_index,
            first_instance_index: instance_span.start,
            instance_count,
            last_transform_revision: 0,
            has_rolled_previous_transform: false,
        };
        self.primitive_shadow[primitive_index as usize] =
            GpuPrimitiveData::with_instance_span(entry.first_instance_index, entry.instance_count);
        for instance_index in instance_span.start..instance_span.end_exclusive() {
            self.instance_shadow[instance_index as usize] =
                GpuInstanceData::for_primitive(primitive_index);
        }
        self.updates.mark_primitive(entry.primitive_index);
        self.updates
            .mark_instances(entry.first_instance_index, entry.instance_count);
        self.entries.insert(stable_instance_key, entry);
        self.pending_prev_transform_rolls
            .insert(stable_instance_key);
        self.refresh_stats(0, self.updates.dirty_entry_count());
        entry
    }

    pub(crate) fn unregister(&mut self, stable_instance_key: u64) -> Option<GpuSceneEntry> {
        let entry = self.entries.remove(&stable_instance_key)?;
        self.pending_prev_transform_rolls
            .remove(&stable_instance_key);
        self.current_skinned_joint_palettes
            .remove(&stable_instance_key);
        self.previous_skinned_joint_palettes
            .remove(&stable_instance_key);
        self.current_skinned_gpu_sources
            .remove(&stable_instance_key);
        self.previous_skinned_gpu_sources
            .remove(&stable_instance_key);
        self.current_morph_weights.remove(&stable_instance_key);
        self.previous_morph_weights.remove(&stable_instance_key);
        self.primitive_ids.free(entry.primitive_index);
        self.instance_ids
            .free_span(entry.first_instance_index, entry.instance_count);
        self.refresh_stats(0, self.updates.dirty_entry_count());
        Some(entry)
    }

    pub(crate) fn retain_registered_keys(&mut self, live_keys: &HashSet<u64>) {
        let stale_keys = self
            .entries
            .keys()
            .copied()
            .filter(|key| !live_keys.contains(key))
            .collect::<Vec<_>>();
        for key in stale_keys {
            self.unregister(key);
        }
        self.current_skinned_joint_palettes
            .retain(|key, _| live_keys.contains(key));
        self.previous_skinned_joint_palettes
            .retain(|key, _| live_keys.contains(key));
        self.current_skinned_gpu_sources
            .retain(|key, _| live_keys.contains(key));
        self.previous_skinned_gpu_sources
            .retain(|key, _| live_keys.contains(key));
        self.current_morph_weights
            .retain(|key, _| live_keys.contains(key));
        self.previous_morph_weights
            .retain(|key, _| live_keys.contains(key));
    }

    pub(crate) fn set_transform_revision(
        &mut self,
        stable_instance_key: u64,
        transform_revision: u64,
    ) {
        if let Some(entry) = self.entries.get_mut(&stable_instance_key) {
            entry.last_transform_revision = transform_revision;
        }
    }

    pub(crate) fn write_primitive(&mut self, entry: GpuSceneEntry, mut data: GpuPrimitiveData) {
        data.first_instance_index = entry.first_instance_index;
        data.instance_count = entry.instance_count;
        let primitive_index = entry.primitive_index as usize;
        if self.primitive_shadow[primitive_index] != data {
            self.primitive_shadow[primitive_index] = data;
            self.updates.mark_primitive(entry.primitive_index);
            self.refresh_stats(0, self.updates.dirty_entry_count());
        }
    }

    pub(crate) fn write_instances(&mut self, entry: GpuSceneEntry, data: &[GpuInstanceData]) {
        assert_eq!(
            data.len() as u32,
            entry.instance_count,
            "gpu scene writes currently replace the whole instance span"
        );
        let start = entry.first_instance_index as usize;
        let mut changed = false;
        let mut transform_changed = false;
        for (offset, instance) in data.iter().copied().enumerate() {
            let mut instance = instance;
            instance.primitive_index = entry.primitive_index;
            let previous = &self.instance_shadow[start + offset];
            changed |= *previous != instance;
            transform_changed |= previous.world_from_local != instance.world_from_local;
        }
        if !changed {
            return;
        }

        for (offset, instance) in data.iter().copied().enumerate() {
            let mut instance = instance;
            instance.primitive_index = entry.primitive_index;
            self.instance_shadow[start + offset] = instance;
        }
        self.updates
            .mark_instances(entry.first_instance_index, entry.instance_count);
        if transform_changed {
            self.pending_prev_transform_rolls
                .insert(entry.stable_instance_key);
        }
        self.refresh_stats(0, self.updates.dirty_entry_count());
    }

    /// Reuses instance-derived metadata while the renderer's current affine
    /// transform still matches the shadow row. Multi-instance spans have no
    /// single transform classification and deliberately fall back to recompute.
    pub(crate) fn instance_flags_for_world_from_local(
        &self,
        entry: GpuSceneEntry,
        world_from_local: &[[f32; 4]; 4],
    ) -> Option<u32> {
        if entry.instance_count != 1 {
            return None;
        }
        self.instance_shadow
            .get(entry.first_instance_index as usize)
            .filter(|instance| &instance.world_from_local == world_from_local)
            .map(|instance| instance.flags)
    }

    pub(crate) fn write_lights(&mut self, device: &wgpu::Device, lights: &[GpuLightData]) {
        if self.light_shadow == lights {
            self.refresh_stats(0, self.updates.dirty_entry_count());
            return;
        }

        let light_count = u32::try_from(lights.len()).expect("gpu scene light count exceeded u32");
        self.ensure_light_capacity(device, light_count);
        if !self.force_full_light_upload {
            for (index, light) in lights.iter().enumerate() {
                if self.light_shadow.get(index) != Some(light) {
                    self.updates.mark_light(
                        u32::try_from(index).expect("gpu scene light index exceeded u32"),
                    );
                }
            }
        }
        self.light_shadow.clear();
        self.light_shadow.extend_from_slice(lights);
        self.refresh_stats(0, self.updates.dirty_entry_count());
    }

    pub(crate) fn primitive_buffer(&self) -> &wgpu::Buffer {
        &self.primitive_buffer
    }

    pub(crate) fn instance_buffer(&self) -> &wgpu::Buffer {
        &self.instance_buffer
    }

    pub(crate) fn light_buffer(&self) -> &wgpu::Buffer {
        &self.light_buffer
    }

    pub(crate) fn scene_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.scene_bind_group_layout
    }

    pub(crate) fn scene_bind_group(&self) -> &wgpu::BindGroup {
        &self.scene_bind_groups[self.skinned_palette_arena.staged_slot()]
    }

    pub(crate) fn create_scene_bind_group_for_visible_instance_remap(
        &self,
        device: &wgpu::Device,
        visible_instance_remap_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        create_gpu_scene_bind_group(
            device,
            &self.scene_bind_group_layout,
            &self.primitive_buffer,
            &self.instance_buffer,
            &self.light_buffer,
            self.skinned_palette_arena.current_buffer(),
            self.skinned_palette_arena.previous_buffer(),
            visible_instance_remap_buffer,
            &self.remapped_visible_instance_remap_params_buffer,
            &self.morph_deltas_buffer,
            &self.morph_weights_buffer,
            &self.virtual_geometry_pages_buffer,
            &self.virtual_geometry_clusters_buffer,
            &self.morph_payloads_buffer,
        )
    }

    pub(crate) fn entry(&self, stable_instance_key: u64) -> Option<GpuSceneEntry> {
        self.entries.get(&stable_instance_key).copied()
    }

    pub(crate) fn stats(&self) -> GpuSceneStats {
        self.stats
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device) {
        let mut rebuild_scene_bind_group = false;
        let required_primitive_capacity = self.primitive_ids.high_water();
        if required_primitive_capacity > self.primitive_capacity {
            self.primitive_capacity = grow_capacity(
                required_primitive_capacity,
                GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY,
            );
            self.primitive_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-primitive-data",
                u64::from(self.primitive_capacity) * GPU_PRIMITIVE_DATA_STRIDE as u64,
            );
            self.force_full_primitive_upload = true;
            rebuild_scene_bind_group = true;
        }

        let required_instance_capacity = self.instance_ids.high_water();
        if required_instance_capacity > self.instance_capacity {
            self.instance_capacity = grow_capacity(
                required_instance_capacity,
                GPU_SCENE_INITIAL_INSTANCE_CAPACITY,
            );
            self.instance_buffer = create_storage_buffer(
                device,
                "zircon-gpu-scene-instance-data",
                u64::from(self.instance_capacity) * GPU_INSTANCE_DATA_STRIDE as u64,
            );
            self.force_full_instance_upload = true;
            rebuild_scene_bind_group = true;
        }

        if rebuild_scene_bind_group {
            self.rebuild_scene_bind_group(device);
        }
    }

    fn ensure_light_capacity(&mut self, device: &wgpu::Device, required_light_count: u32) {
        let required_light_capacity = required_light_count.max(1);
        if required_light_capacity <= self.light_capacity {
            return;
        }

        self.light_capacity =
            grow_capacity(required_light_capacity, GPU_SCENE_INITIAL_LIGHT_CAPACITY);
        self.light_buffer = create_storage_buffer(
            device,
            "zircon-gpu-scene-light-data",
            u64::from(self.light_capacity) * GpuLightData::STRIDE as u64,
        );
        self.force_full_light_upload = true;
        self.rebuild_scene_bind_group(device);
    }

    pub(super) fn rebuild_scene_bind_group(&mut self, device: &wgpu::Device) {
        self.scene_bind_groups = std::array::from_fn(|current_slot| {
            let (current_palette, previous_palette) = self
                .skinned_palette_arena
                .buffers_for_current_slot(current_slot);
            create_gpu_scene_bind_group(
                device,
                &self.scene_bind_group_layout,
                &self.primitive_buffer,
                &self.instance_buffer,
                &self.light_buffer,
                current_palette,
                previous_palette,
                &self.direct_visible_instance_remap_buffer,
                &self.direct_visible_instance_remap_params_buffer,
                &self.morph_deltas_buffer,
                &self.morph_weights_buffer,
                &self.virtual_geometry_pages_buffer,
                &self.virtual_geometry_clusters_buffer,
                &self.morph_payloads_buffer,
            )
        });
    }

    fn ensure_shadow_len(&mut self, primitive_high_water: u32, instance_high_water: u32) {
        let primitive_len = primitive_high_water as usize;
        if self.primitive_shadow.len() < primitive_len {
            self.primitive_shadow
                .resize(primitive_len, GpuPrimitiveData::default());
        }

        let instance_len = instance_high_water as usize;
        if self.instance_shadow.len() < instance_len {
            self.instance_shadow
                .resize(instance_len, GpuInstanceData::default());
        }
    }

    pub(super) fn refresh_stats(&mut self, uploaded_bytes: u64, dirty_entry_count: usize) {
        self.stats = GpuSceneStats {
            primitive_count: self.primitive_ids.live(),
            instance_count: self.instance_ids.live(),
            light_count: self.light_shadow.len() as u32,
            dirty_entry_count,
            uploaded_bytes,
            primitive_capacity: self.primitive_capacity,
            instance_capacity: self.instance_capacity,
            light_capacity: self.light_capacity,
            free_span_count: self.primitive_ids.free_span_count()
                + self.instance_ids.free_span_count(),
        };
    }
}

pub(super) fn create_storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
    size: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    })
}

fn create_remap_params_buffer(
    device: &wgpu::Device,
    label: &'static str,
    params: GpuSceneVisibleInstanceRemapParams,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub(super) fn grow_capacity(required: u32, minimum: u32) -> u32 {
    let mut capacity = minimum.max(1);
    while capacity < required {
        capacity = capacity
            .checked_mul(2)
            .expect("gpu scene buffer capacity overflowed u32");
    }
    capacity
}

#[cfg(test)]
mod tests;
