use std::collections::HashSet;
use std::sync::Arc;

use super::super::{
    retention, slot_capture, RuntimeSessionArchive, RuntimeSessionArchiveCaptureRetentionReport,
    RuntimeSessionArchiveError, RuntimeSessionArchiveManifest, RuntimeSessionArchivePruneReport,
    RuntimeSessionArchiveRetentionPolicy, RuntimeSessionSlot,
    RuntimeSessionSlotCapturePreviewReport,
};

#[derive(Debug)]
pub(super) struct RuntimeSessionArchiveCaptureRetentionPlan {
    target_generation: u64,
    target_revision: u64,
    capture: RuntimeSessionSlotCapturePreviewReport,
    slot: RuntimeSessionSlot,
    prune: RuntimeSessionArchivePruneReport,
}

// 捕获槽位 ID 先加入保护集，再计算全档案或指定标签桶的裁剪结果；标签桶只统计实际带该标签的槽位。
pub(super) fn prepare_capture_preview_with_retention(
    archive: &RuntimeSessionArchive,
    preview: slot_capture::RuntimeSessionSlotCapturePreview,
    tag: Option<&str>,
    policy: RuntimeSessionArchiveRetentionPolicy,
) -> Result<RuntimeSessionArchiveCaptureRetentionPlan, RuntimeSessionArchiveError> {
    let capture = preview.report;
    let slot = preview.slot;
    let policy = policy.with_protected_slot(capture.slot_id.clone());
    let prune = retention::preview_matching_slots_after_upsert(archive, &slot, tag, policy)?;

    Ok(RuntimeSessionArchiveCaptureRetentionPlan {
        target_generation: archive.generation(),
        target_revision: archive.revision(),
        capture,
        slot,
        prune,
    })
}

impl RuntimeSessionArchiveCaptureRetentionPlan {
    // 预览清单按保留集合投影出更新或插入后的状态，不改写原档案；报告与提交共享同一捕获快照。
    pub(super) fn report(
        &self,
        archive: &RuntimeSessionArchive,
    ) -> RuntimeSessionArchiveCaptureRetentionReport {
        RuntimeSessionArchiveCaptureRetentionReport {
            capture: self.capture.clone(),
            prune: self.prune.clone(),
            manifest: self.virtual_manifest(archive),
        }
    }

    // 仅当准备时的档案代际和修订号仍匹配才允许提交；捕获槽的替换或插入与删除集合通过一次暂存提交共同生效。
    pub(super) fn commit(
        self,
        archive: &mut RuntimeSessionArchive,
    ) -> Result<RuntimeSessionArchiveCaptureRetentionReport, RuntimeSessionArchiveError> {
        if archive.generation() != self.target_generation
            || archive.revision() != self.target_revision
        {
            return Err(RuntimeSessionArchiveError::StaleCaptureRetentionPlan {
                expected_generation: self.target_generation,
                expected_revision: self.target_revision,
                current_generation: archive.generation(),
                current_revision: archive.revision(),
            });
        }

        let report = self.report(archive);
        let RuntimeSessionArchiveCaptureRetentionPlan { slot, prune, .. } = self;
        let (replacements, inserts) = match archive.indexed_slot_index(&slot.slot_id) {
            Some(_) => (vec![slot], Vec::new()),
            None => (Vec::new(), vec![slot]),
        };
        archive.commit_staged_slot_rows(
            replacements,
            inserts,
            prune.removed_slot_ids.iter().map(String::as_str),
        );
        Ok(report)
    }

    fn virtual_manifest(&self, archive: &RuntimeSessionArchive) -> RuntimeSessionArchiveManifest {
        let retained_slot_ids = retained_slot_id_index(&self.prune.retained_slot_ids);
        let mut slots = archive
            .iter_canonical_slots()
            .filter(|slot| slot.slot_id != self.slot.slot_id)
            .filter(|slot| retained_slot_ids.contains(slot.slot_id.as_str()))
            .map(RuntimeSessionSlot::summary)
            .collect::<Vec<_>>();
        if retained_slot_ids.contains(self.slot.slot_id.as_str()) {
            slots.push(self.slot.summary());
        }
        slots.sort_by(|left, right| left.slot_id.cmp(&right.slot_id));
        RuntimeSessionArchiveManifest {
            format_version: archive.format_version,
            slots: Arc::new(slots),
        }
    }
}

fn retained_slot_id_index(retained_slot_ids: &[String]) -> HashSet<&str> {
    let mut index = HashSet::with_capacity(retained_slot_ids.len());
    index.extend(retained_slot_ids.iter().map(String::as_str));
    index
}

#[cfg(test)]
#[path = "apply/tests/hash_membership_tests.rs"]
mod hash_membership_tests;
