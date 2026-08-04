use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::RenderGraphAttachmentOps;

use super::super::screen_space_ui_renderer::ScreenSpaceUiRenderer;
use super::{prepare_screen_space_ui, ScreenSpaceUiVertex};

const SCREEN_SPACE_UI_MIN_VERTEX_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;

impl ScreenSpaceUiRenderer {
    pub(crate) fn record(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
        streamer: Option<&ResourceStreamer>,
    ) -> Result<(), GraphicsError> {
        let pass_clear_color = wgpu::Color::TRANSPARENT;
        self.last_attachment_ops = attachment_ops;
        let Some(prepared) = prepare_screen_space_ui(frame, attachment_ops, pass_clear_color)
        else {
            self.last_text_prepare_report = Default::default();
            self.image_system.clear_frame_state();
            record_empty_screen_space_ui_pass(
                encoder,
                color_view,
                attachment_ops,
                pass_clear_color,
            );
            return Ok(());
        };
        self.write_screen_space_ui_vertex_buffer(device, queue, prepared.vertices.as_slice());
        self.text_system
            .prepare(
                device,
                queue,
                frame.viewport_size,
                &prepared.auto_texts,
                &prepared.native_texts,
                &prepared.sdf_texts,
            )
            .map_err(|error| GraphicsError::Asset(error.to_string()))?;
        self.last_text_prepare_report = self.text_system.prepare_report();
        let prepared_images = self.image_system.prepare(
            device,
            queue,
            frame.viewport_size,
            &prepared.images,
            streamer,
        );

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("zircon-screen-space-ui-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: color_attachment_operations(attachment_ops, pass_clear_color),
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        }

        for draw in &prepared.draws {
            pass.set_scissor_rect(
                draw.scissor.x,
                draw.scissor.y,
                draw.scissor.width,
                draw.scissor.height,
            );
            pass.draw(draw.vertices.clone(), 0..1);
        }
        self.image_system.render(&mut pass, &prepared_images);
        pass.set_scissor_rect(
            0,
            0,
            frame.viewport_size.x.max(1),
            frame.viewport_size.y.max(1),
        );
        self.text_system.render(&mut pass);

        if !prepared.post_text_draws.is_empty() {
            pass.set_pipeline(&self.pipeline);
            if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
                pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            }
            for draw in &prepared.post_text_draws {
                pass.set_scissor_rect(
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.width,
                    draw.scissor.height,
                );
                pass.draw(draw.vertices.clone(), 0..1);
            }
        }
        Ok(())
    }

    pub(crate) fn text_prepare_report(&self) -> super::super::text::ScreenSpaceUiTextPrepareReport {
        self.last_text_prepare_report.clone()
    }

    #[cfg(test)]
    pub(crate) fn last_attachment_ops(&self) -> RenderGraphAttachmentOps {
        self.last_attachment_ops
    }

    fn write_screen_space_ui_vertex_buffer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        vertices: &[ScreenSpaceUiVertex],
    ) {
        if vertices.is_empty() {
            return;
        }

        let vertex_bytes = bytemuck::cast_slice(vertices);
        let required_byte_len = vertex_bytes.len();
        let requires_reallocation = self.vertex_buffer.is_none()
            || self.vertex_buffer_capacity_bytes < required_byte_len as u64;
        if requires_reallocation {
            self.vertex_buffer_capacity_bytes =
                screen_space_ui_vertex_buffer_capacity(required_byte_len);
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("zircon-screen-space-ui-vertices"),
                size: self.vertex_buffer_capacity_bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        let payload_hash = *blake3::hash(vertex_bytes).as_bytes();
        let write_required = screen_space_ui_vertex_buffer_write_required(
            requires_reallocation,
            self.vertex_buffer_payload_hash,
            payload_hash,
        );
        if write_required {
            if let Some(vertex_buffer) = self.vertex_buffer.as_ref() {
                queue.write_buffer(vertex_buffer, 0, vertex_bytes);
                self.vertex_buffer_payload_hash = Some(payload_hash);
            }
        }
    }
}

pub(super) fn screen_space_ui_vertex_buffer_write_required(
    requires_reallocation: bool,
    current_payload_hash: Option<[u8; 32]>,
    next_payload_hash: [u8; 32],
) -> bool {
    requires_reallocation || current_payload_hash != Some(next_payload_hash)
}

fn screen_space_ui_vertex_buffer_capacity(required_byte_len: usize) -> u64 {
    (required_byte_len as u64)
        .max(SCREEN_SPACE_UI_MIN_VERTEX_BUFFER_CAPACITY_BYTES)
        .checked_next_power_of_two()
        .unwrap_or(required_byte_len as u64)
}

fn record_empty_screen_space_ui_pass(
    encoder: &mut wgpu::CommandEncoder,
    color_view: &wgpu::TextureView,
    attachment_ops: RenderGraphAttachmentOps,
    clear_color: wgpu::Color,
) {
    if attachment_ops == RenderGraphAttachmentOps::load_store() {
        return;
    }
    let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("zircon-screen-space-ui-empty-pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: color_view,
            depth_slice: None,
            resolve_target: None,
            ops: color_attachment_operations(attachment_ops, clear_color),
        })],
        depth_stencil_attachment: None,
        occlusion_query_set: None,
        timestamp_writes: None,
        multiview_mask: None,
    });
}
