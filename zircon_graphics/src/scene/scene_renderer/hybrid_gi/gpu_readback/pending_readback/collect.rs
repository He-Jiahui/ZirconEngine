use crate::types::GraphicsError;

use super::super::decode::{
    cache_entries, completed_probe_ids, completed_trace_region_ids, probe_irradiance_rgb,
    probe_trace_lighting_rgb,
};
use super::super::readback::HybridGiGpuReadback;
use super::HybridGiGpuPendingReadback;

impl HybridGiGpuPendingReadback {
    pub(in crate::scene::scene_renderer) fn collect(
        self,
        device: &wgpu::Device,
    ) -> Result<HybridGiGpuReadback, GraphicsError> {
        Ok(HybridGiGpuReadback {
            cache_entries: cache_entries(device, &self.cache_buffer, self.cache_word_count)?,
            completed_probe_ids: completed_probe_ids(
                device,
                &self.completed_probe_buffer,
                self.completed_probe_word_count,
            )?,
            completed_trace_region_ids: completed_trace_region_ids(
                device,
                &self.completed_trace_buffer,
                self.completed_trace_word_count,
            )?,
            probe_irradiance_rgb: probe_irradiance_rgb(
                device,
                &self.irradiance_buffer,
                self.irradiance_word_count,
            )?,
            probe_trace_lighting_rgb: probe_trace_lighting_rgb(
                device,
                &self.trace_lighting_buffer,
                self.trace_lighting_word_count,
            )?,
        })
    }
}
