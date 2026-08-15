use std::collections::BTreeSet;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};
use super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};

/// A generation-bound prune decision that can be inspected before it publishes
/// its retained and removed slot rows.
#[derive(Debug)]
pub struct RuntimeSessionArchivePrunePlan {
    target_generation: u64,
    target_revision: u64,
    report: RuntimeSessionArchivePruneReport,
}

impl RuntimeSessionArchivePrunePlan {
    pub fn target_generation(&self) -> u64 {
        self.target_generation
    }

    pub fn target_revision(&self) -> u64 {
        self.target_revision
    }

    pub fn report(&self) -> &RuntimeSessionArchivePruneReport {
        &self.report
    }

    pub fn commit(
        self,
        archive: &mut RuntimeSessionArchive,
    ) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
        if archive.generation() != self.target_generation
            || archive.revision() != self.target_revision
        {
            return Err(RuntimeSessionArchiveError::StalePrunePlan {
                expected_generation: self.target_generation,
                expected_revision: self.target_revision,
                current_generation: archive.generation(),
                current_revision: archive.revision(),
            });
        }

        let report = self.report;
        if !report.removed_slot_ids.is_empty() {
            archive.commit_staged_slot_rows(
                Vec::new(),
                Vec::new(),
                report.removed_slot_ids.iter().map(String::as_str),
            );
        }
        Ok(report)
    }
}

pub(in crate::scene::dynamic_scene::session) fn prepare_prune_slots(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePrunePlan, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let report = preview_matching_slots(archive, policy.normalized(), |_| true)?;
    Ok(RuntimeSessionArchivePrunePlan {
        target_generation: archive.generation(),
        target_revision: archive.revision(),
        report,
    })
}

pub(in crate::scene::dynamic_scene::session) fn prepare_prune_slots_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePrunePlan, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let tag = tag.trim();
    let report = if tag.is_empty() {
        RuntimeSessionArchivePruneReport {
            retained_slot_ids: canonical_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        }
    } else {
        preview_matching_slots_with_tag(archive, tag, policy.normalized())?
    };
    Ok(RuntimeSessionArchivePrunePlan {
        target_generation: archive.generation(),
        target_revision: archive.revision(),
        report,
    })
}

pub(super) fn preview_matching_slots(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    matches_scope: impl Fn(&str) -> bool,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let scoped_slot_ids = archive
        .iter_canonical_slots()
        .filter(|slot| matches_scope(&slot.slot_id))
        .map(|slot| slot.slot_id.clone())
        .collect::<BTreeSet<_>>();
    preview_matching_slot_ids(archive, policy, scoped_slot_ids)
}

pub(super) fn preview_matching_slots_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let scoped_slot_ids = archive
        .indexed_tag_slots(tag)
        .map(|slot| slot.slot_id.clone())
        .collect::<BTreeSet<_>>();
    preview_matching_slot_ids(archive, policy, scoped_slot_ids)
}

pub(in crate::scene::dynamic_scene::session) fn preview_matching_slots_after_upsert(
    archive: &RuntimeSessionArchive,
    captured_slot: &RuntimeSessionSlot,
    tag: Option<&str>,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let all_slot_ids = archive
        .slot_ids()
        .filter(|slot_id| *slot_id != captured_slot.slot_id.as_str())
        .chain(std::iter::once(captured_slot.slot_id.as_str()))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let scoped_slot_ids = match tag {
        Some(tag) if tag.trim().is_empty() => BTreeSet::new(),
        Some(tag) => {
            let tag = tag.trim();
            archive
                .indexed_tag_slots(tag)
                .filter(|slot| slot.slot_id != captured_slot.slot_id)
                .map(|slot| slot.slot_id.clone())
                .chain(
                    captured_slot
                        .metadata
                        .tags
                        .iter()
                        .any(|candidate| candidate == tag)
                        .then(|| captured_slot.slot_id.clone()),
                )
                .collect::<BTreeSet<_>>()
        }
        None => all_slot_ids.clone(),
    };

    preview_matching_slot_ids_after_upsert(
        archive,
        captured_slot,
        policy.normalized(),
        all_slot_ids,
        scoped_slot_ids,
    )
}

fn preview_matching_slot_ids(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    scoped_slot_ids: BTreeSet<String>,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let Some(max_slots) = policy.max_slots else {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: canonical_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    };

    if scoped_slot_ids.len() <= max_slots {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: canonical_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    }

    let protected_slot_ids = policy
        .protected_slot_ids
        .iter()
        .filter(|slot_id| archive.contains_slot(slot_id))
        .filter(|slot_id| scoped_slot_ids.contains(*slot_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut kept_slot_ids = scoped_slot_ids
        .iter()
        .filter(|slot_id| protected_slot_ids.contains(*slot_id))
        .cloned()
        .collect::<BTreeSet<_>>();

    for slot in archive.indexed_slots_by_update().rev() {
        if kept_slot_ids.len() >= max_slots {
            break;
        }
        if scoped_slot_ids.contains(&slot.slot_id) {
            kept_slot_ids.insert(slot.slot_id.clone());
        }
    }

    let removed_slot_ids = canonical_slot_ids(archive)
        .into_iter()
        .filter(|slot_id| scoped_slot_ids.contains(slot_id))
        .filter(|slot_id| !kept_slot_ids.contains(slot_id))
        .collect::<Vec<_>>();
    let removed_set = removed_slot_ids.iter().cloned().collect::<BTreeSet<_>>();

    Ok(RuntimeSessionArchivePruneReport {
        retained_slot_ids: canonical_slot_ids(archive)
            .into_iter()
            .filter(|slot_id| !removed_set.contains(slot_id))
            .collect(),
        removed_slot_ids,
    })
}

fn preview_matching_slot_ids_after_upsert(
    archive: &RuntimeSessionArchive,
    captured_slot: &RuntimeSessionSlot,
    policy: RuntimeSessionArchiveRetentionPolicy,
    all_slot_ids: BTreeSet<String>,
    scoped_slot_ids: BTreeSet<String>,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let Some(max_slots) = policy.max_slots else {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: all_slot_ids.into_iter().collect(),
            removed_slot_ids: Vec::new(),
        });
    };

    if scoped_slot_ids.len() <= max_slots {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: all_slot_ids.into_iter().collect(),
            removed_slot_ids: Vec::new(),
        });
    }

    let mut kept_slot_ids = policy
        .protected_slot_ids
        .iter()
        .filter(|slot_id| all_slot_ids.contains(*slot_id))
        .filter(|slot_id| scoped_slot_ids.contains(*slot_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    let captured_is_candidate = scoped_slot_ids.contains(&captured_slot.slot_id)
        && !kept_slot_ids.contains(&captured_slot.slot_id);
    let mut considered_captured = !captured_is_candidate;
    for slot in archive.indexed_slots_by_update().rev() {
        if kept_slot_ids.len() >= max_slots {
            break;
        }
        if slot.slot_id == captured_slot.slot_id
            || !scoped_slot_ids.contains(&slot.slot_id)
            || kept_slot_ids.contains(&slot.slot_id)
        {
            continue;
        }
        if !considered_captured && slot_update_key(captured_slot) > slot_update_key(slot) {
            kept_slot_ids.insert(captured_slot.slot_id.clone());
            considered_captured = true;
            if kept_slot_ids.len() >= max_slots {
                break;
            }
        }
        kept_slot_ids.insert(slot.slot_id.clone());
    }
    if !considered_captured && kept_slot_ids.len() < max_slots {
        kept_slot_ids.insert(captured_slot.slot_id.clone());
    }

    let removed_slot_ids = all_slot_ids
        .iter()
        .filter(|slot_id| scoped_slot_ids.contains(*slot_id))
        .filter(|slot_id| !kept_slot_ids.contains(*slot_id))
        .cloned()
        .collect::<Vec<_>>();
    let removed_set = removed_slot_ids.iter().cloned().collect::<BTreeSet<_>>();

    Ok(RuntimeSessionArchivePruneReport {
        retained_slot_ids: all_slot_ids
            .into_iter()
            .filter(|slot_id| !removed_set.contains(slot_id))
            .collect(),
        removed_slot_ids,
    })
}

fn slot_update_key(slot: &RuntimeSessionSlot) -> (u64, &str) {
    (
        slot.metadata.updated_at_unix_millis.unwrap_or(0),
        slot.slot_id.as_str(),
    )
}

pub(super) fn canonical_slot_ids(archive: &RuntimeSessionArchive) -> Vec<String> {
    archive.slot_ids().map(str::to_string).collect()
}
