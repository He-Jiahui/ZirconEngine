use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::mesh::build_mesh_draws::IndexedIndirectArgs;

use super::{
    IndirectCompactionPlan, INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES,
    INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES,
};

pub(crate) struct MeshIndirectCompactionResources {
    metadata_buffer: wgpu::Buffer,
    visible_instance_index_buffer: wgpu::Buffer,
    draw_count_buffer: wgpu::Buffer,
    compacted_indirect_args_buffer: wgpu::Buffer,
    metadata_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_buffer_allocation_byte_size: wgpu::BufferAddress,
    draw_count_buffer_byte_size: wgpu::BufferAddress,
    draw_count_buffer_allocation_byte_size: wgpu::BufferAddress,
    compacted_indirect_args_buffer_byte_size: wgpu::BufferAddress,
    visible_instance_index_capacity: u32,
    draw_count_capacity: u32,
}

impl MeshIndirectCompactionResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        label_prefix: &'static str,
        plan: &IndirectCompactionPlan,
    ) -> Self {
        let metadata_label = format!("{label_prefix}-compaction-metadata");
        let metadata_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(metadata_label.as_str()),
            contents: bytemuck::cast_slice(plan.metadata()),
            usage: compaction_storage_usage(),
        });

        let visible_instance_index_buffer_byte_size =
            plan.visible_instance_index_buffer_byte_size();
        let visible_instance_index_buffer_allocation_byte_size =
            bindable_storage_buffer_size(visible_instance_index_buffer_byte_size);
        let visible_instance_index_label = format!("{label_prefix}-visible-instance-index");
        let visible_instance_index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(visible_instance_index_label.as_str()),
            size: visible_instance_index_buffer_allocation_byte_size,
            usage: compaction_storage_usage(),
            mapped_at_creation: false,
        });

        let draw_count_label = format!("{label_prefix}-compaction-draw-count");
        let draw_count_buffer_byte_size = plan.draw_count_buffer_byte_size();
        let draw_count_buffer_allocation_byte_size =
            bindable_draw_count_buffer_size(draw_count_buffer_byte_size);
        let draw_count_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(draw_count_label.as_str()),
            size: draw_count_buffer_allocation_byte_size,
            usage: compaction_draw_count_usage(),
            mapped_at_creation: false,
        });

        let compacted_indirect_args_buffer_byte_size =
            compacted_indirect_args_buffer_byte_size(plan.metadata_count());
        let compacted_indirect_args_label = format!("{label_prefix}-compacted-indirect-args");
        let compacted_indirect_args_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(compacted_indirect_args_label.as_str()),
            size: compacted_indirect_args_buffer_byte_size,
            usage: compacted_indirect_args_usage(),
            mapped_at_creation: false,
        });

        Self {
            metadata_buffer,
            visible_instance_index_buffer,
            draw_count_buffer,
            compacted_indirect_args_buffer,
            metadata_buffer_byte_size: plan.metadata_buffer_byte_size(),
            visible_instance_index_buffer_byte_size,
            visible_instance_index_buffer_allocation_byte_size,
            draw_count_buffer_byte_size,
            draw_count_buffer_allocation_byte_size,
            compacted_indirect_args_buffer_byte_size,
            visible_instance_index_capacity: plan.visible_instance_capacity(),
            draw_count_capacity: plan.draw_count_count(),
        }
    }

    pub(crate) const fn metadata_buffer(&self) -> &wgpu::Buffer {
        &self.metadata_buffer
    }

    pub(crate) const fn visible_instance_index_buffer(&self) -> &wgpu::Buffer {
        &self.visible_instance_index_buffer
    }

    pub(crate) const fn draw_count_buffer(&self) -> &wgpu::Buffer {
        &self.draw_count_buffer
    }

    pub(crate) const fn compacted_indirect_args_buffer(&self) -> &wgpu::Buffer {
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
mod tests {
    use super::*;

    #[test]
    fn bindable_storage_buffer_size_keeps_zero_capacity_buffers_bindable() {
        assert_eq!(
            bindable_storage_buffer_size(0),
            INDIRECT_VISIBLE_INSTANCE_INDEX_STRIDE_BYTES
        );
        assert_eq!(bindable_storage_buffer_size(20), 20);
    }

    #[test]
    fn bindable_draw_count_buffer_size_keeps_zero_capacity_buffers_bindable() {
        assert_eq!(
            bindable_draw_count_buffer_size(0),
            INDIRECT_DRAW_COUNT_BUFFER_SIZE_BYTES
        );
        assert_eq!(bindable_draw_count_buffer_size(8), 8);
    }

    #[test]
    fn mesh_indirect_compaction_resources_reserve_expected_wgpu_usages() {
        let source = include_str!("indirect_compaction_resources.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation source");

        assert!(implementation.contains("create_buffer_init"));
        assert!(implementation.contains("visible-instance-index"));
        assert!(implementation.contains("compacted-indirect-args"));
        assert!(implementation.contains("wgpu::BufferUsages::STORAGE"));
        assert!(implementation.contains("wgpu::BufferUsages::COPY_DST"));
        assert!(implementation.contains("wgpu::BufferUsages::COPY_SRC"));
        assert!(implementation.contains("wgpu::BufferUsages::INDIRECT"));
    }

    #[test]
    fn mesh_indirect_compaction_resources_clear_outputs_without_rewriting_metadata() {
        let source = include_str!("indirect_compaction_resources.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("implementation source");

        assert!(implementation.contains("pub(crate) fn encode_clear_outputs"));
        assert!(implementation.contains("encoder.clear_buffer("));
        assert!(implementation.contains("visible_instance_index_buffer_allocation_byte_size"));
        assert!(implementation.contains("draw_count_buffer"));
        assert!(implementation.contains("compacted_indirect_args_buffer"));
        assert!(!implementation.contains("clear_buffer(\n            &self.metadata_buffer"));
    }
}
