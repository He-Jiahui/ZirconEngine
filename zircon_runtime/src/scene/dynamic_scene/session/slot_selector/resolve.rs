use super::super::{
    RuntimeSessionArchive, RuntimeSessionArchiveError, RuntimeSessionArchiveManifest,
};
use super::report::RuntimeSessionSlotSelectionReport;
use super::selector::RuntimeSessionSlotSelector;

impl RuntimeSessionSlotSelector {
    pub fn slot_id(slot_id: impl Into<String>) -> Self {
        Self::SlotId {
            slot_id: slot_id.into(),
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
            tag: tag.into().trim().to_string(),
        }
    }

    pub fn oldest_updated_with_tag(tag: impl Into<String>) -> Self {
        Self::OldestUpdatedWithTag {
            tag: tag.into().trim().to_string(),
        }
    }

    pub fn resolve(
        &self,
        archive: &RuntimeSessionArchive,
    ) -> Result<RuntimeSessionSlotSelectionReport, RuntimeSessionArchiveError> {
        self.resolve_manifest(&archive.manifest()?)
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
