use crate::graphics::pipeline::RenderPassStage;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_sprite_stage_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
        stage: RenderPassStage,
        attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let sprite_renderer = self.sprite_renderer.ok_or_else(|| {
            format!("sprite graph executor for stage `{stage:?}` requires sprite renderer context")
        })?;
        let streamer = self.streamer.ok_or_else(|| {
            format!(
                "sprite graph executor for stage `{stage:?}` requires resource streamer context"
            )
        })?;
        sprite_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            streamer,
            self.frame,
            stage,
            render_region,
            attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_screen_space_ui_to_resource(
        &mut self,
        resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let color_view = Self::require_texture_view_by_name(
            resources,
            self.resource_resolver,
            resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        self.screen_space_ui_renderer.record(
            self.device,
            self.queue,
            self.encoder,
            color_view,
            self.frame,
            attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_preview_sky_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        if self.overlay_renderer.is_none() {
            return Err(format!(
                "preview sky graph executor for pass `{pass_name}` requires preview sky renderer context"
            ));
        }
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let overlay_renderer = self
            .overlay_renderer
            .as_deref_mut()
            .expect("preview sky renderer context was checked before resource resolution");
        overlay_renderer.record_preview_sky_with_attachment_ops(
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            render_region,
            color_attachment_ops,
            depth_attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_overlay_to_resources(
        &mut self,
        pass_name: &str,
        color_resource_name: &str,
        depth_resource_name: &str,
    ) -> Result<(), String> {
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        let depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            depth_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let render_region = self.render_region_for_write_resource(color_resource_name);
        let overlay_renderer = self.overlay_renderer.as_deref_mut().ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires overlay renderer context"
            )
        })?;
        let prepared_overlays = self.prepared_overlays.ok_or_else(|| {
            format!(
                "overlay graph executor for pass `{pass_name}` requires prepared overlay buffers"
            )
        })?;
        overlay_renderer.record_overlays(
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            prepared_overlays,
            render_region,
        );
        Ok(())
    }
}
