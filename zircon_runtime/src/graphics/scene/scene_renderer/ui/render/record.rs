use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::attachment_ops::color_attachment_operations;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::render_graph::RenderGraphAttachmentOps;
use std::sync::{Arc, Weak};
use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch};

use super::super::resource_upload::ScreenSpaceUiPreparedUpload;
use super::super::screen_space_ui_renderer::{
    ScreenSpaceUiRenderer, ScreenSpaceUiVertexSegmentBuffer,
};
use super::{PlannedScreenSpaceUi, PreparedScreenSpaceUi, framebuffer_background_color};

const SCREEN_SPACE_UI_MIN_VERTEX_BUFFER_CAPACITY_BYTES: u64 = 4 * 1024;

impl ScreenSpaceUiRenderer {
    pub(in crate::graphics::scene::scene_renderer) fn record(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        frame: &ViewportRenderFrame,
        attachment_ops: RenderGraphAttachmentOps,
        streamer: Option<&ResourceStreamer>,
    ) -> Result<ScreenSpaceUiPreparedUpload, GraphicsError> {
        let mut prepared_upload = self.upload_transaction.begin()?;
        let force_full_upload = prepared_upload.force_full_upload();
        let pass_clear_color = wgpu::Color::TRANSPARENT;
        self.last_attachment_ops = attachment_ops;
        let framebuffer_background_color =
            framebuffer_background_color(frame, attachment_ops, pass_clear_color);
        let font_revision = self.text_system.published_font_collection_revision();
        let prepared = frame.ui.as_ref().and_then(|submission| {
            self.plan_cache.prepare_with_font_revision(
                submission,
                frame.viewport_size,
                framebuffer_background_color,
                font_revision,
            )
        });
        let Some(prepared) = prepared else {
            self.vertex_buffer_plan = None;
            for vertex_segment in &mut self.vertex_segments {
                vertex_segment.plan = None;
            }
            if frame.ui.is_none() {
                self.plan_cache.clear();
                self.vertex_segments.clear();
            }
            self.text_prepare_report_valid = false;
            self.text_system.clear_frame_state();
            self.image_system.clear_frame_state();
            record_empty_screen_space_ui_pass(
                encoder,
                color_view,
                attachment_ops,
                pass_clear_color,
            );
            return Ok(prepared_upload);
        };
        self.write_screen_space_ui_vertex_buffers(
            device,
            &prepared,
            prepared_upload.buffer_uploads_mut(),
            force_full_upload,
        );
        self.text_prepare_report_valid = false;
        let (buffer_uploads, texture_uploads) = prepared_upload.resource_uploads_mut();
        self.text_system.prepare(
            device,
            frame.viewport_size,
            &prepared.render_segments,
            prepared.resolved_glyph_artifact_routes,
            buffer_uploads,
            texture_uploads,
            force_full_upload,
        )?;
        self.text_prepare_report_valid = true;
        self.image_system.prepare(
            device,
            frame.viewport_size,
            &prepared.render_segments,
            streamer,
            prepared_upload.buffer_uploads_mut(),
            force_full_upload,
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
        for (segment, vertex_segment) in prepared.render_segments.iter().zip(&self.vertex_segments)
        {
            let Some(vertex_buffer) = vertex_segment.buffer.as_ref() else {
                continue;
            };
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            for draw in &segment.draws {
                pass.set_scissor_rect(
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.width,
                    draw.scissor.height,
                );
                pass.draw(draw.vertices.clone(), 0..1);
            }
        }
        self.image_system.render(&mut pass);
        pass.set_scissor_rect(
            0,
            0,
            frame.viewport_size.x.max(1),
            frame.viewport_size.y.max(1),
        );
        self.text_system.render(&mut pass);

        pass.set_pipeline(&self.pipeline);
        for (segment, vertex_segment) in prepared.render_segments.iter().zip(&self.vertex_segments)
        {
            let Some(vertex_buffer) = vertex_segment.buffer.as_ref() else {
                continue;
            };
            if segment.post_text_draws.is_empty() {
                continue;
            }
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            for draw in &segment.post_text_draws {
                pass.set_scissor_rect(
                    draw.scissor.x,
                    draw.scissor.y,
                    draw.scissor.width,
                    draw.scissor.height,
                );
                pass.draw(draw.vertices.clone(), 0..1);
            }
        }
        prepared_upload.mark_full_upload_prepared();
        Ok(prepared_upload)
    }

    pub(crate) fn text_prepare_report(&self) -> super::super::text::ScreenSpaceUiTextPrepareReport {
        if self.text_prepare_report_valid {
            self.text_system.prepare_report()
        } else {
            Default::default()
        }
    }

    #[cfg(test)]
    pub(crate) fn last_attachment_ops(&self) -> RenderGraphAttachmentOps {
        self.last_attachment_ops
    }

    fn write_screen_space_ui_vertex_buffers(
        &mut self,
        device: &wgpu::Device,
        prepared: &Arc<PreparedScreenSpaceUi>,
        uploads: &mut WgpuBufferUploadBatch,
        force_full_upload: bool,
    ) {
        if !force_full_upload
            && screen_space_ui_vertex_plan_reused(self.vertex_buffer_plan.as_ref(), prepared)
        {
            crate::profile_counter!("runtime", "ui.screen_space_ui_vertex.plan_reuse_count", 1);
            return;
        }
        if self.vertex_segments.len() < prepared.render_segments.len() {
            self.vertex_segments.resize_with(
                prepared.render_segments.len(),
                ScreenSpaceUiVertexSegmentBuffer::default,
            );
        }

        let mut segment_plan_reuse_count = 0_usize;
        let mut segment_hash_count = 0_usize;
        let mut segment_hash_input_bytes = 0_usize;
        let mut segment_write_count = 0_usize;
        let mut segment_write_bytes = 0_usize;
        let mut segment_buffer_allocation_count = 0_usize;
        for (segment, vertex_segment) in prepared
            .render_segments
            .iter()
            .zip(&mut self.vertex_segments)
        {
            if !force_full_upload
                && screen_space_ui_vertex_segment_plan_reused(vertex_segment.plan.as_ref(), segment)
            {
                segment_plan_reuse_count = segment_plan_reuse_count.saturating_add(1);
                continue;
            }

            let vertex_bytes = bytemuck::cast_slice(segment.vertices.as_slice());
            if vertex_bytes.is_empty() {
                vertex_segment.payload_hash = None;
                vertex_segment.plan = Some(Arc::downgrade(segment));
                continue;
            }
            segment_hash_count = segment_hash_count.saturating_add(1);
            segment_hash_input_bytes = segment_hash_input_bytes.saturating_add(vertex_bytes.len());
            let required_byte_len = vertex_bytes.len();
            let requires_reallocation = vertex_segment.buffer.is_none()
                || vertex_segment.capacity_bytes < required_byte_len as u64;
            if requires_reallocation {
                vertex_segment.capacity_bytes =
                    screen_space_ui_vertex_buffer_capacity(required_byte_len);
                vertex_segment.buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("zircon-screen-space-ui-segment-vertices"),
                    size: vertex_segment.capacity_bytes,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
                segment_buffer_allocation_count = segment_buffer_allocation_count.saturating_add(1);
            }

            let payload_hash = *blake3::hash(vertex_bytes).as_bytes();
            if screen_space_ui_vertex_buffer_write_required(
                requires_reallocation || force_full_upload,
                vertex_segment.payload_hash,
                payload_hash,
            ) {
                if let Some(vertex_buffer) = vertex_segment.buffer.as_ref() {
                    uploads.push(WgpuBufferUpload::from_bytes(
                        vertex_buffer.clone(),
                        0,
                        vertex_bytes,
                    ));
                    vertex_segment.payload_hash = Some(payload_hash);
                    segment_write_count = segment_write_count.saturating_add(1);
                    segment_write_bytes = segment_write_bytes.saturating_add(vertex_bytes.len());
                }
            }
            vertex_segment.plan = Some(Arc::downgrade(segment));
        }
        self.vertex_segments
            .truncate(prepared.render_segments.len());
        self.vertex_buffer_plan = Some(Arc::downgrade(prepared));
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                (
                    "ui.screen_space_ui_vertex.segment_plan_reuse_count",
                    segment_plan_reuse_count as f64,
                ),
                (
                    "ui.screen_space_ui_vertex.hash_count",
                    segment_hash_count as f64,
                ),
                (
                    "ui.screen_space_ui_vertex.hash_input_bytes",
                    segment_hash_input_bytes as f64,
                ),
                (
                    "ui.screen_space_ui_vertex.segment_write_count",
                    segment_write_count as f64,
                ),
                (
                    "ui.screen_space_ui_vertex.segment_write_bytes",
                    segment_write_bytes as f64,
                ),
                (
                    "ui.screen_space_ui_vertex.segment_buffer_allocation_count",
                    segment_buffer_allocation_count as f64,
                ),
            ],
        );
    }
}

pub(super) fn screen_space_ui_vertex_plan_reused(
    current: Option<&Weak<PreparedScreenSpaceUi>>,
    next: &Arc<PreparedScreenSpaceUi>,
) -> bool {
    current.is_some_and(|current| std::ptr::eq(current.as_ptr(), Arc::as_ptr(next)))
}

pub(super) fn screen_space_ui_vertex_segment_plan_reused(
    current: Option<&Weak<PlannedScreenSpaceUi>>,
    next: &Arc<PlannedScreenSpaceUi>,
) -> bool {
    current.is_some_and(|current| std::ptr::eq(current.as_ptr(), Arc::as_ptr(next)))
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
