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
