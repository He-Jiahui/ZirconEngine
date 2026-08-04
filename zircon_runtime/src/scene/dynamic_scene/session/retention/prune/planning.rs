use std::collections::BTreeSet;

use super::super::super::{RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionSlot};
use super::super::{RuntimeSessionArchivePruneReport, RuntimeSessionArchiveRetentionPolicy};

pub(super) fn preview_matching_slots(
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
        .filter(|slot_id| scoped_slot_ids.contains(*slot_id))
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
    let mut candidates = archive
        .slots
        .iter()
        .filter(|slot| slot.slot_id != captured_slot.slot_id)
        .filter(|slot| scoped_slot_ids.contains(&slot.slot_id))
        .filter(|slot| !kept_slot_ids.contains(&slot.slot_id))
        .map(|slot| {
            (
                slot.metadata.updated_at_unix_millis.unwrap_or(0),
                slot.slot_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    if scoped_slot_ids.contains(&captured_slot.slot_id)
        && !kept_slot_ids.contains(&captured_slot.slot_id)
    {
        candidates.push((
            captured_slot.metadata.updated_at_unix_millis.unwrap_or(0),
            captured_slot.slot_id.clone(),
        ));
    }
    candidates.sort_by(|left, right| right.cmp(left));

    for (_, slot_id) in candidates {
        if kept_slot_ids.len() >= max_slots {
            break;
        }
        kept_slot_ids.insert(slot_id);
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

pub(super) fn sorted_slot_ids(archive: &RuntimeSessionArchive) -> Vec<String> {
    let mut slot_ids = archive.slot_ids().map(str::to_string).collect::<Vec<_>>();
    slot_ids.sort();
    slot_ids
}
