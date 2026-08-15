use crate::graphics::scene::scene_renderer::history::SceneFrameHistoryTextures;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::super::RenderPassGpuExecutionContext;
use super::effect_stack_uses_reconstructed_velocity;

impl<'a> RenderPassGpuExecutionContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer) fn record_taa_resolve_to_resources(
        &mut self,
        pass_name: &str,
        scene_color_resource_name: &str,
        scene_depth_resource_name: &str,
        scene_velocity_resource_name: &str,
        taa_history_previous_resource_name: &str,
        taa_reactive_mask_resource_name: &str,
        taa_output_resource_name: &str,
        taa_history_current_resource_name: &str,
        taa_output_attachment_ops: RenderGraphAttachmentOps,
        taa_history_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "TAA resolve graph executor for pass `{pass_name}` requires post-process stack context"
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
        let scene_velocity_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_velocity_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let taa_history_previous_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_history_previous_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let taa_reactive_mask_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_reactive_mask_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let taa_output_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_output_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let taa_history_current_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            taa_history_current_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let taa_history_valid = stack
            .history_textures
            .is_some_and(SceneFrameHistoryTextures::taa_scene_color_history_valid);
        let bind_group_created = stack.post_process.execute_taa_resolve(
            self.device,
            self.queue,
            self.encoder,
            stack.target.size,
            scene_color_view,
            scene_depth_view,
            scene_velocity_view,
            taa_history_previous_view,
            taa_reactive_mask_view,
            taa_output_view,
            taa_history_current_view,
            resources.texture_identity(scene_color_resource_name),
            resources.texture_identity(scene_depth_resource_name),
            resources.texture_identity(scene_velocity_resource_name),
            resources.texture_identity(taa_history_previous_resource_name),
            resources.texture_identity(taa_history_current_resource_name),
            resources.texture_identity(taa_reactive_mask_resource_name),
            taa_output_attachment_ops,
            taa_history_attachment_ops,
            taa_history_valid,
            self.frame.extract.view.anti_alias,
        );
        if bind_group_created {
            self.record_taa_resolve_bind_group_create();
        }
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_velocity_camera_to_resource(
        &mut self,
        pass_name: &str,
        scene_depth_resource_name: &str,
        velocity_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "velocity camera graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let velocity_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            velocity_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let camera = self.frame.effective_camera();
        self.motion_vector_camera_status = stack.post_process.execute_velocity_camera(
            self.device,
            self.queue,
            self.encoder,
            stack.target.size,
            scene_depth_view,
            velocity_view,
            attachment_ops,
            &camera,
            self.frame.previous_motion_vector_camera(),
            effect_stack_uses_reconstructed_velocity(self.frame.extract.post_process.effect_stack),
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
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let motion_vector_source_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            motion_vector_source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_tile_max_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            motion_vector_tile_max_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_motion_vector_tile_max(
            self.device,
            self.encoder,
            motion_vector_source_view,
            motion_vector_tile_max_view,
            attachment_ops,
            effect_stack_uses_reconstructed_velocity(self.frame.extract.post_process.effect_stack),
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
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let motion_vector_tile_max_coarse_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            motion_vector_tile_max_coarse_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let motion_vector_neighbor_max_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            motion_vector_neighbor_max_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_motion_vector_neighbor_max(
            self.device,
            self.encoder,
            motion_vector_tile_max_coarse_view,
            motion_vector_neighbor_max_view,
            attachment_ops,
            effect_stack_uses_reconstructed_velocity(self.frame.extract.post_process.effect_stack),
        );
        Ok(())
    }
}
