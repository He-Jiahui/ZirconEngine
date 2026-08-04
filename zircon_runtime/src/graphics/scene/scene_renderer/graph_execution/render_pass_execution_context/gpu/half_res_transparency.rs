use crate::core::framework::render::PostProcessGraphResourceNames;
use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_half_resolution_transparency_depth_downsample(
        &mut self,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            "half-resolution transparency depth executor requires post-process resources"
                .to_string()
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let source_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let half_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
            RenderGraphResourceAccessKind::Write,
        )?;
        let half_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
            RenderGraphResourceAccessKind::Write,
        )?;
        let render_region = self.render_region_for_write_resource(
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
        );
        stack
            .post_process()
            .execute_half_resolution_transparency_depth_downsample(
                self.device,
                self.encoder,
                source_depth_view,
                half_color_view,
                half_depth_view,
                render_region,
                color_attachment_ops,
                depth_attachment_ops,
            );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_half_resolution_transparency_composite(
        &mut self,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            "half-resolution transparency composite executor requires post-process resources"
                .to_string()
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let half_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_COLOR,
            RenderGraphResourceAccessKind::Read,
        )?;
        let half_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::HALF_RES_TRANSPARENCY_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let full_depth_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_DEPTH,
            RenderGraphResourceAccessKind::Read,
        )?;
        let scene_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            PostProcessGraphResourceNames::SCENE_COLOR,
            RenderGraphResourceAccessKind::Write,
        )?;
        let render_region =
            self.render_region_for_write_resource(PostProcessGraphResourceNames::SCENE_COLOR);
        stack
            .post_process()
            .execute_half_resolution_transparency_composite(
                self.device,
                self.queue,
                self.encoder,
                half_color_view,
                half_depth_view,
                full_depth_view,
                scene_color_view,
                render_region,
                attachment_ops,
                self.half_resolution_transparency_depth_sigma,
            );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn half_resolution_transparency_context_uses_declared_graph_resources() {
        let source = include_str!("half_res_transparency.rs");

        assert!(source.contains("HALF_RES_TRANSPARENCY_COLOR"));
        assert!(source.contains("HALF_RES_TRANSPARENCY_DEPTH"));
        assert!(source.contains("RenderGraphResourceAccessKind::Write"));
    }
}
