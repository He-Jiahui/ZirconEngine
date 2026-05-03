use crate::virtual_geometry::renderer::root_render_passes::VirtualGeometryIndirectStats;
use zircon_runtime::core::framework::render::{
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
};

use super::virtual_geometry_readback_outputs::VirtualGeometryReadbackOutputs;

pub(in crate::virtual_geometry::renderer) fn plugin_renderer_outputs_from_indirect_stats(
    stats: &VirtualGeometryIndirectStats,
) -> RenderPluginRendererOutputs {
    plugin_renderer_outputs_from_node_cluster_cull_readback(
        stats.node_and_cluster_cull_readback_outputs(),
    )
}

pub(in crate::virtual_geometry::renderer) fn plugin_renderer_outputs_from_node_cluster_cull_readback(
    node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs,
) -> RenderPluginRendererOutputs {
    let mut readback_outputs = VirtualGeometryReadbackOutputs::default();
    readback_outputs.store_node_cluster_cull_readback(node_cluster_cull);

    RenderPluginRendererOutputs {
        virtual_geometry: readback_outputs.take_neutral_readback_outputs(),
        ..RenderPluginRendererOutputs::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_renderer_outputs_package_node_cluster_cull_readback_under_virtual_geometry() {
        let outputs = plugin_renderer_outputs_from_node_cluster_cull_readback(
            RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                page_request_ids: vec![300, 301],
                ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
            },
        );

        assert_eq!(
            outputs.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![300, 301]
        );
        assert!(outputs.hybrid_gi.is_empty());
        assert!(outputs.particles.is_empty());
        assert!(!outputs.is_empty());
    }
}
