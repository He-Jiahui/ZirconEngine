use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_deferred_gbuffer_to_resources(
        &mut self,
        pass_name: &str,
        gbuffer_albedo_resource_name: &str,
        gbuffer_normal_resource_name: &str,
        gbuffer_material_resource_name: &str,
        gbuffer_emissive_resource_name: &str,
        depth_resource_name: &str,
        albedo_attachment_ops: RenderGraphAttachmentOps,
        normal_attachment_ops: RenderGraphAttachmentOps,
        material_attachment_ops: RenderGraphAttachmentOps,
        emissive_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let gbuffer_albedo_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_albedo_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let gbuffer_material_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_material_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let gbuffer_emissive_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_emissive_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let gbuffer_normal_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_normal_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let deferred = self.deferred.ok_or_else(|| {
            format!(
                "deferred graph executor for pass `{pass_name}` requires deferred renderer context"
            )
        })?;
        let render_region = self.render_region();
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("deferred graph executor for pass `{pass_name}` requires mesh draw context")
        })?;
        let mesh_pipelines = self.mesh_pipelines.as_deref_mut().ok_or_else(|| {
            format!("deferred graph executor for pass `{pass_name}` requires mesh pipeline context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "deferred graph executor for pass `{pass_name}` requires resource streamer context"
            )
        })?;
        let replay_stats = deferred.record_gbuffer_geometry(
            self.device,
            self.encoder,
            gbuffer_albedo_view,
            gbuffer_normal_view,
            gbuffer_material_view,
            gbuffer_emissive_view,
            depth_view,
            self.scene_bind_group,
            mesh_draw_lists.gpu_scene_bind_group,
            streamer,
            mesh_pipelines,
            albedo_attachment_ops,
            normal_attachment_ops,
            material_attachment_ops,
            emissive_attachment_ops,
            render_region,
            [
                mesh_draw_lists.opaque_stream(),
                mesh_draw_lists.alpha_mask_stream(),
            ],
        );
        mesh_draw_lists.replay_stats.record(replay_stats);
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_deferred_lighting_to_resources(
        &mut self,
        pass_name: &str,
        gbuffer_albedo_resource_name: &str,
        gbuffer_normal_resource_name: &str,
        gbuffer_material_resource_name: &str,
        gbuffer_emissive_resource_name: &str,
        scene_depth_resource_name: &str,
        scene_color_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let gbuffer_albedo_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_albedo_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let gbuffer_normal_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_normal_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let gbuffer_material_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_material_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let gbuffer_emissive_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            gbuffer_emissive_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_grid_params_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_zbins_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_ZBINS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let light_tile_masks_buffer = Self::require_buffer_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
            RenderGraphResourceAccessKind::Read,
        )?;
        let integrated_volumetric_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::VOLUMETRIC_INTEGRATED,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            scene_color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let subsurface_diffuse_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SSS_DIFFUSE,
            RenderGraphResourceAccessKind::Write,
        )?;
        let subsurface_retained_view = Self::optional_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SSS_SPECULAR,
            RenderGraphResourceAccessKind::Write,
        )?;
        if subsurface_diffuse_view.is_some() != subsurface_retained_view.is_some() {
            return Err(format!(
                "deferred graph executor for pass `{pass_name}` requires both SSS MRT resources or neither"
            ));
        }
        let render_region = self.render_region();
        let deferred = self.deferred.ok_or_else(|| {
            format!(
                "deferred graph executor for pass `{pass_name}` requires deferred renderer context"
            )
        })?;
        let mesh_draw_lists = self.mesh_draw_lists.ok_or_else(|| {
            format!("deferred graph executor for pass `{pass_name}` requires mesh draw context")
        })?;
        let gpu_scene_bind_group = mesh_draw_lists
            .gpu_scene_bind_group
            .ok_or_else(|| {
                format!(
                    "deferred graph executor for pass `{pass_name}` requires GPUScene bind group"
                )
            })?
            .bind_group();
        deferred.execute_lighting(
            self.device,
            self.encoder,
            self.scene_bind_group,
            gpu_scene_bind_group,
            gbuffer_albedo_view,
            gbuffer_normal_view,
            gbuffer_material_view,
            gbuffer_emissive_view,
            scene_depth_view,
            self.shadow_atlas_resources,
            light_grid_params_buffer,
            light_zbins_buffer,
            light_tile_masks_buffer,
            integrated_volumetric_view,
            self.frame,
            scene_color_view,
            subsurface_diffuse_view,
            subsurface_retained_view,
            attachment_ops,
            render_region,
        );
        Ok(())
    }
}
