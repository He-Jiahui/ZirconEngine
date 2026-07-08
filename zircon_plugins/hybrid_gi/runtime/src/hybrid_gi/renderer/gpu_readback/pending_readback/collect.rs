use super::super::decode::read_buffer_u32s::read_buffer_u32s;
use zircon_runtime::graphics::GraphicsError;

use super::super::decode::{
    cache_entries, completed_probe_ids, completed_trace_region_ids, probe_irradiance_rgb,
    probe_trace_lighting_rgb,
};
use super::super::readback::HybridGiGpuReadback;
use super::HybridGiGpuPendingReadback;

fn texture_sample_rgba(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
) -> Result<[u8; 4], GraphicsError> {
    let sample = read_buffer_u32s(device, buffer, 1)?
        .into_iter()
        .next()
        .unwrap_or_default();
    Ok(sample.to_le_bytes())
}

fn probe_trace_tiles_from_readback(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
    record_count: usize,
) -> Result<Vec<(u32, u32, u32, u32)>, GraphicsError> {
    let words = read_buffer_u32s(device, buffer, word_count)?;
    Ok(words
        .chunks_exact(4)
        .take(record_count)
        .map(|record| (record[0], record[1], record[2], record[3]))
        .collect())
}

fn probe_trace_indirect_args_from_readback(
    device: &wgpu::Device,
    buffer: &wgpu::Buffer,
    word_count: usize,
) -> Result<Vec<u32>, GraphicsError> {
    read_buffer_u32s(device, buffer, word_count)
}

fn probe_trace_dispatch_from_indirect_args(
    indirect_args: &[u32],
    fallback_tile_count: usize,
) -> [u32; 3] {
    let tile_count = indirect_args
        .first()
        .copied()
        .unwrap_or(fallback_tile_count as u32);
    if tile_count == 0 {
        [0; 3]
    } else {
        [
            indirect_args.get(2).copied().unwrap_or(1).max(1),
            indirect_args.get(3).copied().unwrap_or(1).max(1),
            tile_count,
        ]
    }
}

impl HybridGiGpuPendingReadback {
    pub(in crate::hybrid_gi::renderer) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<HybridGiGpuReadback, GraphicsError> {
        let scene_prepare_resources = self
            .scene_prepare_resources
            .map(|mut snapshot| -> Result<_, GraphicsError> {
                let atlas_slot_rgba_samples = self
                    .scene_prepare_atlas_slot_sample_buffers
                    .iter()
                    .map(|(slot_id, buffer)| {
                        texture_sample_rgba(device, buffer).map(|rgba| (*slot_id, rgba))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let capture_slot_rgba_samples = self
                    .scene_prepare_capture_slot_sample_buffers
                    .iter()
                    .map(|(slot_id, buffer)| {
                        texture_sample_rgba(device, buffer).map(|rgba| (*slot_id, rgba))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                snapshot.store_texture_slot_rgba_samples(
                    atlas_slot_rgba_samples,
                    capture_slot_rgba_samples,
                );
                if !self
                    .scene_prepare_surface_cache_depth_slot_sample_buffers
                    .is_empty()
                {
                    let surface_cache_depth_rgba_samples = self
                        .scene_prepare_surface_cache_depth_slot_sample_buffers
                        .iter()
                        .map(|(slot_id, buffer)| {
                            texture_sample_rgba(device, buffer).map(|rgba| (*slot_id, rgba))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    snapshot.store_surface_cache_depth_samples(surface_cache_depth_rgba_samples);
                }
                if let Some(buffer) = self.scene_prepare_probe_trace_tile_readback.as_ref() {
                    let probe_trace_tiles = probe_trace_tiles_from_readback(
                        device,
                        buffer,
                        self.scene_prepare_probe_trace_tile_word_count,
                        self.scene_prepare_probe_trace_tile_record_count,
                    )?;
                    let probe_trace_dispatch = match self
                        .scene_prepare_probe_trace_indirect_args_readback
                        .as_ref()
                    {
                        Some(indirect_args_buffer) => {
                            let indirect_args = probe_trace_indirect_args_from_readback(
                                device,
                                indirect_args_buffer,
                                self.scene_prepare_probe_trace_indirect_arg_word_count,
                            )?;
                            probe_trace_dispatch_from_indirect_args(
                                &indirect_args,
                                probe_trace_tiles.len(),
                            )
                        }
                        None => {
                            probe_trace_dispatch_from_indirect_args(&[], probe_trace_tiles.len())
                        }
                    };
                    snapshot.store_probe_trace_tiles(probe_trace_tiles, probe_trace_dispatch);
                }
                Ok::<_, GraphicsError>(snapshot)
            })
            .transpose()?;

        Ok(HybridGiGpuReadback::new(
            cache_entries(device, &self.cache_buffer, self.cache_word_count)?,
            completed_probe_ids(
                device,
                &self.completed_probe_buffer,
                self.completed_probe_word_count,
            )?,
            completed_trace_region_ids(
                device,
                &self.completed_trace_buffer,
                self.completed_trace_word_count,
            )?,
            probe_irradiance_rgb(device, &self.irradiance_buffer, self.irradiance_word_count)?,
            probe_trace_lighting_rgb(
                device,
                &self.trace_lighting_buffer,
                self.trace_lighting_word_count,
            )?,
            scene_prepare_resources,
        ))
    }
}
