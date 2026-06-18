use super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSessionArchiveMergePolicy {
    RejectConflicts,
    KeepExisting,
    ReplaceExisting,
}

impl Default for RuntimeSessionArchiveMergePolicy {
    fn default() -> Self {
        Self::RejectConflicts
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveMergeReport {
    pub inserted_slot_ids: Vec<String>,
    pub replaced_slot_ids: Vec<String>,
    pub skipped_slot_ids: Vec<String>,
}

impl RuntimeSessionArchiveMergeReport {
    pub fn is_empty(&self) -> bool {
        self.inserted_slot_ids.is_empty()
            && self.replaced_slot_ids.is_empty()
            && self.skipped_slot_ids.is_empty()
    }
}

pub(super) fn preview_merge_archive(
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
