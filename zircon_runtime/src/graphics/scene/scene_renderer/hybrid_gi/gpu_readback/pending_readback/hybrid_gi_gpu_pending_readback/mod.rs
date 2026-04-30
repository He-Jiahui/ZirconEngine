mod scene_prepare_resources_access;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

pub(in crate::graphics::scene::scene_renderer) struct HybridGiGpuPendingReadback {
    pub(super) cache_word_count: usize,
    pub(super) cache_buffer: wgpu::Buffer,
    pub(super) completed_probe_word_count: usize,
    pub(super) completed_probe_buffer: wgpu::Buffer,
    pub(super) completed_trace_word_count: usize,
    pub(super) completed_trace_buffer: wgpu::Buffer,
    pub(super) irradiance_word_count: usize,
    pub(super) irradiance_buffer: wgpu::Buffer,
    pub(super) trace_lighting_word_count: usize,
    pub(super) trace_lighting_buffer: wgpu::Buffer,
    pub(super) scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
    pub(super) _scene_prepare_atlas_texture: Option<wgpu::Texture>,
    pub(super) _scene_prepare_atlas_view: Option<wgpu::TextureView>,
    pub(super) _scene_prepare_atlas_upload_buffer: Option<wgpu::Buffer>,
    pub(super) scene_prepare_atlas_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
    pub(super) _scene_prepare_capture_texture: Option<wgpu::Texture>,
    pub(super) _scene_prepare_capture_views: Vec<wgpu::TextureView>,
    pub(super) _scene_prepare_capture_upload_buffer: Option<wgpu::Buffer>,
    pub(super) scene_prepare_capture_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
}
