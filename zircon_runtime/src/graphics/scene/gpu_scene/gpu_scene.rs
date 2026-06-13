use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::framework::render::GpuLightData;
use wgpu::util::DeviceExt;

use super::binding::{
    create_gpu_scene_bind_group, create_gpu_scene_bind_group_layout,
    GpuSceneVisibleInstanceRemapParams,
};
use super::id_allocator::GpuSceneIdAllocator;
use super::layout::{
    GpuInstanceData, GpuPrimitiveData, GPU_INSTANCE_DATA_STRIDE, GPU_PRIMITIVE_DATA_STRIDE,
};
use super::update_queue::GpuSceneUpdateQueue;
use super::upload::{write_full_pod_buffer, write_upload_ranges};

pub(crate) const GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY: u32 = 64;
pub(crate) const GPU_SCENE_INITIAL_INSTANCE_CAPACITY: u32 = 64;
pub(crate) const GPU_SCENE_INITIAL_LIGHT_CAPACITY: u32 = 1;

const GPU_SCENE_VISIBLE_INSTANCE_REMAP_FALLBACK_BYTES: u64 = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GpuSceneEntry {
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
}

impl GpuSceneUploadPath {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::DirectQueueWrite => "direct_queue_write",
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

/// Owns the GPUScene storage buffers and CPU mirrors before frame-path wiring.
///
/// Registration keeps stable primitive/instance indices until explicit
/// unregister. Upload uses direct queue writes for merged dirty ranges; callers
/// can feed full-frame extracts every frame without re-uploading unchanged
/// entries.
pub(crate) struct GpuScene {
    primitive_buffer: wgpu::Buffer,
    pub(super) instance_buffer: wgpu::Buffer,
    light_buffer: wgpu::Buffer,
    /// Non-skinned draws share fallback palette slots until GS-M2 creates
    /// per-skinned-draw object groups with real current/previous palettes.
    fallback_skinned_joint_palette_buffer: Arc<wgpu::Buffer>,
    direct_visible_instance_remap_buffer: wgpu::Buffer,
    direct_visible_instance_remap_params_buffer: wgpu::Buffer,
    remapped_visible_instance_remap_params_buffer: wgpu::Buffer,
    scene_bind_group_layout: wgpu::BindGroupLayout,
    scene_bind_group: wgpu::BindGroup,
    primitive_shadow: Vec<GpuPrimitiveData>,
    pub(super) instance_shadow: Vec<GpuInstanceData>,
    light_shadow: Vec<GpuLightData>,
    primitive_ids: GpuSceneIdAllocator,
    instance_ids: GpuSceneIdAllocator,
    pub(super) entries: HashMap<u64, GpuSceneEntry>,
    pub(super) updates: GpuSceneUpdateQueue,
    stats: GpuSceneStats,
    primitive_capacity: u32,
    instance_capacity: u32,
    light_capacity: u32,
    force_full_primitive_upload: bool,
    force_full_instance_upload: bool,
    force_full_light_upload: bool,
    uploaded_light_count_params: Option<u32>,
}

impl GpuScene {
    pub(crate) fn new(
        device: &wgpu::Device,
        fallback_skinned_joint_palette_buffer: Arc<wgpu::Buffer>,
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
        let scene_bind_group_layout =
            create_gpu_scene_bind_group_layout(device, skinned_joint_palette_min_binding_size);
        let scene_bind_group = create_gpu_scene_bind_group(
            device,
            &scene_bind_group_layout,
            &primitive_buffer,
            &instance_buffer,
            &light_buffer,
            &fallback_skinned_joint_palette_buffer,
            &fallback_skinned_joint_palette_buffer,
            &direct_visible_instance_remap_buffer,
            &direct_visible_instance_remap_params_buffer,
        );
        Self {
            primitive_buffer,
            instance_buffer,
            light_buffer,
            fallback_skinned_joint_palette_buffer,
            direct_visible_instance_remap_buffer,
            direct_visible_instance_remap_params_buffer,
            remapped_visible_instance_remap_params_buffer,
            scene_bind_group_layout,
            scene_bind_group,
            primitive_shadow: Vec::new(),
            instance_shadow: Vec::new(),
            light_shadow: Vec::new(),
            primitive_ids: GpuSceneIdAllocator::new(),
            instance_ids: GpuSceneIdAllocator::new(),
            entries: HashMap::new(),
            updates: GpuSceneUpdateQueue::new(),
            stats: GpuSceneStats {
                primitive_capacity: GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY,
                instance_capacity: GPU_SCENE_INITIAL_INSTANCE_CAPACITY,
                light_capacity: GPU_SCENE_INITIAL_LIGHT_CAPACITY,
                ..GpuSceneStats::default()
            },
            primitive_capacity: GPU_SCENE_INITIAL_PRIMITIVE_CAPACITY,
            instance_capacity: GPU_SCENE_INITIAL_INSTANCE_CAPACITY,
            light_capacity: GPU_SCENE_INITIAL_LIGHT_CAPACITY,
            force_full_primitive_upload: true,
            force_full_instance_upload: true,
            force_full_light_upload: true,
            uploaded_light_count_params: None,
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
        self.refresh_stats(0, self.updates.dirty_entry_count());
        entry
    }

    pub(crate) fn unregister(&mut self, stable_instance_key: u64) -> Option<GpuSceneEntry> {
        let entry = self.entries.remove(&stable_instance_key)?;
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
        let changed = data.iter().copied().enumerate().any(|(offset, instance)| {
            let mut instance = instance;
            instance.primitive_index = entry.primitive_index;
            self.instance_shadow[start + offset] != instance
        });
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
        self.refresh_stats(0, self.updates.dirty_entry_count());
    }

    pub(crate) fn write_lights(&mut self, device: &wgpu::Device, lights: &[GpuLightData]) {
        self.ensure_light_capacity(device, lights.len() as u32);
        if self.light_shadow == lights {
            self.refresh_stats(0, self.updates.dirty_entry_count());
            return;
        }

        self.light_shadow.clear();
        self.light_shadow.extend_from_slice(lights);
        self.force_full_light_upload = true;
        self.refresh_stats(0, self.updates.dirty_entry_count());
    }

    pub(crate) fn flush_updates(&mut self, queue: &wgpu::Queue) -> GpuSceneUploadReport {
        let dirty_entry_count = self.updates.dirty_entry_count();
        let mut report = GpuSceneUploadReport::default();

        if self.force_full_primitive_upload {
            let active_len = self.primitive_ids.high_water() as usize;
            let uploaded = write_full_pod_buffer(
                queue,
                &self.primitive_buffer,
                &self.primitive_shadow,
                active_len,
            );
            if uploaded > 0 {
                report.primitive_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
            let _ = self
                .updates
                .drain_primitive_upload_ranges(GPU_PRIMITIVE_DATA_STRIDE as u64);
        } else {
            let ranges = self
                .updates
                .drain_primitive_upload_ranges(GPU_PRIMITIVE_DATA_STRIDE as u64);
            report.primitive_upload_range_count = ranges.len();
            report.uploaded_bytes += write_upload_ranges(
                queue,
                &self.primitive_buffer,
                &self.primitive_shadow,
                &ranges,
            );
        }

        if self.force_full_instance_upload {
            let active_len = self.instance_ids.high_water() as usize;
            let uploaded = write_full_pod_buffer(
                queue,
                &self.instance_buffer,
                &self.instance_shadow,
                active_len,
            );
            if uploaded > 0 {
                report.instance_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
            let _ = self
                .updates
                .drain_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64);
        } else {
            let ranges = self
                .updates
                .drain_instance_upload_ranges(GPU_INSTANCE_DATA_STRIDE as u64);
            report.instance_upload_range_count = ranges.len();
            report.uploaded_bytes +=
                write_upload_ranges(queue, &self.instance_buffer, &self.instance_shadow, &ranges);
        }

        if self.force_full_light_upload {
            let uploaded = write_full_pod_buffer(
                queue,
                &self.light_buffer,
                &self.light_shadow,
                self.light_shadow.len(),
            );
            if uploaded > 0 {
                report.light_upload_range_count = 1;
                report.uploaded_bytes += uploaded;
            }
        }
        self.write_light_count_params_if_needed(queue);

        self.force_full_primitive_upload = false;
        self.force_full_instance_upload = false;
        self.force_full_light_upload = false;
        self.primitive_ids.commit_pending_frees();
        self.instance_ids.commit_pending_frees();
        self.refresh_stats(report.uploaded_bytes, dirty_entry_count);
        report
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
        &self.scene_bind_group
    }

    pub(crate) fn create_scene_bind_group_for_palettes(
        &self,
        device: &wgpu::Device,
        skinned_joint_palette_buffer: Option<&wgpu::Buffer>,
        previous_skinned_joint_palette_buffer: Option<&wgpu::Buffer>,
    ) -> wgpu::BindGroup {
        create_gpu_scene_bind_group(
            device,
            &self.scene_bind_group_layout,
            &self.primitive_buffer,
            &self.instance_buffer,
            &self.light_buffer,
            skinned_joint_palette_buffer.unwrap_or(&self.fallback_skinned_joint_palette_buffer),
            previous_skinned_joint_palette_buffer
                .unwrap_or(&self.fallback_skinned_joint_palette_buffer),
            &self.direct_visible_instance_remap_buffer,
            &self.direct_visible_instance_remap_params_buffer,
        )
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
            &self.fallback_skinned_joint_palette_buffer,
            &self.fallback_skinned_joint_palette_buffer,
            visible_instance_remap_buffer,
            &self.remapped_visible_instance_remap_params_buffer,
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

    fn rebuild_scene_bind_group(&mut self, device: &wgpu::Device) {
        self.scene_bind_group = create_gpu_scene_bind_group(
            device,
            &self.scene_bind_group_layout,
            &self.primitive_buffer,
            &self.instance_buffer,
            &self.light_buffer,
            &self.fallback_skinned_joint_palette_buffer,
            &self.fallback_skinned_joint_palette_buffer,
            &self.direct_visible_instance_remap_buffer,
            &self.direct_visible_instance_remap_params_buffer,
        );
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

    fn write_light_count_params_if_needed(&mut self, queue: &wgpu::Queue) {
        let light_count = self.light_shadow.len() as u32;
        if self.uploaded_light_count_params == Some(light_count) {
            return;
        }

        queue.write_buffer(
            &self.direct_visible_instance_remap_params_buffer,
            0,
            bytemuck::bytes_of(
                &GpuSceneVisibleInstanceRemapParams::direct_with_light_count(light_count),
            ),
        );
        queue.write_buffer(
            &self.remapped_visible_instance_remap_params_buffer,
            0,
            bytemuck::bytes_of(
                &GpuSceneVisibleInstanceRemapParams::remapped_with_light_count(light_count),
            ),
        );
        self.uploaded_light_count_params = Some(light_count);
    }

    fn refresh_stats(&mut self, uploaded_bytes: u64, dirty_entry_count: usize) {
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

fn create_storage_buffer(device: &wgpu::Device, label: &'static str, size: u64) -> wgpu::Buffer {
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

fn grow_capacity(required: u32, minimum: u32) -> u32 {
    let mut capacity = minimum.max(1);
    while capacity < required {
        capacity = capacity
            .checked_mul(2)
            .expect("gpu scene buffer capacity overflowed u32");
    }
    capacity
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::graphics::scene::gpu_scene::{
        GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
    };

    const TEST_STABLE_INSTANCE_KEY: u64 = 0x1000_0001;
    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn render_gpu_scene_static_scene_second_frame_uploads_zero_bytes() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        sync_test_entry(&backend.device, &mut scene, 1, 0.0);
        let first_report = scene.flush_updates(&backend.queue);
        assert!(first_report.uploaded_bytes > 0);

        sync_test_entry(&backend.device, &mut scene, 1, 0.0);
        let second_report = scene.flush_updates(&backend.queue);

        assert_eq!(
            second_report.upload_path,
            GpuSceneUploadPath::DirectQueueWrite
        );
        assert_eq!(second_report.upload_path.label(), "direct_queue_write");
        assert_eq!(second_report.uploaded_bytes, 0);
        assert_eq!(second_report.primitive_upload_range_count, 0);
        assert_eq!(second_report.instance_upload_range_count, 0);
        assert_eq!(scene.stats().dirty_entry_count, 0);
    }

    #[test]
    fn render_gpu_scene_single_moving_entity_uploads_only_its_entry() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);

        sync_test_entry(&backend.device, &mut scene, 1, 0.0);
        let _ = scene.flush_updates(&backend.queue);

        sync_test_entry(&backend.device, &mut scene, 2, 5.0);
        let moving_report = scene.flush_updates(&backend.queue);

        assert_eq!(
            moving_report.upload_path,
            GpuSceneUploadPath::DirectQueueWrite
        );
        assert_eq!(
            moving_report.uploaded_bytes,
            GPU_INSTANCE_DATA_STRIDE as u64
        );
        assert_eq!(moving_report.primitive_upload_range_count, 0);
        assert_eq!(moving_report.instance_upload_range_count, 1);
    }

    #[test]
    fn render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads() {
        let Some(backend) = test_backend() else {
            return;
        };
        let mut scene = test_gpu_scene(&backend.device);
        let lights = vec![test_light_data(1), test_light_data(2), test_light_data(3)];

        scene.write_lights(&backend.device, &lights);
        let first_report = scene.flush_updates(&backend.queue);

        assert_eq!(scene.stats().light_count, 3);
        assert!(scene.stats().light_capacity >= 4);
        assert_eq!(first_report.light_upload_range_count, 1);
        assert_eq!(
            first_report.uploaded_bytes,
            (lights.len() * GpuLightData::STRIDE) as u64
        );

        scene.write_lights(&backend.device, &lights);
        let unchanged_report = scene.flush_updates(&backend.queue);

        assert_eq!(unchanged_report.uploaded_bytes, 0);
        assert_eq!(unchanged_report.light_upload_range_count, 0);
    }

    fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
        crate::graphics::backend::RenderBackend::new_offscreen()
            .inspect_err(|error| eprintln!("skipping gpu scene upload test: {error:?}"))
            .ok()
    }

    fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-empty-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }

    fn sync_test_entry(
        device: &wgpu::Device,
        scene: &mut GpuScene,
        transform_revision: u64,
        translate_x: f32,
    ) {
        let entry = scene.register(device, TEST_STABLE_INSTANCE_KEY, 1);
        scene.write_primitive(entry, test_primitive_data());
        scene.write_instances(entry, &[test_instance_data(translate_x)]);
        scene.set_transform_revision(TEST_STABLE_INSTANCE_KEY, transform_revision);
    }

    fn test_primitive_data() -> GpuPrimitiveData {
        GpuPrimitiveData {
            bounds_center: [0.0, 0.0, 0.0],
            bounds_radius: 1.0,
            tint: [1.0, 1.0, 1.0, 1.0],
            shadow_params: [0.0, 0.5, 1.0, 0.0],
            motion_params: [0.0, 0.0, 0.0, 0.0],
            flags: GPU_PRIMITIVE_FLAG_VISIBLE,
            first_instance_index: u32::MAX,
            instance_count: u32::MAX,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        }
    }

    fn test_instance_data(translate_x: f32) -> GpuInstanceData {
        let mut world_from_local = test_identity_matrix();
        world_from_local[3][0] = translate_x;
        GpuInstanceData {
            world_from_local,
            prev_world_from_local: test_identity_matrix(),
            primitive_index: u32::MAX,
            flags: 0,
            payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
            _pad0: 0,
        }
    }

    fn test_light_data(light_id: u32) -> GpuLightData {
        GpuLightData {
            color_intensity: [1.0, 0.5, 0.25, 2.0],
            shadow_slot_layer: [u32::MAX, 1, light_id, 0],
            ..GpuLightData::default()
        }
    }

    fn test_identity_matrix() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}
