use crate::graphics::scene::scene_renderer::overlay::begin_line_pass;
use crate::graphics::types::EditorOrRuntimeFrame;

pub(crate) struct GridPass;

impl GridPass {
    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        line_pipeline: &wgpu::RenderPipeline,
        grid_buffer: &wgpu::Buffer,
        grid_count: u32,
        frame: &EditorOrRuntimeFrame,
    ) {
        if !frame
            .scene
            .overlays
            .grid
            .as_ref()
            .is_some_and(|grid| grid.visible)
        {
            return;
        }
        let mut pass = begin_line_pass(encoder, "GridPass", color_view, depth_view);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_pipeline(line_pipeline);
        pass.set_vertex_buffer(0, grid_buffer.slice(..));
        pass.draw(0..grid_count, 0..1);
    }
}
