use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError};
use super::super::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};
use super::preview;

pub(in crate::scene::dynamic_scene::session) fn merge_archive(
    target: &mut RuntimeSessionArchive,
    incoming: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
    let report = preview::preview_merge_archive(target, incoming, policy)?;

    for slot in &incoming.slots {
        let slot_id = slot.slot_id.clone();
        if target.contains_slot(&slot_id) {
            match policy {
                RuntimeSessionArchiveMergePolicy::RejectConflicts => {
                    unreachable!("reject-conflicts policy scans duplicate slot ids before mutating")
                }
                RuntimeSessionArchiveMergePolicy::KeepExisting => {}
                RuntimeSessionArchiveMergePolicy::ReplaceExisting => {
                    target.upsert_slot(slot.clone())?;
                }
            }
        } else {
            target.push_slot(slot.clone())?;
        }
    }
    Ok(report)
}
