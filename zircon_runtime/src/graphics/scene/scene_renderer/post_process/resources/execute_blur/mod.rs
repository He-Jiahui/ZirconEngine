use crate::core::framework::render::RenderPipelinePhase;
use crate::core::math::UVec2;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;
use zr_rhi_wgpu::WgpuBufferUploadBatch;

use super::super::scene_post_process_resources::ScenePostProcessResources;
use super::super::scene_runtime_feature_flags::SceneRuntimeFeatureFlags;
use super::execute_post_process::{
    build_post_process_params, create_bind_group, post_process_params_upload,
};

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn execute_blur(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        cluster_dimensions: UVec2,
        scene_color_origin: [u32; 2],
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        exposure_buffer: wgpu::BufferBinding<'_>,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> WgpuBufferUploadBatch {
        let render_region = frame
            .render_region_for_phase(RenderPipelinePhase::PostReconstructionScenePostProcess)
            .expect("blur requires the post-reconstruction phase");
        let mut params = build_post_process_params(
            render_region.local_size(),
            cluster_dimensions,
            render_region,
            scene_color_origin,
            &frame.extract,
            frame.post_process(),
            SceneRuntimeFeatureFlags::default(),
            false,
            0,
            0,
            0,
            false,
        );
        params.effect_blur_dof[1] = 0.0;
        params.effect_blur_dof[2] = 0.0;
        params.effect_blur_dof[3] = 0.0;
        params.effect_dof_lens = [0.0; 4];
        let params_buffer = &self.post_process_pass_parameter_buffers.blur;
        let params_uploads = post_process_params_upload(params_buffer, &params);
        let bind_group = create_bind_group(
            self,
            device,
            params_buffer,
            scene_color_view,
            scene_depth_view,
            &self.black_texture_view,
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
            label: Some("BlurPass"),
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
        if !render_region.apply_local_to_render_pass(&mut pass) {
            return params_uploads;
        }
        pass.set_pipeline(&self.blur_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        params_uploads
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::PostProcessGraphResourceNames;

    #[test]
    fn blur_pass_writes_dedicated_intermediate_resource() {
        assert_eq!(
            PostProcessGraphResourceNames::BLURRED,
            "postprocess.blurred"
        );
    }

    #[test]
    fn blur_params_are_returned_as_pre_submit_uploads() {
        let source = include_str!("mod.rs");
        let production = source.split("#[cfg(test)]").next().expect("blur source");

        assert!(!production.contains("queue.write_buffer"));
        assert!(!production.contains("create_post_process_params_buffer"));
        assert!(production.contains("post_process_params_upload("));
        assert!(production.contains("WgpuBufferUploadBatch"));
    }
}
