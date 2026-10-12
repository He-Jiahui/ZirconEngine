use std::sync::Arc;

use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

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

/// 返回的批次保有三个上传范围共用的不可变字节；目标写入沿用各 binding 的 offset，由调用方追加到帧提交队列。
pub(crate) fn prepare_light_grid_buffer_uploads(
    light_grid_params_buffer: wgpu::BufferBinding<'_>,
    light_zbins_buffer: wgpu::BufferBinding<'_>,
    light_tile_masks_buffer: wgpu::BufferBinding<'_>,
    light_grid: &LightGridCpuOutput,
) -> WgpuBufferUploadBatch {
    let params = bytemuck::bytes_of(&light_grid.params);
    let zbins = bytemuck::cast_slice(&light_grid.zbins);
    let tile_masks = bytemuck::cast_slice(&light_grid.tile_masks);
    let mut bytes = Vec::with_capacity(
        params
            .len()
            .saturating_add(zbins.len())
            .saturating_add(tile_masks.len()),
    );
    let params_start = bytes.len();
    bytes.extend_from_slice(params);
    let zbins_start = bytes.len();
    bytes.extend_from_slice(zbins);
    let tile_masks_start = bytes.len();
    bytes.extend_from_slice(tile_masks);
    let payload: Arc<[u8]> = bytes.into();

    let mut uploads = WgpuBufferUploadBatch::new();
    for (binding, source_range) in [
        (light_grid_params_buffer, params_start..zbins_start),
        (light_zbins_buffer, zbins_start..tile_masks_start),
        (light_tile_masks_buffer, tile_masks_start..payload.len()),
    ] {
        if let Some(upload) = WgpuBufferUpload::new(
            binding.buffer.clone(),
            binding.offset,
            payload.clone(),
            source_range,
        ) {
            uploads.push(upload);
        }
    }
    uploads
}

#[cfg(test)]
#[path = "tests/light_grid_pass.rs"]
mod tests;
