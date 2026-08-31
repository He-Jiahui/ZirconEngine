use zr_rhi_wgpu::{WgpuBufferUpload, WgpuBufferUploadBatch, WgpuTextureUploadBatch};

use crate::graphics::scene::scene_renderer::ui::ScreenSpaceUiPreparedUpload;

use super::RenderPassGpuExecutionContext;

/// Pass-scoped capability for CPU buffer writes that must execute before the
/// graph command buffers. It deliberately exposes no queue or submit access.
pub struct RenderPassBufferUploadRecorder<'a> {
    uploads: &'a mut WgpuBufferUploadBatch,
}

pub trait RenderPassBufferUploadSink {
    fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]);
}

impl<'a> RenderPassBufferUploadRecorder<'a> {
    pub(crate) fn new(uploads: &'a mut WgpuBufferUploadBatch) -> Self {
        Self { uploads }
    }

    fn record_write(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        self.uploads
            .push(WgpuBufferUpload::from_bytes(buffer.clone(), offset, bytes));
    }
}

impl RenderPassBufferUploadSink for RenderPassBufferUploadRecorder<'_> {
    fn write_buffer(&mut self, buffer: &wgpu::Buffer, offset: u64, bytes: &[u8]) {
        self.record_write(buffer, offset, bytes);
    }
}

impl RenderPassGpuExecutionContext<'_> {
    /// Returns the pass-scoped writer for CPU data consumed by graph commands.
    ///
    /// The recorder is backed by this pass result and is merged into the frame resource upload
    /// packet only after the graph executor succeeds. It carries no native queue authority.
    pub fn buffer_upload_recorder(&mut self) -> RenderPassBufferUploadRecorder<'_> {
        RenderPassBufferUploadRecorder::new(&mut self.buffer_uploads)
    }

    /// Moves feature-owned CPU initialization writes into this pass result.
    /// Targets must have one CPU producer per frame because the collected
    /// writes execute before all graph command buffers.
    pub fn append_pre_submit_buffer_uploads(&mut self, uploads: &mut WgpuBufferUploadBatch) {
        self.buffer_uploads.append(uploads);
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_buffer_uploads(
        &mut self,
    ) -> WgpuBufferUploadBatch {
        std::mem::take(&mut self.buffer_uploads)
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_texture_uploads(
        &mut self,
    ) -> WgpuTextureUploadBatch {
        std::mem::take(&mut self.texture_uploads)
    }

    pub(in crate::graphics::scene::scene_renderer) fn push_screen_space_ui_upload_commit(
        &mut self,
        prepared: ScreenSpaceUiPreparedUpload,
    ) {
        self.screen_space_ui_upload_commits.push(prepared);
    }

    pub(in crate::graphics::scene::scene_renderer) fn take_screen_space_ui_upload_commits(
        &mut self,
    ) -> Vec<ScreenSpaceUiPreparedUpload> {
        std::mem::take(&mut self.screen_space_ui_upload_commits)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn temporal_and_bloom_passes_append_pre_submit_uploads_to_the_pass_context() {
        let temporal = include_str!("post_process/temporal.rs");
        let effects = include_str!("post_process/effects.rs");

        let taa = temporal
            .find("execute_taa_resolve(")
            .expect("TAA executor call");
        let velocity = temporal
            .find("execute_velocity_camera(")
            .expect("velocity executor call");
        assert!(temporal[taa..velocity].contains("self.append_pre_submit_buffer_uploads("));
        assert!(temporal[velocity..].contains("self.append_pre_submit_buffer_uploads("));

        let bloom = effects.find("execute_bloom(").expect("bloom executor call");
        assert!(effects[bloom..].contains("self.append_pre_submit_buffer_uploads("));
    }
}
