use std::sync::Arc;

use crate::virtual_geometry::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCullInputSnapshot, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
};

#[cfg(test)]
#[path = "virtual_geometry_output_buffers/fixed_packing_tests.rs"]
mod fixed_packing_tests;

#[inline]
fn collect_fixed_packed_words<T, const WORD_COUNT: usize>(
    values: &[T],
    packed_words: impl Fn(&T) -> [u32; WORD_COUNT],
) -> Vec<u32> {
    let capacity = values
        .len()
        .checked_mul(WORD_COUNT)
        .expect("fixed-width output buffer word count exceeds addressable memory");
    let mut words = Vec::with_capacity(capacity);
    for value in values {
        words.extend_from_slice(&packed_words(value));
    }
    words
}

pub(super) fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    packed_words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-selected-cluster-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_cull_input_buffer(
    device: &wgpu::Device,
    cull_input: &RenderVirtualGeometryCullInputSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    let packed_words = cull_input.packed_words();
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-cull-input-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_launch_worklist_buffer(
    device: &wgpu::Device,
    launch_worklist: &RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    let packed_words = launch_worklist.packed_words();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-launch-worklist-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_instance_work_item_buffer(
    device: &wgpu::Device,
    instance_work_items: &[RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem],
) -> Option<Arc<wgpu::Buffer>> {
    if instance_work_items.is_empty() {
        return None;
    }

    let packed_words = collect_fixed_packed_words(
        instance_work_items,
        RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::packed_words,
    );
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-work-item-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_cluster_work_item_buffer(
    device: &wgpu::Device,
    cluster_work_items: &[VirtualGeometryNodeAndClusterCullClusterWorkItem],
) -> Option<Arc<wgpu::Buffer>> {
    if cluster_work_items.is_empty() {
        return None;
    }

    let packed_words = collect_fixed_packed_words(
        cluster_work_items,
        VirtualGeometryNodeAndClusterCullClusterWorkItem::packed_words,
    );
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-cluster-work-item-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_hierarchy_child_id_buffer(
    device: &wgpu::Device,
    hierarchy_child_ids: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if hierarchy_child_ids.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-hierarchy-child-ids"),
            contents: bytemuck::cast_slice(hierarchy_child_ids),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_child_work_item_buffer(
    device: &wgpu::Device,
    child_work_items: &[VirtualGeometryNodeAndClusterCullChildWorkItem],
) -> Option<Arc<wgpu::Buffer>> {
    if child_work_items.is_empty() {
        return None;
    }

    let packed_words = collect_fixed_packed_words(
        child_work_items,
        VirtualGeometryNodeAndClusterCullChildWorkItem::packed_words,
    );
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-child-work-item-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_traversal_record_buffer(
    device: &wgpu::Device,
    traversal_records: &[VirtualGeometryNodeAndClusterCullTraversalRecord],
) -> Option<Arc<wgpu::Buffer>> {
    if traversal_records.is_empty() {
        return None;
    }

    let packed_words = collect_fixed_packed_words(
        traversal_records,
        VirtualGeometryNodeAndClusterCullTraversalRecord::packed_words,
    );
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-traversal-record-buffer"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_page_request_buffer(
    device: &wgpu::Device,
    page_request_ids: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if page_request_ids.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-page-request-buffer"),
            contents: bytemuck::cast_slice(page_request_ids),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_visbuffer64_buffer(
    device: &wgpu::Device,
    packed_words: &[u64],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-visbuffer64-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn pack_hardware_rasterization_records(
    records: &[RenderVirtualGeometryHardwareRasterizationRecord],
) -> Vec<u32> {
    collect_fixed_packed_words(records, |record| record.packed_words())
}

pub(super) fn create_hardware_rasterization_buffer(
    device: &wgpu::Device,
    packed_words: &[u32],
) -> Option<Arc<wgpu::Buffer>> {
    if packed_words.is_empty() {
        return None;
    }

    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-hardware-rasterization-buffer"),
            contents: bytemuck::cast_slice(packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}
