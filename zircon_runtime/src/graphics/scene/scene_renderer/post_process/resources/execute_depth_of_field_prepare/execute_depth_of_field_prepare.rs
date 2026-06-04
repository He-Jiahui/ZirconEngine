use super::super::super::clear_render_target::clear_render_target;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;

impl ScenePostProcessResources {
    pub(crate) fn execute_depth_of_field_prepare(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        coc_view: &wgpu::TextureView,
        bokeh_view: &wgpu::TextureView,
    ) {
        clear_render_target(
            encoder,
            "DepthOfFieldCocPreparePass",
            coc_view,
            wgpu::Color::BLACK,
        );
        clear_render_target(
            encoder,
            "DepthOfFieldBokehPreparePass",
            bokeh_view,
            wgpu::Color::BLACK,
        );
    }
}
