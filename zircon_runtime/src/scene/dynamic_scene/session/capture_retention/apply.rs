use std::collections::BTreeSet;
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
        let retained_slot_ids = self
            .prune
            .retained_slot_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
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
