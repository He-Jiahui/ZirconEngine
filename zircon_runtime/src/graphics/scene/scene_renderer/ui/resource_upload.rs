use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::graphics::types::GraphicsError;
use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

pub(super) struct ScreenSpaceUiUploadTransactionState {
    owner: Arc<()>,
    preparation_outstanding: Arc<AtomicBool>,
    prepared_generation: u64,
    committed_generation: u64,
}

pub(in crate::graphics::scene::scene_renderer) struct ScreenSpaceUiPreparedUpload {
    owner: Arc<()>,
    preparation_outstanding: Arc<AtomicBool>,
    generation: u64,
    force_full_upload: bool,
    full_upload_prepared: bool,
    attached: bool,
    buffer_uploads: WgpuBufferUploadBatch,
    texture_uploads: WgpuTextureUploadBatch,
}

impl Default for ScreenSpaceUiUploadTransactionState {
    fn default() -> Self {
        Self {
            owner: Arc::new(()),
            preparation_outstanding: Arc::new(AtomicBool::new(false)),
            prepared_generation: 0,
            committed_generation: 0,
        }
    }
}

impl ScreenSpaceUiUploadTransactionState {
    pub(super) fn begin(&mut self) -> Result<ScreenSpaceUiPreparedUpload, GraphicsError> {
        self.preparation_outstanding
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                GraphicsError::Asset(
                    "screen-space UI already owns an outstanding resource upload preparation"
                        .to_string(),
                )
            })?;
        let Some(generation) = self.prepared_generation.checked_add(1) else {
            self.preparation_outstanding.store(false, Ordering::Release);
            return Err(GraphicsError::Asset(
                "screen-space UI resource upload generation exhausted".to_string(),
            ));
        };
        let force_full_upload = self.committed_generation != self.prepared_generation;
        self.prepared_generation = generation;
        Ok(ScreenSpaceUiPreparedUpload {
            owner: Arc::clone(&self.owner),
            preparation_outstanding: Arc::clone(&self.preparation_outstanding),
            generation,
            force_full_upload,
            full_upload_prepared: false,
            attached: false,
            buffer_uploads: WgpuBufferUploadBatch::new(),
            texture_uploads: WgpuTextureUploadBatch::new(),
        })
    }

    pub(super) fn commit(&mut self, prepared: ScreenSpaceUiPreparedUpload) -> bool {
        if !prepared.attached
            || !Arc::ptr_eq(&self.owner, &prepared.owner)
            || self.prepared_generation != prepared.generation
        {
            return false;
        }
        if !prepared.force_full_upload || prepared.full_upload_prepared {
            self.committed_generation = prepared.generation;
        }
        true
    }

    fn append(
        &self,
        prepared: &mut ScreenSpaceUiPreparedUpload,
        frame_buffer_uploads: &mut WgpuBufferUploadBatch,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> bool {
        if prepared.attached || !Arc::ptr_eq(&self.owner, &prepared.owner) {
            return false;
        }
        frame_buffer_uploads.append(&mut prepared.buffer_uploads);
        frame_texture_uploads.append(std::mem::take(&mut prepared.texture_uploads));
        prepared.attached = true;
        true
    }
}

impl ScreenSpaceUiPreparedUpload {
    pub(super) const fn force_full_upload(&self) -> bool {
        self.force_full_upload
    }

    pub(super) fn mark_full_upload_prepared(&mut self) {
        self.full_upload_prepared = true;
    }

    pub(super) fn buffer_uploads_mut(&mut self) -> &mut WgpuBufferUploadBatch {
        &mut self.buffer_uploads
    }

    pub(super) fn resource_uploads_mut(
        &mut self,
    ) -> (&mut WgpuBufferUploadBatch, &mut WgpuTextureUploadBatch) {
        (&mut self.buffer_uploads, &mut self.texture_uploads)
    }

    pub(in crate::graphics::scene::scene_renderer) fn append_to(
        &mut self,
        renderer: &super::ScreenSpaceUiRenderer,
        frame_buffer_uploads: &mut WgpuBufferUploadBatch,
        frame_texture_uploads: &mut WgpuTextureUploadBatch,
    ) -> bool {
        renderer
            .upload_transaction
            .append(self, frame_buffer_uploads, frame_texture_uploads)
    }
}

impl Drop for ScreenSpaceUiPreparedUpload {
    fn drop(&mut self) {
        self.preparation_outstanding.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenSpaceUiUploadTransactionState;
    use zr_rhi_wgpu::{WgpuBufferUploadBatch, WgpuTextureUploadBatch};

    #[test]
    fn dropped_preparation_forces_the_next_frame_to_upload_in_full() {
        let mut state = ScreenSpaceUiUploadTransactionState::default();
        let first = state.begin().expect("first preparation");
        assert!(!first.force_full_upload());
        drop(first);

        let retry = state.begin().expect("retry preparation");
        assert!(retry.force_full_upload());
    }

    #[test]
    fn overlapping_preparations_are_rejected_until_drop() {
        let mut state = ScreenSpaceUiUploadTransactionState::default();
        let first = state.begin().expect("first preparation");
        assert!(state.begin().is_err());
        drop(first);
        assert!(state.begin().is_ok());
    }

    #[test]
    fn only_an_appended_preparation_advances_the_committed_generation() {
        let mut state = ScreenSpaceUiUploadTransactionState::default();
        let unappended = state.begin().expect("unappended preparation");
        assert!(!state.commit(unappended));

        let mut retry = state.begin().expect("retry preparation");
        assert!(retry.force_full_upload());
        let mut frame_uploads = WgpuBufferUploadBatch::new();
        let mut frame_texture_uploads = WgpuTextureUploadBatch::new();
        retry.mark_full_upload_prepared();
        assert!(state.append(&mut retry, &mut frame_uploads, &mut frame_texture_uploads));
        assert!(state.commit(retry));

        let stable = state.begin().expect("stable preparation");
        assert!(!stable.force_full_upload());
    }

    #[test]
    fn a_foreign_transaction_cannot_attach_the_prepared_batch() {
        let mut owner = ScreenSpaceUiUploadTransactionState::default();
        let foreign = ScreenSpaceUiUploadTransactionState::default();
        let mut prepared = owner.begin().expect("owned preparation");
        let mut frame_uploads = WgpuBufferUploadBatch::new();
        let mut frame_texture_uploads = WgpuTextureUploadBatch::new();

        assert!(!foreign.append(
            &mut prepared,
            &mut frame_uploads,
            &mut frame_texture_uploads
        ));
        assert!(owner.append(
            &mut prepared,
            &mut frame_uploads,
            &mut frame_texture_uploads
        ));
        assert!(owner.commit(prepared));
    }

    #[test]
    fn an_empty_retry_frame_does_not_clear_the_forced_full_upload() {
        let mut state = ScreenSpaceUiUploadTransactionState::default();
        drop(state.begin().expect("abandoned preparation"));

        let mut empty_retry = state.begin().expect("empty retry preparation");
        assert!(empty_retry.force_full_upload());
        let mut frame_uploads = WgpuBufferUploadBatch::new();
        let mut frame_texture_uploads = WgpuTextureUploadBatch::new();
        assert!(state.append(
            &mut empty_retry,
            &mut frame_uploads,
            &mut frame_texture_uploads
        ));
        assert!(state.commit(empty_retry));

        assert!(
            state
                .begin()
                .expect("retry after empty frame")
                .force_full_upload()
        );
    }

    #[test]
    fn a_prepared_full_retry_clears_the_forced_full_upload_after_commit() {
        let mut state = ScreenSpaceUiUploadTransactionState::default();
        drop(state.begin().expect("abandoned preparation"));

        let mut retry = state.begin().expect("full retry preparation");
        assert!(retry.force_full_upload());
        retry.mark_full_upload_prepared();
        let mut frame_uploads = WgpuBufferUploadBatch::new();
        let mut frame_texture_uploads = WgpuTextureUploadBatch::new();
        assert!(state.append(&mut retry, &mut frame_uploads, &mut frame_texture_uploads));
        assert!(state.commit(retry));

        assert!(
            !state
                .begin()
                .expect("stable preparation")
                .force_full_upload()
        );
    }
}
