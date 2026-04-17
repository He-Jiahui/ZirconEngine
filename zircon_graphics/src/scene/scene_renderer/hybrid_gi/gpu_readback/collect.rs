use crate::types::GraphicsError;

use super::cache_entries::cache_entries;
use super::completed_probe_ids::completed_probe_ids;
use super::completed_trace_region_ids::completed_trace_region_ids;
use super::hybrid_gi_gpu_pending_readback::HybridGiGpuPendingReadback;
use super::hybrid_gi_gpu_readback::HybridGiGpuReadback;
use super::probe_irradiance_rgb::probe_irradiance_rgb;

impl HybridGiGpuPendingReadback {
    pub(crate) fn collect(
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
        })
    }
}
