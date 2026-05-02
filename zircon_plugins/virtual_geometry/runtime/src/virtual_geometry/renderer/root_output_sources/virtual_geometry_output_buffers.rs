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

    let packed_words = instance_work_items
        .iter()
        .flat_map(RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::packed_words)
        .collect::<Vec<_>>();
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

    let packed_words = cluster_work_items
        .iter()
        .flat_map(VirtualGeometryNodeAndClusterCullClusterWorkItem::packed_words)
        .collect::<Vec<_>>();
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

    let packed_words = child_work_items
        .iter()
        .flat_map(VirtualGeometryNodeAndClusterCullChildWorkItem::packed_words)
        .collect::<Vec<_>>();
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

    let packed_words = traversal_records
        .iter()
        .flat_map(VirtualGeometryNodeAndClusterCullTraversalRecord::packed_words)
        .collect::<Vec<_>>();
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
    records
        .iter()
        .flat_map(|record| record.packed_words())
        .collect()
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
