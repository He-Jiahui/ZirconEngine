use std::sync::Arc;

use zr_rhi_wgpu::WgpuBufferUploadBatch;

use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{
    IndirectCompactionBatchMetadata, IndirectCompactionPlan, PodRangeUploadCommit,
    PodRangeUploadShadow, INDIRECT_COMPACTION_METADATA_STRIDE_BYTES,
    INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES, INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
};

#[derive(Clone)]
pub(crate) struct MeshIndirectCompactionResources {
    metadata_buffer: Arc<wgpu::Buffer>,
    visible_instance_index_buffer: Arc<wgpu::Buffer>,
    draw_count_buffer: Arc<wgpu::Buffer>,
    compacted_indirect_args_buffer: Arc<wgpu::Buffer>,
    metadata_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_buffer_allocation_byte_size: wgpu::BufferAddress,
    draw_count_buffer_byte_size: wgpu::BufferAddress,
    draw_count_buffer_allocation_byte_size: wgpu::BufferAddress,
    compacted_indirect_args_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_capacity: u32,
    draw_count_capacity: u32,
}

#[derive(Default)]
pub(crate) struct MeshIndirectCompactionWorkspace {
    metadata_buffer: Option<Arc<wgpu::Buffer>>,
    visible_instance_index_buffer: Option<Arc<wgpu::Buffer>>,
    draw_count_buffer: Option<Arc<wgpu::Buffer>>,
    compacted_indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    metadata_capacity_bytes: wgpu::BufferAddress,
    visible_instance_index_capacity_bytes: wgpu::BufferAddress,
    draw_count_capacity_bytes: wgpu::BufferAddress,
    compacted_indirect_args_capacity_bytes: wgpu::BufferAddress,
    metadata_buffer_revision: u64,
    metadata_shadow: PodRangeUploadShadow<IndirectCompactionBatchMetadata>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MeshIndirectCompactionPrepareStats {
    pub(crate) created_buffer_count: u32,
    pub(crate) uploaded_byte_count: u64,
    pub(crate) upload_range_count: u32,
}

impl MeshIndirectCompactionWorkspace {
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        label_prefix: &'static str,
        plan: &IndirectCompactionPlan,
        uploads: &mut WgpuBufferUploadBatch,
    ) -> (
        MeshIndirectCompactionResources,
        MeshIndirectCompactionPrepareStats,
        Option<PodRangeUploadCommit>,
    ) {
        let mut stats = MeshIndirectCompactionPrepareStats::default();
        let metadata_buffer_recreated = ensure_buffer_capacity(
            device,
            &mut self.metadata_buffer,
            &mut self.metadata_capacity_bytes,
            plan.metadata_buffer_byte_size(),
            INDIRECT_COMPACTION_METADATA_STRIDE_BYTES,
            label_prefix,
            "compaction-metadata",
            compaction_storage_usage(),
        );
        stats.created_buffer_count += u32::from(metadata_buffer_recreated);
        if metadata_buffer_recreated {
            self.metadata_buffer_revision = self.metadata_buffer_revision.wrapping_add(1).max(1);
        }
        let (metadata_upload, metadata_commit) = self.metadata_shadow.prepare(
            self.metadata_buffer
                .as_ref()
                .expect("metadata buffer was prepared"),
            self.metadata_buffer_revision,
            plan.metadata(),
            uploads,
        );
        stats.uploaded_byte_count = metadata_upload.byte_count;
        stats.upload_range_count = metadata_upload.range_count;

        let visible_instance_index_buffer_byte_size =
            plan.visible_instance_index_buffer_byte_size();
        let visible_instance_index_buffer_allocation_byte_size =
            bindable_storage_buffer_size(visible_instance_index_buffer_byte_size);
        let recreated = ensure_buffer_capacity(
            device,
            &mut self.visible_instance_index_buffer,
            &mut self.visible_instance_index_capacity_bytes,
            visible_instance_index_buffer_allocation_byte_size,
            INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
            label_prefix,
            "visible-instance-index",
            compaction_storage_usage(),
        );
        stats.created_buffer_count += u32::from(recreated);

        let draw_count_buffer_byte_size = plan.draw_count_buffer_byte_size();
        let draw_count_buffer_allocation_byte_size =
            bindable_draw_count_buffer_size(draw_count_buffer_byte_size);
        let recreated = ensure_buffer_capacity(
            device,
            &mut self.draw_count_buffer,
            &mut self.draw_count_capacity_bytes,
            draw_count_buffer_allocation_byte_size,
            INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
            label_prefix,
            "compaction-draw-count",
            compaction_draw_count_usage(),
        );
        stats.created_buffer_count += u32::from(recreated);

        let compacted_indirect_args_buffer_byte_size =
            compacted_indirect_args_buffer_byte_size(plan.metadata_count());
        let recreated = ensure_buffer_capacity(
            device,
            &mut self.compacted_indirect_args_buffer,
            &mut self.compacted_indirect_args_capacity_bytes,
            compacted_indirect_args_buffer_byte_size,
            std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress,
            label_prefix,
            "compacted-indirect-args",
            compacted_indirect_args_usage(),
        );
        stats.created_buffer_count += u32::from(recreated);

        (
            MeshIndirectCompactionResources {
                metadata_buffer: Arc::clone(
                    self.metadata_buffer
                        .as_ref()
                        .expect("metadata buffer was prepared"),
                ),
                visible_instance_index_buffer: Arc::clone(
                    self.visible_instance_index_buffer
                        .as_ref()
                        .expect("visible instance index buffer was prepared"),
                ),
                draw_count_buffer: Arc::clone(
                    self.draw_count_buffer
                        .as_ref()
                        .expect("draw count buffer was prepared"),
                ),
                compacted_indirect_args_buffer: Arc::clone(
                    self.compacted_indirect_args_buffer
                        .as_ref()
                        .expect("compacted indirect args buffer was prepared"),
                ),
                metadata_buffer_byte_size: plan.metadata_buffer_byte_size(),
                visible_instance_index_buffer_byte_size,
                visible_instance_index_buffer_allocation_byte_size,
                draw_count_buffer_byte_size,
                draw_count_buffer_allocation_byte_size,
                compacted_indirect_args_buffer_byte_size,
                visible_instance_index_capacity: plan.visible_instance_capacity(),
                draw_count_capacity: plan.draw_count_count(),
            },
            stats,
            metadata_commit,
        )
    }

    pub(crate) fn accepts_metadata_upload(&self, commit: PodRangeUploadCommit) -> bool {
        self.metadata_shadow.accepts(commit)
    }

    pub(crate) fn commit_metadata_upload(&mut self, commit: PodRangeUploadCommit) -> bool {
        self.metadata_shadow.commit(commit)
    }
}

impl MeshIndirectCompactionResources {
    pub(crate) fn metadata_buffer(&self) -> &wgpu::Buffer {
        &self.metadata_buffer
    }

    pub(crate) fn visible_instance_index_buffer(&self) -> &wgpu::Buffer {
        &self.visible_instance_index_buffer
    }

    pub(crate) fn draw_count_buffer(&self) -> &wgpu::Buffer {
        &self.draw_count_buffer
    }

    pub(crate) fn compacted_indirect_args_buffer(&self) -> &wgpu::Buffer {
        &self.compacted_indirect_args_buffer
    }

    pub(crate) const fn metadata_buffer_byte_size(&self) -> wgpu::BufferAddress {
        self.metadata_buffer_byte_size
    }

    pub(crate) const fn visible_instance_index_buffer_byte_size(&self) -> wgpu::BufferAddress {
        self.visible_instance_index_buffer_byte_size
    }

    pub(crate) const fn visible_instance_index_buffer_allocation_byte_size(
        &self,
    ) -> wgpu::BufferAddress {
        self.visible_instance_index_buffer_allocation_byte_size
    }

    pub(crate) const fn visible_instance_index_capacity(&self) -> u32 {
        self.visible_instance_index_capacity
    }

    pub(crate) const fn draw_count_buffer_byte_size(&self) -> wgpu::BufferAddress {
        self.draw_count_buffer_byte_size
    }

    pub(crate) const fn draw_count_buffer_allocation_byte_size(&self) -> wgpu::BufferAddress {
        self.draw_count_buffer_allocation_byte_size
    }

    pub(crate) const fn compacted_indirect_args_buffer_byte_size(&self) -> wgpu::BufferAddress {
        self.compacted_indirect_args_buffer_byte_size
    }

    pub(crate) const fn draw_count_capacity(&self) -> u32 {
        self.draw_count_capacity
    }

    // 每帧重置 GPU 生成的索引、计数和压缩参数；metadata 已由本帧上传批提供，须保留。
    pub(crate) fn encode_clear_outputs(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(
            &self.visible_instance_index_buffer,
            0,
            Some(self.visible_instance_index_buffer_allocation_byte_size),
        );
        encoder.clear_buffer(
            &self.draw_count_buffer,
            0,
            Some(self.draw_count_buffer_allocation_byte_size),
        );
        encoder.clear_buffer(
            &self.compacted_indirect_args_buffer,
            0,
            Some(self.compacted_indirect_args_buffer_byte_size),
        );
    }
}

pub(super) fn grow_indirect_buffer_capacity(
    current: wgpu::BufferAddress,
    required: wgpu::BufferAddress,
) -> wgpu::BufferAddress {
    if current >= required {
        return current;
    }
    required.checked_next_power_of_two().unwrap_or(required)
}

fn ensure_buffer_capacity(
    device: &wgpu::Device,
    buffer: &mut Option<Arc<wgpu::Buffer>>,
    capacity: &mut wgpu::BufferAddress,
    required: wgpu::BufferAddress,
    minimum: wgpu::BufferAddress,
    label_prefix: &str,
    label_suffix: &str,
    usage: wgpu::BufferUsages,
) -> bool {
    let required = required.max(minimum);
    if buffer.is_some() && *capacity >= required {
        return false;
    }
    *capacity = grow_indirect_buffer_capacity(*capacity, required);
    let label = format!("{label_prefix}-{label_suffix}");
    *buffer = Some(Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label.as_str()),
        size: *capacity,
        usage,
        mapped_at_creation: false,
    })));
    true
}

fn bindable_storage_buffer_size(byte_size: wgpu::BufferAddress) -> wgpu::BufferAddress {
    byte_size.max(INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES)
}

fn bindable_draw_count_buffer_size(byte_size: wgpu::BufferAddress) -> wgpu::BufferAddress {
    byte_size.max(INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES)
}

fn compacted_indirect_args_buffer_byte_size(args_count: u32) -> wgpu::BufferAddress {
    (u64::from(args_count) * std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress)
        .max(std::mem::size_of::<IndexedIndirectArgs>() as wgpu::BufferAddress)
}

fn compaction_storage_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}

fn compaction_draw_count_usage() -> wgpu::BufferUsages {
    compaction_storage_usage() | wgpu::BufferUsages::INDIRECT
}

fn compacted_indirect_args_usage() -> wgpu::BufferUsages {
    compaction_storage_usage() | wgpu::BufferUsages::INDIRECT
}

#[cfg(test)]
#[path = "tests/indirect_compaction_resources.rs"]
mod tests;
