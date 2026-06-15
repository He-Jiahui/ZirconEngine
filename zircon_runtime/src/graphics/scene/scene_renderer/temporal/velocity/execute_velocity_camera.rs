use crate::core::framework::render::{MotionVectorCameraStatus, ViewportCameraSnapshot};
use crate::core::math::UVec2;
use crate::render_graph::RenderGraphAttachmentOps;

use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::{
    PostProcessDepthSamplingMode, ScenePostProcessResources,
};

use super::velocity_camera_params::VelocityCameraParams;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_velocity_camera(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_depth_view: &wgpu::TextureView,
        velocity_view: &wgpu::TextureView,
        attachment_ops: RenderGraphAttachmentOps,
        current_camera: &ViewportCameraSnapshot,
        previous_camera: Option<&ViewportCameraSnapshot>,
        enabled: bool,
    ) -> MotionVectorCameraStatus {
        if !enabled {
            return MotionVectorCameraStatus::NotRequested;
        }
        let Some(previous_camera) = previous_camera else {
            clear_velocity_target(encoder, "ClearVelocityCameraPass", velocity_view);
            return MotionVectorCameraStatus::MissingPreviousCamera;
        };
        let params = VelocityCameraParams::from_cameras(
            viewport_size,
            current_camera,
            previous_camera,
            enabled,
        );
        if !params.is_enabled() {
            clear_velocity_target(encoder, "ClearVelocityCameraPass", velocity_view);
            return MotionVectorCameraStatus::CameraCutOrInvalid;
        }

        queue.write_buffer(
            &self.velocity_camera_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
        let scene_depth_binding_view = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
            PostProcessDepthSamplingMode::ViewportDepthFallback => &self.black_texture_view,
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-velocity-camera-bind-group"),
            layout: &self.velocity_camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(scene_depth_binding_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.velocity_camera_params_buffer.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("VelocityCameraPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: velocity_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.velocity_camera_pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
        MotionVectorCameraStatus::Ready
    }
}

fn clear_velocity_target(
    encoder: &mut wgpu::CommandEncoder,
    pass_label: &'static str,
    velocity_view: &wgpu::TextureView,
) {
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(pass_label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: velocity_view,
            resolve_target: None,
            depth_slice: None,
            ops: color_attachment_operations(
                RenderGraphAttachmentOps::clear_store(),
                wgpu::Color::BLACK,
            ),
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
}
