use crate::core::framework::render::{PostProcessGraphResourceNames, RenderPipelinePhase};
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::super::RenderPassGpuExecutionContext;
use super::{
    latest_scene_color_resource, optional_texture_resource_is_bound,
    optional_texture_view_or_black, require_post_process_render_region,
};

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_motion_blur_to_resource(
        &mut self,
        pass_name: &str,
        output_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "motion blur graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let source_resource_name = if optional_texture_resource_is_bound(
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
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_neighbor_max_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
            RenderGraphResourceAccessKind::Read,
        )?;
        let output_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            output_resource_name,
            RenderGraphResourceAccessKind::Write,
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
        let post_process_cluster_dimensions =
            stack.post_process_cluster_dimensions(self.frame, pass_name)?;
        let mut params_uploads = stack.post_process.execute_motion_blur(
            self.device,
            self.encoder,
            post_process_cluster_dimensions,
            super::post_process_texture_origin(self.frame, source_resource_name),
            scene_color_view,
            scene_depth_view,
            motion_vector_neighbor_max_view,
            output_view,
            exposure_buffer,
            self.frame,
            attachment_ops,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_scene_composite_to_resource(
        &mut self,
        pass_name: &str,
        output_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "scene-composite graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_resource = latest_scene_color_resource(resources, resource_resolver)?;
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
        let screen_space_reflection_history_view = optional_texture_view_or_black(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
            stack.post_process,
        )?;
        let output_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            output_resource_name,
            RenderGraphResourceAccessKind::Write,
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
        let post_process_cluster_dimensions =
            stack.post_process_cluster_dimensions(self.frame, pass_name)?;
        let mut params_uploads = stack.post_process.execute_scene_composite(
            self.device,
            self.encoder,
            post_process_cluster_dimensions,
            super::post_process_texture_origin(self.frame, scene_color_resource),
            scene_color_view,
            scene_depth_view,
            screen_space_reflection_history_view,
            output_view,
            exposure_buffer,
            self.frame,
            attachment_ops,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_blur_to_resource(
        &mut self,
        pass_name: &str,
        output_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "blur graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let source_resource_name = if optional_texture_resource_is_bound(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_COMPOSITED,
            RenderGraphResourceAccessKind::Read,
        )? {
            PostProcessGraphResourceNames::SCENE_COMPOSITED
        } else {
            latest_scene_color_resource(resources, resource_resolver)?
        };
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let output_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            output_resource_name,
            RenderGraphResourceAccessKind::Write,
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
        let post_process_cluster_dimensions =
            stack.post_process_cluster_dimensions(self.frame, pass_name)?;
        let mut params_uploads = stack.post_process.execute_blur(
            self.device,
            self.encoder,
            post_process_cluster_dimensions,
            super::post_process_texture_origin(self.frame, source_resource_name),
            scene_color_view,
            scene_depth_view,
            output_view,
            exposure_buffer,
            self.frame,
            attachment_ops,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_depth_of_field_to_resource(
        &mut self,
        pass_name: &str,
        output_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "depth-of-field graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let source_resource_name = PostProcessGraphResourceNames::SCENE_COLOR;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let depth_of_field_coc_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
            RenderGraphResourceAccessKind::Read,
        )?;
        let depth_of_field_bokeh_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let output_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            output_resource_name,
            RenderGraphResourceAccessKind::Write,
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
        let post_process_cluster_dimensions =
            stack.post_process_cluster_dimensions(self.frame, pass_name)?;
        let mut params_uploads = stack.post_process.execute_depth_of_field(
            self.device,
            self.encoder,
            post_process_cluster_dimensions,
            super::post_process_texture_origin(self.frame, source_resource_name),
            scene_color_view,
            scene_depth_view,
            depth_of_field_coc_view,
            depth_of_field_bokeh_view,
            output_view,
            exposure_buffer,
            self.frame,
            attachment_ops,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
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
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let bloom_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            bloom_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let phase = RenderPipelinePhase::PostReconstructionScenePostProcess;
        let render_region = require_post_process_render_region(
            pass_name,
            "bloom",
            phase,
            self.frame.render_region_for_phase(phase),
        )?;
        let mut params_uploads = stack.post_process.execute_bloom(
            self.device,
            self.encoder,
            render_region.local_size(),
            super::post_process_texture_origin(self.frame, scene_color_resource_name),
            scene_color_view,
            bloom_view,
            self.frame.post_process().bloom,
            stack.runtime_features.bloom_enabled,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
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
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let coc_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            coc_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let bokeh_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            bokeh_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let camera = self.frame.effective_camera();
        let scene_linear_size = stack.scene_linear_size(self.frame, pass_name)?;
        let mut params_uploads = stack.post_process.execute_depth_of_field_prepare(
            self.device,
            self.encoder,
            scene_linear_size,
            super::post_process_texture_origin(self.frame, scene_color_resource_name),
            scene_color_view,
            scene_depth_view,
            coc_view,
            bokeh_view,
            self.frame.post_process().effect_stack.depth_of_field,
            &camera,
        );
        self.append_pre_submit_buffer_uploads(&mut params_uploads);
        Ok(())
    }
}
