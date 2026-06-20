use crate::render_graph::{RenderGraphAttachmentOps, RenderGraphResourceAccessKind};

use super::super::RenderPassGpuExecutionContext;

impl<'a> RenderPassGpuExecutionContext<'a> {
    pub(in crate::graphics::scene::scene_renderer) fn record_output_transfer_to_resource(
        &mut self,
        pass_name: &str,
        tonemapped_resource_name: &str,
        final_color_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "output-transfer graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let tonemapped_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            tonemapped_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let final_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            final_color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_output_transfer(
            self.device,
            self.encoder,
            tonemapped_view,
            final_color_view,
            final_color_resource_name,
            attachment_ops,
            self.frame.render_region(),
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_upscale_to_resource(
        &mut self,
        pass_name: &str,
        source_resource_name: &str,
        upscaled_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "upscale graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let source_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            source_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let upscaled_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            upscaled_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_upscale(
            self.device,
            self.encoder,
            source_view,
            upscaled_view,
            attachment_ops,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_fxaa_to_resource(
        &mut self,
        pass_name: &str,
        terminal_input_resource_name: &str,
        final_color_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "FXAA graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let terminal_input_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            terminal_input_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let final_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            final_color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_fxaa(
            self.device,
            self.encoder,
            terminal_input_view,
            final_color_view,
            attachment_ops,
            self.frame.render_region(),
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_smaa_to_resource(
        &mut self,
        pass_name: &str,
        terminal_input_resource_name: &str,
        final_color_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let stack = self.post_process_stack.ok_or_else(|| {
            format!(
                "SMAA graph executor for pass `{pass_name}` requires post-process stack context"
            )
        })?;
        let resources = &*self.resources;
        let resource_resolver = self.resource_resolver;
        let terminal_input_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            terminal_input_resource_name,
            RenderGraphResourceAccessKind::Read,
        )?;
        let final_color_view = Self::require_texture_view_by_name(
            resources,
            resource_resolver,
            final_color_resource_name,
            RenderGraphResourceAccessKind::Write,
        )?;
        stack.post_process.execute_smaa(
            self.device,
            self.encoder,
            stack.target.size,
            terminal_input_view,
            final_color_view,
            attachment_ops,
            self.frame.render_region(),
        );
        Ok(())
    }
}
