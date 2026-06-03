use super::text::{ScreenSpaceUiTextPrepareReport, ScreenSpaceUiTextSystem};
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct ScreenSpaceUiRenderer {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) text_system: ScreenSpaceUiTextSystem,
    pub(super) last_text_prepare_report: ScreenSpaceUiTextPrepareReport,
    pub(super) last_attachment_ops: RenderGraphAttachmentOps,
}
