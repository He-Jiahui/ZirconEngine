use crate::render_graph::RenderGraphAttachmentOps;

use super::RenderPassGpuExecutionContext;

impl RenderPassGpuExecutionContext<'_> {
    pub fn record_particle_billboards_to_resources(
        &mut self,
        color_resource_name: &str,
        depth_resource_name: &str,
    ) -> Result<(), String> {
        let color_view = self.resources.require_texture_view(color_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let particle_renderer = self.particle_renderer.ok_or_else(|| {
            format!(
                "particle graph executor requires particle renderer context for resources `{color_resource_name}` and `{depth_resource_name}`"
            )
        })?;
        particle_renderer.record(
            self.device,
            self.encoder,
            color_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
        );
        Ok(())
    }

    pub(in crate::graphics::scene::scene_renderer) fn record_particle_velocity_to_resource(
        &mut self,
        pass_name: &str,
        velocity_resource_name: &str,
        depth_resource_name: &str,
        attachment_ops: RenderGraphAttachmentOps,
    ) -> Result<(), String> {
        let velocity_view = self
            .resources
            .require_texture_view(velocity_resource_name)?;
        let depth_view = self.resources.require_texture_view(depth_resource_name)?;
        let particle_renderer = self.particle_renderer.ok_or_else(|| {
            format!(
                "particle velocity graph executor for pass `{pass_name}` requires particle renderer context for resources `{velocity_resource_name}` and `{depth_resource_name}`"
            )
        })?;
        particle_renderer.record_velocity(
            self.device,
            self.encoder,
            pass_name,
            velocity_view,
            depth_view,
            self.scene_bind_group,
            self.frame,
            attachment_ops,
        );
        Ok(())
    }
}
