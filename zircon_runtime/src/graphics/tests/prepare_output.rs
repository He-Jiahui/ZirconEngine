use super::*;
use crate::core::framework::render::{
    RenderVirtualGeometryNodeClusterCullReadbackOutputs, RenderVirtualGeometryReadbackOutputs,
};

#[test]
fn prepare_output_carries_neutral_virtual_geometry_renderer_outputs() {
    let output = VirtualGeometryRuntimePrepareOutput::new(vec![3]).with_renderer_outputs(
        RenderPluginRendererOutputs {
            virtual_geometry: RenderVirtualGeometryReadbackOutputs {
                node_cluster_cull: RenderVirtualGeometryNodeClusterCullReadbackOutputs {
                    page_request_ids: vec![300, 301],
                    ..RenderVirtualGeometryNodeClusterCullReadbackOutputs::default()
                },
                ..RenderVirtualGeometryReadbackOutputs::default()
            },
            ..RenderPluginRendererOutputs::default()
        },
    );

    assert_eq!(
        output
            .renderer_outputs()
            .virtual_geometry
            .node_cluster_cull
            .page_request_ids,
        vec![300, 301]
    );

    let (evictable_page_ids, renderer_outputs) = output.into_parts();
    assert_eq!(evictable_page_ids, vec![3]);
    assert_eq!(
        renderer_outputs
            .virtual_geometry
            .node_cluster_cull
            .page_request_ids,
        vec![300, 301]
    );
}
