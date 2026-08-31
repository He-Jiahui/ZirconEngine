use super::image::ScreenSpaceUiImageSystem;
use std::sync::Weak;

use super::render::{PlannedScreenSpaceUi, PreparedScreenSpaceUi, ScreenSpaceUiPlanCache};
use super::resource_upload::{ScreenSpaceUiPreparedUpload, ScreenSpaceUiUploadTransactionState};
use super::text::ScreenSpaceUiTextSystem;
use crate::render_graph::RenderGraphAttachmentOps;

pub(crate) struct ScreenSpaceUiRenderer {
    pub(super) pipeline: wgpu::RenderPipeline,
    pub(super) vertex_segments: Vec<ScreenSpaceUiVertexSegmentBuffer>,
    pub(super) vertex_buffer_plan: Option<Weak<PreparedScreenSpaceUi>>,
    pub(super) image_system: ScreenSpaceUiImageSystem,
    pub(super) plan_cache: ScreenSpaceUiPlanCache,
    pub(super) text_system: ScreenSpaceUiTextSystem,
    pub(super) text_prepare_report_valid: bool,
    pub(super) last_attachment_ops: RenderGraphAttachmentOps,
    pub(super) upload_transaction: ScreenSpaceUiUploadTransactionState,
}

impl ScreenSpaceUiRenderer {
    pub(in crate::graphics::scene::scene_renderer) fn commit_prepared_upload(
        &mut self,
        prepared: ScreenSpaceUiPreparedUpload,
    ) -> bool {
        if !self.upload_transaction.commit(prepared) {
            return false;
        }
        self.text_system.commit_prepared_uploads();
        true
    }
}

#[derive(Default)]
pub(super) struct ScreenSpaceUiVertexSegmentBuffer {
    pub(super) buffer: Option<wgpu::Buffer>,
    pub(super) capacity_bytes: u64,
    pub(super) payload_hash: Option<[u8; 32]>,
    pub(super) plan: Option<Weak<PlannedScreenSpaceUi>>,
}
