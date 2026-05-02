use super::super::super::gpu_pending_probe_input::GpuPendingProbeInput;
use super::super::super::gpu_resident_probe_input::GpuResidentProbeInput;
use super::super::super::gpu_trace_region_input::GpuTraceRegionInput;
use crate::hybrid_gi::types::{
    HybridGiPrepareCardCaptureRequest, HybridGiPrepareSurfaceCachePageContent,
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap,
};
use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
    RenderSpotLightSnapshot,
};

pub(super) struct HybridGiPrepareExecutionInputs {
    pub(super) cache_entries: Vec<[u32; 2]>,
    pub(super) resident_probe_inputs: Vec<GpuResidentProbeInput>,
    pub(super) pending_probe_inputs: Vec<GpuPendingProbeInput>,
    pub(super) trace_region_inputs: Vec<GpuTraceRegionInput>,
    pub(super) scene_card_capture_requests: Vec<HybridGiPrepareCardCaptureRequest>,
    pub(super) scene_surface_cache_page_contents: Vec<HybridGiPrepareSurfaceCachePageContent>,
    pub(super) scene_card_capture_descriptor_count: usize,
    pub(super) scene_voxel_clipmaps: Vec<HybridGiPrepareVoxelClipmap>,
    pub(super) scene_voxel_cells: Vec<HybridGiPrepareVoxelCell>,
    pub(super) scene_meshes: Vec<RenderMeshSnapshot>,
    pub(super) directional_lights: Vec<RenderDirectionalLightSnapshot>,
    pub(super) point_lights: Vec<RenderPointLightSnapshot>,
    pub(super) spot_lights: Vec<RenderSpotLightSnapshot>,
    pub(super) cache_word_count: usize,
    pub(super) completed_probe_word_count: usize,
    pub(super) completed_trace_word_count: usize,
    pub(super) irradiance_word_count: usize,
    pub(super) trace_lighting_word_count: usize,
}
