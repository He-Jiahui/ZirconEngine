use super::HybridGiScenePrepareResourcesSnapshot;
use zircon_runtime::core::framework::render::RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HybridGiGpuReadback {
    pub(super) cache_entries: Vec<(u32, u32)>,
    pub(super) completed_probe_ids: Vec<u32>,
    pub(super) completed_trace_region_ids: Vec<u32>,
    pub(super) probe_irradiance_rgb: Vec<(u32, [u8; 3])>,
    pub(super) probe_trace_lighting_rgb: Vec<(u32, [u8; 3])>,
    pub(super) radiance_cache_gpu_stage_dispatch_counts:
        [u32; RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT],
    pub(super) scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
}
