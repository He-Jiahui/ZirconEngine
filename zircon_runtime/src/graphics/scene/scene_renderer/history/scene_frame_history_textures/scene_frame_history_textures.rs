use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::temporal::taa::{
    TemporalHistoryKey, TemporalHistoryStore,
};

pub(crate) struct SceneFrameHistoryTextures {
    pub(crate) size: UVec2,
    pub(crate) hzb_furthest_size: UVec2,
    pub(crate) hzb_furthest_mip_count: u32,
    pub(super) taa_scene_color: TemporalHistoryStore,
    pub(crate) global_illumination: wgpu::Texture,
    pub(crate) global_illumination_view: wgpu::TextureView,
    pub(crate) global_illumination_temporal_metadata: wgpu::Texture,
    pub(crate) global_illumination_temporal_metadata_view: wgpu::TextureView,
    pub(super) global_illumination_history_valid: bool,
    pub(crate) ambient_occlusion: wgpu::Texture,
    pub(crate) ambient_occlusion_view: wgpu::TextureView,
    pub(crate) screen_space_reflection: wgpu::Texture,
    pub(crate) screen_space_reflection_view: wgpu::TextureView,
    pub(crate) hzb_furthest: wgpu::Texture,
    pub(crate) hzb_furthest_view: wgpu::TextureView,
    pub(super) exposure_read: wgpu::Buffer,
    pub(super) exposure_write: wgpu::Buffer,
}

impl SceneFrameHistoryTextures {
    pub(crate) fn global_illumination_history_valid(&self) -> bool {
        self.global_illumination_history_valid
    }

    pub(crate) fn set_global_illumination_history_valid(&mut self, valid: bool) {
        self.global_illumination_history_valid = valid;
    }

    pub(crate) fn taa_scene_color_history_matches(&self, key: TemporalHistoryKey) -> bool {
        self.taa_scene_color.matches_key(key)
    }

    pub(crate) fn taa_scene_color_history_valid(&self) -> bool {
        self.taa_scene_color.is_valid()
    }

    pub(crate) fn invalidate_taa_scene_color_history(&mut self) {
        self.taa_scene_color.invalidate();
    }

    pub(crate) fn flip_taa_scene_color_history(&mut self) {
        self.taa_scene_color.flip_after_success();
    }

    pub(crate) fn taa_scene_color_previous_view(&self) -> wgpu::TextureView {
        self.taa_scene_color.previous_view().clone()
    }

    pub(crate) fn taa_scene_color_current_view(&self) -> wgpu::TextureView {
        self.taa_scene_color.current_view().clone()
    }

    pub(crate) fn invalidate_exposure_history(&mut self, queue: &wgpu::Queue) {
        let default_words =
            crate::graphics::scene::scene_renderer::post_process::params::exposure_params::default_exposure_buffer_words();
        queue.write_buffer(&self.exposure_read, 0, bytemuck::cast_slice(&default_words));
        queue.write_buffer(
            &self.exposure_write,
            0,
            bytemuck::cast_slice(&default_words),
        );
    }

    pub(crate) fn flip_exposure_history(&mut self) {
        std::mem::swap(&mut self.exposure_read, &mut self.exposure_write);
    }

    pub(crate) fn exposure_previous_buffer(&self) -> wgpu::Buffer {
        self.exposure_read.clone()
    }

    pub(crate) fn exposure_current_buffer(&self) -> wgpu::Buffer {
        self.exposure_write.clone()
    }
}
