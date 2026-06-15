use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::super::execute_post_process::{build_post_process_params, create_bind_group};

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_screen_space_reflection_resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        cluster_dimensions: UVec2,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        motion_vector_neighbor_max_view: &wgpu::TextureView,
        scene_normal_view: &wgpu::TextureView,
        scene_material_view: Option<&wgpu::TextureView>,
        ao_view: &wgpu::TextureView,
        previous_scene_color_view: Option<&wgpu::TextureView>,
        previous_global_illumination_view: Option<&wgpu::TextureView>,
        previous_screen_space_reflection_history_view: Option<&wgpu::TextureView>,
        bloom_view: &wgpu::TextureView,
        depth_of_field_coc_view: &wgpu::TextureView,
        depth_of_field_bokeh_view: &wgpu::TextureView,
        screen_space_reflection_history_view: &wgpu::TextureView,
        screen_space_reflection_specular_occlusion_view: &wgpu::TextureView,
        screen_space_reflection_depth_pyramid_view: &wgpu::TextureView,
        screen_space_reflection_reflection_pyramid_view: &wgpu::TextureView,
        screen_space_reflection_depth_pyramid_coarse_view: &wgpu::TextureView,
        screen_space_reflection_reflection_pyramid_coarse_view: &wgpu::TextureView,
        cluster_buffer: &wgpu::Buffer,
        frame: &ViewportRenderFrame,
        features: SceneRuntimeFeatureFlags,
        history_available: bool,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let params = build_post_process_params(
            viewport_size,
            cluster_dimensions,
            &frame.extract,
            features,
            history_available,
            0,
            0,
            0,
        );
        queue.write_buffer(
            &self.post_process_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );

        let bind_group = create_bind_group(
            self,
            device,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ao_view,
            self.white_texture_view(),
            previous_scene_color_view,
            previous_global_illumination_view,
            previous_screen_space_reflection_history_view,
            // The resolve pass writes the current SSR history target, so binding 21
            // must stay on the fallback view instead of sampling its color attachment.
            None,
            Some(screen_space_reflection_specular_occlusion_view),
            Some(screen_space_reflection_depth_pyramid_view),
            Some(screen_space_reflection_reflection_pyramid_view),
            Some(screen_space_reflection_depth_pyramid_coarse_view),
            Some(screen_space_reflection_reflection_pyramid_coarse_view),
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            &self.effect_lut_texture_view,
            &self.effect_lut_texture_3d_view,
            cluster_buffer,
            self.default_exposure_buffer(),
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ScreenSpaceReflectionResolvePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: screen_space_reflection_history_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.screen_space_reflection_resolve_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
