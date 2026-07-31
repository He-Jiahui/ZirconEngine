use crate::virtual_geometry::renderer::root_render_passes::VirtualGeometryIndirectStats;
use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::{
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;

use super::virtual_geometry_readback_outputs::VirtualGeometryReadbackOutputs;

const VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER: &str = "virtual-geometry-feedback";
const VIRTUAL_GEOMETRY_FEEDBACK_BACKING: &str =
    "virtual-geometry-feedback:runtime-prepare-page-requests";

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
    context: &mut RuntimePrepareCollectorContext<'_>,
) -> RenderPluginRendererOutputs {
    // The frame sideband remains the feedback owner and is moved into runtime feedback after
    // rendering. Mirroring it here would deep-clone large readback vectors and merge them twice.
    register_prepared_virtual_geometry_feedback_buffer(context);
    RenderPluginRendererOutputs::default()
}

fn register_prepared_virtual_geometry_feedback_buffer(
    context: &mut RuntimePrepareCollectorContext<'_>,
) {
    let page_request_ids = &context
        .prepared_virtual_geometry_readback_outputs()
        .node_cluster_cull
        .page_request_ids;
    if page_request_ids.is_empty() {
        return;
    }

    let buffer = context
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-vg-runtime-prepare-feedback-page-requests"),
            contents: bytemuck::cast_slice(page_request_ids),
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });
    context.register_external_buffer_binding_with_backing(
        VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER,
        VIRTUAL_GEOMETRY_FEEDBACK_BACKING,
        &buffer,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_prepare_keeps_frame_sideband_as_the_feedback_owner() {
        let source = include_str!("virtual_geometry_plugin_renderer_outputs.rs");

        assert!(!source.contains(concat!(
            "prepared_virtual_geometry_readback_outputs().",
            "clone()"
        )));
        assert!(source.contains(concat!(
            "register_prepared_virtual_geometry_feedback_buffer(context);",
            "\n    RenderPluginRendererOutputs::default()"
        )));
    }

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

    #[test]
    fn virtual_geometry_feedback_binding_names_stay_stable() {
        assert_eq!(
            VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER,
            "virtual-geometry-feedback"
        );
        assert_eq!(
            VIRTUAL_GEOMETRY_FEEDBACK_BACKING,
            "virtual-geometry-feedback:runtime-prepare-page-requests"
        );
    }
}
