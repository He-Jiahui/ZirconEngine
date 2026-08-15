use crate::core::framework::render::AntiAliasSettings;
use crate::core::math::UVec2;
use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::post_process::{
    PostProcessDepthSamplingMode, ScenePostProcessResources,
};
use crate::render_graph::RenderGraphAttachmentOps;

use super::taa_resolve_bind_group_cache::{create_bind_group, TaaResolveBindGroupKey};
use super::taa_resolve_params::TaaResolveParams;

impl ScenePostProcessResources {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_taa_resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        viewport_size: UVec2,
        scene_color_view: &wgpu::TextureView,
        scene_depth_view: &wgpu::TextureView,
        scene_velocity_view: &wgpu::TextureView,
        taa_history_previous_view: &wgpu::TextureView,
        taa_reactive_mask_view: &wgpu::TextureView,
        taa_output_view: &wgpu::TextureView,
        taa_history_current_view: &wgpu::TextureView,
        scene_color_identity: Option<SampledTextureIdentity>,
        scene_depth_identity: Option<SampledTextureIdentity>,
        scene_velocity_identity: Option<SampledTextureIdentity>,
        taa_history_previous_identity: Option<SampledTextureIdentity>,
        taa_history_current_identity: Option<SampledTextureIdentity>,
        taa_reactive_mask_identity: Option<SampledTextureIdentity>,
        taa_output_attachment_ops: RenderGraphAttachmentOps,
        taa_history_attachment_ops: RenderGraphAttachmentOps,
        history_valid: bool,
        anti_alias: AntiAliasSettings,
    ) -> bool {
        let params = TaaResolveParams::new(
            viewport_size,
            anti_alias.mode == crate::core::framework::render::AntiAliasMode::Taa && history_valid,
            anti_alias.taa_quality,
        );
        queue.write_buffer(
            &self.taa_resolve_params_buffer,
            0,
            bytemuck::bytes_of(&params),
        );
        let scene_depth_binding_view = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_view,
            PostProcessDepthSamplingMode::ViewportDepthFallback => &self.black_texture_view,
        };
        let scene_depth_binding_identity = match self.depth_sampling_mode {
            PostProcessDepthSamplingMode::RawDepthTexture => scene_depth_identity,
            PostProcessDepthSamplingMode::ViewportDepthFallback => {
                Some(self.black_texture_identity)
            }
        };
        let key = (taa_reactive_mask_identity == Some(self.black_texture_identity))
            .then(|| {
                match (
                    scene_color_identity,
                    scene_depth_binding_identity,
                    scene_velocity_identity,
                    taa_history_previous_identity,
                    taa_reactive_mask_identity,
                ) {
                    (
                        Some(scene_color),
                        Some(scene_depth),
                        Some(scene_velocity),
                        Some(history_previous),
                        Some(reactive_mask),
                    ) => Some(TaaResolveBindGroupKey::new(
                        scene_color,
                        scene_depth,
                        scene_velocity,
                        history_previous,
                        reactive_mask,
                    )),
                    _ => None,
                }
            })
            .flatten();
        let prepared_bind_group =
            key.zip(taa_history_current_identity)
                .map(|(key, history_current_identity)| {
                    self.taa_resolve_bind_group_cache
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .prepare(
                            device,
                            &self.taa_resolve_bind_group_layout,
                            key,
                            history_current_identity,
                            scene_color_view,
                            scene_depth_binding_view,
                            scene_velocity_view,
                            taa_history_previous_view,
                            &self.taa_resolve_params_buffer,
                            taa_reactive_mask_view,
                        )
                });
        let uncached_bind_group = prepared_bind_group.is_none().then(|| {
            create_bind_group(
                device,
                &self.taa_resolve_bind_group_layout,
                scene_color_view,
                scene_depth_binding_view,
                scene_velocity_view,
                taa_history_previous_view,
                &self.taa_resolve_params_buffer,
                taa_reactive_mask_view,
            )
        });
        let bind_group = prepared_bind_group
            .as_ref()
            .map(|prepared| &prepared.bind_group)
            .unwrap_or_else(|| {
                uncached_bind_group
                    .as_ref()
                    .expect("uncached TAA bind group")
            });
        let bind_group_created = prepared_bind_group
            .as_ref()
            .map(|prepared| prepared.created)
            .unwrap_or(true);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("TaaResolvePass"),
            color_attachments: &[
                Some(wgpu::RenderPassColorAttachment {
                    view: taa_output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(taa_output_attachment_ops, wgpu::Color::BLACK),
                }),
                Some(wgpu::RenderPassColorAttachment {
                    view: taa_history_current_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: color_attachment_operations(
                        taa_history_attachment_ops,
                        wgpu::Color::BLACK,
                    ),
                }),
            ],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.taa_resolve_pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        bind_group_created
    }
}
