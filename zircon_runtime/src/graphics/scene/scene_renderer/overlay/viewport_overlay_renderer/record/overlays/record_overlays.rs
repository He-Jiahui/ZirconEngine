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
        self.selection_outline.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.line_pipeline,
            prepared.selection_buffer.as_ref(),
            render_region,
        );
        self.wireframe.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.line_pipeline,
            prepared.wireframe_buffer.as_ref(),
            frame,
            render_region,
        );
        self.grid.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.line_pipeline,
            &self.grid_vertex_buffer,
            self.grid_vertex_count,
            frame,
            render_region,
        );
        self.scene_gizmo.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.line_pipeline,
            prepared.scene_gizmo.line_buffer.as_ref(),
            &prepared.scene_gizmo.icon_draws,
            render_region,
        );
        self.handle.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.line_pipeline,
            prepared.handle_buffer.as_ref(),
            render_region,
        );
    }
}
