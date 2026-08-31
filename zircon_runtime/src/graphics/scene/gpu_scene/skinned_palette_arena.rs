use std::collections::HashMap;
use std::sync::Arc;

use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use crate::graphics::scene::scene_renderer::SkinnedMeshJointPaletteStorage;

const SKINNED_PALETTE_MATRIX_BYTES: u64 = std::mem::size_of::<[[f32; 4]; 4]>() as u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct SkinnedPaletteSpan {
    matrix_base: u32,
    joint_count: u32,
}

impl SkinnedPaletteSpan {
    const fn params(self, previous: Option<Self>) -> [u32; 4] {
        let previous = match previous {
            Some(previous) => previous,
            None => Self {
                matrix_base: 0,
                joint_count: 0,
            },
        };
        [
            self.matrix_base,
            self.joint_count,
            previous.matrix_base,
            previous.joint_count,
        ]
    }
}

pub(super) struct GpuSceneSkinnedPaletteArena {
    buffers: [Arc<wgpu::Buffer>; 2],
    capacity_bytes: [u64; 2],
    minimum_binding_bytes: u64,
    committed_slot: Option<usize>,
    staged_slot: usize,
    committed_spans: HashMap<u64, SkinnedPaletteSpan>,
    staged_spans: HashMap<u64, SkinnedPaletteSpan>,
    staged_matrices: Vec<[[f32; 4]; 4]>,
}

pub(super) struct PreparedSkinnedPaletteArenaUpload {
    pub(super) batch: WgpuBufferUploadBatch,
    pub(super) uploaded_bytes: u64,
    pub(super) buffer_recreated: bool,
}

impl GpuSceneSkinnedPaletteArena {
    pub(super) fn new(
        device: &wgpu::Device,
        first_buffer: Arc<wgpu::Buffer>,
        minimum_binding_size: wgpu::BufferSize,
    ) -> Self {
        let minimum_binding_bytes = minimum_binding_size.get();
        let second_buffer = create_palette_buffer(
            device,
            minimum_binding_bytes,
            "zircon-gpu-scene-skinned-palette-arena-1",
        );
        Self {
            buffers: [first_buffer, second_buffer],
            capacity_bytes: [minimum_binding_bytes; 2],
            minimum_binding_bytes,
            committed_slot: None,
            staged_slot: 0,
            committed_spans: HashMap::new(),
            staged_spans: HashMap::new(),
            staged_matrices: Vec::new(),
        }
    }

    pub(super) fn begin_frame(&mut self) {
        self.staged_slot = self.committed_slot.map(|slot| 1 - slot).unwrap_or(0);
        self.staged_spans.clear();
        self.staged_matrices.clear();
    }

    pub(super) fn stage_palette(
        &mut self,
        stable_instance_key: u64,
        storage: Option<&SkinnedMeshJointPaletteStorage>,
        expose_current: bool,
        expose_previous: bool,
    ) -> [u32; 4] {
        let Some(storage) = storage else {
            return [0; 4];
        };
        let current = stage_palette_span(
            &mut self.staged_spans,
            &mut self.staged_matrices,
            stable_instance_key,
            storage,
        );

        if !expose_current {
            return [0; 4];
        }
        let previous = expose_previous
            .then(|| self.committed_spans.get(&stable_instance_key).copied())
            .flatten();
        current.params(previous)
    }

    pub(super) fn prepare_upload(
        &mut self,
        device: &wgpu::Device,
    ) -> PreparedSkinnedPaletteArenaUpload {
        let required_bytes = u64::try_from(self.staged_matrices.len())
            .expect("skinned palette matrix count did not fit u64")
            .checked_mul(SKINNED_PALETTE_MATRIX_BYTES)
            .expect("skinned palette upload byte count overflowed u64");
        let required_capacity = required_bytes.max(self.minimum_binding_bytes);
        let buffer_recreated = self.capacity_bytes[self.staged_slot] < required_capacity;
        if buffer_recreated {
            self.capacity_bytes[self.staged_slot] = grow_palette_capacity(required_capacity);
            self.buffers[self.staged_slot] = create_palette_buffer(
                device,
                self.capacity_bytes[self.staged_slot],
                if self.staged_slot == 0 {
                    "zircon-gpu-scene-skinned-palette-arena-0"
                } else {
                    "zircon-gpu-scene-skinned-palette-arena-1"
                },
            );
        }

        let mut batch = WgpuBufferUploadBatch::new();
        if required_bytes > 0 {
            batch.push(WgpuBufferUpload::from_bytes(
                self.buffers[self.staged_slot].as_ref().clone(),
                0,
                bytemuck::cast_slice(&self.staged_matrices),
            ));
        }
        PreparedSkinnedPaletteArenaUpload {
            batch,
            uploaded_bytes: required_bytes,
            buffer_recreated,
        }
    }

    pub(super) fn current_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.staged_slot]
    }

    pub(super) fn previous_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[1 - self.staged_slot]
    }

    pub(super) fn buffers_for_current_slot(
        &self,
        current_slot: usize,
    ) -> (&wgpu::Buffer, &wgpu::Buffer) {
        (&self.buffers[current_slot], &self.buffers[1 - current_slot])
    }

    pub(super) const fn staged_slot(&self) -> usize {
        self.staged_slot
    }

    pub(super) fn commit_after_scene_success(&mut self) {
        self.committed_slot = Some(self.staged_slot);
        std::mem::swap(&mut self.committed_spans, &mut self.staged_spans);
        self.staged_spans.clear();
    }
}

fn stage_palette_span(
    staged_spans: &mut HashMap<u64, SkinnedPaletteSpan>,
    staged_matrices: &mut Vec<[[f32; 4]; 4]>,
    stable_instance_key: u64,
    storage: &SkinnedMeshJointPaletteStorage,
) -> SkinnedPaletteSpan {
    if let Some(span) = staged_spans.get(&stable_instance_key).copied() {
        assert_eq!(
            span.joint_count,
            storage.joint_count(),
            "one stable skinned instance key must resolve one palette span per frame"
        );
        return span;
    }

    let matrix_base = u32::try_from(staged_matrices.len())
        .expect("skinned palette arena matrix base exceeded u32");
    staged_matrices.extend_from_slice(storage.active_joint_matrices());
    let span = SkinnedPaletteSpan {
        matrix_base,
        joint_count: storage.joint_count(),
    };
    staged_spans.insert(stable_instance_key, span);
    span
}

fn create_palette_buffer(
    device: &wgpu::Device,
    size: u64,
    label: &'static str,
) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    }))
}

fn grow_palette_capacity(required: u64) -> u64 {
    required.checked_next_power_of_two().unwrap_or(required)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::core::math::Mat4;
    use crate::graphics::scene::scene_renderer::SkinnedMeshJointPaletteStorage;

    use super::{SkinnedPaletteSpan, grow_palette_capacity, stage_palette_span};

    #[test]
    fn palette_span_projects_current_and_previous_instance_indirection() {
        let current = SkinnedPaletteSpan {
            matrix_base: 7,
            joint_count: 64,
        };
        let previous = SkinnedPaletteSpan {
            matrix_base: 11,
            joint_count: 32,
        };

        assert_eq!(current.params(Some(previous)), [7, 64, 11, 32]);
        assert_eq!(current.params(None), [7, 64, 0, 0]);
    }

    #[test]
    fn palette_capacity_is_power_of_two_and_grow_only_at_the_owner() {
        assert_eq!(grow_palette_capacity(1), 1);
        assert_eq!(grow_palette_capacity(4_096), 4_096);
        assert_eq!(grow_palette_capacity(4_097), 8_192);
    }

    #[test]
    fn palette_packing_is_contiguous_and_deduplicates_stable_instances() {
        let mut spans = HashMap::new();
        let mut matrices = Vec::new();
        let first = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY; 2])
            .expect("test palette fits CPU snapshot");
        let second = SkinnedMeshJointPaletteStorage::from_matrices(&[Mat4::IDENTITY])
            .expect("test palette fits CPU snapshot");

        let first_span = stage_palette_span(&mut spans, &mut matrices, 11, &first);
        let second_span = stage_palette_span(&mut spans, &mut matrices, 22, &second);
        let repeated_first_span = stage_palette_span(&mut spans, &mut matrices, 11, &first);

        assert_eq!(
            first_span,
            SkinnedPaletteSpan {
                matrix_base: 0,
                joint_count: 2
            }
        );
        assert_eq!(
            second_span,
            SkinnedPaletteSpan {
                matrix_base: 2,
                joint_count: 1
            }
        );
        assert_eq!(repeated_first_span, first_span);
        assert_eq!(matrices.len(), 3);
        assert_eq!(spans.len(), 2);
    }
}
