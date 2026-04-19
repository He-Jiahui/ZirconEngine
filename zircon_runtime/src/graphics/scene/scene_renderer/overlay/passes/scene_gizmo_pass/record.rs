use crate::graphics::scene::scene_renderer::overlay::{begin_line_pass, PreparedIconDraw};

use super::scene_gizmo_pass::SceneGizmoPass;

impl SceneGizmoPass {
    pub(crate) fn record(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        line_pipeline: &wgpu::RenderPipeline,
        line_buffer: Option<&(wgpu::Buffer, u32)>,
        icon_draws: &[PreparedIconDraw],
    ) {
        if line_buffer.is_none() && icon_draws.is_empty() {
            return;
        }
        let mut pass = begin_line_pass(encoder, "SceneGizmoPass", color_view, depth_view);
        pass.set_bind_group(0, scene_bind_group, &[]);
        if let Some((buffer, count)) = line_buffer {
            pass.set_pipeline(line_pipeline);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..*count, 0..1);
        }
        if !icon_draws.is_empty() {
            pass.set_pipeline(&self.icon_pipeline);
            for draw in icon_draws {
                pass.set_bind_group(1, draw.bind_group.as_ref(), &[]);
                pass.set_vertex_buffer(0, draw.vertex_buffer.slice(..));
                pass.draw(0..draw.vertex_count, 0..1);
            }
        }
    }
}
