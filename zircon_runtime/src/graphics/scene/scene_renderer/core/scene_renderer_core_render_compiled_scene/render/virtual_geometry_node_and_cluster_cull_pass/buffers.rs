use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
};
use crate::graphics::types::{
    VirtualGeometryNodeAndClusterCullChildWorkItem,
    VirtualGeometryNodeAndClusterCullClusterWorkItem,
    VirtualGeometryNodeAndClusterCullTraversalRecord,
};
use wgpu::util::DeviceExt;

pub(super) fn create_node_and_cluster_cull_dispatch_setup_buffer(
    device: &wgpu::Device,
    dispatch_setup: &RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
) -> Option<Arc<wgpu::Buffer>> {
    let packed_words = dispatch_setup.packed_words();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-dispatch-setup"),
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
            label: Some("zircon-vg-node-and-cluster-cull-launch-worklist"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}

pub(super) fn create_node_and_cluster_cull_instance_seed_buffer(
    device: &wgpu::Device,
    instance_seeds: &[RenderVirtualGeometryNodeAndClusterCullInstanceSeed],
) -> Option<Arc<wgpu::Buffer>> {
    if instance_seeds.is_empty() {
        return None;
    }

    let packed_words = instance_seeds
        .iter()
        .flat_map(RenderVirtualGeometryNodeAndClusterCullInstanceSeed::packed_words)
        .collect::<Vec<_>>();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-node-and-cluster-cull-instance-seeds"),
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
            label: Some("zircon-vg-node-and-cluster-cull-cluster-work-items"),
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
            label: Some("zircon-vg-node-and-cluster-cull-child-work-items"),
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
            label: Some("zircon-vg-node-and-cluster-cull-traversal-records"),
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
            label: Some("zircon-vg-node-and-cluster-cull-page-requests"),
            contents: bytemuck::cast_slice(page_request_ids),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}
