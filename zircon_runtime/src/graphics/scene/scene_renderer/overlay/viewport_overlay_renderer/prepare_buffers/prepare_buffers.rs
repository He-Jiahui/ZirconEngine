use super::super::super::{PreparedOverlayBuffers, PreparedSceneGizmoPass};
use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl ViewportOverlayRenderer {
    pub(crate) fn prepare_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<PreparedOverlayBuffers, GraphicsError> {
        let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {
            return Ok(PreparedOverlayBuffers {
                selection_buffer: None,
                wireframe_buffer: None,
                scene_gizmo: PreparedSceneGizmoPass {
                    line_buffer: None,
                    icon_draws: Vec::new(),
                },
                handle_buffer: None,
            });
        };

        Ok(PreparedOverlayBuffers {
            selection_buffer: super::super::super::super::primitives::build_line_buffer(
                device,
                "zircon-selection-buffer",
                &super::super::super::super::primitives::build_selection_vertices(frame, streamer),
            ),
            wireframe_buffer: super::super::super::super::primitives::build_line_buffer(
                device,
                "zircon-wireframe-buffer",
                &super::super::super::super::primitives::build_wireframe_vertices(frame, streamer),
            ),
            scene_gizmo: interaction_overlays.scene_gizmo.prepare(
                device,
                queue,
                texture_layout,
                frame,
            )?,
            handle_buffer: super::super::super::super::primitives::build_line_buffer(
                device,
                "zircon-handle-buffer",
                &super::super::super::super::primitives::build_handle_vertices(frame),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("prepare_buffers.rs");

    fn production_source() -> &'static str {
        SOURCE
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("overlay preparation should retain a test-module boundary")
    }

    #[test]
    fn disabled_interaction_overlays_skip_cpu_vertex_generation() {
        let source = production_source();
        let interaction_guard = source
            .find("let Some(interaction_overlays) = self.interaction_overlays.as_mut() else {")
            .expect("overlay preparation should exit when interaction resources are absent");
        let selection_generation = source
            .find("build_selection_vertices")
            .expect("interactive overlays should still prepare selection geometry");

        assert!(
            interaction_guard < selection_generation,
            "the EnvironmentOnly path must exit before generating interaction vertices"
        );
        assert!(source.contains("selection_buffer: None"));
        assert!(source.contains("wireframe_buffer: None"));
        assert!(source.contains("handle_buffer: None"));
    }
}
