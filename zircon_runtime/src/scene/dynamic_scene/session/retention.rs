use std::collections::BTreeSet;

use super::{RuntimeSessionArchive, RuntimeSessionArchiveError};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveRetentionPolicy {
    pub max_slots: Option<usize>,
    pub protected_slot_ids: Vec<String>,
}

impl RuntimeSessionArchiveRetentionPolicy {
    pub fn keep_latest(max_slots: usize) -> Self {
        Self {
            max_slots: Some(max_slots),
            protected_slot_ids: Vec::new(),
        }
    }

    pub fn with_protected_slot(mut self, slot_id: impl Into<String>) -> Self {
        self.protected_slot_ids.push(slot_id.into());
        self.normalize();
        self
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn normalize(&mut self) {
        normalize_protected_slot_ids(&mut self.protected_slot_ids);
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchivePruneReport {
    pub retained_slot_ids: Vec<String>,
    pub removed_slot_ids: Vec<String>,
}

impl RuntimeSessionArchivePruneReport {
    pub fn removed_count(&self) -> usize {
        self.removed_slot_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.removed_slot_ids.is_empty()
    }
}

pub(super) fn prune_slots(
    archive: &mut RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    archive.sort_slots();
    let policy = policy.normalized();
    let report = preview_matching_slots(archive, policy, |_| true)?;
    apply_prune_report(archive, &report);
    Ok(report)
}

pub(super) fn preview_prune_slots(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    preview_matching_slots(archive, policy.normalized(), |_| true)
}

pub(super) fn prune_slots_with_tag(
    archive: &mut RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    archive.sort_slots();
    let tag = tag.trim();
    let policy = policy.normalized();
    if tag.is_empty() {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: archive.slot_ids().map(str::to_string).collect(),
            removed_slot_ids: Vec::new(),
        });
    }
    let report = preview_matching_slots_with_tag(archive, tag, policy)?;
    apply_prune_report(archive, &report);
    Ok(report)
}

pub(super) fn preview_prune_slots_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    archive.ensure_supported()?;
    let tag = tag.trim();
    if tag.is_empty() {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: sorted_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    }
    preview_matching_slots_with_tag(archive, tag, policy.normalized())
}

fn preview_matching_slots(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    matches_scope: impl Fn(&str) -> bool,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let scoped_slot_ids = archive
        .slots
        .iter()
        .filter(|slot| matches_scope(&slot.slot_id))
        .map(|slot| slot.slot_id.clone())
        .collect::<BTreeSet<_>>();
    preview_matching_slot_ids(archive, policy, scoped_slot_ids)
}

fn preview_matching_slots_with_tag(
    archive: &RuntimeSessionArchive,
    tag: &str,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let scoped_slot_ids = archive
        .slots
        .iter()
        .filter(|slot| slot.metadata.tags.iter().any(|candidate| candidate == tag))
        .map(|slot| slot.slot_id.clone())
        .collect::<BTreeSet<_>>();
    preview_matching_slot_ids(archive, policy, scoped_slot_ids)
}

fn preview_matching_slot_ids(
    archive: &RuntimeSessionArchive,
    policy: RuntimeSessionArchiveRetentionPolicy,
    scoped_slot_ids: BTreeSet<String>,
) -> Result<RuntimeSessionArchivePruneReport, RuntimeSessionArchiveError> {
    let Some(max_slots) = policy.max_slots else {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: sorted_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    };

    if scoped_slot_ids.len() <= max_slots {
        return Ok(RuntimeSessionArchivePruneReport {
            retained_slot_ids: sorted_slot_ids(archive),
            removed_slot_ids: Vec::new(),
        });
    }

    let protected_slot_ids = policy
        .protected_slot_ids
        .iter()
        .filter(|slot_id| archive.contains_slot(slot_id))
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut kept_slot_ids = scoped_slot_ids
        .iter()
        .filter(|slot_id| protected_slot_ids.contains(*slot_id))
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut candidates = archive
        .slots
        .iter()
        .filter(|slot| scoped_slot_ids.contains(&slot.slot_id))
        .filter(|slot| !kept_slot_ids.contains(&slot.slot_id))
        .map(|slot| {
            (
                slot.metadata.updated_at_unix_millis.unwrap_or(0),
                slot.slot_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| right.cmp(left));

    for (_, slot_id) in candidates {
        if kept_slot_ids.len() >= max_slots {
            break;
        }
        kept_slot_ids.insert(slot_id);
    }

    let removed_slot_ids = sorted_slot_ids(archive)
        .into_iter()
        .filter(|slot_id| scoped_slot_ids.contains(slot_id))
        .filter(|slot_id| !kept_slot_ids.contains(slot_id))
        .collect::<Vec<_>>();
    let removed_set = removed_slot_ids.iter().cloned().collect::<BTreeSet<_>>();

    Ok(RuntimeSessionArchivePruneReport {
        retained_slot_ids: sorted_slot_ids(archive)
            .into_iter()
            .filter(|slot_id| !removed_set.contains(slot_id))
            .collect(),
        removed_slot_ids,
    })
}

fn apply_prune_report(
    archive: &mut RuntimeSessionArchive,
    report: &RuntimeSessionArchivePruneReport,
) {
    if report.removed_slot_ids.is_empty() {
        return;
    }
    let removed_slot_ids = report
        .removed_slot_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    archive
        .slots
        .retain(|slot| !removed_slot_ids.contains(slot.slot_id.as_str()));
    archive.sort_slots();
}

fn sorted_slot_ids(archive: &RuntimeSessionArchive) -> Vec<String> {
    let mut slot_ids = archive.slot_ids().map(str::to_string).collect::<Vec<_>>();
    slot_ids.sort();
    slot_ids
}

fn normalize_protected_slot_ids(slot_ids: &mut Vec<String>) {
    for slot_id in slot_ids.iter_mut() {
        *slot_id = slot_id.trim().to_string();
    }
    slot_ids.retain(|slot_id| !slot_id.is_empty());
    slot_ids.sort();
    slot_ids.dedup();
}
