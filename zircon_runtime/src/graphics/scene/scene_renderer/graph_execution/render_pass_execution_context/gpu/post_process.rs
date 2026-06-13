use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, RenderPostProcessEffectStackSettings,
};
use crate::graphics::backend::OffscreenTarget;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::graphics::scene::scene_renderer::lighting::light_grid_pass::{
    build_light_grid_for_frame, write_light_grid_buffers,
};
use crate::graphics::scene::scene_renderer::post_process::{
    clustered_lighting_dispatch_groups, clustered_lighting_workgroup_size,
    hzb_build_dispatch_groups, hzb_build_workgroup_size, ssao_dispatch_groups, ssao_workgroup_size,
    ScenePostProcessResources, SceneRuntimeFeatureFlags,
};
use crate::graphics::visibility::HzbBuilder;
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::super::{
    RenderGraphComputeDispatchRecord, RenderGraphExecutionResources, RenderGraphLightGridReport,
};
use super::RenderPassGpuExecutionContext;

mod screen_space_reflection;

const HZB_BUILD_PIPELINE_LABEL: &str = "zircon-hzb-build-pipeline";

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
        let graph = &self.frame.extract.post_process.graph;
        let motion_vector_neighbor_max_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            stack.post_process,
        )?;
        let scene_normal_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            stack.post_process,
        )?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                optional_texture_view_or_black(
                    self.resources,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    stack.post_process,
                )
            })
            .transpose()?;
        let ambient_occlusion_view = optional_texture_view_or_white(
            self.resources,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            stack.post_process,
        )?;
        let contact_shadow_view = optional_texture_view_or_white(
            self.resources,
            PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
            stack.post_process,
        )?;
        let bloom_view = if post_process_graph_has_node(graph, PostProcessEffectKind::Bloom) {
            self.resources
                .require_texture_view(PostProcessGraphResourceNames::BLOOM)?
        } else {
            stack.post_process.black_texture_view()
        };
        let depth_of_field_coc_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            stack.post_process,
        )?;
        let depth_of_field_bokeh_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            stack.post_process,
        )?;
        let _final_composited_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::FINAL_COMPOSITED,
            stack.post_process,
        )?;
        let final_color_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::FINAL_COLOR)?;
        let global_illumination_view = self
            .resources
            .require_texture_view(PostProcessGraphResourceNames::GLOBAL_ILLUMINATION)?;
        let screen_space_reflection_history_view = optional_texture_view_or_black(
            self.resources,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            stack.post_process,
        )?;
        let screen_space_reflection_specular_occlusion_view = optional_texture_view_or_white(
            self.resources,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
            stack.post_process,
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
            contact_shadow_view,
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
        hzb_furthest_resource_name: &str,
        ambient_occlusion_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "SSAO graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let normal_view = self.resources.require_texture_view(normal_resource_name)?;
        let hzb_furthest_view = self
            .resources
            .require_texture_view(hzb_furthest_resource_name)?;
        let hzb_furthest_full_mip_view = self
            .resources
            .owned_texture_full_mip_view(hzb_furthest_resource_name)
            .ok();
        let hzb_furthest_sampling_view = hzb_furthest_full_mip_view
            .as_ref()
            .unwrap_or(hzb_furthest_view);
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
            hzb_furthest_sampling_view,
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
        light_grid_params_resource_name: &str,
        light_zbins_resource_name: &str,
        light_tile_masks_resource_name: &str,
        legacy_light_list_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "light grid graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let light_grid_params_buffer = self
            .resources
            .require_buffer(light_grid_params_resource_name)?;
        let light_zbins_buffer = self.resources.require_buffer(light_zbins_resource_name)?;
        let light_tile_masks_buffer = self
            .resources
            .require_buffer(light_tile_masks_resource_name)?;
        let legacy_light_list_buffer = self
            .resources
            .require_buffer(legacy_light_list_resource_name)?;
        let target = stack.target;
        let enabled = stack.runtime_features.clustered_lighting_enabled;
        let light_grid = build_light_grid_for_frame(&self.frame.extract, target.size, enabled);
        self.light_grid_report = Some(RenderGraphLightGridReport::from_stats(&light_grid.stats));
        write_light_grid_buffers(
            self.queue,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            &light_grid,
        );
        let dispatch_groups = clustered_lighting_dispatch_groups(target.cluster_dimensions);
        let workgroup_size = clustered_lighting_workgroup_size();
        stack.post_process.execute_clustered_lighting(
            self.device,
            self.queue,
            self.encoder,
            target.size,
            target.cluster_dimensions,
            legacy_light_list_buffer,
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
                    vec![
                        light_grid_params_resource_name.to_string(),
                        light_zbins_resource_name.to_string(),
                        light_tile_masks_resource_name.to_string(),
                        legacy_light_list_resource_name.to_string(),
                    ],
                ));
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_hzb_build_to_resource(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        depth_resource_name: &str,
        hzb_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!("HZB graph executor for pass `{pass_name}` requires post-process stack context")
        })?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let _hzb_view = self.resources.require_texture_view(hzb_resource_name)?;
        let plan = HzbBuilder::new(self.frame.extract.view.effective_render_size()).build_plan();
        for mip_level in 0..plan.mip_count {
            let source_view = if mip_level == 0 {
                None
            } else {
                Some(
                    self.resources
                        .owned_texture_mip_view(hzb_resource_name, mip_level - 1)?,
                )
            };
            let target_view = self
                .resources
                .owned_texture_mip_view(hzb_resource_name, mip_level)?;
            stack.post_process.execute_hzb_build_mip(
                self.device,
                self.queue,
                self.encoder,
                depth_view,
                source_view.as_ref(),
                &target_view,
                plan.mip_size(mip_level),
                mip_level,
            );
        }
        let dispatch_groups = hzb_build_dispatch_groups(plan.hzb_size);
        let workgroup_size = hzb_build_workgroup_size();
        self.compute_dispatches
            .push(RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                HZB_BUILD_PIPELINE_LABEL,
                workgroup_size,
                dispatch_groups,
                vec![hzb_resource_name.to_string()],
            ));
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

fn post_process_graph_has_node(
    graph: &crate::core::framework::render::PostProcessPassGraph,
    kind: PostProcessEffectKind,
) -> bool {
    graph.nodes.iter().any(|node| node.kind == kind)
}

fn optional_texture_view_or_black<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_name: &str,
    post_process: &'a ScenePostProcessResources,
) -> Result<&'a wgpu::TextureView, String> {
    optional_texture_view_or(resources, resource_name, post_process.black_texture_view())
}

fn optional_texture_view_or_white<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_name: &str,
    post_process: &'a ScenePostProcessResources,
) -> Result<&'a wgpu::TextureView, String> {
    optional_texture_view_or(resources, resource_name, post_process.white_texture_view())
}

fn optional_texture_view_or<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_name: &str,
    fallback: &'a wgpu::TextureView,
) -> Result<&'a wgpu::TextureView, String> {
    if resources.has_texture_view(resource_name) {
        resources.require_texture_view(resource_name)
    } else {
        Ok(fallback)
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

    pub(in crate::graphics::scene::scene_renderer) fn white_texture_view(
        &self,
    ) -> &'a wgpu::TextureView {
        self.post_process.white_texture_view()
    }
}
