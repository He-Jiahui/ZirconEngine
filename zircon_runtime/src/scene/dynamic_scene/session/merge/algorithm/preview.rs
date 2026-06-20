use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};

pub(in crate::scene::dynamic_scene::session) fn preview_merge_archive(
    target: &RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    target.ensure_supported()?;
    incoming.ensure_supported()?;

    if policy == RuntimeSessionArchiveMergePolicy::RejectConflicts {
        if let Some(conflicting) = incoming
            .slots
            .iter()
            .find(|slot| target.contains_slot(&slot.slot_id))
        {
            return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                slot_id: conflicting.slot_id.clone(),
            });
        }
    }

    let mut report = RuntimeSessionArchiveMergeReport::default();
    for slot in &incoming.slots {
        let slot_id = slot.slot_id.clone();
        if target.contains_slot(&slot_id) {
            match policy {
                RuntimeSessionArchiveMergePolicy::RejectConflicts => unreachable!(
                    "reject-conflicts policy scans duplicate slot ids before reporting"
                ),
                RuntimeSessionArchiveMergePolicy::KeepExisting => {
                    report.skipped_slot_ids.push(slot_id);
                }
                RuntimeSessionArchiveMergePolicy::ReplaceExisting => {
                    report.replaced_slot_ids.push(slot_id);
                }
            }
        } else {
            report.inserted_slot_ids.push(slot_id);
        }
    }
    Ok(report)
}
