use crate::graphics::scene::scene_renderer::core::scene_renderer_core::SceneRendererNeutralGraphBuffers;
use crate::graphics::scene::scene_renderer::graph_execution::RenderGraphExecutionResources;
use crate::graphics::RuntimePrepareExternalBufferBindingPacket;
use crate::render_graph::{
    CompiledRenderGraph, RenderGraphExternalResourceType, RenderGraphResourceDesc,
};

const MIN_PLUGIN_EXTERNAL_BUFFER_SIZE: wgpu::BufferAddress =
    std::mem::size_of::<u32>() as wgpu::BufferAddress;

pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn bind_plugin_graph_resources(
    device: &wgpu::Device,
    neutral_buffers: &mut SceneRendererNeutralGraphBuffers,
    graph: &CompiledRenderGraph,
    external_buffer_binding_packet: Option<&RuntimePrepareExternalBufferBindingPacket>,
    resources: &mut RenderGraphExecutionResources,
) -> Result<(), String> {
    if let Some(binding_packet) = external_buffer_binding_packet {
        binding_packet.ensure_device_epoch(resources.device_epoch())?;
        for binding in binding_packet.bindings() {
            let logical_name = binding.logical_name();
            if !graph_declares_typed_external_buffer(graph, logical_name) {
                continue;
            }

            if let Some(physical_desc) = binding.physical_desc() {
                resources.bind_borrowed_buffer_with_physical_desc(
                    logical_name,
                    binding.backing_name(),
                    binding.buffer(),
                    physical_desc.clone(),
                )?;
            } else {
                resources.bind_execution_owned_buffer(
                    logical_name,
                    binding.backing_name(),
                    binding.buffer(),
                );
            }
        }
    }

    for logical_name in FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS {
        if PARTICLE_PLUGIN_EXTERNAL_BUFFERS.contains(logical_name) {
            continue;
        }
        if resources.has_buffer(logical_name) {
            continue;
        }
        if !graph_declares_typed_external_buffer(graph, logical_name) {
            continue;
        }

        if let Some((buffer, backing_name)) = neutral_buffers.plugin_buffer(
            device,
            logical_name,
            plugin_external_fallback_size(logical_name),
        ) {
            resources.bind_execution_owned_buffer(*logical_name, backing_name, buffer);
        }
    }
    Ok(())
}

fn graph_declares_typed_external_buffer(graph: &CompiledRenderGraph, logical_name: &str) -> bool {
    graph
        .resource_lifetime_by_name(logical_name)
        .is_some_and(|lifetime| {
            matches!(&lifetime.desc, RenderGraphResourceDesc::External)
                && lifetime.external_binding.resource_type
                    == RenderGraphExternalResourceType::Buffer
        })
}

fn plugin_external_fallback_size(logical_name: &str) -> wgpu::BufferAddress {
    plugin_external_buffer_min_size(logical_name).max(MIN_PLUGIN_EXTERNAL_BUFFER_SIZE)
}

fn plugin_external_buffer_min_size(logical_name: &str) -> wgpu::BufferAddress {
    match logical_name {
        "particles.gpu.indirect-draw-args" => {
            (4 * std::mem::size_of::<u32>()) as wgpu::BufferAddress
        }
        "particles.gpu.debug-readback" => (8 * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
        "particles.gpu.emitter-params" => 256,
        _ => MIN_PLUGIN_EXTERNAL_BUFFER_SIZE,
    }
}

const FIRST_PARTY_PLUGIN_EXTERNAL_BUFFERS: &[&str] = &[
    "particles.gpu.particles-a",
    "particles.gpu.emitter-params",
    "particles.gpu.particles-b",
    "particles.gpu.counters",
    "particles.gpu.alive-indices",
    "particles.gpu.indirect-draw-args",
    "particles.gpu.debug-readback",
    "virtual-geometry-feedback",
];
const PARTICLE_PLUGIN_EXTERNAL_BUFFERS: &[&str] = &[
    "particles.gpu.particles-a",
    "particles.gpu.emitter-params",
    "particles.gpu.particles-b",
    "particles.gpu.counters",
    "particles.gpu.alive-indices",
    "particles.gpu.indirect-draw-args",
    "particles.gpu.debug-readback",
];

#[cfg(test)]
#[path = "tests/bind_plugin_graph_resources.rs"]
mod tests;
