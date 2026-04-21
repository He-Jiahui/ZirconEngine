use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;

use super::HybridGiGpuPendingReadback;

impl HybridGiGpuPendingReadback {
    pub(in crate::graphics::scene::scene_renderer::hybrid_gi) fn new(
        cache_word_count: usize,
        cache_buffer: wgpu::Buffer,
        completed_probe_word_count: usize,
        completed_probe_buffer: wgpu::Buffer,
        completed_trace_word_count: usize,
        completed_trace_buffer: wgpu::Buffer,
        irradiance_word_count: usize,
        irradiance_buffer: wgpu::Buffer,
        trace_lighting_word_count: usize,
        trace_lighting_buffer: wgpu::Buffer,
        scene_prepare_resources: Option<HybridGiScenePrepareResourcesSnapshot>,
        scene_prepare_atlas_texture: Option<wgpu::Texture>,
        scene_prepare_atlas_view: Option<wgpu::TextureView>,
        scene_prepare_atlas_upload_buffer: Option<wgpu::Buffer>,
        scene_prepare_atlas_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
        scene_prepare_capture_texture: Option<wgpu::Texture>,
        scene_prepare_capture_views: Vec<wgpu::TextureView>,
        scene_prepare_capture_upload_buffer: Option<wgpu::Buffer>,
        scene_prepare_capture_slot_sample_buffers: Vec<(u32, wgpu::Buffer)>,
    ) -> Self {
        Self {
            cache_word_count,
            cache_buffer,
            completed_probe_word_count,
            completed_probe_buffer,
            completed_trace_word_count,
            completed_trace_buffer,
            irradiance_word_count,
            irradiance_buffer,
            trace_lighting_word_count,
            trace_lighting_buffer,
            scene_prepare_resources,
            _scene_prepare_atlas_texture: scene_prepare_atlas_texture,
            _scene_prepare_atlas_view: scene_prepare_atlas_view,
            _scene_prepare_atlas_upload_buffer: scene_prepare_atlas_upload_buffer,
            scene_prepare_atlas_slot_sample_buffers,
            _scene_prepare_capture_texture: scene_prepare_capture_texture,
            _scene_prepare_capture_views: scene_prepare_capture_views,
            _scene_prepare_capture_upload_buffer: scene_prepare_capture_upload_buffer,
            scene_prepare_capture_slot_sample_buffers,
        }
    }
}
