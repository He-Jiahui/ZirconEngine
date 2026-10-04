use super::*;
use crate::core::framework::render::RenderFrameSubmissionTransaction;
use zr_rhi::{
    DeviceGeneration, DeviceId, RenderQueueClass, SubmissionPollReceipt, SubmissionTicket,
};

fn ticket(sequence: u64) -> SubmissionTicket {
    SubmissionTicket::new(
        DeviceId::new(3),
        DeviceGeneration::new(2),
        RenderQueueClass::Graphics,
        sequence,
    )
}

#[test]
fn only_unsuccessful_texture_submissions_revoke_resource_publication() {
    let cancelled_texture = ResourceId::from_stable_label("cancelled-texture");
    let submitted_texture = ResourceId::from_stable_label("submitted-texture");
    let completed_texture = ResourceId::from_stable_label("completed-texture");
    let mut transaction = RenderFrameSubmissionTransaction::begin(
        7,
        SubmissionPollReceipt::new(DeviceId::new(3), DeviceGeneration::new(2), 11),
    );
    for (resource_id, sequence) in [
        (cancelled_texture, 36),
        (submitted_texture, 37),
        (completed_texture, 38),
    ] {
        transaction
            .record_pre_scene_resource_submission(
                RenderFrameSubmissionProducer::TextureCopyUpload,
                resource_id,
                ticket(sequence),
            )
            .expect("texture submission identity");
    }
    let receipt = transaction
        .abort(vec![
            SubmissionStatus::Cancelled,
            SubmissionStatus::Submitted,
            SubmissionStatus::Completed,
        ])
        .expect("settled failure receipt");

    assert_eq!(
        invalid_texture_resource_ids(&receipt),
        HashSet::from([cancelled_texture])
    );
}

#[test]
fn failure_rollback_is_scoped_to_texture_producers_and_dependent_materials() {
    let source = include_str!("../resource_streamer_submission_failure.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("submission rollback test boundary");

    assert!(source.contains("self.textures"));
    assert!(source.contains("self.post_process_lut_textures"));
    assert!(source.contains("self.mip_streaming_states"));
    assert!(source.contains("self.last_ui_texture_prepare_receipt = None"));
    assert!(source.contains("prepared_material_uses_any_texture"));
    assert!(source.contains("self.active_staged_material_ids.remove"));
    assert!(!source.contains("self.materials.clear"));
}
