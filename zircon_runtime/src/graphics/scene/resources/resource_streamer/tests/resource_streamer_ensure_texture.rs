use crate::asset::{AssetUri, TextureAsset};

use super::center_texture_asset_rgba;

#[test]
fn capture_sample_reads_the_center_texel_once_from_rgba8_source() {
    let texture = TextureAsset::new_rgba8(
        AssetUri::parse("res://textures/capture-center.png").expect("texture uri"),
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 64, 128, 192, 255,
        ],
    );

    assert_eq!(
        center_texture_asset_rgba(&texture),
        Some([64.0 / 255.0, 128.0 / 255.0, 192.0 / 255.0, 1.0])
    );
}

#[test]
fn gpu_texture_publication_uses_one_atomic_asset_revision_snapshot() {
    let production = include_str!("../resource_streamer_ensure_texture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("texture preparation test boundary");

    assert!(production.contains("load_texture_asset_snapshot(id)"));
    assert!(production.contains("let revision = texture.revision();"));
    assert!(!production.contains("let revision = self.resource_revision(id)?;"));
}

#[test]
fn frame_texture_upload_records_pre_copy_post_ticket_order() {
    let production = include_str!("../resource_streamer_ensure_texture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("texture preparation test boundary");
    let pre = production
        .find("RenderFrameSubmissionProducer::TexturePreUpload")
        .expect("pre-upload producer");
    let copy = production
        .find("RenderFrameSubmissionProducer::TextureCopyUpload")
        .expect("copy-upload producer");
    let post = production
        .find("RenderFrameSubmissionProducer::TexturePostUpload")
        .expect("post-upload producer");

    assert!(pre < copy && copy < post);
    assert_eq!(production.matches("let ticket = backend.").count(), 3);
    assert_eq!(
        production
            .matches("record_pre_scene_resource_submission(")
            .count(),
        2
    );
    assert_eq!(
        production
            .matches("record_pre_scene_resource_submission_with_boundary(")
            .count(),
        1
    );
    assert!(production
        .contains("RenderFrameSubmissionBoundaryReason::TextureMipPreservationBeforeUpload"));
}

#[test]
fn texture_prepare_profiling_covers_load_clone_prepare_and_enqueue_stages() {
    let source = include_str!("../resource_streamer_ensure_texture.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("texture preparation test boundary");

    for marker in [
        "texture_snapshot_load_count",
        "texture_cpu_payload_clone_bytes",
        "prepare_gpu_resource",
        "enqueue_gpu_upload",
        "texture_upload_payload_bytes",
        "texture_publication_before_upload_completion",
    ] {
        assert!(source.contains(marker), "missing profiling marker {marker}");
    }
}
