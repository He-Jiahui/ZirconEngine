use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::execute_post_process::{
    build_post_process_params, create_bind_group, create_post_process_params_buffer,
};

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn execute_motion_blur(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        cluster_dimensions: UVec2,
        scene_color_origin: [u32; 2],
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        motion_vector_neighbor_max_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        exposure_buffer: &wgpu::Buffer,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
    ) {
        let params = build_post_process_params(
            frame.extract.view.effective_render_size(),
            cluster_dimensions,
            frame.render_region(),
            scene_color_origin,
            &frame.extract,
            SceneRuntimeFeatureFlags::default(),
            false,
            0,
            0,
            0,
        );
        let params_buffer =
            create_post_process_params_buffer(device, queue, "zircon-motion-blur-params", &params);
        let bind_group = create_bind_group(
            self,
            device,
            &params_buffer,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            &self.black_texture_view,
            None,
            &self.white_texture_view,
            &self.white_texture_view,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            &self.black_texture_view,
            &self.black_texture_view,
            &self.black_texture_view,
            &self.effect_lut_texture_view,
            &self.effect_lut_texture_3d_view,
            &self.light_buffer,
            exposure_buffer,
        );
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MotionBlurPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.motion_blur_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::PostProcessGraphResourceNames;

    #[test]
    fn motion_blur_pass_writes_dedicated_intermediate_resource() {
        assert_eq!(
            PostProcessGraphResourceNames::MOTION_BLURRED,
            "postprocess.motion-blurred"
        );
    }
}
