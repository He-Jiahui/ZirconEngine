use crate::virtual_geometry::renderer::root_render_passes::VirtualGeometryIndirectStats;
use wgpu::util::DeviceExt;
use zircon_runtime::core::framework::render::{
    RenderPluginRendererOutputs, RenderVirtualGeometryNodeClusterCullReadbackOutputs,
    RenderVirtualGeometryReadbackOutputs,
};
use zircon_runtime::graphics::RuntimePrepareCollectorContext;
use zircon_runtime::rhi::{BufferDesc, BufferUsage};

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
    // GPU feedback always has a count-prefixed bounded destination, including frames whose
    // CPU sideband contains no requests. The sideband retains its single-consumer ownership.
    let words = vec![0_u32; (crate::VIRTUAL_GEOMETRY_GRAPH_ID_CAPACITY + 1) as usize];

    let buffer = {
        let gpu = context.gpu_recording_context();
        gpu.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-vg-runtime-prepare-feedback-page-requests"),
                contents: bytemuck::cast_slice(&words),
                usage: wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::STORAGE,
            })
    };
    context.register_external_buffer_binding_with_backing_and_physical_desc(
        VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER,
        VIRTUAL_GEOMETRY_FEEDBACK_BACKING,
        &buffer,
        BufferDesc::new(
            VIRTUAL_GEOMETRY_FEEDBACK_EXTERNAL_BUFFER,
            buffer.size(),
            BufferUsage::COPY_SRC | BufferUsage::COPY_DST | BufferUsage::STORAGE,
        ),
    );
}

#[cfg(test)]
#[path = "tests/virtual_geometry_plugin.rs"]
mod tests;
