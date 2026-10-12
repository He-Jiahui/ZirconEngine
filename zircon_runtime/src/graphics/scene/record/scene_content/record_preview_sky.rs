use crate::graphics::scene::scene_renderer::overlay::ViewportOverlayRenderer;
use crate::graphics::types::{ViewportRenderFrame, ViewportRenderRegion};
use crate::render_graph::RenderGraphAttachmentOps;

impl ViewportOverlayRenderer {
    /// 直接场景和环境捕获的天空入口；使用清理附件策略，调用方需提供独占的当前场景目标。
    pub(crate) fn record_preview_sky(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
    ) {
        self.preview_sky.record(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            &self.sky_pipeline,
            &self.sky_volumetric_layout,
            &self.sky_volumetric_apply,
            frame,
        );
    }

    /// 编译图入口复用天空资源，附件加载策略和区域由图传入，避免隐式清除共享目标。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_preview_sky_with_attachment_ops(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        frame: &ViewportRenderFrame,
        render_region: ViewportRenderRegion,
        color_attachment_ops: RenderGraphAttachmentOps,
        depth_attachment_ops: RenderGraphAttachmentOps,
        integrated_volumetric_view: Option<&wgpu::TextureView>,
    ) {
        self.preview_sky.record_with_attachment_ops(
            encoder,
            device,
            color_view,
            depth_view,
            scene_bind_group,
            &self.sky_pipeline,
            &self.sky_volumetric_layout,
            &self.sky_volumetric_apply,
            integrated_volumetric_view,
            frame,
            render_region,
            color_attachment_ops,
            depth_attachment_ops,
        );
    }
}
