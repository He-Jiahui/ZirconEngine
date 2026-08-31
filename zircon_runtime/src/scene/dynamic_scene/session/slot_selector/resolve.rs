use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};
use super::report::{RuntimeSessionSlotSelection, RuntimeSessionSlotSelectionReport};
use super::selector::RuntimeSessionSlotSelector;

impl RuntimeSessionSlotSelector {
    pub fn slot_id(slot_id: impl Into<String>) -> Self {
        Self::SlotId {
            slot_id: normalize_selector_value(slot_id),
        }
    }

    pub fn latest_updated() -> Self {
        Self::LatestUpdated
    }

    pub fn oldest_updated() -> Self {
        Self::OldestUpdated
    }

    pub fn latest_updated_with_tag(tag: impl Into<String>) -> Self {
        Self::LatestUpdatedWithTag {
            tag: normalize_selector_value(tag),
        }
    }

    pub fn oldest_updated_with_tag(tag: impl Into<String>) -> Self {
        Self::OldestUpdatedWithTag {
            tag: normalize_selector_value(tag),
        }
    }

    pub fn resolve(
        &self,
        archive: &RuntimeSessionArchive,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        let selection = self.resolve_slot(archive)?;
        let slot = selection.slot();

        Ok(RuntimeSessionSlotSelectionReport {
            selector: self.clone(),
            selected_slot_id: slot.slot_id.clone(),
            summary: slot.summary(),
        })
    }

    pub fn resolve_slot<'archive>(
        &self,
        archive: &'archive RuntimeSessionArchive,
    ) -> Result<RuntimeSessionSlotSelection<'archive>, RuntimeSessionArchiveError> {
        let slot = match self {
            Self::SlotId { slot_id } => archive.slot(slot_id),
            Self::LatestUpdated => archive.indexed_latest_slot(),
            Self::OldestUpdated => archive.indexed_oldest_slot(),
            Self::LatestUpdatedWithTag { tag } => archive.indexed_latest_tag_slot(tag.trim()),
            Self::OldestUpdatedWithTag { tag } => archive.indexed_oldest_tag_slot(tag.trim()),
        }
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: self.missing_slot_label(),
        })?;

        Ok(RuntimeSessionSlotSelection::new(
            archive.generation(),
            archive.revision(),
            slot,
        ))
    }

    pub fn resolve_manifest(
        &self,
        manifest: &RuntimeSessionArchiveManifest,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        let summary = match self {
            Self::SlotId { slot_id } => manifest.slot(slot_id),
            Self::LatestUpdated => manifest.latest_updated_slot(),
            Self::OldestUpdated => manifest.oldest_updated_slot(),
            Self::LatestUpdatedWithTag { tag } => manifest.latest_updated_slot_with_tag(tag),
            Self::OldestUpdatedWithTag { tag } => manifest.oldest_updated_slot_with_tag(tag),
        }
        .ok_or_else(|| RuntimeSessionArchiveError::MissingSlot {
            slot_id: self.missing_slot_label(),
        })?;

        Ok(RuntimeSessionSlotSelectionReport {
            selector: self.clone(),
            selected_slot_id: summary.slot_id.clone(),
            summary: summary.clone(),
        })
    }

    fn missing_slot_label(&self) -> String {
        match self {
            Self::SlotId { slot_id } => slot_id.clone(),
            Self::LatestUpdated => "<latest-updated>".to_string(),
            Self::OldestUpdated => "<oldest-updated>".to_string(),
            Self::LatestUpdatedWithTag { tag } => {
                format!("<latest-updated tag=\"{}\">", tag.trim())
            }
            Self::OldestUpdatedWithTag { tag } => {
                format!("<oldest-updated tag=\"{}\">", tag.trim())
            }
        }
    }
}

fn normalize_selector_value(value: impl Into<String>) -> String {
    let mut value = value.into();
    let trimmed_end = value.trim_end().len();
    value.truncate(trimmed_end);

    let trimmed_start = value.len() - value.trim_start().len();
    if trimmed_start != 0 {
        value.drain(..trimmed_start);
    }
    value
}

#[cfg(test)]
#[path = "resolve/in_place_tests.rs"]
mod in_place_tests;
