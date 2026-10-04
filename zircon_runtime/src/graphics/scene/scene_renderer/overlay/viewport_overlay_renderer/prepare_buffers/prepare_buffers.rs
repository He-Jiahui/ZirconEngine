use super::super::super::{PreparedOverlayBuffers, PreparedSceneGizmoPass};
use super::super::viewport_overlay_renderer::ViewportOverlayRenderer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use zr_rhi_wgpu::WgpuTextureUploadBatch;

impl ViewportOverlayRenderer {
    /// 在图执行前生成当前帧的辅助缓冲，并累计图标上传债务。
    /// 返回值不能复用于其他相机；交互资源缺席时直接给出空层，环境捕获无需准备辅助几何。
    pub(crate) fn prepare_buffers(
        &mut self,
        device: &wgpu::Device,
        texture_layout: &wgpu::BindGroupLayout,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
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
                texture_layout,
                frame,
                frame_texture_uploads,
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
#[path = "tests/prepare_buffers.rs"]
mod tests;
