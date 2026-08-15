use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};

impl ViewportOverlayRenderer {
    pub(crate) fn record_overlays(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
        prepared: &PreparedOverlayBuffers,
        render_region: ViewportRenderRegion,
    ) {
        let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {
            return;
        };

        interaction_overlays.selection_outline.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &interaction_overlays.line_pipeline,
            prepared.selection_buffer.as_ref(),
            render_region,
        );
        interaction_overlays.wireframe.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &interaction_overlays.line_pipeline,
            prepared.wireframe_buffer.as_ref(),
            frame,
            render_region,
        );
        interaction_overlays.grid.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &interaction_overlays.line_pipeline,
            &interaction_overlays.grid_vertex_buffer,
            interaction_overlays.grid_vertex_count,
            frame,
            render_region,
        );
        interaction_overlays.scene_gizmo.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &interaction_overlays.line_pipeline,
            prepared.scene_gizmo.line_buffer.as_ref(),
            &prepared.scene_gizmo.icon_draws,
            render_region,
        );
        interaction_overlays.handle.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &interaction_overlays.line_pipeline,
            prepared.handle_buffer.as_ref(),
            render_region,
        );
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("record_overlays.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("overlay recording should retain a test-module boundary")
    }

    #[test]
    fn disabled_interaction_overlays_skip_render_pass_recording() {
        let source = production_source();
        let interaction_guard = source
            .find("let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {")
            .expect("overlay recording should exit when interaction resources are absent");
        let selection_record = source
            .find("interaction_overlays.selection_outline.record(")
            .expect("interactive overlays should still record selection outlines");

        assert!(
            interaction_guard < selection_record,
            "the EnvironmentOnly path must exit before recording overlay passes"
        );
    }
}
