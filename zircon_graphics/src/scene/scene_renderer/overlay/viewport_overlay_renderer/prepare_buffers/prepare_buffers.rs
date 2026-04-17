use super::super::super::PreparedOverlayBuffers;
use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;
use crate::scene::resources::ResourceStreamer;
use crate::types::{EditorOrRuntimeFrame, GraphicsError};

impl ViewportOverlayRenderer {
    pub(crate) fn prepare_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        frame: &EditorOrRuntimeFrame,
    ) -> Result<PreparedOverlayBuffers, GraphicsError> {
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
            scene_gizmo: self
                .scene_gizmo
                .prepare(device, queue, texture_layout, frame)?,
            handle_buffer: super::super::super::super::primitives::build_line_buffer(
                device,
                "zircon-handle-buffer",
                &super::super::super::super::primitives::build_handle_vertices(frame),
            ),
        })
    }
}
