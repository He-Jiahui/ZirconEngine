use std::sync::Arc;

use crate::core::framework::render::RenderVirtualGeometrySelectedCluster;
use wgpu::util::DeviceExt;

pub(super) fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Option<Arc<wgpu::Buffer>> {
    if selected_clusters.is_empty() {
        return None;
    }

    let packed_words = selected_clusters
        .iter()
        .flat_map(RenderVirtualGeometrySelectedCluster::packed_words)
        .collect::<Vec<_>>();
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-executed-selected-clusters"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}
