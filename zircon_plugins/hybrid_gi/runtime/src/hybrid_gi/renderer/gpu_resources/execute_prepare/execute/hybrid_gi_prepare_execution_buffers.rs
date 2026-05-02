use crate::hybrid_gi::renderer::HybridGiScenePrepareResourcesSnapshot;

pub(super) struct HybridGiPrepareScenePrepareResources {
    pub(super) snapshot: HybridGiScenePrepareResourcesSnapshot,
    pub(super) atlas_texture: Option<wgpu::Texture>,
    pub(super) atlas_view: Option<wgpu::TextureView>,
    pub(super) atlas_upload_buffer: Option<wgpu::Buffer>,
    pub(super) atlas_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
    pub(super) capture_texture: Option<wgpu::Texture>,
    pub(super) capture_views: Vec<wgpu::TextureView>,
    pub(super) capture_upload_buffer: Option<wgpu::Buffer>,
    pub(super) capture_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
}

pub(super) struct HybridGiPrepareExecutionBuffers {
    pub(super) cache_readback: wgpu::Buffer,
    pub(super) resident_probe_buffer: wgpu::Buffer,
    pub(super) pending_probe_buffer: wgpu::Buffer,
    pub(super) trace_region_buffer: wgpu::Buffer,
    pub(super) scene_prepare_descriptor_buffer: wgpu::Buffer,
    pub(super) completed_probe_buffer: wgpu::Buffer,
    pub(super) completed_trace_buffer: wgpu::Buffer,
    pub(super) completed_probe_readback: wgpu::Buffer,
    pub(super) completed_trace_readback: wgpu::Buffer,
    pub(super) irradiance_buffer: wgpu::Buffer,
    pub(super) irradiance_readback: wgpu::Buffer,
    pub(super) trace_lighting_buffer: wgpu::Buffer,
    pub(super) trace_lighting_readback: wgpu::Buffer,
    pub(super) scene_prepare_resources: Option<HybridGiPrepareScenePrepareResources>,
}
