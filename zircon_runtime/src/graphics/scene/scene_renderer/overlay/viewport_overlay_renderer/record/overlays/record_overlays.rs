use crate::graphics::scene::scene_renderer::overlay::{
    PreparedOverlayBuffers, ViewportOverlayRenderer,
};
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};

impl ViewportOverlayRenderer {
    /// 在最终场景目标上依次叠加选中线框、显示线框、网格、实体辅助图形和操纵手柄。
    /// prepared 必须来自同帧准备，图标上传必须在绘制前进入本帧提交事务。
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
#[path = "tests/record_overlays.rs"]
mod tests;
