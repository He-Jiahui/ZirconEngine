use std::sync::Arc;

use crate::core::framework::render::RenderFrameExtract;
use crate::core::math::UVec2;
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::light_buffer::pack_lighting_extract_with_cookies;
use super::light_grid_builder::{LightGridCpuOutput, LightGridViewInfo, build_light_grid};

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
mod tests {
    #[test]
    fn light_grid_uses_one_packed_payload_and_no_direct_queue_write() {
        let source = include_str!("light_grid_pass.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("light-grid production source");

        assert!(!production.contains("queue.write_buffer"));
        assert_eq!(production.matches("let payload: Arc<[u8]>").count(), 1);
        assert_eq!(production.matches("WgpuBufferUpload::new(").count(), 1);
        assert!(production.contains("light_grid_params_buffer"));
        assert!(production.contains("light_zbins_buffer"));
        assert!(production.contains("light_tile_masks_buffer"));
        assert!(production.contains("binding.offset"));
        assert!(production.contains("BufferBinding<'_>"));
    }
}
