use super::super::super::primitives::{
    build_handle_vertices, build_line_buffer, build_selection_vertices, build_wireframe_vertices,
};
use super::super::PreparedOverlayBuffers;
use super::viewport_overlay_renderer::ViewportOverlayRenderer;
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
            selection_buffer: build_line_buffer(
                device,
                "zircon-selection-buffer",
                &build_selection_vertices(frame, streamer),
            ),
            wireframe_buffer: build_line_buffer(
                device,
                "zircon-wireframe-buffer",
                &build_wireframe_vertices(frame, streamer),
            ),
            scene_gizmo: self
                .scene_gizmo
                .prepare(device, queue, texture_layout, frame)?,
            handle_buffer: build_line_buffer(
                device,
                "zircon-handle-buffer",
                &build_handle_vertices(frame),
            ),
        })
    }
}
