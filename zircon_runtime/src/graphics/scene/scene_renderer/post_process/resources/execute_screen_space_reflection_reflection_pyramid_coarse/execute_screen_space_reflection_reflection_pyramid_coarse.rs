use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::super::execute_post_process::{
    build_post_process_params, create_bind_group, post_process_params_upload,
};

impl ScenePostProcessResources {
    pub(crate) fn prepare_screen_space_reflection_reflection_pyramid_coarse_params(
        &self,
        cluster_dimensions: UVec2,
        scene_color_origin: [u32; 2],
        frame: &ViewportRenderFrame,
        features: SceneRuntimeFeatureFlags,
        history_available: bool,
    ) -> WgpuBufferUploadBatch {
        let render_region = frame
            .render_region_for_phase(
                crate::core::framework::render::RenderPipelinePhase::
                    PostReconstructionScenePostProcess,
            )
            .expect("SSR coarse pyramid requires post-reconstruction phase");
        let params = build_post_process_params(
            render_region.local_size(),
            cluster_dimensions,
            render_region,
            scene_color_origin,
            &frame.extract,
            frame.post_process(),
            features,
            history_available,
            0,
            0,
            0,
            false,
        );
        post_process_params_upload(
            &self
                .post_process_pass_parameter_buffers
                .reflection_pyramid_coarse,
            &params,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_screen_space_reflection_reflection_pyramid_coarse(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
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
        screen_space_reflection_reflection_pyramid_view: &wgpu::TextureView,
        screen_space_reflection_reflection_pyramid_coarse_view: &wgpu::TextureView,
        cluster_buffer: wgpu::BufferBinding<'_>,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let params_buffer = &self
            .post_process_pass_parameter_buffers
            .reflection_pyramid_coarse;

        let bind_group = create_bind_group(
            self,
            device,
            params_buffer,
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
            None,
            None,
            None,
            Some(screen_space_reflection_reflection_pyramid_view),
            None,
            // The coarse reflection-pyramid pass writes binding 26's target,
            // so it binds the neutral fallback instead of sampling the attachment.
            None,
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            &self.effect_lut_texture_view,
            &self.effect_lut_texture_3d_view,
            cluster_buffer,
            self.default_exposure_buffer_binding(),
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ScreenSpaceReflectionReflectionPyramidCoarsePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: screen_space_reflection_reflection_pyramid_coarse_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.screen_space_reflection_reflection_pyramid_coarse_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn coarse_pyramid_params_have_one_producer_outside_the_mip_loop() {
        let source = include_str!(
            "../../../graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection.rs"
        );
        let method = source
            .split("fn record_screen_space_reflection_reflection_pyramid_coarse_to_resource")
            .nth(1)
            .expect("coarse-pyramid graph method")
            .split("fn record_screen_space_reflection_reflection_pyramid_to_resource")
            .next()
            .expect("coarse-pyramid graph method end");

        assert_eq!(
            method
                .matches("prepare_screen_space_reflection_reflection_pyramid_coarse_params(")
                .count(),
            1
        );
        assert!(method.contains("self.append_pre_submit_buffer_uploads("));
    }
}
