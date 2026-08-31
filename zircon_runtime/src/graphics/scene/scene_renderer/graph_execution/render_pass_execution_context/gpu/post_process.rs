use crate::core::framework::render::{
    PostProcessEffectKind, PostProcessGraphResourceNames, RenderPipelinePhase,
    RenderPostProcessEffectStackSettings, RenderViewFamilyPhaseTargets,
};
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::cluster_dimensions_for_size;
use crate::graphics::scene::scene_renderer::history::{
    SceneFrameHistoryTextures, SceneHistoryAvailability, SceneHistoryDomain,
};
use crate::graphics::scene::scene_renderer::post_process::{
    ScenePostProcessResources, SceneRuntimeFeatureFlags, color_lut_bake_dispatch_groups,
    color_lut_bake_workgroup_size,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
use crate::render_graph::{
    RenderGraphPassResourceAccess, RenderGraphResourceAccessKind, RenderGraphResourceKind,
};

use super::super::super::{RenderGraphComputeDispatchRecord, RenderGraphExecutionResources};
use super::RenderPassGpuExecutionContext;

mod computed_resources;
mod effects;
mod screen_space_reflection;
mod temporal;
mod terminal;

pub(super) use crate::graphics::shader::HZB_BUILD_PIPELINE_LABEL;
const EXPOSURE_HISTOGRAM_PIPELINE_LABEL: &str = "zircon-exposure-histogram-pipeline";
const EXPOSURE_RESOLVE_PIPELINE_LABEL: &str = "zircon-exposure-resolve-pipeline";
const COLOR_LUT_BAKE_PIPELINE_LABEL: &str = "zircon-color-lut-bake-pipeline";

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn with_post_process_stack_context(
        mut self,
        post_process_stack: RenderPassPostProcessStackContext<'a>,
    ) -> Self {
        self.post_process_stack = Some(post_process_stack);
        self
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
        let features = stack.runtime_features;
        let graph = &self.frame.post_process().graph;
        let scene_composite_split =
            post_process_graph_has_node(graph, PostProcessEffectKind::SceneComposite);
        let blur_split = post_process_graph_has_node(graph, PostProcessEffectKind::Blur);
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_resource =
            latest_scene_color_after_composite_resource(resources, resource_resolver)?;
        let scene_color_origin = post_process_texture_origin(self.frame, scene_color_resource);
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_neighbor_max_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            stack.post_process,
        )?;
        let scene_normal_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GBUFFER_NORMAL,
            stack.post_process,
        )?;
        let scene_material_view = stack
            .material_gbuffer_valid
            .then(|| {
                optional_texture_view_or_black(
                    resources,
                    resource_resolver,
                    PostProcessGraphResourceNames::GBUFFER_MATERIAL,
                    stack.post_process,
                )
            })
            .transpose()?;
        let ambient_occlusion_view = optional_texture_view_or_white(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::AMBIENT_OCCLUSION,
            stack.post_process,
        )?;
        let contact_shadow_view = optional_texture_view_or_white(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::CONTACT_SHADOW_OCCLUSION,
            stack.post_process,
        )?;
        let bloom_view = if post_process_graph_has_node(graph, PostProcessEffectKind::Bloom) {
            Self::require_texture_view_by_name(
                resources,
                resource_resolver,
                PostProcessGraphResourceNames::BLOOM,
                RenderGraphResourceAccessKind::Read,
            )?
        } else {
            stack.post_process.black_texture_view()
        };
        let depth_of_field_coc_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            stack.post_process,
        )?;
        let depth_of_field_bokeh_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            stack.post_process,
        )?;
        let tonemapped_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::TONEMAPPED,
            RenderGraphResourceAccessKind::Write,
        )?;
        let global_illumination_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::GLOBAL_ILLUMINATION,
            RenderGraphResourceAccessKind::Write,
        )?;
        let current_hybrid_gi_lighting_view = optional_single_sample_color_texture_view(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HYBRID_GI_LIGHTING,
        )?;
        let previous_global_illumination_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI,
            RenderGraphResourceAccessKind::Read,
        )?;
        let screen_space_reflection_history_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            stack.post_process,
        )?;
        let screen_space_reflection_specular_occlusion_view = optional_texture_view_or_white(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
            stack.post_process,
        )?;
        let baked_color_lut_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::COLOR_LUT,
            RenderGraphResourceAccessKind::Read,
        )?;
        let cluster_buffer = Self::require_buffer_binding_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_LIST,
            RenderGraphResourceAccessKind::Read,
        )?;
        let exposure_buffer = if let Some(exposure_buffer) = Self::optional_buffer_binding_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Read,
        )? {
            exposure_buffer
        } else {
            stack.post_process.default_exposure_buffer_binding()
        };
        let post_process_size = stack.post_process_size(self.frame, pass_name)?;
        let post_process_cluster_dimensions = cluster_dimensions_for_size(post_process_size);
        let mut pass_uploads = stack.post_process.execute_post_process(
            self.device,
            self.encoder,
            post_process_size,
            post_process_cluster_dimensions,
            scene_color_origin,
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            scene_normal_view,
            scene_material_view,
            ambient_occlusion_view,
            contact_shadow_view,
            None,
            current_hybrid_gi_lighting_view,
            previous_global_illumination_view,
            None,
            bloom_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            tonemapped_view,
            global_illumination_view,
            screen_space_reflection_history_view,
            screen_space_reflection_specular_occlusion_view,
            baked_color_lut_view,
            cluster_buffer,
            exposure_buffer,
            self.frame,
            stack.streamer,
            features,
            false,
            stack.history_available(SceneHistoryDomain::HybridGlobalIllumination),
            post_process_graph_has_node(graph, PostProcessEffectKind::DepthOfField),
            post_process_graph_has_node(graph, PostProcessEffectKind::MotionBlur),
            blur_split,
            scene_composite_split,
        );
        self.append_pre_submit_buffer_uploads(&mut pass_uploads);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_color_lut_bake_to_resource(
        &mut self,
        pass_name: &str,
        executor_id: &str,
        color_lut_resource_name: &str,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "color LUT bake graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_lut_texture = Self::require_owned_texture_by_name(
            resources,
            resource_resolver,
            color_lut_resource_name,
            RenderGraphResourceAccessKind::Write,
        )
        .map_err(|error| {
            format!("{error}; color LUT bake graph executor for pass `{pass_name}` requires owned transient texture `{color_lut_resource_name}`")
        })?;
        let exposure_buffer = if let Some(exposure_buffer) = Self::optional_buffer_binding_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT,
            RenderGraphResourceAccessKind::Read,
        )? {
            exposure_buffer
        } else {
            stack.post_process.default_exposure_buffer_binding()
        };
        let (_, mut params_uploads) = stack.post_process.execute_color_lut_bake(
            self.device,
            self.encoder,
            &color_lut_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            exposure_buffer,
            self.frame,
            stack.streamer,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
        self.compute_dispatches.push(
            RenderGraphComputeDispatchRecord::new(
                pass_name,
                executor_id,
                COLOR_LUT_BAKE_PIPELINE_LABEL,
                color_lut_bake_workgroup_size(),
                color_lut_bake_dispatch_groups(),
                vec![color_lut_resource_name.to_string()],
            )
            .with_resource_accesses(color_lut_bake_dispatch_resource_accesses(
                color_lut_resource_name,
            )),
        );
        Ok(())
    }
}

pub(super) fn post_process_texture_origin(
    _frame: &ViewportRenderFrame,
    _resource_name: &str,
) -> [u32; 2] {
    [0, 0]
}

fn post_process_graph_has_node(
    graph: &crate::core::framework::render::PostProcessPassGraph,
    kind: PostProcessEffectKind,
) -> bool {
    graph.nodes.iter().any(|node| node.kind == kind)
}

fn latest_scene_color_resource(
    resources: &RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
) -> Result<&'static str, String> {
    let resource_name = if optional_texture_resource_is_bound(
        resources,
        resource_resolver,
        PostProcessGraphResourceNames::MOTION_BLURRED,
        RenderGraphResourceAccessKind::Read,
    )? {
        PostProcessGraphResourceNames::MOTION_BLURRED
    } else if optional_texture_resource_is_bound(
        resources,
        resource_resolver,
        PostProcessGraphResourceNames::TAA_OUTPUT,
        RenderGraphResourceAccessKind::Read,
    )? {
        PostProcessGraphResourceNames::TAA_OUTPUT
    } else if optional_texture_resource_is_bound(
        resources,
        resource_resolver,
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        RenderGraphResourceAccessKind::Read,
    )? {
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED
    } else {
        PostProcessGraphResourceNames::SCENE_COLOR
    };
    Ok(resource_name)
}

fn latest_scene_color_after_composite_resource(
    resources: &RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
) -> Result<&'static str, String> {
    let resource_name = if optional_texture_resource_is_bound(
        resources,
        resource_resolver,
        PostProcessGraphResourceNames::BLURRED,
        RenderGraphResourceAccessKind::Read,
    )? {
        PostProcessGraphResourceNames::BLURRED
    } else if optional_texture_resource_is_bound(
        resources,
        resource_resolver,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        RenderGraphResourceAccessKind::Read,
    )? {
        PostProcessGraphResourceNames::SCENE_COMPOSITED
    } else {
        latest_scene_color_resource(resources, resource_resolver)?
    };
    Ok(resource_name)
}

fn optional_texture_view_or_black<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
    resource_name: &str,
    post_process: &'a ScenePostProcessResources,
) -> Result<&'a wgpu::TextureView, String> {
    optional_texture_view_or(
        resources,
        resource_resolver,
        resource_name,
        post_process.black_texture_view(),
    )
}

fn optional_texture_view_or_white<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
    resource_name: &str,
    post_process: &'a ScenePostProcessResources,
) -> Result<&'a wgpu::TextureView, String> {
    optional_texture_view_or(
        resources,
        resource_resolver,
        resource_name,
        post_process.white_texture_view(),
    )
}

fn optional_single_sample_color_texture_view<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
    resource_name: &str,
) -> Result<Option<&'a wgpu::TextureView>, String> {
    let Some(view) = RenderPassGpuExecutionContext::optional_texture_view_by_name(
        resources,
        resource_resolver,
        resource_name,
        RenderGraphResourceAccessKind::Read,
    )?
    else {
        return Ok(None);
    };
    let desc = RenderPassGpuExecutionContext::require_texture_desc_by_name(
        resources,
        resource_resolver,
        resource_name,
        RenderGraphResourceAccessKind::Read,
    )?;
    if desc.sample_count != 1 || desc.format.is_depth() {
        return Ok(None);
    }
    Ok(Some(view))
}

fn optional_texture_view_or<'a>(
    resources: &'a RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
    resource_name: &str,
    fallback: &'a wgpu::TextureView,
) -> Result<&'a wgpu::TextureView, String> {
    RenderPassGpuExecutionContext::optional_texture_view_by_name(
        resources,
        resource_resolver,
        resource_name,
        RenderGraphResourceAccessKind::Read,
    )
    .map(|view| view.unwrap_or(fallback))
}

fn optional_texture_resource_is_bound(
    resources: &RenderGraphExecutionResources,
    resource_resolver: Option<super::super::RgResourceResolver<'_>>,
    resource_name: &str,
    access: RenderGraphResourceAccessKind,
) -> Result<bool, String> {
    RenderPassGpuExecutionContext::optional_texture_view_by_name(
        resources,
        resource_resolver,
        resource_name,
        access,
    )
    .map(|view| view.is_some())
}

fn effect_stack_uses_reconstructed_velocity(
    effect_stack: RenderPostProcessEffectStackSettings,
) -> bool {
    effect_stack.motion_blur.is_enabled() || effect_stack.screen_space_reflection.is_enabled()
}

fn require_post_process_render_region(
    pass_name: &str,
    executor_name: &str,
    phase: RenderPipelinePhase,
    render_region: Option<ViewportRenderRegion>,
) -> Result<ViewportRenderRegion, String> {
    render_region.ok_or_else(|| {
        format!("{executor_name} graph executor for pass `{pass_name}` requires phase {phase:?}")
    })
}

fn require_post_process_phase_targets(
    pass_name: &str,
    phase: RenderPipelinePhase,
    phase_targets: Option<RenderViewFamilyPhaseTargets>,
) -> Result<RenderViewFamilyPhaseTargets, String> {
    phase_targets.ok_or_else(|| {
        format!("post-process stack graph executor for pass `{pass_name}` requires phase {phase:?}")
    })
}

fn color_lut_bake_dispatch_resource_accesses(
    color_lut_resource_name: &str,
) -> Vec<RenderGraphPassResourceAccess> {
    vec![
        RenderGraphPassResourceAccess {
            name: PostProcessGraphResourceNames::EXPOSURE_CURRENT.to_string(),
            kind: RenderGraphResourceKind::TransientBuffer,
            access: RenderGraphResourceAccessKind::Read,
            attachment_ops: None,
        },
        RenderGraphPassResourceAccess {
            name: color_lut_resource_name.to_string(),
            kind: RenderGraphResourceKind::TransientTexture,
            access: RenderGraphResourceAccessKind::Write,
            attachment_ops: None,
        },
    ]
}

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer) struct RenderPassPostProcessStackContext<'a> {
    post_process: &'a ScenePostProcessResources,
    streamer: &'a ResourceStreamer,
    runtime_features: SceneRuntimeFeatureFlags,
    history_textures: Option<&'a SceneFrameHistoryTextures>,
    history_availability: SceneHistoryAvailability,
    material_gbuffer_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        color_lut_bake_dispatch_resource_accesses, effect_stack_uses_reconstructed_velocity,
        require_post_process_phase_targets, require_post_process_render_region,
    };
    use crate::core::framework::render::{
        PostProcessGraphResourceNames, RenderMotionBlurSettings, RenderPipelinePhase,
        RenderPostProcessEffectStackSettings, RenderScreenSpaceReflectionSettings,
    };
    use crate::core::math::UVec2;
    use crate::graphics::types::ViewportRenderRegion;
    use crate::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceKind};

    #[test]
    fn color_lut_bake_dispatch_reports_exposure_read_and_lut_write() {
        let accesses =
            color_lut_bake_dispatch_resource_accesses(PostProcessGraphResourceNames::COLOR_LUT);

        assert_eq!(accesses.len(), 2);
        assert_eq!(
            accesses[0].name,
            PostProcessGraphResourceNames::EXPOSURE_CURRENT
        );
        assert_eq!(accesses[0].kind, RenderGraphResourceKind::TransientBuffer);
        assert_eq!(accesses[0].access, RenderGraphResourceAccessKind::Read);
        assert_eq!(accesses[1].name, PostProcessGraphResourceNames::COLOR_LUT);
        assert_eq!(accesses[1].kind, RenderGraphResourceKind::TransientTexture);
        assert_eq!(accesses[1].access, RenderGraphResourceAccessKind::Write);
    }

    #[test]
    fn reconstructed_velocity_is_requested_for_temporal_effects() {
        assert!(!effect_stack_uses_reconstructed_velocity(
            RenderPostProcessEffectStackSettings::default()
        ));

        assert!(effect_stack_uses_reconstructed_velocity(
            RenderPostProcessEffectStackSettings {
                motion_blur: RenderMotionBlurSettings {
                    shutter_angle: 0.5,
                    samples: 4,
                },
                ..Default::default()
            }
        ));

        assert!(effect_stack_uses_reconstructed_velocity(
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

    #[test]
    fn missing_post_process_phase_is_a_fallible_graph_error() {
        let phase = RenderPipelinePhase::DisplayPostProcess;
        let error = require_post_process_render_region("post.fxaa", "FXAA", phase, None)
            .expect_err("missing phase must fail without panicking");

        assert_eq!(
            error,
            "FXAA graph executor for pass `post.fxaa` requires phase DisplayPostProcess"
        );
        let region = ViewportRenderRegion::full_target(UVec2::new(640, 360));
        assert_eq!(
            require_post_process_render_region("post.fxaa", "FXAA", phase, Some(region)),
            Ok(region)
        );

        let target_error =
            require_post_process_phase_targets("post.hzb", RenderPipelinePhase::SceneLinear, None)
                .expect_err("missing phase targets must fail without panicking");
        assert_eq!(
            target_error,
            "post-process stack graph executor for pass `post.hzb` requires phase SceneLinear"
        );
    }
}

impl<'a> RenderPassPostProcessStackContext<'a> {
    pub(super) fn scene_linear_size(
        self,
        frame: &ViewportRenderFrame,
        pass_name: &str,
    ) -> Result<crate::core::math::UVec2, String> {
        let phase = RenderPipelinePhase::SceneLinear;
        require_post_process_render_region(
            pass_name,
            "post-process stack",
            phase,
            frame.render_region_for_phase(phase),
        )
        .map(ViewportRenderRegion::local_size)
    }

    pub(super) fn scene_linear_allocation_size(
        self,
        frame: &ViewportRenderFrame,
        pass_name: &str,
    ) -> Result<crate::core::math::UVec2, String> {
        let phase = RenderPipelinePhase::SceneLinear;
        require_post_process_phase_targets(
            pass_name,
            phase,
            frame.view_family_pipeline().phase_targets(phase),
        )
        .map(|targets| targets.output().allocation_extent())
    }

    pub(super) fn post_process_size(
        self,
        frame: &ViewportRenderFrame,
        pass_name: &str,
    ) -> Result<crate::core::math::UVec2, String> {
        let phase = RenderPipelinePhase::DisplayMapping;
        require_post_process_render_region(
            pass_name,
            "post-process stack",
            phase,
            frame.render_region_for_phase(phase),
        )
        .map(ViewportRenderRegion::local_size)
    }

    pub(super) fn post_process_cluster_dimensions(
        self,
        frame: &ViewportRenderFrame,
        pass_name: &str,
    ) -> Result<crate::core::math::UVec2, String> {
        self.post_process_size(frame, pass_name)
            .map(cluster_dimensions_for_size)
    }

    pub(in crate::graphics::scene::scene_renderer) fn post_process(
        &self,
    ) -> &'a ScenePostProcessResources {
        self.post_process
    }

    pub(super) fn hybrid_gi_history_available(self) -> bool {
        self.history_textures.is_some()
            && self.history_available(SceneHistoryDomain::HybridGlobalIllumination)
    }

    pub(super) const fn history_available(self, domain: SceneHistoryDomain) -> bool {
        self.history_availability.is_available(domain)
    }

    pub(in crate::graphics::scene::scene_renderer) fn new(
        post_process: &'a ScenePostProcessResources,
        streamer: &'a ResourceStreamer,
        runtime_features: SceneRuntimeFeatureFlags,
        history_textures: Option<&'a SceneFrameHistoryTextures>,
        history_availability: SceneHistoryAvailability,
    ) -> Self {
        Self {
            post_process,
            streamer,
            runtime_features,
            history_textures,
            history_availability,
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

    pub(in crate::graphics::scene::scene_renderer) fn hzb_history_resource_identity(
        &self,
    ) -> Option<crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity> {
        if !self
            .history_availability
            .is_available(SceneHistoryDomain::HzbFurthest)
        {
            return None;
        }
        self.history_textures
            .and_then(SceneFrameHistoryTextures::hzb_resource_identity)
    }

    pub(in crate::graphics::scene::scene_renderer) fn hzb_fallback_resource_identity(
        &self,
    ) -> crate::graphics::scene::scene_renderer::hzb::HzbSampledResourceIdentity {
        self.post_process.hzb_fallback_resource_identity()
    }
}
