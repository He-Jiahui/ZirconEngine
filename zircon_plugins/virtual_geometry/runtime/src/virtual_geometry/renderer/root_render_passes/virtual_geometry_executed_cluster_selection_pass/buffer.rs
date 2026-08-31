use std::sync::Arc;

use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster;

use super::super::packed_words::collect_fixed_packed_words;

pub(super) fn create_selected_cluster_buffer(
    device: &wgpu::Device,
    selected_clusters: &[RenderVirtualGeometrySelectedCluster],
) -> Option<Arc<wgpu::Buffer>> {
    if selected_clusters.is_empty() {
        return None;
    }

    let packed_words = collect_fixed_packed_words(
        selected_clusters,
        RenderVirtualGeometrySelectedCluster::packed_words,
    );
    Some(Arc::new(device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-executed-selected-clusters"),
            contents: bytemuck::cast_slice(&packed_words),
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE,
        },
    )))
}
