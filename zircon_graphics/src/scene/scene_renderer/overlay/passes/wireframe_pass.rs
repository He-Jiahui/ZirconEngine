use zircon_scene::DisplayMode;

use crate::scene::scene_renderer::overlay::begin_line_pass;
use crate::types::EditorOrRuntimeFrame;

pub(crate) struct WireframePass;

impl WireframePass {
    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        line_pipeline: &wgpu::RenderPipeline,
        buffer: Option<&(wgpu::Buffer, u32)>,
        frame: &EditorOrRuntimeFrame,
    ) {
        if frame.scene.overlays.display_mode == DisplayMode::Shaded {
            return;
        }
        let Some((buffer, count)) = buffer else {
            return;
        };
        let mut pass = begin_line_pass(encoder, "WireframePass", color_view, depth_view);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_pipeline(line_pipeline);
        pass.set_vertex_buffer(0, buffer.slice(..));
        pass.draw(0..*count, 0..1);
    }
}
