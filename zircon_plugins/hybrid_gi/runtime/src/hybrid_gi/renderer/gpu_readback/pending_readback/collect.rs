use zircon_runtime::core::framework::render::RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT;
use zircon_runtime::graphics::{GraphicsError, RuntimeGpuReadback};

use super::super::decode::{
    cache_entries, completed_probe_ids, completed_trace_region_ids, probe_irradiance_rgb,
    probe_trace_diagnostics, probe_trace_lighting_rgb, read_buffer_u32s::read_buffer_u32s,
};
use super::super::readback::HybridGiGpuReadback;
use super::hybrid_gi_gpu_readback_future::HybridGiGpuReadbackFuture;

impl HybridGiGpuReadbackFuture {
    pub(in crate::hybrid_gi::renderer) fn try_collect(
        self,
    ) -> Option<Result<HybridGiGpuReadback, GraphicsError>> {
        if !self.is_ready() {
            return None;
        }
        Some(self.collect_ready())
    }

    fn collect_ready(self) -> Result<HybridGiGpuReadback, GraphicsError> {
        let trace_diagnostic_word_count = self.trace_diagnostics.word_count;
        let trace_diagnostics =
            probe_trace_diagnostics(&self.trace_diagnostics.take()?, trace_diagnostic_word_count)?;
        let mut scene_prepare_resources = self.scene_prepare_resources;
        if let Some(snapshot) = scene_prepare_resources.as_mut() {
            snapshot.store_probe_trace_diagnostics(trace_diagnostics);
            snapshot.store_texture_slot_rgba_samples(
                take_slot_samples(self.atlas_slot_samples)?,
                take_slot_samples(self.capture_slot_samples)?,
            );
            if !self.surface_cache_depth_slot_samples.is_empty() {
                snapshot.store_surface_cache_depth_samples(take_slot_samples(
                    self.surface_cache_depth_slot_samples,
                )?);
            }
            if let Some(trace_tiles) = self.probe_trace_tiles {
                let trace_tile_word_count = trace_tiles.word_count;
                let words = read_buffer_u32s(&trace_tiles.take()?, trace_tile_word_count)?;
                let tiles = words
                    .chunks_exact(4)
                    .take(self.probe_trace_tile_record_count)
                    .map(|record| (record[0], record[1], record[2], record[3]))
                    .collect::<Vec<_>>();
                let indirect_args = self
                    .probe_trace_indirect_args
                    .map(|readback| {
                        let word_count = readback.word_count;
                        read_buffer_u32s(&readback.take()?, word_count)
                    })
                    .transpose()?
                    .unwrap_or_default();
                snapshot.store_probe_trace_tiles(
                    tiles.clone(),
                    probe_trace_dispatch(&indirect_args, tiles.len()),
                );
            }
        }

        let cache_word_count = self.cache.word_count;
        let completed_probe_word_count = self.completed_probes.word_count;
        let completed_trace_word_count = self.completed_traces.word_count;
        let irradiance_word_count = self.irradiance.word_count;
        let trace_lighting_word_count = self.trace_lighting.word_count;
        let radiance_cache_dispatch_word_count = self.radiance_cache_dispatch_counts.word_count;
        let radiance_cache_gpu_stage_dispatch_counts = read_buffer_u32s(
            &self.radiance_cache_dispatch_counts.take()?,
            radiance_cache_dispatch_word_count,
        )?
        .try_into()
        .map_err(|counts: Vec<u32>| {
            GraphicsError::BufferMap(format!(
                "hybrid GI radiance-cache dispatch readback returned {} words, expected {}",
                counts.len(),
                RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
            ))
        })?;
        Ok(HybridGiGpuReadback::new(
            cache_entries(&self.cache.take()?, cache_word_count)?,
            completed_probe_ids(&self.completed_probes.take()?, completed_probe_word_count)?,
            completed_trace_region_ids(&self.completed_traces.take()?, completed_trace_word_count)?,
            probe_irradiance_rgb(&self.irradiance.take()?, irradiance_word_count)?,
            probe_trace_lighting_rgb(&self.trace_lighting.take()?, trace_lighting_word_count)?,
            scene_prepare_resources,
        )
        .with_radiance_cache_gpu_stage_dispatch_counts(radiance_cache_gpu_stage_dispatch_counts))
    }
}

fn take_slot_samples(
    readbacks: Vec<(u32, RuntimeGpuReadback)>,
) -> Result<Vec<(u32, [u8; 4])>, GraphicsError> {
    readbacks
        .into_iter()
        .map(|(slot_id, readback)| {
            let bytes = readback
                .try_take()
                .expect("ready hybrid GI slot sample remains available")?;
            let rgba = bytes.get(..4).ok_or_else(|| {
                GraphicsError::BufferMap(format!(
                    "hybrid GI slot {slot_id} returned fewer than four bytes"
                ))
            })?;
            Ok((slot_id, [rgba[0], rgba[1], rgba[2], rgba[3]]))
        })
        .collect()
}

fn probe_trace_dispatch(indirect_args: &[u32], fallback_tile_count: usize) -> [u32; 3] {
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
