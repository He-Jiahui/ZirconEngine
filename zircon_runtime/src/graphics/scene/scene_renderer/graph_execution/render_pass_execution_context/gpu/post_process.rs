use crate::core::framework::render::{
    PostProcessGraphResourceNames, RenderPostProcessEffectStackSettings,
};
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::post_process::{
    clustered_lighting_dispatch_groups, clustered_lighting_workgroup_size, ssao_dispatch_groups,
    ssao_workgroup_size, ScenePostProcessResources, SceneRuntimeFeatureFlags,
};
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::RenderGraphComputeDispatchRecord;
use super::RenderPassGpuExecutionContext;

mod screen_space_reflection;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn with_post_process_stack_context(
        mut self,
        post_process_stack: RenderPassPostProcessStackContext<'a>,
    ) -> Self {
        self.post_process_stack = Some(post_process_stack);
        self
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_motion_vector_clear_to_resource(
        &mut self,
        pass_name: &str,
        motion_vector_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let motion_vector_view = self
            .resources
            .require_texture_view(motion_vector_resource_name)?;
        let _pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(pass_name),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: motion_vector_view,
                resolve_target: None,
                depth_slice: None,
                ops: color_attachment_operations(attachment_ops, wgpu::Color::BLACK),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_post_process_stack(
        &mut self,
        pass_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "post-process stack graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let target = stack.target;
        let history = stack.history_textures;
        let features = stack.runtime_features;
        let scene_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_COLOR)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCENE_DEPTH)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX)?;
        let scene_normal_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GBUFFER_NORMAL)?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                self.resources
                    .require_texture_view(PostProcessGraphResourceNames::GBUFFER_MATERIAL)
            })
            .transpose()?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::AMBIENT_OCCLUSION)?;
        let bloom_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::BLOOM)?;
        let depth_of_field_coc_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC)?;
        let depth_of_field_bokeh_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH)?;
        let _final_composited_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::FINAL_COMPOSITED)?;
        let final_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::FINAL_COLOR)?;
        let global_illumination_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION)?;
        let screen_space_reflection_history_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY)?;
        let screen_space_reflection_specular_occlusion_view = self.resources.require_texture_view(
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        )?;
        let cluster_buffer = self
            .resources
            .require_buffer(PostProcessGraphResourceNames::LIGHT_LIST)?;
        stack.post_process.execute_post_process(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ambient_occlusion_view,
            history.map(|history| &history.scene_color_view),
            history.map(|history| &history.global_illumination_view),
            history.map(|history| &history.screen_space_reflection_view),
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            final_color_view,
            global_illumination_view,
            screen_space_reflection_history_view,
            screen_space_reflection_specular_occlusion_view,
            cluster_buffer,
            self.frame,
            stack.streamer,
            features,
            stack.history_available,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_ssao_to_resources(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        normal_resource_name: &str,
        ambient_occlusion_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "SSAO graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let normal_view = self.resources.require_texture_view(normal_resource_name)?;
        let ambient_occlusion_view = self
            .resources
            .require_texture_view(ambient_occlusion_resource_name)?;
        let target = stack.target;
        let enabled = stack.runtime_features.ssao_enabled;
        let dispatch_groups = ssao_dispatch_groups(target.size);
        let workgroup_size = ssao_workgroup_size();
        stack.post_process.execute_ssao(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            depth_view,
            normal_view,
            stack
                .history_textures
                .map(|history| &history.ambient_occlusion_view),
            ambient_occlusion_view,
            enabled,
            stack.history_available,
        );
        if enabled {
            self.compute_dispatches
                .push(RenderGraphComputeDispatchRecord::new(
                    pass_name,
                    executor_id,
                    "zircon-ssao-pipeline",
                    workgroup_size,
                    dispatch_groups,
                    vec![ambient_occlusion_resource_name.to_string()],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_clustered_lighting_to_resources(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        light_list_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "clustered lighting graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let _depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let light_list_buffer = self.resources.require_buffer(light_list_resource_name)?;
        let target = stack.target;
        let enabled = stack.runtime_features.clustered_lighting_enabled;
        let dispatch_groups = clustered_lighting_dispatch_groups(target.cluster_dimensions);
        let workgroup_size = clustered_lighting_workgroup_size();
        stack.post_process.execute_clustered_lighting(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            light_list_buffer,
            target.cluster_buffer_bytes,
            &self.frame.extract.lighting.directional_lights,
            enabled,
        );
        if enabled {
            self.compute_dispatches
                .push(RenderGraphComputeDispatchRecord::new(
                    pass_name,
                    executor_id,
                    "zircon-cluster-pipeline",
                    workgroup_size,
                    dispatch_groups,
                    vec![light_list_resource_name.to_string()],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_bloom_to_resources(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        bloom_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "bloom graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let bloom_view = self.resources.require_texture_view(bloom_resource_name)?;
        let target = stack.target;
        stack.post_process.execute_bloom(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            scene_color_view,
            bloom_view,
            self.frame.extract.post_process.bloom,
            stack.runtime_features.bloom_enabled,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_depth_of_field_prepare_to_resources(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        scene_depth_resource_name: &str,
        coc_resource_name: &str,
        bokeh_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "depth-of-field prepare graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let scene_color_view = self
            .resources
            .require_texture_view(scene_color_resource_name)?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let coc_view = self.resources.require_texture_view(coc_resource_name)?;
        let bokeh_view = self.resources.require_texture_view(bokeh_resource_name)?;
        stack.post_process.execute_depth_of_field_prepare(
            self.device,
            self.queue,
            self.encoder,
            stack.target.size,
            scene_color_view,
            scene_depth_view,
            coc_view,
            bokeh_view,
            self.frame.extract.post_process.effect_stack.depth_of_field,
            &self.frame.extract.view.camera,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_motion_vector_camera_to_resource(
        &mut self,
        pass_name: &str,
        scene_depth_resource_name: &str,
        motion_vector_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "motion-vector camera graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let scene_depth_view = self
            .resources
            .require_texture_view(scene_depth_resource_name)?;
        let motion_vector_view = self
            .resources
            .require_texture_view(motion_vector_resource_name)?;
        self.motion_vector_camera_status = stack.post_process.execute_motion_vector_camera(
            self.device,
            self.queue,
            self.encoder,
            stack.target.size,
            scene_depth_view,
            motion_vector_view,
            attachment_ops,
            &self.frame.extract.view.camera,
            self.frame.previous_motion_vector_camera(),
            effect_stack_uses_reconstructed_motion_vectors(
                self.frame.extract.post_process.effect_stack,
            ),
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_motion_vector_tile_max_to_resource(
        &mut self,
        pass_name: &str,
        motion_vector_source_resource_name: &str,
        motion_vector_tile_max_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "motion-vector tile-max graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let motion_vector_source_view = self
            .resources
            .require_texture_view(motion_vector_source_resource_name)?;
        let motion_vector_tile_max_view = self
            .resources
            .require_texture_view(motion_vector_tile_max_resource_name)?;
        stack.post_process.execute_motion_vector_tile_max(
            self.device,
            self.encoder,
            motion_vector_source_view,
            motion_vector_tile_max_view,
            attachment_ops,
            effect_stack_uses_reconstructed_motion_vectors(
                self.frame.extract.post_process.effect_stack,
            ),
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_motion_vector_neighbor_max_to_resource(
        &mut self,
        pass_name: &str,
        motion_vector_tile_max_coarse_resource_name: &str,
        motion_vector_neighbor_max_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "motion-vector neighbor-max graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let motion_vector_tile_max_coarse_view = self
            .resources
            .require_texture_view(motion_vector_tile_max_coarse_resource_name)?;
        let motion_vector_neighbor_max_view = self
            .resources
            .require_texture_view(motion_vector_neighbor_max_resource_name)?;
        stack.post_process.execute_motion_vector_neighbor_max(
            self.device,
            self.encoder,
            motion_vector_tile_max_coarse_view,
            motion_vector_neighbor_max_view,
            attachment_ops,
            effect_stack_uses_reconstructed_motion_vectors(
                self.frame.extract.post_process.effect_stack,
            ),
        );
        Ok(())
    }
}

fn effect_stack_uses_reconstructed_motion_vectors(
    effect_stack: RenderPostProcessEffectStackSettings,
) -> bool {
    effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled()
}

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassPostProcessStackContext<'a> {
    post_process: &'a ScenePostProcessResources,
    target: &'a OffscreenTarget,
    streamer: &'a ResourceStreamer,
    runtime_features: SceneRuntimeFeatureFlags,
    history_textures: Option<&'a SceneFrameHistoryTextures>,
    history_available: bool,
    material_gbuffer_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::effect_stack_uses_reconstructed_motion_vectors;
    use crate::core::framework::render::{
        RenderMotionBlurSettings, RenderPostProcessEffectStackSettings,
        RenderScreenSpaceReflectionSettings,
    };

    #[test]
    fn reconstructed_motion_vectors_are_requested_for_temporal_effects() {
        assert!(!effect_stack_uses_reconstructed_motion_vectors(
            RenderPostProcessEffectStackSettings::default()
        ));

        assert!(effect_stack_uses_reconstructed_motion_vectors(
            RenderPostProcessEffectStackSettings {
                motion_blur: RenderMotionBlurSettings {
                    shutter_angle: 0.5,
                    samples: 4,
                },
                ..Default::default()
            }
        ));

        assert!(effect_stack_uses_reconstructed_motion_vectors(
            RenderPostProcessEffectStackSettings {
                screen_space_reflection: RenderScreenSpaceReflectionSettings {
                    intensity: 0.5,
                    max_steps: 16,
                    ..Default::default()
                },
                ..Default::default()
            }
        ));
    }
}

impl<'a> RenderPassPostProcessStackContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn new(
        post_process: &'a ScenePostProcessResources,
        target: &'a OffscreenTarget,
        streamer: &'a ResourceStreamer,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&'a SceneFrameHistoryTextures>,
        history_available: bool,
    ) -> Self {
        Self {
            post_process,
            target,
            streamer,
            runtime_features,
            history_textures,
            history_available,
            material_gbuffer_valid: false,
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn with_material_gbuffer_valid(
        mut self,
        material_gbuffer_valid: bool,
    ) -> Self {
        self.material_gbuffer_valid = material_gbuffer_valid;
        self
    }
}
