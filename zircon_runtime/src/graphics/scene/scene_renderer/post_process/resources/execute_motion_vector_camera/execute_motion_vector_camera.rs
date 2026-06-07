use crate::core::framework::render::{MotionVectorCameraStatus, ViewportCameraSnapshot};
use crate::core::math::UVec2;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::clear_render_target::clear_render_target;
use super::super::super::motion_vector_camera_params::MotionVectorCameraParams;
use super::super::super::resources::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::scene_post_process_resources::ScenePostProcessResources;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_motion_vector_camera(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_depth_view: &wgpu::TextureView,
        motion_vector_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        current_camera: &ViewportCameraSnapshot,
        previous_camera: Option<&ViewportCameraSnapshot>,
        enabled: bool,
    ) -> MotionVectorCameraStatus {
        if !enabled {
            return MotionVectorCameraStatus::NotRequested;
        }
        let Some(previous_camera) = previous_camera else {
            clear_render_target(
                encoder,
                "ClearMotionVectorCameraPass",
                motion_vector_view,
                wgpu::Color::BLACK,
            );
            return MotionVectorCameraStatus::MissingPreviousCamera;
        };
        let params = MotionVectorCameraParams::from_cameras(
            viewport_size,
            current_camera,
            previous_camera,
            enabled,
        );
        if !params.is_enabled() {
            clear_render_target(
                encoder,
                "ClearMotionVectorCameraPass",
                motion_vector_view,
                wgpu::Color::BLACK,
            );
            return MotionVectorCameraStatus::CameraCutOrInvalid;
        }

        queue.write_buffer(
            &self.motion_vector_camera_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
        let scene_depth_binding_view = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
            PostProcessDepthSamplingMode::ViewportDepthFallback => &self.black_texture_view,
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-motion-vector-camera-bind-group"),
            layout: &self.motion_vector_camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_depth_binding_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.motion_vector_camera_params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("MotionVectorCameraPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: motion_vector_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.motion_vector_camera_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        MotionVectorCameraStatus::Ready
    }
}
