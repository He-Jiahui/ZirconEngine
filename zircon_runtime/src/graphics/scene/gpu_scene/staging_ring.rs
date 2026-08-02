use bytemuck::Pod;

pub(crate) const GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES: u64 = 256 * 1024;

const GPU_SCENE_STAGING_RING_SLOT_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GpuSceneStagingDestination {
    Primitive,
    Instance,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GpuSceneStagedCopy {
    destination: GpuSceneStagingDestination,
    source_offset: u64,
    destination_offset: u64,
    byte_len: u64,
}

/// Reuses three COPY_SRC buffers and CPU scratch across large scene uploads.
///
/// All copies in one frame reference a single immutable staging blob, so a
/// later queue write cannot overwrite bytes needed by an earlier encoded copy.
pub(super) struct GpuSceneStagingRing {
    slots: Vec<wgpu::Buffer>,
    capacity_bytes: u64,
    next_slot: usize,
    bytes: Vec<u8>,
    copies: Vec<GpuSceneStagedCopy>,
}

impl Default for GpuSceneStagingRing {
    fn default() -> Self {
        Self {
            slots: Vec::with_capacity(GPU_SCENE_STAGING_RING_SLOT_COUNT),
            capacity_bytes: 0,
            next_slot: 0,
            bytes: Vec::new(),
            copies: Vec::new(),
        }
    }
}

impl GpuSceneStagingRing {
    pub(super) const fn should_stage(total_upload_bytes: u64) -> bool {
        total_upload_bytes >= GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES
    }

    pub(super) fn begin_frame(&mut self) {
        self.bytes.clear();
        self.copies.clear();
    }

    pub(super) fn stage_pod_slice<T: Pod>(
        &mut self,
        destination: GpuSceneStagingDestination,
        destination_offset: u64,
        values: &[T],
    ) -> u64 {
        let bytes = bytemuck::cast_slice(values);
        if bytes.is_empty() {
            return 0;
        }

        let byte_len = u64::try_from(bytes.len()).expect("gpu scene staging upload exceeds u64");
        assert_eq!(
            byte_len % wgpu::COPY_BUFFER_ALIGNMENT as u64,
            0,
            "gpu scene staging copy size must satisfy WGPU buffer-copy alignment"
        );
        let source_offset =
            u64::try_from(self.bytes.len()).expect("gpu scene staging source offset exceeds u64");
        assert_eq!(
            source_offset % wgpu::COPY_BUFFER_ALIGNMENT as u64,
            0,
            "gpu scene staging copy offset must satisfy WGPU buffer-copy alignment"
        );
        self.bytes.extend_from_slice(bytes);
        self.copies.push(GpuSceneStagedCopy {
            destination,
            source_offset,
            destination_offset,
            byte_len,
        });
        byte_len
    }

    pub(super) fn submit(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        primitive_buffer: &wgpu::Buffer,
        instance_buffer: &wgpu::Buffer,
        light_buffer: &wgpu::Buffer,
    ) {
        if self.copies.is_empty() {
            return;
        }

        let required_bytes = u64::try_from(self.bytes.len())
            .expect("gpu scene staging upload byte length exceeds u64");
        self.ensure_capacity(device, required_bytes);
        let staging_buffer = &self.slots[self.next_slot];
        queue.write_buffer(staging_buffer, 0, &self.bytes);
        for copy in &self.copies {
            let destination = match copy.destination {
                GpuSceneStagingDestination::Primitive => primitive_buffer,
                GpuSceneStagingDestination::Instance => instance_buffer,
                GpuSceneStagingDestination::Light => light_buffer,
            };
            encoder.copy_buffer_to_buffer(
                staging_buffer,
                copy.source_offset,
                destination,
                copy.destination_offset,
                copy.byte_len,
            );
        }
        self.next_slot = (self.next_slot + 1) % GPU_SCENE_STAGING_RING_SLOT_COUNT;
    }

    fn ensure_capacity(&mut self, device: &wgpu::Device, required_bytes: u64) {
        if required_bytes <= self.capacity_bytes {
            return;
        }

        self.capacity_bytes = GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES;
        while self.capacity_bytes < required_bytes {
            self.capacity_bytes = self
                .capacity_bytes
                .checked_mul(2)
                .expect("gpu scene staging capacity overflowed u64");
        }
        self.slots.clear();
        for _ in 0..GPU_SCENE_STAGING_RING_SLOT_COUNT {
            self.slots
                .push(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("zircon-gpu-scene-upload-staging"),
                    size: self.capacity_bytes,
                    usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        self.next_slot = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GpuSceneStagingDestination, GpuSceneStagingRing, GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES,
    };

    #[test]
    fn render_gpu_scene_staging_ring_selects_only_large_uploads() {
        assert!(!GpuSceneStagingRing::should_stage(
            GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES - 1
        ));
        assert!(GpuSceneStagingRing::should_stage(
            GPU_SCENE_STAGING_COPY_THRESHOLD_BYTES
        ));
    }

    #[test]
    fn render_gpu_scene_staging_ring_keeps_copy_offsets_in_one_frame_blob() {
        let mut ring = GpuSceneStagingRing::default();
        ring.begin_frame();
        ring.stage_pod_slice(GpuSceneStagingDestination::Primitive, 32, &[1u32; 4]);
        ring.stage_pod_slice(GpuSceneStagingDestination::Instance, 64, &[2u32; 4]);

        assert_eq!(ring.copies.len(), 2);
        assert_eq!(ring.copies[0].source_offset, 0);
        assert_eq!(ring.copies[1].source_offset, 16);
        assert_eq!(ring.bytes.len(), 32);
    }
}
