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

    assert!(state
        .begin()
        .expect("retry after empty frame")
        .force_full_upload());
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

    assert!(!state
        .begin()
        .expect("stable preparation")
        .force_full_upload());
}
