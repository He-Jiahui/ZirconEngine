use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;

use super::light_buffer::pack_lighting_extract_with_cookies;
use super::light_grid_builder::{build_light_grid, LightGridCpuOutput, LightGridViewInfo};

pub(crate) fn build_light_grid_for_frame(
    extract: &RenderFrameExtract,
    viewport_size: UVec2,
    lighting_enabled: bool,
) -> LightGridCpuOutput {
    let packed_lights = pack_lighting_extract_with_cookies(
        &extract.lighting,
        &extract.lighting.advanced_lighting.cookies,
        lighting_enabled,
    );
    let view = LightGridViewInfo::from_camera(&extract.view.camera, viewport_size);
    build_light_grid(&packed_lights.lights, &view)
}

pub(crate) fn write_light_grid_buffers(
    queue: &wgpu::Queue,
    light_grid_params_buffer: &wgpu::Buffer,
    light_zbins_buffer: &wgpu::Buffer,
    light_tile_masks_buffer: &wgpu::Buffer,
    light_grid: &LightGridCpuOutput,
) {
    queue.write_buffer(
        light_grid_params_buffer,
        0,
        bytemuck::bytes_of(&light_grid.params),
    );
    queue.write_buffer(
        light_zbins_buffer,
        0,
        bytemuck::cast_slice(&light_grid.zbins),
    );
    queue.write_buffer(
        light_tile_masks_buffer,
        0,
        bytemuck::cast_slice(&light_grid.tile_masks),
    );
}
