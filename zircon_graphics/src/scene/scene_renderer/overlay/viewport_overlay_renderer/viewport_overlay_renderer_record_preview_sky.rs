use super::viewport_overlay_renderer::ViewportOverlayRenderer;
use crate::types::EditorOrRuntimeFrame;

impl ViewportOverlayRenderer {
    pub(crate) fn record_preview_sky(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &EditorOrRuntimeFrame,
    ) {
        self.preview_sky.record(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.sky_pipeline,
            frame,
        );
    }
}
