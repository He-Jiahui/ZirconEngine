use crate::graphics::types::ViewportRenderRegion;

use wgpu::util::DeviceExt;

use super::super::post_process_params::TerminalRegionParams;

pub(super) fn apply_local_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_local_to_render_pass(pass)
}

pub(super) fn apply_physical_render_region_to_pass(
    pass: &mut wgpu::RenderPass<'_>,
    render_region: ViewportRenderRegion,
) -> bool {
    render_region.apply_physical_to_render_pass(pass)
}

pub(super) fn create_local_terminal_region_params_buffer(
    device: &wgpu::Device,
    label: &'static str,
    render_region: ViewportRenderRegion,
) -> wgpu::Buffer {
    let origin = render_region.local_position();
    let params = TerminalRegionParams {
        viewport_origin: [origin[0], origin[1], 0, 0],
    };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}

pub(super) fn create_physical_terminal_region_params_buffer(
    device: &wgpu::Device,
    label: &'static str,
    render_region: ViewportRenderRegion,
) -> wgpu::Buffer {
    let origin = render_region.physical_origin();
    let params = TerminalRegionParams {
        viewport_origin: [origin[0], origin[1], 0, 0],
    };
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::bytes_of(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    })
}
