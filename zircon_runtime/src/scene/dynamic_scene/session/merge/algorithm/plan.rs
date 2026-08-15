use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};
use super::super::{RuntimeSessionArchiveMergePolicy, RuntimeSessionArchiveMergeReport};

#[derive(Debug)]
pub struct RuntimeSessionArchiveMergePlan<'incoming> {
    target_generation: u64,
    target_revision: u64,
    report: RuntimeSessionArchiveMergeReport,
    replacements: Vec<&'incoming RuntimeSessionSlot>,
    inserts: Vec<&'incoming RuntimeSessionSlot>,
}

impl RuntimeSessionArchiveMergePlan<'_> {
    pub fn target_generation(&self) -> u64 {
        self.target_generation
    }

    pub fn target_revision(&self) -> u64 {
        self.target_revision
    }

    pub fn report(&self) -> &RuntimeSessionArchiveMergeReport {
        &self.report
    }

    pub fn commit(
        self,
        target: &mut RuntimeSessionArchive,
    ) -> Result<RuntimeSessionArchiveMergeReport, RuntimeSessionArchiveError> {
        if target.generation() != self.target_generation
            || target.revision() != self.target_revision
        {
            return Err(RuntimeSessionArchiveError::StaleMergePlan {
                expected_generation: self.target_generation,
                expected_revision: self.target_revision,
                current_generation: target.generation(),
                current_revision: target.revision(),
            });
        }
        let report = self.report;
        if self.replacements.is_empty() && self.inserts.is_empty() {
            return Ok(report);
        }

        // Clone only after all validation and conflict planning has completed.
        // Preview consumes the borrowed plan without cloning slot payloads.
        let replacements = self.replacements.into_iter().cloned().collect();
        let inserts = self.inserts.into_iter().cloned().collect();
        target.commit_staged_slot_rows(replacements, inserts, std::iter::empty());
        Ok(report)
    }
}

pub(in crate::scene::dynamic_scene::session) fn prepare_merge_archive<'incoming>(
    target: &RuntimeSessionArchive,
    incoming: &'incoming RuntimeSessionArchive,
    policy: RuntimeSessionArchiveMergePolicy,
) -> Result<RuntimeSessionArchiveMergePlan<'incoming>, RuntimeSessionArchiveError> {
    target.ensure_supported()?;
    incoming.ensure_supported()?;

    let mut report = RuntimeSessionArchiveMergeReport::default();
    let mut replacements = Vec::new();
    let mut inserts = Vec::new();
    for slot in incoming.iter_canonical_slots() {
        match target.indexed_slot_index(&slot.slot_id) {
            Some(_) => match policy {
                RuntimeSessionArchiveMergePolicy::RejectConflicts => {
                    return Err(RuntimeSessionArchiveError::DuplicateSlotId {
                        slot_id: slot.slot_id.clone(),
                    });
                }
                RuntimeSessionArchiveMergePolicy::KeepExisting => {
                    report.skipped_slot_ids.push(slot.slot_id.clone());
                }
                RuntimeSessionArchiveMergePolicy::ReplaceExisting => {
                    report.replaced_slot_ids.push(slot.slot_id.clone());
                    replacements.push(slot);
                }
            },
            None => {
                report.inserted_slot_ids.push(slot.slot_id.clone());
                inserts.push(slot);
            }
        }
    }

    Ok(RuntimeSessionArchiveMergePlan {
        target_generation: target.generation(),
        target_revision: target.revision(),
        report,
        replacements,
        inserts,
    })
}
