use super::image::ScreenSpaceUiImageSystem;
use super::text::{ScreenSpaceUiTextPrepareReport, ScreenSpaceUiTextSystem};
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct ScreenSpaceUiRenderer {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) vertex_buffer: Option<wgpu::Buffer>,
    pub(super) vertex_buffer_capacity_bytes: u64,
    pub(super) vertex_buffer_payload_hash: Option<[u8; 32]>,
    pub(super) image_system: ScreenSpaceUiImageSystem,
    pub(super) text_system: ScreenSpaceUiTextSystem,
    pub(super) last_text_prepare_report: ScreenSpaceUiTextPrepareReport,
    pub(super) last_attachment_ops: RenderGraphAttachmentOps,
}
