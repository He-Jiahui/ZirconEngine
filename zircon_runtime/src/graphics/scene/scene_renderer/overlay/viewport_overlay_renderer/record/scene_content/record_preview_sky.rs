use crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::graphics::types::ViewportRenderFrame;
use crate::render_graph::RenderGraphAttachmentOps;

impl ViewportOverlayRenderer {
    pub(crate) fn record_preview_sky(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_preview_sky_with_attachment_ops(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
    ) {
        self.preview_sky.record_with_attachment_ops(
            encoder,
            color_view,
            depth_view,
            scene_bind_group,
            &self.sky_pipeline,
            frame,
            color_attachment_ops,
            depth_attachment_ops,
        );
    }
}
