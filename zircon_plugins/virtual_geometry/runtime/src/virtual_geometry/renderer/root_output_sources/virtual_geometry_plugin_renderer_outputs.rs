use crate::virtual_geometry::renderer::root_render_passes::VirtualGeometryIndirectStats;
use zircon_runtime::core::framework::render::{
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;

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

pub(in crate::virtual_geometry::renderer) fn plugin_renderer_outputs_from_virtual_geometry_readback(
    virtual_geometry: RenderVirtualGeometryReadbackOutputs,
) -> RenderPluginRendererOutputs {
    RenderPluginRendererOutputs {
        virtual_geometry,
        ..RenderPluginRendererOutputs::default()
    }
}

pub(crate) fn runtime_prepare_renderer_outputs(
    context: &RuntimePrepareCollectorContext<'_>,
) -> RenderPluginRendererOutputs {
    // The collector mirrors only neutral sidebands prepared by runtime providers;
    // it does not synthesize GPU/readback feedback from CPU prepare heuristics.
    plugin_renderer_outputs_from_virtual_geometry_readback(
        context.prepared_virtual_geometry_readback_outputs().clone(),
    )
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

    #[test]
    fn runtime_prepare_renderer_outputs_do_not_fabricate_virtual_geometry_readbacks() {
        let outputs = plugin_renderer_outputs_from_virtual_geometry_readback(
            RenderVirtualGeometryReadbackOutputs::default(),
        );

        assert!(outputs.is_empty());
        assert!(outputs.virtual_geometry.is_empty());
    }

    #[test]
    fn runtime_prepare_renderer_outputs_package_prepared_virtual_geometry_sideband() {
        let outputs = plugin_renderer_outputs_from_virtual_geometry_readback(
            RenderVirtualGeometryReadbackOutputs {
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![401, 402],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
        );

        assert_eq!(
            outputs.virtual_geometry.node_cluster_cull.page_request_ids,
            vec![401, 402]
        );
        assert!(outputs.hybrid_gi.is_empty());
        assert!(outputs.particles.is_empty());
    }
}
