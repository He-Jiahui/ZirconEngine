use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError,
    RuntimeSessionArchiveRetentionPolicy, slot_capture,
};

pub(super) fn apply_capture_preview_with_retention(
    archive: &mut RuntimeSessionArchive,
    preview: slot_capture::RuntimeSessionSlotCapturePreview,
    tag: Option<&str>,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
    let capture = preview.report;
    let captured_slot_id = capture.slot_id.clone();
    archive.upsert_slot(preview.slot)?;

    let policy = policy.with_protected_slot(captured_slot_id);
    let prune = match tag {
        Some(tag) => archive.prune_slots_with_tag(tag, policy)?,
        None => archive.prune_slots(policy)?,
    };
    let manifest = archive.manifest()?;

    Ok(RuntimeSessionArchiveCaptureRetentionReport {
        capture,
        prune,
        manifest,
    })
}
